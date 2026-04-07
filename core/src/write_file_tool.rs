// write_file_tool.rs — Запись в файл

use crate::tool::{Tool, ExecutionContext};
use crate::resolve::resolve_value;
use anyhow::{anyhow, Result};
use std::io::Write;

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
        // Резолвим переменные и конкатенацию
        let resolved_path = resolve_value(&self.file_path, ctx);
        let resolved_content = resolve_value(&self.content, ctx);

        if self.append {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&resolved_path)
                .map_err(|e| anyhow!("Не удалось открыть '{}': {}", resolved_path, e))?;
            file.write_all(resolved_content.as_bytes())
                .map_err(|e| anyhow!("Не удалось записать '{}': {}", resolved_path, e))?;
        } else {
            std::fs::write(&resolved_path, &resolved_content)
                .map_err(|e| anyhow!("Не удалось записать '{}': {}", resolved_path, e))?;
        }

        ctx.log(format!("📝 Файл записан: {} ({} байт){}", resolved_path, resolved_content.len(),
            if self.append { " (дописано)" } else { "" }));
        ctx.log(format!("      📝 Содержимое (первые 100 символов): \"{}\"",
            resolved_content.chars().take(100).collect::<String>()));

        Ok(())
    }
}
