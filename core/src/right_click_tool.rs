// right_click_tool.rs — Правый клик по элементу

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use uiautomation::UIAutomation;

pub struct RightClickTool {
    pub selector: Selector,
    pub process_pid: Option<u32>,
}

impl RightClickTool {
    pub fn new(selector: Selector, process_pid: Option<u32>) -> Self {
        Self { selector, process_pid }
    }
}

impl Tool for RightClickTool {
    fn name(&self) -> &str { "RightClick" }
    fn description(&self) -> &str { "Правый клик по элементу (контекстное меню)" }
    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let element = self.selector.find_with_pid(automation, &root, self.process_pid)?;
        element.right_click()?;
        ctx.log(format!("✅ Правый клик: {:?}", self.selector));
        Ok(())
    }
}
