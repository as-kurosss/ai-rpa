// examples/record_selector.rs
//
// Пример использования SelectorRecorder с подсветкой элементов в реальном времени:
// - Наведите мышь на любой UI элемент - увидите подсветку рамки
// - Нажмите Ctrl для захвата селектора
// - Программа выведет полное дерево селектора

use ai_rpa::{SelectorRecorder, parse_app_arg, launch_app_and_wait};
use uiautomation::UIAutomation;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::Foundation::{POINT, RECT, COLORREF};

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
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
    use std::sync::Arc;

    // Общие координаты текущего элемента для подсветки
    let hl_x = Arc::new(AtomicI32::new(0));
    let hl_y = Arc::new(AtomicI32::new(0));
    let hl_w = Arc::new(AtomicI32::new(0));
    let hl_h = Arc::new(AtomicI32::new(0));
    let hl_active = Arc::new(AtomicBool::new(false));
    let hl_stop = Arc::new(AtomicBool::new(false));

    // Запускаем поток подсветки
    let hl_x_c = hl_x.clone();
    let hl_y_c = hl_y.clone();
    let hl_w_c = hl_w.clone();
    let hl_h_c = hl_h.clone();
    let hl_active_c = hl_active.clone();
    let hl_stop_c = hl_stop.clone();

    let highlight_thread = thread::spawn(move || {
        let mut prev_x = 0i32;
        let mut prev_y = 0i32;
        let mut prev_w = 0i32;
        let mut prev_h = 0i32;
        let mut is_drawn = false;
        let mut hdc_screen: Option<windows::Win32::Graphics::Gdi::HDC> = None;
        let mut old_pen: Option<windows::Win32::Graphics::Gdi::HGDIOBJ> = None;
        let mut old_brush: Option<windows::Win32::Graphics::Gdi::HGDIOBJ> = None;

        use windows::Win32::Graphics::Gdi::*;

        // Создаём перо один раз
        let pen = unsafe { CreatePen(PS_SOLID, 3, COLORREF(0x00FF00)) };

        loop {
            if hl_stop_c.load(Ordering::Relaxed) {
                break;
            }

            // Всегда читаем актуальные координаты
            let cx = hl_x_c.load(Ordering::Relaxed);
            let cy = hl_y_c.load(Ordering::Relaxed);
            let cw = hl_w_c.load(Ordering::Relaxed);
            let ch = hl_h_c.load(Ordering::Relaxed);
            let active = hl_active_c.load(Ordering::Relaxed);

            if !active {
                // Не активно — стираем если что-то осталось
                if is_drawn && hdc_screen.is_some() {
                    unsafe {
                        let hdc = hdc_screen.unwrap();
                        if let Some(op) = old_pen { let _ = SelectObject(hdc, op); }
                        if let Some(ob) = old_brush { let _ = SelectObject(hdc, ob); }
                        let rect = RECT {
                            left: prev_x - 5, top: prev_y - 5,
                            right: prev_x + prev_w + 5, bottom: prev_y + prev_h + 5,
                        };
                        let _ = InvalidateRect(None, Some(&rect), false);
                        let _ = ReleaseDC(None, hdc);
                    }
                    hdc_screen = None;
                    old_pen = None;
                    old_brush = None;
                    is_drawn = false;
                }
                thread::sleep(Duration::from_millis(30));
                continue;
            }

            // Защита от нулевых/маленьких размеров — минимум 10px
            let cw = if cw < 10 { 10 } else { cw };
            let ch = if ch < 10 { 10 } else { ch };

            // Получаем позицию курсора
            let mut point = POINT { x: 0, y: 0 };
            let cursor_inside = unsafe {
                GetCursorPos(&mut point).is_ok() &&
                point.x >= cx && point.x <= (cx + cw) &&
                point.y >= cy && point.y <= (cy + ch)
            };

            if cursor_inside {
                // Курсор на элементе — рисуем/обновляем рамку
                if !is_drawn || cx != prev_x || cy != prev_y || cw != prev_w || ch != prev_h {
                    // Стираем старую рамку
                    if is_drawn && hdc_screen.is_some() {
                        unsafe {
                            let hdc = hdc_screen.unwrap();
                            if let Some(op) = old_pen { let _ = SelectObject(hdc, op); }
                            if let Some(ob) = old_brush { let _ = SelectObject(hdc, ob); }
                            let rect = RECT {
                                left: prev_x - 5, top: prev_y - 5,
                                right: prev_x + prev_w + 5, bottom: prev_y + prev_h + 5,
                            };
                            let _ = InvalidateRect(None, Some(&rect), false);
                            let _ = ReleaseDC(None, hdc);
                        }
                    }

                    // Рисуем новую
                    unsafe {
                        let hdc = GetDC(None);
                        if !hdc.is_invalid() {
                            old_pen = Some(SelectObject(hdc, pen.into()));
                            old_brush = Some(SelectObject(hdc, GetStockObject(NULL_BRUSH).into()));
                            let _ = Rectangle(hdc, cx - 2, cy - 2, cx + cw + 2, cy + ch + 2);
                            hdc_screen = Some(hdc);
                        }
                    }

                    prev_x = cx; prev_y = cy; prev_w = cw; prev_h = ch;
                    is_drawn = true;
                }
            } else {
                // Курсор ушёл — стираем
                if is_drawn && hdc_screen.is_some() {
                    unsafe {
                        let hdc = hdc_screen.unwrap();
                        if let Some(op) = old_pen { let _ = SelectObject(hdc, op); }
                        if let Some(ob) = old_brush { let _ = SelectObject(hdc, ob); }
                        let rect = RECT {
                            left: prev_x - 5, top: prev_y - 5,
                            right: prev_x + prev_w + 5, bottom: prev_y + prev_h + 5,
                        };
                        let _ = InvalidateRect(None, Some(&rect), false);
                        let _ = ReleaseDC(None, hdc);
                    }
                    hdc_screen = None;
                    old_pen = None;
                    old_brush = None;
                    is_drawn = false;
                }
            }

            thread::sleep(Duration::from_millis(30));
        }

        // Чистим перо
        unsafe { let _ = DeleteObject(pen.into()); }
    });

    // Основной цикл — только клавиши и обновление координат
    let mut ctrl_was_pressed = false;

    println!("📍 Режим записи активен. Наведите мышь на элемент...");

    loop {
        // Проверяем нажатие Esc для выхода
        if is_key_pressed(windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE) {
            println!("\n👋 Выход по запросу пользователя");
            hl_stop.store(true, Ordering::Relaxed);
            hl_active.store(false, Ordering::Relaxed);
            let _ = highlight_thread.join();
            break;
        }

        // Проверяем нажатие Ctrl для захвата
        let ctrl_pressed = is_key_pressed(windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL);

        if ctrl_pressed && !ctrl_was_pressed {
            println!("\n⏳ Захват элемента...");
            hl_active.store(false, Ordering::Relaxed); // отключаем подсветку на время захвата
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

        // Получаем элемент под курсором и обновляем координаты для подсветки
        if let Some(element) = get_element_under_cursor(recorder) {
            if let Ok(rect) = element.get_bounding_rectangle() {
                hl_x.store(rect.get_left(), Ordering::Relaxed);
                hl_y.store(rect.get_top(), Ordering::Relaxed);
                hl_w.store(rect.get_width(), Ordering::Relaxed);
                hl_h.store(rect.get_height(), Ordering::Relaxed);
                hl_active.store(true, Ordering::Relaxed);
            }
        }
        // Если элемент не получен — не меняем hl_active, чтобы рамка не мерцала
        // Поток подсветки сам проверит позицию курсора и решит, стирать или нет

        // Короткая пауза
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn capture_and_display(recorder: &SelectorRecorder) -> anyhow::Result<()> {
    // Получаем элемент по ТЕКУЩЕЙ позиции курсора (в момент нажатия Ctrl)
    let element = match get_element_under_cursor(recorder) {
        Some(el) => el,
        None => {
            eprintln!("❌ Не удалось получить элемент под курсором");
            return Ok(());
        }
    };

    match recorder.capture_element(&element) {
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
