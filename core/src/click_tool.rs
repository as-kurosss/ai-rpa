// click_tool.rs

use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use anyhow::Result;
use uiautomation::UIAutomation;

/// Инструмент для клика по элементу UI
pub struct ClickTool {
    /// Селектор для поиска элемента
    pub selector: Selector,

    /// Описание для пользователя (опционально)
    pub description: String,
}

impl ClickTool {
    /// Создает новый инструмент с селектором
    pub fn new(selector: Selector) -> Self {
        Self {
            selector,
            description: "Click on UI element".to_string(),
        }
    }
}

impl Tool for ClickTool {
    fn name(&self) -> &str {
        "Click"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        // 1. Получаем корневой элемент дерева UI (весь экран)
        let root = automation.get_root_element()?;

        // 2. Ищем целевой элемент по селектору
        let element = self.selector.find(automation, &root)?;

        // 3. Кликаем по элементу
        element.click()?;

        // 4. Логируем успешное действие в контекст
        ctx.log(format!("✅ Clicked on element: {:?}", self.selector));

        Ok(())
    }
}