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

    /// PID процесса для ограничения поиска (None = весь экран)
    pub process_pid: Option<u32>,
}

impl ExtractTool {
    pub fn new(selector: Selector, var_name: String, process_pid: Option<u32>) -> Self {
        Self { selector, var_name, process_pid }
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
        use uiautomation::patterns::UIValuePattern;

        let root = automation.get_root_element()?;
        let element = self.selector.find_with_pid(automation, &root, self.process_pid)?;

        // Извлекаем текст в правильном порядке приоритета:
        // 1. Value Pattern — содержимое редактируемых полей (Edit, ComboBox)
        // 2. Name — заголовок/имя элемента (кнопки, статический текст)

        let text = element
            .get_pattern::<UIValuePattern>()
            .ok()
            .and_then(|vp| vp.get_value().ok())
            .filter(|s| !s.is_empty())
            .or_else(|| element.get_name().ok().filter(|s| !s.is_empty()))
            .unwrap_or_default();

        if text.is_empty() {
            return Err(anyhow!("Элемент не содержит текста: {:?}", self.selector));
        }

        ctx.variables.insert(self.var_name.clone(), json!(text));
        ctx.log(format!("✅ Извлечён текст ({} символов) → ${}", text.len(), self.var_name));
        ctx.log(format!("      📝 Значение: \"{}\"", text.chars().take(100).collect::<String>()));

        Ok(())
    }
}
