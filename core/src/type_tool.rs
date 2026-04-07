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

    /// PID процесса для ограничения поиска (None = весь экран)
    pub process_pid: Option<u32>,

    /// Текст для ввода
    pub text: String,

    /// Описание для пользователя (опционально)
    pub description: String,
}

impl TypeTool {
    /// Создает новый инструмент с селектором и текстом
    pub fn new(selector: Selector, text: String, process_pid: Option<u32>) -> Self {
        Self {
            selector,
            process_pid,
            text,
            description: "Type text into UI element".to_string(),
        }
    }

    /// Резолвит текст: подставляет переменные, обрабатывает кавычки и конкатенацию.
    fn resolve_text_with_variables(&self, ctx: &ExecutionContext) -> String {
        crate::resolve::resolve_value(&self.text, ctx)
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

        // 2. Ищем целевой элемент по селектору с учётом PID процесса
        let element = self.selector.find_with_pid(automation, &root, self.process_pid)?;

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
