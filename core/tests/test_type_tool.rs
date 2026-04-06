// tests/test_type_tool.rs

use ai_rpa::type_tool::TypeTool;
use ai_rpa::selector::Selector;
use ai_rpa::tool::ExecutionContext;
use serde_json::json;

#[test]
fn test_resolve_variables_simple() {
    // Тест: простые переменные без кавычек
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("name".to_string(), json!("Мир"));
    ctx.variables.insert("greeting".to_string(), json!("Привет"));
    
    let selector = Selector::Classname("Edit".to_string());
    let text = "greeting name";
    let _tool = TypeTool::new(selector, text.to_string());
    
    // Проверяем через вспомогательную функцию
    let result = ai_rpa::type_tool::resolve_variables(text, &ctx);
    assert_eq!(result, "Привет Мир");
}

#[test]
fn test_resolve_variables_with_quotes() {
    // Тест: текст в кавычках не резолвится
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("name".to_string(), json!("Мир"));
    
    let text = "Hello \"name\" world";
    let result = ai_rpa::type_tool::resolve_variables(text, &ctx);
    assert_eq!(result, "Hello name world");
}

#[test]
fn test_resolve_variables_mixed() {
    // Тест: смесь переменных и текста
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("var1".to_string(), json!("VALUE1"));
    ctx.variables.insert("var2".to_string(), json!("VALUE2"));
    
    let text = "var1 text \"var2\" var2";
    let result = ai_rpa::type_tool::resolve_variables(text, &ctx);
    assert_eq!(result, "VALUE1 text var2 VALUE2");
}

#[test]
fn test_resolve_variables_unknown_vars() {
    // Тест: неизвестные переменные остаются как текст
    let ctx = ExecutionContext::new();
    
    let text = "hello world test";
    let result = ai_rpa::type_tool::resolve_variables(text, &ctx);
    assert_eq!(result, "hello world test");
}

#[test]
fn test_resolve_variables_with_newlines() {
    // Тест: многострочный текст
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("line1".to_string(), json!("Первая строка"));
    
    let text = "line1\n\"quoted text\"";
    let result = ai_rpa::type_tool::resolve_variables(text, &ctx);
    assert_eq!(result, "Первая строка\nquoted text");
}
