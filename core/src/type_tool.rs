// type_tool.rs

use crate::tool::{
    Tool,
    ExecutionContext
};
use crate::selector::Selector;
use anyhow::Result;
use uiautomation::UIAutomation;

/// Инструмент для ввода текста в элемент UI
pub struct TypeTool {
    /// Селектор для поиска элемента
    pub selector: Selector,

    /// Текст для ввода
    pub text: String,

    /// Описание для пользователя (опционально)
    pub description: String,
}

impl TypeTool {
    /// Создает новый инструмент с селектором и текстом
    pub fn new(selector: Selector, text: String) -> Self {
        Self {
            selector,
            text,
            description: "Type text into UI element".to_string(),
        }
    }

    /// Резолвит текст, подставляя переменные из контекста.
    /// - `var` (без кавычек) = переменная из ctx.variables
    /// - `"var"` (в кавычках) =.literal текст "var"
    fn resolve_text_with_variables(&self, ctx: &ExecutionContext) -> String {
        resolve_variables(&self.text, ctx)
    }
}

impl Tool for TypeTool {
    fn name(&self) -> &str {
        "Type"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, automation: &UIAutomation, ctx: &mut ExecutionContext) -> Result<()> {
        // 1. Получаем корневой элемент дерева UI (весь экран)
        let root = automation.get_root_element()?;

        // 2. Ищем целевой элемент по селектору
        let element = self.selector.find(automation, &root)?;

        // 3. Кликаем по элементу для фокуса
        element.click()?;

        // 4. Резолвим текст: подставляем переменные из контекста
        let resolved_text = self.resolve_text_with_variables(ctx);

        // 5. Вводим текст — экранируем фигурные скобки, чтобы `{Enter}`, `{Tab}`
        //    не интерпретировались как спецклавиши SendKeys.
        let escaped = escape_send_keys(&resolved_text);
        element.send_text(&escaped, 42)?;

        // 6. Логируем успешное действие в контекст
        ctx.log(format!("✅ Typed '{}' into element: {:?}", resolved_text, self.selector));

        Ok(())
    }
}

/// Экранирует спецсимволы SendKeys: `{` → `{{}`, `}` → `{}}`, `+` → `{+}`, `^` → `{^}`, `%` → `{%}`
fn escape_send_keys(s: &str) -> String {
    s.replace('{', "{{}")
     .replace('}', "{}}")
     .replace('+', "{+}")
     .replace('^', "{^}")
     .replace('%', "{%}")
}

/// Резолвит текст, подставляя переменные из контекста.
/// 
/// Правила:
/// - `var` (без кавычек) = переменная из ctx.variables
/// - `"var"` (в кавычках) = literal текст "var" (без кавычек)
/// 
/// Примеры:
/// - `Привет name` -> если name="Мир", то "Привет Мир"
/// - `Привет "name"` -> "Привет name"
/// - `Hello world Test123` -> если Hello="Привет", world="Мир", то "Привет Мир Test123"
pub fn resolve_variables(text: &str, ctx: &ExecutionContext) -> String {
    let mut result = String::new();
    let mut in_quotes = false;
    let mut current_word = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // Переключение режима кавычек
                in_quotes = !in_quotes;
                if !in_quotes {
                    // Закрываем кавычку - добавляем текст как есть (без кавычек)
                    // Но не добавляем пробел - просто помечаем, что слово закончилось
                    if !current_word.is_empty() {
                        result.push_str(&current_word);
                        current_word.clear();
                    }
                }
                // Если открываем кавычку - просто пропускаем
            }
            ' ' | '\n' | '\t' => {
                if in_quotes {
                    // Внутри кавычек - добавляем пробел/newline как текст
                    current_word.push(ch);
                } else {
                    // Снаружи кавычек - это разделитель слов
                    if !current_word.is_empty() {
                        // Пробуем найти переменную
                        if let Some(value) = ctx.variables.get(&current_word) {
                            if let Some(s) = value.as_str() {
                                result.push_str(s);
                            } else {
                                result.push_str(&value.to_string());
                            }
                        } else {
                            // Не переменная - добавляем как текст
                            result.push_str(&current_word);
                        }
                        current_word.clear();
                    }
                    // Добавляем разделитель
                    result.push(ch);
                }
            }
            _ => {
                current_word.push(ch);
            }
        }
    }
    
    // Обрабатываем последнее слово
    if !current_word.is_empty() {
        if in_quotes {
            // Не закрытая кавычка - считаем как текст
            result.push_str(&current_word);
        } else if let Some(value) = ctx.variables.get(&current_word) {
            if let Some(s) = value.as_str() {
                result.push_str(s);
            } else {
                result.push_str(&value.to_string());
            }
        } else {
            result.push_str(&current_word);
        }
    }
    
    result
}
