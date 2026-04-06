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

        // 4. Вводим текст — экранируем фигурные скобки, чтобы `{Enter}`, `{Tab}`
        //    не интерпретировались как спецклавиши SendKeys.
        let escaped = escape_send_keys(&self.text);
        element.send_text(&escaped, 42)?;

        // 5. Логируем успешное действие в контекст
        ctx.log(format!("✅ Typed '{}' into element: {:?}", self.text, self.selector));

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
