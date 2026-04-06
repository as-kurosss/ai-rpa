// condition_tool.rs — Условное выполнение

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use serde_json::json;
use uiautomation::UIAutomation;

/// Проверяет наличие элемента и сохраняет результат в переменную.
/// Следующие шаги могут использовать `${condition_result}` для ветвления.
pub struct ConditionTool {
    pub selector: Selector,
    /// Имя переменной для сохранения результата (по умолчанию "condition_result")
    pub var_name: String,
}

impl ConditionTool {
    pub fn new(selector: Selector, var_name: String) -> Self {
        Self { selector, var_name }
    }
}

impl Tool for ConditionTool {
    fn name(&self) -> &str {
        "Condition"
    }

    fn description(&self) -> &str {
        "Проверить наличие элемента и сохранить true/false в переменную"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let exists = self.selector.find(automation, &root).is_ok();
        ctx.variables.insert(self.var_name.clone(), json!(exists));
        ctx.log(format!("🔍 {}: {}", self.var_name, if exists { "true" } else { "false" }));
        Ok(())
    }
}
