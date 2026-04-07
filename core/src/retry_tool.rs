// retry_tool.rs — Повтор действия до успеха

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use uiautomation::UIAutomation;

/// Повторяет поиск элемента и действие до успеха или достижения лимита попыток.
/// Действие — клик по найденному элементу.
pub struct RetryTool {
    pub selector: Selector,
    pub max_attempts: u32,
    pub delay_ms: u64,
    /// PID процесса для ограничения поиска (None = весь экран)
    pub process_pid: Option<u32>,
}

impl RetryTool {
    pub fn new(selector: Selector, max_attempts: u32, delay_ms: u64, process_pid: Option<u32>) -> Self {
        Self { selector, max_attempts, delay_ms, process_pid }
    }
}

impl Tool for RetryTool {
    fn name(&self) -> &str {
        "Retry"
    }

    fn description(&self) -> &str {
        "Повторять клик по элементу до успеха N раз"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;

        for attempt in 1..=self.max_attempts {
            match self.selector.find_with_pid(automation, &root, self.process_pid) {
                Ok(element) => {
                    match element.click() {
                        Ok(()) => {
                            ctx.log(format!("✅ Retry: успех на попытке {}/{}", attempt, self.max_attempts));
                            return Ok(());
                        }
                        Err(_) => {
                            ctx.log(format!("⚠️  Retry: попытка {}/{} — клик не удался", attempt, self.max_attempts));
                        }
                    }
                }
                Err(_) => {
                    ctx.log(format!("⚠️  Retry: попытка {}/{} — элемент не найден", attempt, self.max_attempts));
                }
            }
            if attempt < self.max_attempts {
                std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
            }
        }

        Err(anyhow::anyhow!("Не удалось выполнить действие за {} попыток", self.max_attempts))
    }
}
