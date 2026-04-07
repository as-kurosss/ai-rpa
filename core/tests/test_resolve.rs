use ai_rpa::resolve::resolve_value;
use ai_rpa::tool::ExecutionContext;
use serde_json::json;

#[test]
fn test_literal_in_quotes() {
    let ctx = ExecutionContext::new();
    assert_eq!(resolve_value(r#""C:\file.txt""#, &ctx), r#"C:\file.txt"#);
}

#[test]
fn test_variable_resolution() {
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("my_path".to_string(), json!("C:\\actual\\path.txt"));
    assert_eq!(resolve_value("my_path", &ctx), r#"C:\actual\path.txt"#);
}

#[test]
fn test_variable_not_found() {
    let ctx = ExecutionContext::new();
    assert_eq!(resolve_value("unknown_var", &ctx), "unknown_var");
}

#[test]
fn test_concat_variable_and_literal() {
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("text".to_string(), json!("Hello"));
    assert_eq!(resolve_value(r#"text + " world""#, &ctx), "Hello world");
}

#[test]
fn test_concat_literal_and_variable() {
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("name".to_string(), json!("Мир"));
    assert_eq!(resolve_value(r#""Привет, " + name"#, &ctx), "Привет, Мир");
}

#[test]
fn test_concat_multiple() {
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("a".to_string(), json!("AAA"));
    ctx.variables.insert("b".to_string(), json!("BBB"));
    assert_eq!(
        resolve_value(r#""start " + a + " middle " + b + " end""#, &ctx),
        "start AAA middle BBB end"
    );
}

#[test]
fn test_concat_with_unknown_variable() {
    let ctx = ExecutionContext::new();
    // unknown_var не в variables — возвращается как есть
    assert_eq!(
        resolve_value(r#"unknown_var + " suffix""#, &ctx),
        "unknown_var suffix"
    );
}

#[test]
fn test_empty_string() {
    let ctx = ExecutionContext::new();
    assert_eq!(resolve_value("", &ctx), "");
}

#[test]
fn test_whitespace_handling() {
    let mut ctx = ExecutionContext::new();
    ctx.variables.insert("x".to_string(), json!("VAL"));
    assert_eq!(resolve_value(r#"  x  +  " ok "  "#, &ctx), "VAL ok ");
}
