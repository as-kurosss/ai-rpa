// click_tool.rs

use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use anyhow::Result;
use uiautomation::UIAutomation;

/// Инстпумент для клика по элементу UI
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
}