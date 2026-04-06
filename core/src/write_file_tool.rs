// write_file_tool.rs — Запись в файл

use crate::tool::{Tool, ExecutionContext};
use anyhow::{anyhow, Result};
use std::fs;

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
        if self.append {
            fs::write(&self.file_path, &self.content)
                .map_err(|e| anyhow!("Не удалось записать '{}': {}", self.file_path, e))?;
        } else {
            fs::write(&self.file_path, &self.content)
                .map_err(|e| anyhow!("Не удалось записать '{}': {}", self.file_path, e))?;
        }
        ctx.log(format!("📝 Файл записан: {} ({} байт)", self.file_path, self.content.len()));
        Ok(())
    }
}
