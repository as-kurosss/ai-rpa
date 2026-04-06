// close_tool.rs — Закрытие приложения по имени процесса

use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;

/// Инструмент для закрытия приложения по имени процесса (без расширения .exe).
/// Поддерживает graceful (WM_CLOSE) и forceful (taskkill /F) завершение.
pub struct CloseTool {
    /// Имя процесса без расширения, например "notepad"
    pub process_name: String,

    /// Принудительное закрытие (taskkill /F) или graceful (по умолчанию graceful)
    pub force: bool,

    /// Таймаут ожидания закрытия в миллисекундах (по умолчанию 3000)
    pub timeout_ms: u64,
}

impl CloseTool {
    pub fn new(process_name: String, force: bool, timeout_ms: u64) -> Self {
        Self {
            process_name,
            force,
            timeout_ms,
        }
    }
}

impl Tool for CloseTool {
    fn name(&self) -> &str {
        "CloseApp"
    }

    fn description(&self) -> &str {
        "Закрыть приложение по имени процесса"
    }

    fn execute(&self, _automation: &uiautomation::UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let exe_name = if self.process_name.ends_with(".exe") {
            self.process_name.clone()
        } else {
            format!("{}.exe", self.process_name)
        };

        // Пробуем graceful закрытие через WM_CLOSE (taskkill без /F)
        let flag = if self.force { "/F" } else { "" };
        let args = if self.force {
            vec!["/IM", &exe_name, flag]
        } else {
            vec!["/IM", &exe_name]
        };

        let output = Command::new("taskkill")
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| anyhow!("Не удалось выполнить taskkill: {}", e))?;

        if output.status.success() {
            ctx.log(format!("✅ Приложение '{}' закрыто", exe_name));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // ERROR: The process "notepad.exe" not found — не считается ошибкой
            if stderr.contains("not found") || stderr.contains("не найдено") {
                ctx.log(format!("⚠️  Приложение '{}' не запущено", exe_name));
            } else {
                return Err(anyhow!("Не удалось закрыть '{}': {}", exe_name, stderr.trim()));
            }
        }

        // Если не force — даём время на graceful завершение
        if !self.force && self.timeout_ms > 0 {
            std::thread::sleep(Duration::from_millis(self.timeout_ms));
        }

        Ok(())
    }
}
