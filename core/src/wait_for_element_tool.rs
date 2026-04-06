// wait_for_element_tool.rs — Ожидание появления элемента

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use std::time::Duration;
use uiautomation::UIAutomation;

/// Ждёт появления элемента по селектору с таймаутом
pub struct WaitForElementTool {
    pub selector: Selector,
    pub timeout_ms: u64,
    pub interval_ms: u64,
}

impl WaitForElementTool {
    pub fn new(selector: Selector, timeout_ms: u64, interval_ms: u64) -> Self {
        Self { selector, timeout_ms, interval_ms }
    }
}

impl Tool for WaitForElementTool {
    fn name(&self) -> &str {
        "WaitForElement"
    }

    fn description(&self) -> &str {
        "Ждать появления элемента по селектору"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(self.timeout_ms);
        let interval = Duration::from_millis(self.interval_ms);

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Элемент не появился за {}ms: {:?}", self.timeout_ms, self.selector));
            }

            if self.selector.find(automation, &root).is_ok() {
                ctx.log(format!("✅ Элемент появился за {}ms", start.elapsed().as_millis()));
                return Ok(());
            }

            std::thread::sleep(interval);
        }
    }
}
