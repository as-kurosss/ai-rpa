// selector_recorder.rs

use anyhow::{anyhow, Result};
use uiautomation::{
    UIAutomation,
    UIElement,
    types::ControlType,
};
use crate::selector::Selector;

/// Один шаг в дереве селектора
#[derive(Debug, Clone)]
pub struct SelectorStep {
    /// Имя класса (например, "Button", "Edit", "Notepad")
    pub classname: Option<String>,
    /// Тип элемента управления
    pub control_type: Option<ControlType>,
    /// Имя/заголовок элемента
    pub name: Option<String>,
    /// Автоматизированный ID (если есть)
    pub automation_id: Option<String>,
}

impl SelectorStep {
    /// Создает шаг из UI элемента
    pub fn from_element(_automation: &UIAutomation, element: &UIElement) -> Result<Self> {
        let classname = element.get_classname().ok();
        let control_type = element.get_control_type().ok();
        let name = element.get_name().ok();
        let automation_id = element.get_automation_id().ok();

        Ok(Self {
            classname,
            control_type,
            name,
            automation_id,
        })
    }

    /// Преобразует шаг в Selector
    pub fn to_selector(&self) -> Option<Selector> {
        // Приоритет: automation_id (не пустой) > classname > control_type > name (не пустой)

        // 1. Automation ID - самый надёжный, если не пустой
        if let Some(ref automation_id) = self.automation_id {
            if !automation_id.is_empty() {
                return Some(Selector::AutomationId(automation_id.clone()));
            }
        }
        
        // 2. Classname - надёжный, не зависит от языка
        if let Some(ref classname) = self.classname {
            if !classname.is_empty() {
                return Some(Selector::Classname(classname.clone()));
            }
        }
        
        // 3. Control Type - хороший вариант
        if let Some(control_type) = self.control_type {
            return Some(Selector::ControlType(control_type));
        }
        
        // 4. Name - только если не пустой
        if let Some(ref name) = self.name {
            if !name.is_empty() {
                return Some(Selector::Name(name.clone()));
            }
        }
        
        // Ничего не подошло
        None
    }
}

/// Полный записанный селектор с деревом шагов
#[derive(Debug, Clone)]
pub struct RecordedSelector {
    /// Дерево шагов от Application до целевого элемента
    pub steps: Vec<SelectorStep>,
    /// Глубина элемента (количество шагов от корня)
    pub depth: usize,
}

impl RecordedSelector {
    /// Создает финальный Selector из последнего шага
    pub fn to_selector(&self) -> Option<Selector> {
        self.steps.last().and_then(|step| step.to_selector())
    }

    /// Печатает читаемое представление селектора
    pub fn print_tree(&self) {
        println!("📋 Записанный селектор ({} шагов):", self.steps.len());
        for (i, step) in self.steps.iter().enumerate() {
            let indent = "  ".repeat(i);
            let classname = step.classname.as_deref().unwrap_or("?");
            let control_type = step.control_type
                .map(|ct| format!("{:?}", ct))
                .unwrap_or_else(|| "?".to_string());
            let name = step.name.as_deref().unwrap_or("");
            let automation_id = step.automation_id.as_deref().unwrap_or("");

            print!("{}[{}] classname={}, type={}", indent, i, classname, control_type);
            if !name.is_empty() {
                print!(", name=\"{}\"", name);
            }
            if !automation_id.is_empty() {
                print!(", automation_id=\"{}\"", automation_id);
            }
            println!();
        }
    }
}

/// Рекордер для записи селекторов
pub struct SelectorRecorder {
    pub automation: UIAutomation,
}

impl SelectorRecorder {
    /// Создает новый рекордер
    pub fn new(automation: UIAutomation) -> Self {
        Self { automation }
    }

    /// Захватывает элемент под курсором мыши и строит полный селектор
    pub fn capture_element_under_cursor(&self) -> Result<RecordedSelector> {
        // Получаем элемент под курсором
        let element = self.get_element_under_cursor()?;
        
        // Строим полный путь от корня до элемента
        let steps = self.build_full_selector_tree(&element)?;
        
        let depth = steps.len();

        Ok(RecordedSelector { steps, depth })
    }

    /// Захватывает переданный элемент и строит полный селектор
    pub fn capture_element(&self, element: &UIElement) -> Result<RecordedSelector> {
        // Строим полный путь от корня до элемента
        let steps = self.build_full_selector_tree(element)?;

        let depth = steps.len();

        Ok(RecordedSelector { steps, depth })
    }

    /// Получает UI элемент под текущей позицией курсора
    fn get_element_under_cursor(&self) -> Result<UIElement> {
        // Получаем курсор через Win32 API
        let (x, y) = get_cursor_position()?;
        
        // Создаем точку для UI Automation (tuple struct)
        let point = uiautomation::types::Point::new(x, y);
        
        // Получаем элемент по координатам
        let element = self.automation
            .element_from_point(point)
            .map_err(|e| anyhow!("Не удалось получить элемент по координатам ({}, {}): {}", x, y, e))?;

        Ok(element)
    }

    /// Строит полное дерево селекторов от Application до элемента
    fn build_full_selector_tree(&self, element: &UIElement) -> Result<Vec<SelectorStep>> {
        let mut steps = Vec::new();
        let mut current = element.clone();

        // Создаем TreeWalker для обхода дерева
        let walker = self.automation.create_tree_walker()?;

        const MAX_DEPTH: usize = 100;

        // Поднимаемся по дереву от элемента к корню
        loop {
            if steps.len() >= MAX_DEPTH {
                return Err(anyhow!("Selector tree exceeds maximum depth of {}", MAX_DEPTH));
            }

            let step = SelectorStep::from_element(&self.automation, &current)?;
            steps.push(step);

            // Пытаемся получить родительский элемент через TreeWalker
            match walker.get_parent(&current) {
                Ok(parent) => {
                    current = parent;
                }
                Err(_) => {
                    // Достигли корня
                    break;
                }
            }
        }

        // Разворачиваем, чтобы идти от Application к элементу
        steps.reverse();

        Ok(steps)
    }

    /// Получает все свойства элемента в читаемом формате
    pub fn get_element_properties(&self, element: &UIElement) -> Result<ElementProperties> {
        Ok(ElementProperties {
            classname: element.get_classname().ok(),
            control_type: element.get_control_type().ok(),
            name: element.get_name().ok(),
            automation_id: element.get_automation_id().ok(),
            localized_control_type: element.get_localized_control_type().ok(),
            bounding_rectangle: element.get_bounding_rectangle().ok(),
            is_enabled: element.is_enabled().ok(),
            is_keyboard_focusable: element.is_keyboard_focusable().ok(),
            has_keyboard_focus: element.has_keyboard_focus().ok(),
            help_text: element.get_help_text().ok(),
        })
    }
}

/// Свойства элемента для детального просмотра
#[derive(Debug, Clone)]
pub struct ElementProperties {
    pub classname: Option<String>,
    pub control_type: Option<ControlType>,
    pub name: Option<String>,
    pub automation_id: Option<String>,
    pub localized_control_type: Option<String>,
    pub bounding_rectangle: Option<uiautomation::types::Rect>,
    pub is_enabled: Option<bool>,
    pub is_keyboard_focusable: Option<bool>,
    pub has_keyboard_focus: Option<bool>,
    pub help_text: Option<String>,
}

impl ElementProperties {
    /// Печатает все свойства в читаемом формате
    pub fn print(&self) {
        println!("🔍 Свойства элемента:");
        println!("  classname:            {:?}", self.classname);
        println!("  control_type:         {:?}", self.control_type);
        println!("  name:                 {:?}", self.name);
        println!("  automation_id:        {:?}", self.automation_id);
        println!("  localized_control_type: {:?}", self.localized_control_type);
        
        if let Some(ref rect) = self.bounding_rectangle {
            println!("  bounding_rectangle:   x={}, y={}, w={}, h={}", 
                rect.get_left(), rect.get_top(), rect.get_width(), rect.get_height()
            );
        }
        
        println!("  is_enabled:           {:?}", self.is_enabled);
        println!("  is_keyboard_focusable: {:?}", self.is_keyboard_focusable);
        println!("  has_keyboard_focus:   {:?}", self.has_keyboard_focus);
        println!("  help_text:            {:?}", self.help_text);
    }
}

/// Получает текущую позицию курсора через Win32 API
fn get_cursor_position() -> Result<(i32, i32)> {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::Foundation::POINT;

    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut point)
            .map_err(|e| anyhow!("Не удалось получить позицию курсора: {}", e))?;
    }
    Ok((point.x, point.y))
}

/// Проверяет, принадлежит ли элемент Electron-приложению (VSCode, Slack, Discord и т.д.)
/// Electron использует classname "Chrome_WidgetWin_0", "Chrome_WidgetWin_1" и т.д.
///
/// Если элемент принадлежит Electron, UI Automation может не видеть внутренние элементы.
/// Для таких приложений рекомендуется использовать веб-инструменты (Playwright, Spectron).
pub fn is_electron_element(element: &uiautomation::UIElement) -> bool {
    element.get_classname()
        .ok()
        .map(|name| name.starts_with("Chrome_WidgetWin_"))
        .unwrap_or(false)
}
