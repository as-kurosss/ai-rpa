use uiautomation::{
    UIAutomation,
    types::ControlType,
    patterns::UIValuePattern
};
use std::process::Command;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Запускаем Блокнот
    Command::new("notepad").spawn()?;
    thread::sleep(Duration::from_secs(2));

    let automation = UIAutomation::new()?;

    // Ищем окно Блокнота
    let window = automation.create_matcher()
        .debug(true)
        .timeout(5000)
        .classname("Notepad")
        .find_first()?;

    println!("Найдено окно: {}", window.get_name()?);

    // Ищем текстовое поле
    let edit = automation.create_matcher()
        .debug(true)
        .from_ref(&window)
        .control_type(ControlType::Edit)
        .timeout(3000)
        .find_first()?;

    edit.click()?;
    edit.send_text("text", 42)?;
    edit.set_focus()?;
    let value_pattern = edit.get_pattern::<UIValuePattern>()?;
    value_pattern.set_value("Сюда его!")?;

    Ok(())
}
