use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::State;

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

// ─── Tauri Commands ───────────────────────────────────────────

fn run_scenario(steps: Vec<ScenarioStep>) -> Result<ExecutionResult, String> {
    EXECUTION_STOPPED.store(false, Ordering::SeqCst);
    let mut logs: Vec<String> = vec!["▶ Запуск сценария...".to_string()];

    // CoInitializeEx для STA — Tauri команды запускаются на MTA потоках,
    // а UI Automation требует STA.
    unsafe {
        let hr = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
        // S_OK или S_FALSE (уже инициализировано) — оба OK
        if hr.is_err() && hr != windows::Win32::Foundation::S_FALSE {
            return Err(format!("CoInitializeEx failed: {hr:?}"));
        }
    }

    let automation = uiautomation::UIAutomation::new().map_err(|e| e.to_string())?;
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
            "Click" => {
                let selector_str = step.config.get("selector").cloned().unwrap_or_default();
                logs.push(format!("  [{step_num}] 🖱 Клик: {}", selector_str));

                if let Ok(selector) = parse_selector(&selector_str) {
                    match registry.execute_tool("Click", selector, &automation, &mut ctx) {
                        Ok(()) => {
                            logs.push("      ✓ Клик выполнен".to_string());
                        }
                        Err(e) => {
                            logs.push(format!("      ❌ {}", e));
                        }
                    }
                } else {
                    logs.push(format!("      ❌ Невалидный селектор: {}", selector_str));
                }
            }
            "TypeText" => {
                let selector_str = step.config.get("selector").cloned().unwrap_or_default();
                let text = step.config.get("text").cloned().unwrap_or_default();
                logs.push(format!("  [{step_num}] ⌨ Ввод: \"{}\"", text));

                if let Ok(selector) = parse_selector(&selector_str) {
                    match registry.execute_tool_with_text("Type", selector, &text, &automation, &mut ctx) {
                        Ok(()) => {
                            logs.push("      ✓ Ввод выполнен".to_string());
                        }
                        Err(e) => {
                            logs.push(format!("      ❌ {}", e));
                        }
                    }
                } else {
                    logs.push(format!("      ❌ Невалидный селектор: {}", selector_str));
                }
            }
            other => {
                logs.push(format!("  [{step_num}] ❓ Неизвестный тип: {}", other));
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

    // CoUninitialize — освобождаем COM на этом потоке
    unsafe {
        windows::Win32::System::Com::CoUninitialize();
    }

    Ok(result)
}

#[tauri::command]
fn execute_scenario(steps: Vec<ScenarioStep>, state: State<AppState>) -> Result<ExecutionResult, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<ExecutionResult, String>>();

    std::thread::spawn(move || {
        let result = run_scenario(steps);
        let _ = tx.send(result);
    });

    let result = rx.recv().map_err(|_| "Execution thread panicked")??;

    // Сохраняем логи в AppState
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
