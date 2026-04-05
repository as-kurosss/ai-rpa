// tool_registry.rs

use std::collections::HashMap;
use anyhow::Result;
use crate::ClickTool;
use crate::TypeTool;
use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use uiautomation::UIAutomation;

/// Реестр инструментов для динамического вызова по имени
pub struct ToolRegistry {
    /// Карта: имя инструмента -> конструктор (функция, создающая инструмент)
    tools: HashMap<String, Box<dyn Fn(Selector) -> Box<dyn Tool>>>,
}

impl ToolRegistry {
    /// Создает новый реестр с зарегистрированными инструментами
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Регистрируем ClickTool под именем "Click"
        registry.register("Click".to_string(), Box::new(|selector| {
            Box::new(ClickTool::new(selector))
        }));

        // Регистрируем TypeTool под именем "Type"
        registry.register("Type".to_string(), Box::new(|selector| {
            // TypeTool требует текст — по умолчанию пустой, задаётся через execute_tool_with_text
            Box::new(TypeTool::new(selector, String::new()))
        }));

        registry
    }

    /// Регистрируем новый инструмент в реестре
    pub fn register(
        &mut self, 
        name: String, 
        constructor: Box<dyn Fn(Selector) -> Box<dyn Tool>>,
    ) {
        self.tools.insert(name, constructor);
    }

    /// Создает инструмент по имени и селектору
    pub fn create_tool(
        &self,
        name: &str,
        selector: Selector,
    ) -> Result<Box<dyn Tool>> {
        let constructor = self.tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        Ok(constructor(selector))
    }

    /// Возвращает список зарегистрированных инструментов
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
        let tool = self.create_tool(name, selector)?;
        tool.execute(automation, ctx)
    }

    /// Выполняет инструмент с дополнительным текстом (для TypeTool и подобных)
    pub fn execute_tool_with_text(
        &self,
        name: &str,
        selector: Selector,
        text: &str,
        automation: &UIAutomation,
        ctx: &mut ExecutionContext,
    ) -> Result<()> {
        let constructor = self.tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;

        // Для TypeTool создаём с нужным текстом
        if name == "Type" {
            let tool = TypeTool::new(selector, text.to_string());
            return tool.execute(automation, ctx);
        }

        // Для остальных — стандартный конструктор
        let tool = constructor(selector);
        tool.execute(automation, ctx)
    }
}