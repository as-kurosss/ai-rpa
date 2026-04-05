// src/lib.rs

pub mod tool;
pub mod selector;
pub mod click_tool;
pub mod type_tool;
pub mod tool_registry;
pub mod selector_recorder;
pub mod highlight;
pub mod highlight_overlay;
pub mod app_launcher;

// Экспортируем для удобства использования в примерах
pub use tool::ExecutionContext;
pub use selector::Selector;
pub use click_tool::ClickTool;
pub use type_tool::TypeTool;
pub use selector_recorder::{SelectorRecorder, RecordedSelector, SelectorStep, ElementProperties, is_electron_element};
pub use highlight::{HighlightConfig, highlight_element, highlight_element_animated, highlight_selector_tree};
pub use highlight_overlay::{draw_highlight_rect_async, draw_highlight_rect_blocking, draw_highlight_rect_animated, draw_highlight_rect_track_cursor, get_dpi_scale, scale_rect, ensure_dpi_aware};
pub use app_launcher::{find_executable, launch_app, launch_app_and_wait, parse_app_arg};