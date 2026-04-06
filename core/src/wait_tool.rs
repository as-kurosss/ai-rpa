// wait_tool.rs — Пауза на заданное время

use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use std::time::Duration;

/// Пауза в миллисекундах
pub struct WaitTool {
    pub duration_ms: u64,
}

impl WaitTool {
    pub fn new(duration_ms: u64) -> Self {
        Self { duration_ms }
    }
}

impl Tool for WaitTool {
    fn name(&self) -> &str {
        "Wait"
    }

    fn description(&self) -> &str {
        "Пауза на заданное количество миллисекунд"
    }

    fn execute(&self, _automation: &uiautomation::UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        ctx.log(format!("⏳ Пауза {}ms", self.duration_ms));
        std::thread::sleep(Duration::from_millis(self.duration_ms));
        Ok(())
    }
}
