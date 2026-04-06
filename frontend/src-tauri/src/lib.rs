use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::State;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED};

use ai_rpa::tool::ExecutionContext;
use ai_rpa::tool_registry::ToolRegistry;
use ai_rpa::selector::Selector;
use ai_rpa::app_launcher::{find_executable, launch_app_and_wait};
use ai_rpa::highlight_overlay::draw_highlight_rect_async;

// ─── Global state ─────────────────────────────────────────────

static EXECUTION_STOPPED: AtomicBool = AtomicBool::new(false);

pub struct AppState {
    pub logs: Mutex<Vec<String>>,
}

// ─── IPC types ────────────────────────────────────────────────

#[derive(Deserialize, Clone)]
pub struct ScenarioStep {
    #[serde(rename = "type")]
    pub step_type: String,
    pub config: std::collections::HashMap<String, String>,
}

#[derive(Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub log: Vec<String>,
}

// ─── COM RAII Guard ───────────────────────────────────────────

/// Гарантирует CoUninitialize при выходе из scope — даже при early return или panic.
struct ComGuard;

impl ComGuard {
    fn new() -> Result<Self, String> {
        // Пробуем STA — UI Automation работает и в STA.
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_ok() {
            return Ok(ComGuard);
        }
        // Если уже инициализирован с другим режимом — пробуем MTA.
        let hr2 = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr2.is_ok() {
            return Ok(ComGuard);
        }
        Err(format!("CoInitializeEx failed: STA=0x{:08X}, MTA=0x{:08X}",
            hr.0 as u32, hr2.0 as u32))
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize(); }
    }
}

// ─── Tauri Commands ───────────────────────────────────────────

fn run_scenario(steps: Vec<ScenarioStep>) -> Result<ExecutionResult, String> {
    EXECUTION_STOPPED.store(false, Ordering::SeqCst);
    let mut logs: Vec<String> = vec!["▶ Запуск сценария...".to_string()];

    let _com = ComGuard::new()?;

    let automation = uiautomation::UIAutomation::new_direct().map_err(|e| e.to_string())?;
    let registry = ToolRegistry::new();
    let mut ctx = ExecutionContext::new();

    for (i, step) in steps.iter().enumerate() {
        if EXECUTION_STOPPED.load(Ordering::SeqCst) {
            logs.push("⏹ Сценарий остановлен".to_string());
            break;
        }

        let step_num = i + 1;

        match step.step_type.as_str() {
            "LaunchApp" => {
                let app_name = step.config.get("app").map(|s| s.as_str()).unwrap_or("notepad");
                logs.push(format!("  [{step_num}] 🚀 Запуск: {}", app_name));

                match find_executable(app_name) {
                    Ok(app_path) => {
                        match launch_app_and_wait(&app_path, &[], 1000) {
                            Ok(pid) => {
                                logs.push(format!("      ✓ PID: {}", pid));
                            }
                            Err(e) => {
                                logs.push(format!("      ❌ {}", e));
                                logs.push("⏹ Сценарий остановлен из-за ошибки".to_string());
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        logs.push(format!("      ❌ {}", e));
                        logs.push("⏹ Сценарий остановлен из-за ошибки".to_string());
                        break;
                    }
                }
            }
            "CloseApp" => {
                let process_name = step.config.get("process_name").cloned().unwrap_or_default();
                let force = step.config.get("force").map(|v| v == "true").unwrap_or(false);
                logs.push(format!("  [{step_num}] 🛑 Закрытие: {}{}", process_name, if force { " (force)" } else { "" }));

                let mut type_config = std::collections::HashMap::new();
                type_config.insert("process_name".to_string(), process_name.clone());
                type_config.insert("force".to_string(), force.to_string());
                match registry.execute_tool_with_config(
                    "CloseApp",
                    Selector::Classname("_".to_string()),
                    &type_config,
                    &automation,
                    &mut ctx,
                ) {
                    Ok(()) => { logs.push(format!("      ✓ '{}' закрыт", process_name)); }
                    Err(e) => { logs.push(format!("      ❌ {}", e)); }
                }
            }
            _ => {
                // Все остальные инструменты обрабатываются через registry
                let tool_name = match step.step_type.as_str() {
                    "Click" => "Click",
                    "TypeText" => "Type",
                    "ExtractText" => "ExtractText",
                    "Wait" => "Wait",
                    "WaitForElement" => "WaitForElement",
                    "DoubleClick" => "DoubleClick",
                    "RightClick" => "RightClick",
                    "KeyPress" => "KeyPress",
                    "MoveMouse" => "MoveMouse",
                    "DragAndDrop" => "DragAndDrop",
                    "Condition" => "Condition",
                    "Retry" => "Retry",
                    "ReadFile" => "ReadFile",
                    "WriteFile" => "WriteFile",
                    "Screenshot" => "Screenshot",
                    other => {
                        logs.push(format!("  [{step_num}] ❓ Неизвестный тип: {}", other));
                        continue;
                    }
                };

                // Для инструментов, требующих селектор
                let selector = if matches!(tool_name, "Click" | "Type" | "ExtractText" | "WaitForElement"
                    | "DoubleClick" | "RightClick" | "MoveMouse" | "DragAndDrop" | "Condition" | "Retry" | "Screenshot") {
                    let sel_str = step.config.get("selector").cloned().unwrap_or_default();
                    match parse_selector(&sel_str) {
                        Ok(s) => s,
                        Err(e) => {
                            logs.push(format!("  [{step_num}] ❌ {}", e));
                            continue;
                        }
                    }
                } else {
                    Selector::Classname("_".to_string())
                };

                let label = step.config.get("selector")
                    .or_else(|| step.config.get("keys"))
                    .or_else(|| step.config.get("file_path"))
                    .or_else(|| step.config.get("duration_ms"))
                    .map(|v| v.as_str())
                    .unwrap_or(tool_name);
                logs.push(format!("  [{step_num}] {} {}", get_emoji(tool_name), label));

                match registry.execute_tool_with_config(tool_name, selector, &step.config, &automation, &mut ctx) {
                    Ok(()) => {
                        if let Some(v) = ctx.variables.values().last() {
                            let s = v.as_str().unwrap_or("?");
                            if s.len() < 200 {
                                logs.push(format!("      ✓ {}", s));
                            } else {
                                logs.push("      ✓".to_string());
                            }
                        } else {
                            logs.push(format!("      ✓ {}", tool_name));
                        }
                    }
                    Err(e) => {
                        logs.push(format!("      ❌ {}", e));
                    }
                }
            }
        }
    }

    if !logs.iter().any(|l| l.contains("остановлен")) {
        logs.push("✓ Сценарий завершён".to_string());
    }

    let result = ExecutionResult {
        success: !logs.iter().any(|l| l.starts_with("❌")),
        log: logs.clone(),
    };

    Ok(result)
}

fn get_emoji(tool: &str) -> &'static str {
    match tool {
        "Click" => "🖱",
        "Type" => "⌨",
        "ExtractText" => "📄",
        "Wait" => "⏳",
        "WaitForElement" => "⏱",
        "DoubleClick" => "🖱🖱",
        "RightClick" => "🖱R",
        "KeyPress" => "⌨️",
        "MoveMouse" => "🔹",
        "DragAndDrop" => "↔️",
        "Condition" => "🔍",
        "Retry" => "🔄",
        "ReadFile" => "📖",
        "WriteFile" => "📝",
        "Screenshot" => "📸",
        _ => "⚙",
    }
}

#[tauri::command]
async fn execute_scenario(
    steps: Vec<ScenarioStep>,
    state: State<'_, AppState>,
) -> Result<ExecutionResult, String> {
    // spawn_blocking — не блокирует Tauri event loop, UI остаётся отзывчивым.
    let result = tokio::task::spawn_blocking(move || run_scenario(steps))
        .await
        .map_err(|e| format!("Execution panicked: {}", e))??;

    *state.logs.lock().unwrap() = result.log.clone();
    Ok(result)
}

#[tauri::command]
fn stop_execution() {
    EXECUTION_STOPPED.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn highlight_element(selector: String) -> Result<(), String> {
    // Подсветка элемента через GDI overlay
    // CoInitializeEx вызывается внутри UIAutomation::new()
    let automation = uiautomation::UIAutomation::new().map_err(|e| e.to_string())?;
    let root = automation.get_root_element().map_err(|e| e.to_string())?;

    if let Ok(sel) = parse_selector(&selector) {
        if let Ok(element) = sel.find(&automation, &root) {
            if let Ok(rect) = element.get_bounding_rectangle() {
                draw_highlight_rect_async(
                    rect.get_left(),
                    rect.get_top(),
                    rect.get_width(),
                    rect.get_height(),
                    2000,
                );
                // COM освободится при завершении потока Tauri
                return Ok(());
            }
        }
    }
    Err("Элемент не найден".to_string())
}

// ─── Helpers ──────────────────────────────────────────────────

fn parse_selector(s: &str) -> Result<Selector, String> {
    if let Some(rest) = s.strip_prefix("classname=") {
        Ok(Selector::Classname(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name=") {
        Ok(Selector::Name(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("id=") {
        Ok(Selector::AutomationId(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name_contains=") {
        Ok(Selector::NameContains(rest.to_string()))
    } else {
        Err(format!("Неизвестный формат селектора: '{}'", s))
    }
}

// ─── Tauri App Setup ──────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            logs: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            execute_scenario,
            stop_execution,
            highlight_element,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}
