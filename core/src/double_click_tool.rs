// double_click_tool.rs — Двойной клик по элементу

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use uiautomation::UIAutomation;

pub struct DoubleClickTool {
    pub selector: Selector,
}

impl DoubleClickTool {
    pub fn new(selector: Selector) -> Self {
        Self { selector }
    }
}

impl Tool for DoubleClickTool {
    fn name(&self) -> &str {
        "DoubleClick"
    }

    fn description(&self) -> &str {
        "Двойной клик по элементу"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let element = self.selector.find(automation, &root)?;
        element.double_click()?;
        ctx.log(format!("✅ Двойной клик: {:?}", self.selector));
        Ok(())
    }
}
