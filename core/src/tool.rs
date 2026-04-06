// tool.rs

use anyhow::Result;
use uiautomation::UIAutomation;
use std::collections::HashMap;

/// Контекст выполнения - общая память между шагами
#[derive(Default)]
pub struct ExecutionContext {
    /// Переменные, которые инструменты могут читать/писать
    pub variables: HashMap<String, serde_json::Value>,

    /// Лог выполнения (для отладки)
    pub log: Vec<String>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет запись в лог, ограничивая размер 500 записями.
    pub fn log(&mut self, message: String) {
        self.log.push(message);
        if self.log.len() > 500 {
            self.log.drain(..100);
        }
    }
}

/// Единый интерфейс для всех инструментов
pub trait Tool {
    /// Уникальное имя инструмента (например, "ckick", "type")
    fn name(&self) -> &str;

    /// Описание для пользователя
    fn description(&self) -> &str;

    /// Основной метод: выполнить действие
    /// 
    /// Принимает:
    /// - `automation`: доступ к UI Automation API
    /// - `ctx`: общая память между шагами
    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()>;
}