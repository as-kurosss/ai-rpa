// extract_tool.rs — Извлечение текста из UI элемента

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use serde_json::json;
use uiautomation::UIAutomation;

/// Инструмент для извлечения текста из UI элемента (Edit, Text, Document и т.д.).
/// Извлечённый текст сохраняется в переменную контекста.
pub struct ExtractTool {
    /// Селектор для поиска элемента
    pub selector: Selector,

    /// Имя переменной для сохранения результата (по умолчанию "extracted_text")
    pub var_name: String,
}

impl ExtractTool {
    pub fn new(selector: Selector, var_name: String) -> Self {
        Self { selector, var_name }
    }
}

impl Tool for ExtractTool {
    fn name(&self) -> &str {
        "ExtractText"
    }

    fn description(&self) -> &str {
        "Извлечь текст из элемента и сохранить в переменную"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let element = self.selector.find(automation, &root)?;

        // Пробуем несколько источников текста:
        // 1. Value Pattern (редактируемые поля)
        // 2. Name (статический текст)
        // 3. Text Pattern (документы, RichEdit)

        let text = element.get_name().unwrap_or_default();

        if text.is_empty() {
            return Err(anyhow!("Элемент не содержит текста: {:?}", self.selector));
        }

        ctx.variables.insert(self.var_name.clone(), json!(text));
        ctx.log(format!("✅ Извлечён текст ({} символов) → ${}", text.len(), self.var_name));

        Ok(())
    }
}
