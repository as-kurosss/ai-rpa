# Как использовать записанные селекторы

## Краткий ответ

**Записанный селектор** → **Копируете из консоли** → **Вставляете в код** → **Используете с ClickTool/TypeTool**

## Полная инструкция

### Шаг 1: Запись селектора

```bash
cargo run --example record_selector
```

Наводите мышь на элемент, нажимаете **Ctrl**, видите результат:

```
🎯 Финальный селектор для использования:
   Classname("menubar-menu-button")
```

### Шаг 2: Копирование в код

Копируете селектор и создаёте инструмент:

```rust
use ai_rpa::{Selector, ClickTool, TypeTool, ExecutionContext};
use uiautomation::UIAutomation;

fn main() -> anyhow::Result<()> {
    let automation = UIAutomation::new()?;
    let mut ctx = ExecutionContext::new();
    
    // Создаёте селектор из записанного значения
    let selector = Selector::Classname("menubar-menu-button".to_string());
    
    // Используете с ClickTool
    let click_tool = ClickTool::new(selector.clone());
    click_tool.execute(&automation, &mut ctx)?;
    
    // Или с TypeTool
    let type_tool = TypeTool::new(selector, "Привет!".to_string());
    type_tool.execute(&automation, &mut ctx)?;
    
    Ok(())
}
```

### Шаг 3: Реальный пример - автоматизация Notepad

```rust
use ai_rpa::{Selector, ClickTool, TypeTool, ExecutionContext};
use uiautomation::UIAutomation;

fn automate_notepad() -> anyhow::Result<()> {
    let automation = UIAutomation::new()?;
    let mut ctx = ExecutionContext::new();
    
    // Записали селектор окна Notepad
    let window_selector = Selector::Classname("Notepad".to_string());
    
    // Записали селектор поля ввода
    let edit_selector = Selector::Classname("RichEditD2D".to_string());
    
    // Кликаем по окну
    ClickTool::new(window_selector).execute(&automation, &mut ctx)?;
    
    // Печатаем текст
    TypeTool::new(edit_selector, "Автоматизированный текст!".to_string())
        .execute(&automation, &mut ctx)?;
    
    println!("✅ Лог выполнения:");
    for entry in &ctx.log {
        println!("  {}", entry);
    }
    
    Ok(())
}
```

## Для AI-агентов

### Структура записанного селектора

```json
{
  "selector": "Classname(\"menubar-menu-button\")",
  "element_properties": {
    "classname": "menubar-menu-button",
    "control_type": "MenuItem",
    "name": "Help",
    "bounding_rectangle": {"x": 355, "y": 0, "w": 45, "h": 35}
  },
  "tree_depth": 19,
  "full_tree": [
    {"level": 0, "classname": "Chrome_WidgetWin_1", "type": "Window"},
    {"level": 1, "classname": "menubar", "type": "MenuBar"},
    {"level": 2, "classname": "menubar-menu-button", "type": "MenuItem"}
  ]
}
```

### Как агенту использовать

1. **Записать селектор** через `record_selector`
2. **Распарсить вывод** консоли (регулярками или через JSON)
3. **Сгенерировать код** с нужным селектором
4. **Выполнить** с ClickTool/TypeTool

#### Пример для AI-агента

```python
# Агент парсит вывод record_selector
output = """
🎯 Финальный селектор для использования:
   Classname("menubar-menu-button")
"""

# Извлекает селектор
import re
match = re.search(r'Classname\("([^"]+)"\)', output)
classname = match.group(1)  # "menubar-menu-button"

# Генерирует Rust код
code = f'''
let selector = Selector::Classname("{classname}".to_string());
let tool = ClickTool::new(selector);
tool.execute(&automation, &mut ctx)?;
'''
```

## Типы селекторов и приоритеты

После записи `to_selector()` выбирает лучший вариант:

```
1. automation_id (если не пустой) → Name("...")
2. classname (если не пустой)     → Classname("...")
3. control_type (всегда есть)     → ControlType(...)
4. name (если не пустой)          → Name("...")
```

### Какой тип выбрать?

| Тип | Надёность | Зависит от языка? | Пример |
|-----|-----------|-------------------|--------|
| `Classname` | ⭐⭐⭐⭐⭐ | ❌ Нет | `Classname("Notepad")` |
| `ControlType` | ⭐⭐⭐⭐ | ❌ Нет | `ControlType(Button)` |
| `Name` | ⭐⭐ | ✅ Да | `Name("Сохранить")` |

## Сохранение селекторов (в будущем)

Планируется:

```rust
// Сохранение
let selector = Selector::Classname("Notepad".to_string());
selector.save("selectors/notepad_window.json")?;

// Загрузка
let selector = Selector::load("selectors/notepad_window.json")?;

// Библиотека селекторов
let selectors = SelectorLibrary::load("selectors/")?;
let notepad = selectors.get("notepad_window")?;
```

## Быстрая шпаргалка

```
1. cargo run --example record_selector  → Записать селектор
2. Скопировать "Classname(\"...\")"     → Из консоли
3. Selector::Classname("...".to_string()) → Вставить в код
4. ClickTool::new(selector)             → Создать инструмент
5. tool.execute(&automation, &mut ctx)? → Выполнить
```
