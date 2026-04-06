// tool_registry.rs

use std::collections::HashMap;
use anyhow::Result;
use crate::ClickTool;
use crate::TypeTool;
use crate::CloseTool;
use crate::ExtractTool;
use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use uiautomation::UIAutomation;

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

        registry.register("Click".to_string(), Box::new(|selector, _config| {
            Ok(Box::new(ClickTool::new(selector)))
        }));

        registry.register("Type".to_string(), Box::new(|selector, config| {
            let text = config.get("text")
                .cloned()
                .unwrap_or_default();
            Ok(Box::new(TypeTool::new(selector, text)))
        }));

        registry.register("CloseApp".to_string(), Box::new(|_selector, config| {
            let process_name = config.get("process_name")
                .cloned()
                .unwrap_or_default();
            let force = config.get("force")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            let timeout_ms = config.get("timeout_ms")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(3000);
            Ok(Box::new(CloseTool::new(process_name, force, timeout_ms)))
        }));

        registry.register("ExtractText".to_string(), Box::new(|selector, config| {
            let var_name = config.get("var_name")
                .cloned()
                .unwrap_or_else(|| "extracted_text".to_string());
            Ok(Box::new(ExtractTool::new(selector, var_name)))
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
        let tool = self.create_tool(name, selector, config)?;
        tool.execute(automation, ctx)
    }
}
