// examples/explore_notepad.rs
// Скрипт для исследования UI-элементов блокнота
// Запуск: cargo run --example explore_notepad

use anyhow::Result;
use uiautomation::UIAutomation;

fn main() -> Result<()> {
    println!("⏳ Запуск через 3 секунды... Откройте Блокнот и нажмите 'Файл'!");
    std::thread::sleep(std::time::Duration::from_secs(3));

    let automation = UIAutomation::new()?;

    println!("🔍 Ищем окно Блокнота...\n");

    // Ищем окно Notepad
    let window = automation.create_matcher()
        .classname("Notepad")
        .find_first();

    let window = match window {
        Ok(w) => w,
        Err(_) => {
            println!("❌ Блокнот не найден! Откройте notepad.exe и попробуйте снова.");
            return Ok(());
        }
    };

    let window_name = window.get_name().unwrap_or("?".into());
    println!("✅ Найдено окно: '{}'\n", window_name);

    // Ищем все элементы типа MenuItem и выводим их имена
    println!("� Ищем элементы меню (MenuItem):\n");
    let menu_items = automation.create_matcher()
        .from_ref(&window)
        .timeout(5000)
        .find_all();

    if let Ok(items) = menu_items {
        for (i, item) in items.iter().enumerate() {
            let name = item.get_name().unwrap_or_default();
            let ct = item.get_control_type().map(|c| format!("{:?}", c)).unwrap_or("?".into());
            let class = item.get_classname().unwrap_or_default();
            println!("  {}. {} | name='{}' | class='{}'", i + 1, ct, name, class);
        }
    }

    // Ищем конкретно элементы содержащие "Сохранить" / "Save"
    println!("\n🔎 Ищем элементы с 'Сохранить'/'Save' в имени:\n");
    let all = automation.create_matcher()
        .from_ref(&window)
        .timeout(2000)
        .find_all();

    if let Ok(items) = all {
        for item in &items {
            if let Ok(name) = item.get_name() {
                if name.to_lowercase().contains("сохран") || name.to_lowercase().contains("save") {
                    let ct = item.get_control_type().map(|c| format!("{:?}", c)).unwrap_or("?".into());
                    let class = item.get_classname().unwrap_or_default();
                    println!("  name='{}' | type={} | class='{}'", name, ct, class);
                }
            }
        }
    }

    println!("\n💡 Совет: откройте меню 'Файл' в блокноте перед запуском, чтобы увидеть пункт 'Сохранить как...'");

    Ok(())
}
