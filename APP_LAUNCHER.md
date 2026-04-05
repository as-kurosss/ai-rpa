# Запуск приложений с записью селекторов

## Использование

### Базовое использование (без запуска приложения)

```bash
cargo run --example record_selector
```

Работает с уже открытыми окнами.

### С запуском приложения

```bash
# Запуск по имени (ищет в PATH)
cargo run --example record_selector -- notepad

# Запуск по полному пути
cargo run --example record_selector -- "C:\Windows\System32\notepad.exe"

# Приложение с аргументами
cargo run --example record_selector -- "mspaint file.png"

# Другие известные приложения
cargo run --example record_selector -- calc
cargo run --example record_selector -- mspaint
```

## Правила поиска

### 1. Просто имя (notepad, calc)

```
notepad → Ищем в PATH
  ✅ Найдено → Запускаем
  ❌ Не найдено → Ошибка с подсказками
```

**Пример ошибки:**
```
❌ Ошибка: Приложение 'unknown_app' не найдено в PATH

💡 Варианты:
- Укажите полный путь: "C:\Windows\System32\notepad.exe"
- Добавьте приложение в PATH
- Используйте известное имя: "notepad", "calc", "mspaint"
```

### 2. Полный путь (C:\path\to\app.exe)

```
C:\Windows\System32\notepad.exe → Проверяем существование
  ✅ Файл есть → Запускаем
  ❌ Файла нет → Ошибка с путём
```

### 3. Как определить, что указан путь?

| Формат | Считается | Пример |
|--------|-----------|--------|
| `notepad` | Имя (ищем в PATH) | ✅ |
| `calc` | Имя (ищем в PATH) | ✅ |
| `C:\path\app.exe` | Путь (проверяем файл) | ✅ |
| `.\app.exe` | Путь (проверяем файл) | ✅ |
| `app.exe` | Имя (ищем в PATH) | ✅ |

## API для разработчиков

### Поиск исполняемого файла

```rust
use ai_rpa::find_executable;

// Ищем в PATH
let path = find_executable("notepad")?;
// ✅ Ok("C:\Windows\System32\notepad.exe")

// Не найдено
let path = find_executable("unknown_app")?;
// ❌ Err("Приложение 'unknown_app' не найдено в PATH")

// Полный путь
let path = find_executable("C:\\Windows\\System32\\calc.exe")?;
// ✅ Ok("C:\Windows\System32\calc.exe")
```

### Запуск приложения

```rust
use ai_rpa::{find_executable, launch_app, launch_app_and_wait};
use std::path::Path;

// Находим и запускаем
let app_path = find_executable("notepad")?;
let pid = launch_app(&app_path, &[])?;
// 🚀 Запуск приложения: C:\Windows\System32\notepad.exe
// ✅ Приложение запущено (PID: 12345)

// С ожиданием
let pid = launch_app_and_wait(&app_path, &[], 2000)?;
// ⏳ Ожидание загрузки приложения (2000ms)...
```

### Парсинг аргумента приложения

```rust
use ai_rpa::parse_app_arg;

// Просто имя
let (path, args) = parse_app_arg("notepad")?;
// path = "C:\Windows\System32\notepad.exe"
// args = []

// С аргументами
let (path, args) = parse_app_arg("mspaint file.png")?;
// path = "C:\Windows\System32\mspaint.exe"
// args = ["file.png"]

// Полный путь
let (path, args) = parse_app_arg("C:\\MyApp\\app.exe --debug")?;
// path = "C:\MyApp\app.exe"
// args = ["--debug"]
```

## Тестирование

```bash
# Запустить все тесты app_launcher
cargo test app_launcher

# Тесты включают:
# - Поиск notepad в PATH
# - Поиск calc в PATH
# - Проверка полного пути
# - Ошибка при несуществующем пути
# - Ошибка при неизвестном имени
# - Парсинг аргументов
```

## Примеры использования

### Автоматизация Notepad

```bash
# 1. Запускаем Notepad и записываем селекторы
cargo run --example record_selector -- notepad

# 2. Наводим мышь на элементы
# 3. Нажимаем Ctrl для записи
# 4. Копируем селектор в код
```

### Автоматизация калькулятора

```bash
cargo run --example record_selector -- calc
```

### Кастомное приложение

```bash
cargo run --example record_selector -- "C:\MyApp\app.exe"
```
