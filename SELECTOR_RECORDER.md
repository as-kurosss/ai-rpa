# Selector Recorder & Highlight

Инструменты для записи и визуализации UI селекторов (аналог UIPath/SheRPA "Record Selector").

## 📋 Возможности

### SelectorRecorder

**Запись полного дерева селектора** от Application до целевого элемента:

```rust
use ai_rpa::{SelectorRecorder, RecordedSelector};
use uiautomation::UIAutomation;

let automation = UIAutomation::new()?;
let recorder = SelectorRecorder::new(automation);

// Захват элемента под курсором
let recorded: RecordedSelector = recorder.capture_element_under_cursor()?;

// Печать дерева
recorded.print_tree();

// Получение финального селектора
if let Some(selector) = recorded.to_selector() {
    println!("Селектор: {:?}", selector);
}
```

**Получение свойств элемента:**

```rust
let element = /* ... */;
let props = recorder.get_element_properties(&element)?;
props.print();
// Выводит: classname, control_type, name, automation_id, bounding_rectangle, ...
```

### Highlight

**Визуальная подсветка элементов** для лучшего понимания что было записано:

```rust
use ai_rpa::{highlight_element, highlight_element_animated, HighlightConfig};

// Простая подсветка
highlight_element(&element, None)?;

// Анимированная подсветка с миганием
highlight_element_animated(&element, 3)?; // 3 мигания

// С конфигурацией
let config = HighlightConfig {
    duration_ms: 3000,  // 3 секунды
    flashes: 5,         // 5 миганий
};
highlight_element(&element, Some(config))?;
```

**Подсветка дерева селектора:**

```rust
use ai_rpa::highlight_selector_tree;

// Подсвечивает каждый шаг в дереве от Application до элемента
highlight_selector_tree(&automation, &recorded.steps)?;
```

## 🚀 Примеры

### Запись селектора с подсветкой

```bash
cargo run --example record_selector
```

**Инструкция:**
1. Откройте приложение (например, Notepad)
2. Запустите пример
3. Наведите мышь на элемент
4. Нажмите Enter
5. Программа:
   - Подсветит элемент (мигание)
   - Захватит полное дерево селектора
   - Выведет все свойства элемента
   - Покажет финальный селектор

## 📦 Структура данных

### RecordedSelector

```rust
pub struct RecordedSelector {
    /// Дерево шагов от Application до целевого элемента
    pub steps: Vec<SelectorStep>,
    /// Глубина элемента (количество шагов от корня)
    pub depth: usize,
}
```

### SelectorStep

```rust
pub struct SelectorStep {
    pub classname: Option<String>,          // Имя класса (Button, Edit, ...)
    pub control_type: Option<ControlType>,  // Тип элемента
    pub name: Option<String>,               // Заголовок/текст
    pub automation_id: Option<String>,      // Автоматизированный ID
}
```

### ElementProperties

```rust
pub struct ElementProperties {
    pub classname: Option<String>,
    pub control_type: Option<ControlType>,
    pub name: Option<String>,
    pub automation_id: Option<String>,
    pub localized_control_type: Option<String>,
    pub bounding_rectangle: Option<Rect>,
    pub is_enabled: Option<bool>,
    pub is_keyboard_focusable: Option<bool>,
    pub has_keyboard_focus: Option<bool>,
    pub help_text: Option<String>,
}
```

### HighlightConfig

```rust
pub struct HighlightConfig {
    pub duration_ms: u64,   // Длительность подсветки
    pub flashes: u32,       // Количество миганий
}
```

## 🎯 Workflow записи селектора

```
1. Пользователь наводит мышь на элемент
2. SelectorRecorder.get_element_under_cursor()
   ├─ GetCursorPos() → (x, y)
   └─ element_from_point(Point::new(x, y)) → UIElement
3. SelectorRecorder.build_full_selector_tree()
   ├─ create_tree_walker() → UITreeWalker
   └─ Цикл: walker.get_parent() до корня
4. SelectorRecorder.get_element_properties()
   └─ Извлечение всех атрибутов элемента
5. highlight_element_animated()
   └─ Мигание через set_focus() + send_keys("{ESC}")
6. Вывод RecordedSelector.print_tree()
```

## 🔧 API

### SelectorRecorder

| Метод | Описание |
|-------|----------|
| `new(automation)` | Создать рекордер |
| `capture_element_under_cursor()` | Захватить элемент под курсором |
| `get_element_properties(&element)` | Получить все свойства элемента |
| `build_full_selector_tree(&element)` | Построить дерево селектора |

### Highlight

| Функция | Описание |
|---------|----------|
| `highlight_element(&element, config)` | Подсветить элемент |
| `highlight_element_animated(&element, flashes)` | Анимированная подсветка |
| `highlight_selector_tree(&automation, steps)` | Подсветка всего дерева |
| `highlight_at_position(x, y, w, h)` | Подсветка по координатам |

## 💡 Советы

1. **Для надёжных селекторов** используйте `automation_id` или `classname`
2. **Избегайте `name`** - зависит от локализации
3. **Проверяйте глубину** - слишком глубокие селекторы хрупкие
4. **Используйте подсветку** для верификации что захвачен правильный элемент

## 📝 Пример вывода

```
🎯 Запись селектора с подсветкой
════════════════════════════════════

✨ Анимированная подсветка: 3 миганий
   Границы: x=8, y=8, w=884, h=462
  🟢 Мигание 1/3 ✅
  🟢 Мигание 2/3 ✅
  🟢 Мигание 3/3 ✅

✅ Селектор успешно записан!

📋 Записанный селектор (5 шагов):
[0] classname="Notepad", type=Window, name="Безымянный - Блокнот"
  [1] classname="DirectUIHWND", type=Pane
    [2] classname="Floater", type=Pane
      [3] classname="RichEditD2D", type=Edit
        [4] classname="RichEditD2D", type=Edit, name="Текстовый редактор"

📊 Статистика:
   Глубина элемента: 5
   Количество шагов: 5

🔍 Свойства элемента:
  classname:            Some("RichEditD2D")
  control_type:         Some(Edit)
  name:                 Some("Текстовый редактор")
  bounding_rectangle:   x=8, y=8, w=884, h=462
  is_enabled:           Some(true)
  is_keyboard_focusable: Some(true)
```
