// read_file_tool.rs — Чтение файла

use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use serde_json::json;
use std::fs;

/// Резолвит значение с поддержкой кавычек:
/// - `"C:\path\file.txt"` — литерал (кавычки убираются)
/// - `my_path_var` — ищет переменную, если нет — возвращает как есть
fn resolve_path(value: &str, ctx: &ExecutionContext) -> String {
    let trimmed = value.trim();

    // Если в кавычках — литерал
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }

    // Пробуем как переменную
    if let Some(v) = ctx.variables.get(trimmed) {
        if let Some(s) = v.as_str() {
            return s.to_string();
        }
        return v.to_string();
    }

    value.to_string()
}

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
        let resolved_path = resolve_path(&self.file_path, ctx);

        let content = fs::read_to_string(&resolved_path)
            .map_err(|e| anyhow!("Не удалось прочитать '{}': {}", resolved_path, e))?;

        ctx.variables.insert(self.var_name.clone(), json!(content));
        ctx.log(format!("📖 Файл прочитан ({} байт) → {}", content.len(), self.var_name));
        ctx.log(format!("      📝 Путь: {}", resolved_path));

        Ok(())
    }
}
