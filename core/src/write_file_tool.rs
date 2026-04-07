// write_file_tool.rs — Запись в файл

use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use std::fs;

/// Резолвит значение с поддержкой кавычек:
/// - `var_name` — ищет переменную в контексте, если есть — возвращает значение, иначе — строку как есть
/// - `"literal text"` — возвращает текст без кавычек
fn resolve_var(value: &str, ctx: &ExecutionContext) -> String {
    let trimmed = value.trim();

    // Если в кавычках — это литерал
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed[1..trimmed.len() - 1].to_string();
    }

    // Пробуем найти как переменную
    if let Some(v) = ctx.variables.get(trimmed) {
        if let Some(s) = v.as_str() {
            return s.to_string();
        }
        return v.to_string();
    }

    // Иначе возвращаем как есть
    value.to_string()
}

pub struct WriteFileTool {
    pub file_path: String,
    pub content: String,
    /// true — дописать, false — перезаписать
    pub append: bool,
}

impl WriteFileTool {
    pub fn new(file_path: String, content: String, append: bool) -> Self {
        Self { file_path, content, append }
    }
}

impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "WriteFile"
    }

    fn description(&self) -> &str {
        "Записать текст в файл"
    }

    fn execute(&self, _automation: &uiautomation::UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        // Резолвим переменные
        let resolved_path = resolve_var(&self.file_path, ctx);
        let resolved_content = resolve_var(&self.content, ctx);

        fs::write(&resolved_path, &resolved_content)
            .map_err(|e| anyhow!("Не удалось записать '{}': {}", resolved_path, e))?;

        ctx.log(format!("📝 Файл записан: {} ({} байт)", resolved_path, resolved_content.len()));
        ctx.log(format!("      📝 Содержимое (первые 100 символов): \"{}\"",
            resolved_content.chars().take(100).collect::<String>()));

        Ok(())
    }
}
