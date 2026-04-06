// examples/registry_demo.rs
// Пример: работа с реестром инструментов
// Запуск: cargo run --example registry_demo

use anyhow::Result;
use ai_rpa::{
    tool::ExecutionContext,
    selector::Selector,
    tool_registry::ToolRegistry,
};
use uiautomation::UIAutomation;

fn main() -> Result<()> {
    println!("🚀 Запуск демонстрации реестра инструментов\n");
    
    // 1. Инициализация
    let automation = UIAutomation::new()?;
    let mut context = ExecutionContext::new();
    let registry = ToolRegistry::new();
    
    println!("✅ Реестр инструментов создан");
    println!("   Зарегистрированные инструменты: {:?}", registry.tool_names());
    
    // 2. Подготовка: открываем Блокнот (вручную или через Command)
    println!("\n📝 ПОДГОТОВКА:");
    println!("   1. Открой Блокнот (notepad.exe)");
    println!("   2. Убедись, что окно активно");
    println!("   3. Нажми Enter для продолжения...");
    std::io::stdin().read_line(&mut String::new())?;
    
    // 3. Выполнение шага 1: клик по меню "Файл"
    println!("\n🖱️  ШАГ 1: Клик по меню 'Файл'");
    match registry.execute_tool(
        "Click",
        Selector::Name("Файл".to_string()),
        &automation,
        &mut context,
    ) {
        Ok(_) => println!("   ✅ Клик выполнен"),
        Err(e) => println!("   ❌ Ошибка: {}", e),
    }
    
    // 4. Выполнение шага 2: клик по "Сохранить как"
    println!("\n🖱️  ШАГ 2: Клик по 'Сохранить как'");
    match registry.execute_tool(
        "Click",
        Selector::NameContains("Сохранить как".to_string()),
        &automation,
        &mut context,
    ) {
        Ok(_) => println!("   ✅ Клик выполнен"),
        Err(e) => println!("   ❌ Ошибка: {}", e),
    }

    // 5. Ввод текста в поле "Имя файла"
    println!("\n⌨️  ШАГ 3: Ввод текста в поле 'Имя файла'");
    let mut type_config = std::collections::HashMap::new();
    type_config.insert("text".to_string(), "привет_мир.txt".to_string());
    match registry.execute_tool_with_config(
        "Type",
        Selector::NameContains("Имя".to_string()),
        &type_config,
        &automation,
        &mut context,
    ) {
        Ok(_) => println!("   ✅ Текст введён"),
        Err(e) => println!("   ❌ Ошибка: {}", e),
    }

    // 6. Вывод лога выполнения
    println!("\n📊 ЛОГ ВЫПОЛНЕНИЯ:");
    for (i, entry) in context.log.iter().enumerate() {
        println!("   {}. {}", i + 1, entry);
    }

    // 6. Статистика
    println!("\n📈 СТАТИСТИКА:");
    println!("   Всего шагов: {}", context.log.len());
    println!("   Успешных: {}", context.log.iter().filter(|s| s.contains("✅")).count());
    println!("   Ошибок: {}", context.log.iter().filter(|s| s.contains("❌")).count());

    println!("\n🎉 Демонстрация завершена!");
    println!("   Проверь, открылось ли окно сохранения в Блокноте");

    Ok(())
}