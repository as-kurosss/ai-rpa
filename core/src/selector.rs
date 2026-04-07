// selector.rs

use anyhow::anyhow;
use anyhow::Result;
use uiautomation::{
    UIAutomation,
    UIElement,
    types::ControlType
};

/// Способы поиска элементов в UI
#[derive(Debug, Clone)]
pub enum Selector {
    /// По имени класса окна (надежно, не зависит от языка)
    Classname(String),

    /// По типу элемента (Button, Edit, Window...)
    ControlType(ControlType),

    /// По имени элемента (заголовок, текст - зависит от локализации!)
    Name(String),

    /// По частичному вхождению в имя (гибче, чем Name)
    NameContains(String),

    /// По Automation ID (самый надёжный для автоматизации)
    AutomationId(String),

    /// Комбинация условий (ИЛИ)
    Or(Vec<Selector>),

    /// Фильтрация по ProcessId — ограничивает поиск элементами конкретного процесса
    /// Полезно при нескольких экземплярах одного приложения
    ProcessId(u32),
}

impl Selector {
    /// Находит первый элемент, соответствующий селектору.
    /// Все варианты используют `find_first` с tree walker — O(depth) вместо O(n) full-tree scan.
    pub fn find(&self, automation: &UIAutomation, root: &UIElement) -> Result<UIElement, anyhow::Error> {
        self.find_with_pid(automation, root, None)
    }

    /// Находит элемент с опциональной фильтрацией по PID процесса.
    /// Если PID указан — сначала ищется root-элемент процесса, затем внутри него — по селектору.
    pub fn find_with_pid(&self, automation: &UIAutomation, root: &UIElement, active_pid: Option<u32>) -> Result<UIElement, anyhow::Error> {
        // Определяем корневой элемент для поиска
        let search_root = if let Some(pid) = active_pid {
            // Ищем root-элемент процесса по PID
            match find_process_root(automation, pid) {
                Some(proc_root) => proc_root,
                None => {
                    // Процесс не найден — fallback на общий root
                    root.clone()
                }
            }
        } else {
            root.clone()
        };

        // Ищем по селектору внутри выбранного корня
        self.find_inside(automation, &search_root)
    }

    /// Внутренний метод — поиск внутри конкретного root-элемента
    fn find_inside(&self, automation: &UIAutomation, root: &UIElement) -> Result<UIElement, anyhow::Error> {
        match self {
            Selector::Classname(classname) => {
                let element = automation.create_matcher()
                    .from_ref(root)
                    .classname(classname)
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: classname={}: {}", classname, e))?;
                Ok(element)
            }

            Selector::ControlType(control_type) => {
                let element = automation.create_matcher()
                    .from_ref(root)
                    .control_type(*control_type)
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: control_type={:?}: {}", control_type, e))?;
                Ok(element)
            }

            Selector::Name(name) => {
                let element = automation.create_matcher()
                    .from_ref(root)
                    .name(name)
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: name={}: {}", name, e))?;
                Ok(element)
            }

            Selector::NameContains(substring) => {
                // Встроенный contains_name использует filter_fn с tree walker,
                // а не find_all с итерацией по всему дереву.
                let substr = substring.clone();
                let element = automation.create_matcher()
                    .from_ref(root)
                    .timeout(5000)
                    .contains_name(&substr)
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: name contains '{}': {}", substr, e))?;
                Ok(element)
            }

            Selector::AutomationId(automation_id) => {
                // Нет встроенного automation_id фильтра — используем filter_fn,
                // который применяется во время tree walk (гораздо быстрее find_all).
                let target_id = automation_id.clone();
                let element = automation.create_matcher()
                    .from_ref(root)
                    .timeout(5000)
                    .filter_fn(Box::new(move |el: &UIElement| {
                        el.get_automation_id()
                            .map(|id| id == target_id)
                    }))
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: automation_id='{}': {}", automation_id, e))?;
                Ok(element)
            }

            Selector::Or(selectors) => {
                for selector in selectors {
                    if let Ok(element) = selector.find(automation, root) {
                        return Ok(element);
                    }
                }
                Err(anyhow!("Element not found: {:?}", selectors))
            }

            Selector::ProcessId(pid) => {
                // Фильтруем элементы по ProcessId
                let target_pid = *pid;
                let element = automation.create_matcher()
                    .from_ref(root)
                    .timeout(5000)
                    .filter_fn(Box::new(move |el: &UIElement| {
                        match el.get_process_id() {
                            Ok(p) => Ok(p == target_pid),
                            Err(_) => Ok(false),
                        }
                    }))
                    .find_first()
                    .map_err(|e| anyhow!("Element not found: process_id={}: {}", target_pid, e))?;
                Ok(element)
            }
        }
    }
}

/// Находит root-элемент процесса по PID.
/// Ищет среди элементов верхнего уровня первый элемент с нужным PID.
fn find_process_root(automation: &UIAutomation, pid: u32) -> Option<UIElement> {
    let root = automation.get_root_element().ok()?;
    automation.create_matcher()
        .from_ref(&root)
        .timeout(2000)
        .filter_fn(Box::new(move |el: &UIElement| {
            match el.get_process_id() {
                Ok(p) => Ok(p == pid),
                Err(_) => Ok(false),
            }
        }))
        .find_first()
        .ok()
}
