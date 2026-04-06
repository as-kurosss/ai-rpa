// drag_drop_tool.rs — Перетаскивание между элементами

use crate::selector::Selector;
use crate::tool::{Tool, ExecutionContext};
use anyhow::Result;
use std::time::Duration;
use uiautomation::UIAutomation;

pub struct DragDropTool {
    /// Селектор источника
    pub source: Selector,
    /// Селектор цели
    pub target: Selector,
    /// Задержка перед отпусканием (мс)
    pub delay_ms: u64,
}

impl DragDropTool {
    pub fn new(source: Selector, target: Selector, delay_ms: u64) -> Self {
        Self { source, target, delay_ms }
    }
}

impl Tool for DragDropTool {
    fn name(&self) -> &str {
        "DragAndDrop"
    }

    fn description(&self) -> &str {
        "Перетащить элемент к другому элементу"
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let root = automation.get_root_element()?;
        let src = self.source.find(automation, &root)?;
        let tgt = self.target.find(automation, &root)?;

        let src_rect = src.get_bounding_rectangle()?;
        let tgt_rect = tgt.get_bounding_rectangle()?;

        let src_x = src_rect.get_left() + src_rect.get_width() / 2;
        let src_y = src_rect.get_top() + src_rect.get_height() / 2;
        let tgt_x = tgt_rect.get_left() + tgt_rect.get_width() / 2;
        let tgt_y = tgt_rect.get_top() + tgt_rect.get_height() / 2;

        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        unsafe {
            // Move to source
            let inputs_move = [
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: src_x,
                            dy: src_y,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            SendInput(&inputs_move, std::mem::size_of::<INPUT>() as i32);

            std::thread::sleep(Duration::from_millis(50));

            // Mouse down (left)
            let inputs_down = [
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            SendInput(&inputs_down, std::mem::size_of::<INPUT>() as i32);

            std::thread::sleep(Duration::from_millis(200));

            // Move to target
            let inputs_drag = [
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: tgt_x - src_x,
                            dy: tgt_y - src_y,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            SendInput(&inputs_drag, std::mem::size_of::<INPUT>() as i32);

            if self.delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.delay_ms));
            }

            // Mouse up (left)
            let inputs_up = [
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            SendInput(&inputs_up, std::mem::size_of::<INPUT>() as i32);
        }

        ctx.log(format!("✅ DragAndDrop: {:?} → {:?}", self.source, self.target));
        Ok(())
    }
}
