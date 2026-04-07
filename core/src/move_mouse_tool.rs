// move_mouse_tool.rs — Перемещение курсора к элементу

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use uiautomation::UIAutomation;

pub struct MoveMouseTool {
    pub selector: Selector,
    pub process_pid: Option<u32>,
}

impl MoveMouseTool {
    pub fn new(selector: Selector, process_pid: Option<u32>) -> Self {
        Self { selector, process_pid }
    }
}

impl Tool for MoveMouseTool {
    fn name(&self) -> &str { "MoveMouse" }
    fn description(&self) -> &str { "Переместить курсор к элементу без клика" }
    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let element = self.selector.find_with_pid(automation, &root, self.process_pid)?;
        let point = element.get_clickable_point()?;

        if let Some(_pt) = point {
            let rect = element.get_bounding_rectangle()?;
            let center_x = rect.get_left() + rect.get_width() / 2;
            let center_y = rect.get_top() + rect.get_height() / 2;

            unsafe {
                windows::Win32::UI::Input::KeyboardAndMouse::SendInput(
                    &[windows::Win32::UI::Input::KeyboardAndMouse::INPUT {
                        r#type: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE,
                        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                            mi: windows::Win32::UI::Input::KeyboardAndMouse::MOUSEINPUT {
                                dx: center_x,
                                dy: center_y,
                                mouseData: 0,
                                dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_ABSOLUTE
                                    | windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_MOVE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    }],
                    std::mem::size_of::<windows::Win32::UI::Input::KeyboardAndMouse::INPUT>() as i32,
                );
            }

            ctx.log(format!("✅ Курсор перемещён: {:?}", self.selector));
        } else {
            ctx.log(format!("⚠️  Не удалось определить точку клика: {:?}", self.selector));
        }

        Ok(())
    }
}
