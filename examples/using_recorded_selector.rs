// examples/using_recorded_selector.rs
//
// Примеры использования записанных селекторов в реальном коде
// для разработчиков и AI-агентов

use ai_rpa::{
    SelectorRecorder, ClickTool, TypeTool,
    Selector, ExecutionContext
};
use uiautomation::UIAutomation;
use anyhow::Result;

fn main() -> Result<()> {
    println!("📋 Примеры использования записанных селекторов");
    println!("══════════════════════════════════════════════");
    println!();

    // ==========================================
    // СПОСОБ 1: Ручное создание из консоли
    // ==========================================
    println!("📝 СПОСОБ 1: Ручное создание селектора");
    println!("───────────────────────────────────────");
    println!();
    println!("После записи вы видите в консоли:");
    println!("  🎯 Финальный селектор: Classname(\"Notepad\")");
    println!();
    println!("Копируете в код:");
    
    example_manual_selector()?;

    // ==========================================
    // СПОСОБ 2: Автоматическая запись и использование
    // ==========================================
    println!();
    println!("🤖 СПОСОБ 2: Автоматическая запись через SelectorRecorder");
    println!("─────────────────────────────────────────────────────────");
    println!();
    println!("Записываем селектор и сразу используем...");
    
    example_auto_record()?;

    // ==========================================
    // СПОСОБ 3: Сохранение в файл (JSON)
    // ==========================================
    println!();
    println!("💾 СПОСОБ 3: Сохранение в JSON файл");
    println!("────────────────────────────────────");
    println!();
    println!("Селекторы можно сохранять и загружать из файлов...");
    
    example_json_selector()?;

    println!();
    println!("✅ Все примеры завершены!");
    Ok(())
}

// ==========================================
// СПОСОБ 1: Ручное создание
// ==========================================

fn example_manual_selector() -> Result<()> {
    println!("  Создать селектор вручную по данным из консоли:");
    println!();

    // После записи вы видите:
    //   classname: Some("Notepad")
    //   control_type: Some(Window)
    //   name: Some("Безымянный - Блокнот")
    
    // Копируете нужный параметр:
    let selector = Selector::Classname("Notepad".to_string());
    
    println!("  ✅ Создан селектор: {:?}", selector);
    println!("  Теперь используйте с инструментами:");
    println!("     ClickTool::new(selector)");
    println!("     TypeTool::new(selector, \"текст\")");
    println!();
    
    Ok(())
}

// ==========================================
// СПОСОБ 2: Автоматическая запись
// ==========================================

fn example_auto_record() -> Result<()> {
    let automation = UIAutomation::new()?;
    let recorder = SelectorRecorder::new(automation.clone());

    // Захватываем элемент (нужно навести мышь)
    println!("  ⏳ Наведите мышь на элемент для записи...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    match recorder.capture_element_under_cursor() {
        Ok(recorded) => {
            println!("  ✅ Записано {} шагов", recorded.steps.len());
            
            // Получаем готовый селектор
            if let Some(selector) = recorded.to_selector() {
                println!("  ✅ Готовый селектор: {:?}", selector);
                println!();
                println!("  Использование:");
                println!("     let tool = ClickTool::new(selector);");
                println!("     tool.execute(&automation, &mut ctx)?;");
            }
        }
        Err(e) => {
            println!("  ⚠️  Не удалось записать: {}", e);
        }
    }

    Ok(())
}

// ==========================================
// СПОСОБ 3: JSON сохранение
// ==========================================

fn example_json_selector() -> Result<()> {
    use std::fs;

    // Пример структуры для сохранения
    let selector_data = serde_json::json!({
        "type": "Classname",
        "value": "Notepad",
        "description": "Главное окно Блокнота",
        "recorded_at": "2026-04-04T10:30:00Z",
        "full_tree": [
            {"classname": "Notepad", "type": "Window"},
            {"classname": "Edit", "type": "Edit"}
        ]
    });

    // Сохраняем
    let json_str = serde_json::to_string_pretty(&selector_data)?;
    
    println!("  Пример JSON:");
    println!("  {}", json_str.replace("\n", "\n  "));
    println!();
    println!("  Загрузка из JSON:");
    println!("     let data: Value = serde_json::from_str(&json)?;");
    println!("     let selector = Selector::Classname(data[\"value\"].as_str().unwrap().to_string());");
    
    // В будущем можно сделать:
    // fs::write("selectors/notepad.json", json_str)?;
    // let selector = load_selector("selectors/notepad.json")?;

    Ok(())
}
