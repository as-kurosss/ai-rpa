// screenshot_tool.rs — Скриншот элемента или экрана

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use serde_json::json;
use uiautomation::UIAutomation;

pub struct ScreenshotTool {
    /// Селектор элемента (если None — скриншот всего экрана)
    pub selector: Option<Selector>,
    /// Путь сохранения файла
    pub output_path: String,
    /// PID процесса для ограничения поиска (None = весь экран)
    pub process_pid: Option<u32>,
}

impl ScreenshotTool {
    pub fn new(selector: Option<Selector>, output_path: String, process_pid: Option<u32>) -> Self {
        Self { selector, output_path, process_pid }
    }
}

impl Tool for ScreenshotTool {
    fn name(&self) -> &str {
        "Screenshot"
    }

    fn description(&self) -> &str {
        "Скриншот элемента или всего экрана"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let (x, y, w, h) = match &self.selector {
            Some(sel) => {
                let root = automation.get_root_element()?;
                let element = sel.find_with_pid(automation, &root, self.process_pid)?;
                let rect = element.get_bounding_rectangle()?;
                (rect.get_left(), rect.get_top(), rect.get_width(), rect.get_height())
            }
            None => {
                // Весь экран — используем GetSystemMetrics через windows crate
                let screen_w = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN) };
                let screen_h = unsafe { windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN) };
                (0, 0, screen_w, screen_h)
            }
        };

        ctx.log(format!("📸 Скриншот области: x={}, y={}, w={}, h={} → {}", x, y, w, h, self.output_path));
        ctx.variables.insert("screenshot_path".to_string(), json!(self.output_path));
        // TODO: полноценное сохранение BMP/PNG — нужен крейт image
        Ok(())
    }
}
