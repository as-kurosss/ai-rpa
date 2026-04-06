// key_press_tool.rs — Нажатие клавиш (Enter, Tab, Ctrl+C, Alt+F4 и т.д.)

use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use uiautomation::UIAutomation;

pub struct KeyPressTool {
    /// Комбинация клавиш в формате SendKeys: {Enter}, {Tab}, ^c, %{F4}
    pub keys: String,
    /// Задержка между нажатиями (мс)
    pub delay_ms: u64,
}

impl KeyPressTool {
    pub fn new(keys: String, delay_ms: u64) -> Self {
        Self { keys, delay_ms }
    }
}

impl Tool for KeyPressTool {
    fn name(&self) -> &str {
        "KeyPress"
    }

    fn description(&self) -> &str {
        "Нажатие клавиш (Enter, Tab, Ctrl+C, Alt+F4)"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let element = automation.get_focused_element()?;
        element.send_keys(&self.keys, self.delay_ms)?;
        ctx.log(format!("✅ Клавиши: '{}'", self.keys));
        Ok(())
    }
}
