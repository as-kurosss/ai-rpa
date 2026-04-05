// examples/click_notepad.rs

use anyhow::Result;
use ai_rpa::{
    tool::{ExecutionContext, Tool},
    selector::Selector,
    click_tool::ClickTool
};
use uiautomation::UIAutomation;

fn main() -> Result<()> {
    // 1. Инициализация
    let automation = UIAutomation::new()?;
    let mut ctx = ExecutionContext::new();

    // 2. Создаем инструмент: клик по кнопке "File" в блокноте
    // (предварительно открой блокнот)
    let tool = ClickTool::new(Selector::Name("Файл".to_string()));

    // 3. Выполняем
    println!("Выполняю: {}", tool.description());
    tool.execute(&automation, &mut ctx)?;

    // 4. Выводим лог
    println!("\nЛог выполнения:");
    for entry in &ctx.log {
        println!("- {}", entry);
    }

    Ok(())
}
