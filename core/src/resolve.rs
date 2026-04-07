/// Единый модуль резолвинга значений с поддержкой переменных и конкатенации.
///
/// Правила:
/// - `"literal text"` — текст в кавычках (кавычки убираются)
/// - `var_name` — ищет переменную в ctx.variables, если нет — возвращает как есть
/// - `var + " текст " + var2` — конкатенация: разбивает по `+`, каждый токен резолвит

use crate::tool::ExecutionContext;

/// Резолвит значение с поддержкой переменных и конкатенации.
///
/// Примеры:
/// - `"C:\\file.txt"` → `C:\file.txt`
/// - `my_var` → значение переменной my_var
/// - `my_var + " проверка "` → значение my_var + " проверка "
/// - `"start " + my_var + " end"` → `start <value> end`
pub fn resolve_value(value: &str, ctx: &ExecutionContext) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return value.to_string();
    }

    // Если содержит `+` вне кавычек — это конкатенация
    if contains_plus_outside_quotes(trimmed) {
        return resolve_concat(trimmed, ctx);
    }

    // Простое значение: кавычки или переменная
    resolve_token(trimmed, ctx)
}

/// Резолвит один токен (без `+`).
fn resolve_token(token: &str, ctx: &ExecutionContext) -> String {
    let trimmed = token.trim();

    // Если в кавычках — literal
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return trimmed.trim_matches('"').to_string();
    }

    // Ищем переменную
    if let Some(v) = ctx.variables.get(trimmed) {
        if let Some(s) = v.as_str() {
            return s.to_string();
        }
        return v.to_string();
    }

    // Не переменная — возвращаем как есть
    trimmed.to_string()
}

/// Резолвит конкатенацию: разбивает по `+`, резолвит каждый токен, склеивает.
fn resolve_concat(expr: &str, ctx: &ExecutionContext) -> String {
    let tokens = split_by_plus(expr);
    tokens
        .iter()
        .map(|t| resolve_token(t, ctx))
        .collect::<Vec<_>>()
        .concat()
}

/// Проверяет, есть ли `+` вне кавычек.
fn contains_plus_outside_quotes(s: &str) -> bool {
    let mut in_quotes = false;
    for ch in s.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '+' if !in_quotes => return true,
            _ => {}
        }
    }
    false
}

/// Разбивает выражение по `+`, игнорируя `+` внутри кавычек.
fn split_by_plus(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in expr.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '+' if !in_quotes => {
                let token = current.trim().to_string();
                if !token.is_empty() {
                    tokens.push(token);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let token = current.trim().to_string();
    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}
