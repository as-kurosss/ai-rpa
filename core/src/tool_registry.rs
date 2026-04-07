// tool_registry.rs

use std::collections::HashMap;
use anyhow::Result;
use crate::ClickTool;
use crate::TypeTool;
use crate::CloseTool;
use crate::ExtractTool;
use crate::WaitTool;
use crate::WaitForElementTool;
use crate::DoubleClickTool;
use crate::RightClickTool;
use crate::KeyPressTool;
use crate::MoveMouseTool;
use crate::DragDropTool;
use crate::ConditionTool;
use crate::RetryTool;
use crate::ReadFileTool;
use crate::WriteFileTool;
use crate::ScreenshotTool;
use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use uiautomation::UIAutomation;

fn parse_selector_str(s: &str) -> Option<Selector> {
    parse_selector_str_with_vars(s, None)
}

/// Резолвит PID из config: `pid` может быть числом или именем переменной
fn resolve_pid(config: &HashMap<String, String>, variables: &HashMap<String, serde_json::Value>) -> Option<u32> {
    let pid_str = config.get("pid")?;

    // Пробуем как число
    if let Ok(pid) = pid_str.parse::<u32>() {
        return Some(pid);
    }

    // Ищем в переменных
    if let Some(val) = variables.get(pid_str) {
        if let Some(pid) = val.as_u64() {
            return Some(pid as u32);
        }
    }

    None
}

/// Парсит строку селектора, опционально резолвя PID из переменных
fn parse_selector_str_with_vars(s: &str, variables: Option<&serde_json::Value>) -> Option<Selector> {
    if s.is_empty() { return None; }
    if let Some(rest) = s.strip_prefix("classname=") {
        Some(Selector::Classname(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name=") {
        Some(Selector::Name(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("id=") {
        Some(Selector::AutomationId(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name_contains=") {
        Some(Selector::NameContains(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("process_id=") {
        // Пробуем как число
        if let Ok(pid) = rest.parse::<u32>() {
            return Some(Selector::ProcessId(pid));
        }
        // Ищем в переменных
        if let Some(vars) = variables {
            if let Some(val) = vars.get(rest) {
                if let Some(pid) = val.as_u64() {
                    return Some(Selector::ProcessId(pid as u32));
                }
            }
        }
        None
    } else {
        None
    }
}

type ToolConstructor = Box<dyn Fn(Selector, &HashMap<String, String>) -> Result<Box<dyn Tool>>>;

/// Реестр инструментов для динамического вызова по имени.
/// Каждый инструмент получает селектор + карту параметров из сценария.
pub struct ToolRegistry {
    tools: HashMap<String, ToolConstructor>,
}

impl ToolRegistry {
    /// Создаёт новый реестр с зарегистрированными инструментами
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        registry.register("Click".to_string(), Box::new(|selector, config| {
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(ClickTool::new(selector, pid)))
        }));

        registry.register("Type".to_string(), Box::new(|selector, config| {
            let text = config.get("text")
                .cloned()
                .unwrap_or_default();
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(TypeTool::new(selector, text, pid)))
        }));

        registry.register("CloseApp".to_string(), Box::new(|_selector, config| {
            let process_name = config.get("process_name")
                .cloned()
                .unwrap_or_default();
            let process_pid = config.get("process_pid")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
            let force = config.get("force")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            let timeout_ms = config.get("timeout_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(3000);
            Ok(Box::new(CloseTool::new(process_name, process_pid, force, timeout_ms)))
        }));

        registry.register("ExtractText".to_string(), Box::new(|selector, config| {
            let var_name = config.get("var_name")
                .cloned()
                .unwrap_or_else(|| "extracted_text".to_string());
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(ExtractTool::new(selector, var_name, pid)))
        }));

        registry.register("Wait".to_string(), Box::new(|_selector, config| {
            let duration_ms = config.get("duration_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1000);
            Ok(Box::new(WaitTool::new(duration_ms)))
        }));

        registry.register("WaitForElement".to_string(), Box::new(|selector, config| {
            let timeout_ms = config.get("timeout_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(10000);
            let interval_ms = config.get("interval_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(500);
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(WaitForElementTool::new(selector, timeout_ms, interval_ms, pid)))
        }));

        registry.register("DoubleClick".to_string(), Box::new(|selector, config| {
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(DoubleClickTool::new(selector, pid)))
        }));

        registry.register("RightClick".to_string(), Box::new(|selector, config| {
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(RightClickTool::new(selector, pid)))
        }));

        registry.register("KeyPress".to_string(), Box::new(|_selector, config| {
            let keys = config.get("keys")
                .cloned()
                .unwrap_or_default();
            let delay_ms = config.get("delay_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(42);
            Ok(Box::new(KeyPressTool::new(keys, delay_ms)))
        }));

        registry.register("MoveMouse".to_string(), Box::new(|selector, config| {
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(MoveMouseTool::new(selector, pid)))
        }));

        registry.register("DragAndDrop".to_string(), Box::new(|selector, config| {
            let target_str = config.get("target_selector")
                .cloned()
                .unwrap_or_default();
            let delay_ms = config.get("delay_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(500);
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            // source — основной селектор, target — из config
            let target_sel = parse_selector_str(&target_str);
            Ok(Box::new(DragDropTool::new(selector.clone(), target_sel.unwrap_or(selector.clone()), delay_ms, pid)))
        }));

        registry.register("Condition".to_string(), Box::new(|selector, config| {
            let var_name = config.get("var_name")
                .cloned()
                .unwrap_or_else(|| "condition_result".to_string());
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(ConditionTool::new(selector, var_name, pid)))
        }));

        registry.register("Retry".to_string(), Box::new(|selector, config| {
            let max_attempts = config.get("max_attempts")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(3);
            let delay_ms = config.get("delay_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1000);
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            Ok(Box::new(RetryTool::new(selector, max_attempts, delay_ms, pid)))
        }));

        registry.register("ReadFile".to_string(), Box::new(|_selector, config| {
            let file_path = config.get("file_path")
                .cloned()
                .unwrap_or_default();
            let var_name = config.get("var_name")
                .cloned()
                .unwrap_or_else(|| "file_content".to_string());
            Ok(Box::new(ReadFileTool::new(file_path, var_name)))
        }));

        registry.register("WriteFile".to_string(), Box::new(|_selector, config| {
            let file_path = config.get("file_path")
                .cloned()
                .unwrap_or_default();
            let content = config.get("content")
                .cloned()
                .unwrap_or_default();
            let append = config.get("append")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            Ok(Box::new(WriteFileTool::new(file_path, content, append)))
        }));

        registry.register("Screenshot".to_string(), Box::new(|selector, config| {
            let output_path = config.get("output_path")
                .cloned()
                .unwrap_or_else(|| "screenshot.bmp".to_string());
            let pid = config.get("pid").and_then(|v| v.parse::<u32>().ok());
            // Если селектор пустой — скриншот экрана
            let sel = if config.get("selector").map(|s| !s.is_empty()).unwrap_or(false) {
                Some(selector)
            } else {
                None
            };
            Ok(Box::new(ScreenshotTool::new(sel, output_path, pid)))
        }));

        registry
    }

    /// Регистрирует новый инструмент
    pub fn register(
        &mut self,
        name: String,
        constructor: ToolConstructor,
    ) {
        self.tools.insert(name, constructor);
    }

    /// Создаёт инструмент по имени, селектору и параметрам
    pub fn create_tool(
        &self,
        name: &str,
        selector: Selector,
        config: &HashMap<String, String>,
    ) -> Result<Box<dyn Tool>> {
        let constructor = self.tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        constructor(selector, config)
    }

    /// Список зарегистрированных инструментов
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Выполняет инструмент по имени и селектору
    pub fn execute_tool(
        &self,
        name: &str,
        selector: Selector,
        automation: &UIAutomation,
        ctx: &mut ExecutionContext,
    ) -> Result<()> {
        let tool = self.create_tool(name, selector, &HashMap::new())?;
        tool.execute(automation, ctx)
    }

    /// Выполняет инструмент с параметрами
    pub fn execute_tool_with_config(
        &self,
        name: &str,
        selector: Selector,
        config: &HashMap<String, String>,
        automation: &UIAutomation,
        ctx: &mut ExecutionContext,
    ) -> Result<()> {
        let tool = self.create_tool_with_pid(name, selector, config, &ctx.variables)?;
        tool.execute(automation, ctx)
    }

    /// Создаёт инструмент с резолвингом PID из config/variables
    pub fn create_tool_with_pid(
        &self,
        name: &str,
        selector: Selector,
        config: &HashMap<String, String>,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<Box<dyn Tool>> {
        let pid = resolve_pid(config, variables);

        match name {
            "Click" => {
                Ok(Box::new(ClickTool::new(selector, pid)))
            }
            "Type" => {
                let text = config.get("text").cloned().unwrap_or_default();
                Ok(Box::new(TypeTool::new(selector, text, pid)))
            }
            "ExtractText" => {
                let var_name = config.get("var_name")
                    .cloned()
                    .unwrap_or_else(|| "extracted_text".to_string());
                Ok(Box::new(ExtractTool::new(selector, var_name, pid)))
            }
            "WaitForElement" => {
                let timeout_ms = config.get("timeout_ms")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(10000);
                let interval_ms = config.get("interval_ms")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(500);
                Ok(Box::new(WaitForElementTool::new(selector, timeout_ms, interval_ms, pid)))
            }
            "DoubleClick" => {
                Ok(Box::new(DoubleClickTool::new(selector, pid)))
            }
            "RightClick" => {
                Ok(Box::new(RightClickTool::new(selector, pid)))
            }
            "MoveMouse" => {
                Ok(Box::new(MoveMouseTool::new(selector, pid)))
            }
            "DragAndDrop" => {
                let target_str = config.get("target_selector")
                    .cloned()
                    .unwrap_or_default();
                let delay_ms = config.get("delay_ms")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(500);
                let target_sel = parse_selector_str_with_vars(&target_str, serde_json::to_value(variables).ok().as_ref());
                Ok(Box::new(DragDropTool::new(
                    selector.clone(),
                    target_sel.unwrap_or(selector.clone()),
                    delay_ms,
                    pid,
                )))
            }
            "Condition" => {
                let var_name = config.get("var_name")
                    .cloned()
                    .unwrap_or_else(|| "condition_result".to_string());
                Ok(Box::new(ConditionTool::new(selector, var_name, pid)))
            }
            "Retry" => {
                let max_attempts = config.get("max_attempts")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(3);
                let delay_ms = config.get("delay_ms")
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(1000);
                Ok(Box::new(RetryTool::new(selector, max_attempts, delay_ms, pid)))
            }
            "Screenshot" => {
                let output_path = config.get("output_path")
                    .cloned()
                    .unwrap_or_else(|| "screenshot.bmp".to_string());
                let sel = if config.get("selector").map(|s| !s.is_empty()).unwrap_or(false) {
                    Some(selector)
                } else {
                    None
                };
                Ok(Box::new(ScreenshotTool::new(sel, output_path, pid)))
            }
            // Инструменты без PID
            _ => {
                let constructor = self.tools
                    .get(name)
                    .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
                constructor(selector, config)
            }
        }
    }
}
