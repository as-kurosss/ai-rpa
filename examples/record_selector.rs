// examples/record_selector.rs
//
// Пример использования SelectorRecorder с подсветкой элементов в реальном времени:
// - Наведите мышь на любой UI элемент - увидите подсветку рамки
// - Нажмите Ctrl для захвата селектора
// - Программа выведет полное дерево селектора

use ai_rpa::{SelectorRecorder, HighlightConfig, highlight_element, highlight_element_animated};
use ai_rpa::{parse_app_arg, launch_app_and_wait};
use ai_rpa::highlight_overlay::draw_highlight_rect_blocking;
use uiautomation::UIAutomation;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::Foundation::POINT;

fn main() -> anyhow::Result<()> {
    println!("🎯 Запись селектора с подсветкой (аналог UIPath/SheRPA)");
    println!("════════════════════════════════════════════════════════");
    println!();

    let args: Vec<String> = std::env::args().collect();

    let _app_pid = if args.len() > 1 {
        let app_spec = &args[1];
        println!("📦 Приложение для запуска: {}", app_spec);
        println!();
        launch_specified_app(app_spec)?
    } else {
        println!("⚠️  Приложение не указано");
        println!("   Для запуска приложения укажите его в аргументах:");
        println!("     cargo run --example record_selector -- notepad");
        println!("     cargo run --example record_selector -- \"C:\\path\\to\\app.exe\"");
        println!();
        None
    };

    println!("Инструкция:");
    println!("1. Наведите мышь на нужный UI элемент");
    println!("2. При наведении элемент будет подсвечиваться зелёной рамкой");
    println!("3. Нажмите Ctrl для захвата селектора");
    println!("4. Программа выведет полное дерево селектора");
    println!();
    println!("⚠️  Убедитесь, что целевое приложение видно на экране!");
    println!();
    println!("🔴 Для выхода нажмите Esc");
    println!();

    print!("⏳ Запуск режима записи...");
    io::stdout().flush()?;

    let automation = UIAutomation::new()?;
    let recorder = SelectorRecorder::new(automation);

    println!("✅ Готово!\n");

    run_selector_recorder_loop(&recorder)?;

    println!("\n✨ Готово!");
    Ok(())
}

fn launch_specified_app(app_spec: &str) -> anyhow::Result<Option<u32>> {
    let (app_path, app_args) = parse_app_arg(app_spec).map_err(|e| {
        anyhow::anyhow!(
            "❌ Ошибка: {}\n\n\
             💡 Проверьте:\n\
             - Имя приложения правильное (например: notepad, calc)\n\
             - Полный путь существует (например: C:\\Windows\\System32\\notepad.exe)",
            e
        )
    })?;

    println!("🔍 Найдено: {}", app_path.display());

    let args_ref: Vec<&str> = app_args.iter().map(|s| s.as_str()).collect();
    let pid = launch_app_and_wait(&app_path, &args_ref, 2000)?;

    Ok(Some(pid))
}

/// Основной цикл записи селектора с отслеживанием мыши
fn run_selector_recorder_loop(recorder: &SelectorRecorder) -> anyhow::Result<()> {
    let mut last_element_hash = 0u64;
    let mut ctrl_was_pressed = false;

    println!("📍 Режим записи активен. Наведите мышь на элемент...");

    loop {
        // Проверяем нажатие Esc для выхода
        if is_key_pressed(windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE) {
            println!("\n👋 Выход по запросу пользователя");
            break;
        }

        // Проверяем нажатие Ctrl для захвата
        let ctrl_pressed = is_key_pressed(windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL);

        if ctrl_pressed && !ctrl_was_pressed {
            println!("\n⏳ Захват элемента...");
            thread::sleep(Duration::from_millis(100));

            match capture_and_display(recorder) {
                Ok(_) => {
                    println!("\n📍 Продолжайте наведение или нажмите Esc для выхода...");
                }
                Err(e) => {
                    eprintln!("\n❌ Ошибка захвата: {}", e);
                }
            }
        }
        ctrl_was_pressed = ctrl_pressed;

        // Получаем элемент под курсором
        if let Some(element) = get_element_under_cursor(recorder) {
            let element_hash = compute_element_hash(&element);

            if element_hash != last_element_hash {
                // Элемент изменился — подсвечиваем новый
                last_element_hash = element_hash;

                if let Ok(rect) = element.get_bounding_rectangle() {
                    let left = rect.get_left();
                    let top = rect.get_top();
                    let width = rect.get_width();
                    let height = rect.get_height();

                    // Рисуем рамку
                    draw_highlight_rect_blocking(left, top, width, height, 150);
                }
            }
        }

        // Короткая пауза
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn capture_and_display(recorder: &SelectorRecorder) -> anyhow::Result<()> {
    let element = match get_element_under_cursor(recorder) {
        Some(el) => el,
        None => {
            eprintln!("❌ Не удалось получить элемент под курсором");
            return Ok(());
        }
    };

    // Анимированная подсветка (в отдельном потоке)
    println!("\n✨ Подсветка элемента...");
    highlight_element_animated(&element, 3)?;

    // Ждём завершения анимации
    thread::sleep(Duration::from_millis(1600));

    match recorder.capture_element_under_cursor() {
        Ok(recorded) => {
            println!("\n✅ Селектор успешно записан!\n");

            recorded.print_tree();

            println!("\n📊 Статистика:");
            println!("   Глубина элемента: {}", recorded.depth);
            println!("   Количество шагов: {}", recorded.steps.len());

            let props = recorder.get_element_properties(&element)?;
            println!();
            props.print();

            if let Some(selector) = recorded.to_selector() {
                println!("\n🎯 Финальный селектор для использования:");
                println!("   {:?}", selector);
            }

            // Финальная подсветка (в отдельном потоке)
            println!("\n🔲 Финальная подсветка записанного элемента (3 сек)...");
            let config = HighlightConfig {
                duration_ms: 3000,
                ..Default::default()
            };
            highlight_element(&element, Some(config))?;
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Ошибка захвата селектора: {}", e));
        }
    }

    Ok(())
}

fn is_key_pressed(vk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY) -> bool {
    unsafe {
        let state = GetAsyncKeyState(vk.0 as i32);
        (state & 0x8000u16 as i16) != 0
    }
}

fn get_element_under_cursor(recorder: &SelectorRecorder) -> Option<uiautomation::UIElement> {
    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut point).is_ok() {
            let automation_point = uiautomation::types::Point::new(point.x, point.y);
            if let Ok(element) = recorder.automation.element_from_point(automation_point) {
                return Some(element);
            }
        }
    }
    None
}

fn compute_element_hash(element: &uiautomation::UIElement) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    if let Ok(name) = element.get_name() {
        name.hash(&mut hasher);
    }
    if let Ok(classname) = element.get_classname() {
        classname.hash(&mut hasher);
    }
    if let Ok(rect) = element.get_bounding_rectangle() {
        rect.get_left().hash(&mut hasher);
        rect.get_top().hash(&mut hasher);
        rect.get_width().hash(&mut hasher);
        rect.get_height().hash(&mut hasher);
    }

    hasher.finish()
}
