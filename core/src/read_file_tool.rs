// read_file_tool.rs — Чтение файла

use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::fs;

pub struct ReadFileTool {
    pub file_path: String,
    /// Имя переменной для сохранения содержимого
    pub var_name: String,
}

impl ReadFileTool {
    pub fn new(file_path: String, var_name: String) -> Self {
        Self { file_path, var_name }
    }
}

impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "ReadFile"
    }

    fn description(&self) -> &str {
        "Прочитать файл и сохранить содержимое в переменную"
    }

    fn execute(&self, _automation: &uiautomation::UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| anyhow!("Не удалось прочитать '{}': {}", self.file_path, e))?;
        ctx.variables.insert(self.var_name.clone(), json!(content));
        ctx.log(format!("📖 Файл прочитан ({} байт) → ${}", content.len(), self.var_name));
        Ok(())
    }
}
