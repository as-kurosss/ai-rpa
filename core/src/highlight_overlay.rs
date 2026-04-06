// highlight_overlay.rs

use std::sync::Once;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::{GetDpiForSystem, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// Толщина рамки подсветки в пикселях
const BORDER_THICKNESS: i32 = 3;

/// Цвет рамки (COLORREF: 0x00BBGGRR)
const GREEN_COLOR: COLORREF = COLORREF(0x00FF00);

/// Базовый DPI (100% масштаб)
const BASE_DPI: f32 = 96.0;

/// Однократная инициализация DPI awareness
static DPI_INIT: Once = Once::new();

/// Делает процесс DPI-aware (вызывается автоматически при первом обращении)
pub fn ensure_dpi_aware() {
    DPI_INIT.call_once(|| {
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
    });
}

/// Получает текущий DPI экрана и возвращает масштаб
/// Например: 120 DPI -> 1.25, 96 DPI -> 1.0
/// Автоматически делает процесс DPI-aware при первом вызове
pub fn get_dpi_scale() -> f32 {
    ensure_dpi_aware();
    let dpi = unsafe { GetDpiForSystem() };
    if dpi == 0 {
        return 1.0;
    }
    dpi as f32 / BASE_DPI
}

/// Масштабирует координаты из UI Automation (logical pixels) в physical pixels для GDI
pub fn scale_rect(x: i32, y: i32, w: i32, h: i32) -> (i32, i32, i32, i32) {
    let scale = get_dpi_scale();
    (
        (x as f32 * scale) as i32,
        (y as f32 * scale) as i32,
        (w as f32 * scale) as i32,
        (h as f32 * scale) as i32,
    )
}

/// Рисует зелёную рамку поверх элемента прямо на экране
/// Блокирует вызывающий поток на duration_ms миллисекунд
/// Координаты принимаются в physical pixels (экранное разрешение).
/// Если процесс DPI-aware (вызван ensure_dpi_aware), координаты от UI Automation
/// уже в physical pixels — передавайте напрямую.
pub fn draw_highlight_rect_blocking(x: i32, y: i32, width: i32, height: i32, duration_ms: u64) {
    unsafe {
        // Получаем DC всего экрана (None = desktop)
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return;
        }

        // Создаём зелёное перо
        let green_color = COLORREF(0x00FF00);
        let pen = CreatePen(PS_SOLID, BORDER_THICKNESS, green_color);
        let old_pen = SelectObject(hdc_screen, pen.into());

        // Прозрачная кисть — не заполняем центр
        let old_brush = SelectObject(hdc_screen, GetStockObject(NULL_BRUSH).into());

        // Рисуем прямоугольник
        let left = x - (BORDER_THICKNESS / 2);
        let top = y - (BORDER_THICKNESS / 2);
        let right = x + width + (BORDER_THICKNESS / 2);
        let bottom = y + height + (BORDER_THICKNESS / 2);

        let _ = Rectangle(hdc_screen, left, top, right, bottom);

        // Восстанавливаем
        SelectObject(hdc_screen, old_pen);
        SelectObject(hdc_screen, old_brush);

        // Ждём
        thread::sleep(Duration::from_millis(duration_ms));

        // Перерисовываем область, чтобы стереть рамку
        let rect = RECT {
            left: left - 2,
            top: top - 2,
            right: right + 2,
            bottom: bottom + 2,
        };

        // Инвалидируем регион — система перерисует
        let _ = InvalidateRect(None, Some(&rect), false);

        // Освобождаем ресурсы
        let _ = DeleteObject(pen.into());
        ReleaseDC(None, hdc_screen);
    }
}

/// Рисует зелёную рамку в отдельном потоке (не блокирует вызывающий поток)
pub fn draw_highlight_rect_async(x: i32, y: i32, width: i32, height: i32, duration_ms: u64) {
    thread::spawn(move || {
        draw_highlight_rect_blocking(x, y, width, height, duration_ms);
    });
}

/// Рисует анимированную рамку с миганием (в отдельном потоке)
pub fn draw_highlight_rect_animated(x: i32, y: i32, width: i32, height: i32, flashes: u32) {
    thread::spawn(move || {
        for i in 0..flashes {
            draw_highlight_rect_blocking(x, y, width, height, 300);

            // Стираем рамку перед следующей вспышкой
            let left = x - (BORDER_THICKNESS / 2) - 2;
            let top = y - (BORDER_THICKNESS / 2) - 2;
            let right = x + width + (BORDER_THICKNESS / 2) + 2;
            let bottom = y + height + (BORDER_THICKNESS / 2) + 2;

            let rect = RECT {
                left,
                top,
                right,
                bottom,
            };

            unsafe {
                let _ = InvalidateRect(None, Some(&rect), false);
            }

            // Не ждём после последней вспышки
            if i < flashes - 1 {
                thread::sleep(Duration::from_millis(200));
            }
        }
    });
}

/// Рисует рамку на элементе и отслеживает курсор мыши.
/// Рамка остаётся, пока курсор находится в пределах элемента.
/// Как только курсор уходит — рамка стирается.
/// Блокирует вызывающий поток до тех пор, пока курсор не покинет элемент.
/// Имеет таймаут 10 секунд для предотвращения бесконечного цикла.
/// Координаты принимаются в physical pixels.
pub fn draw_highlight_rect_track_cursor(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        // Создаём перо один раз
        let pen = CreatePen(PS_SOLID, BORDER_THICKNESS, GREEN_COLOR);
        if pen.is_invalid() {
            return;
        }

        let mut is_drawn = false;
        let mut last_hdc = None;
        let mut last_old_pen = None;
        let mut last_old_brush = None;

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);

        loop {
            // Таймаут для предотвращения бесконечного цикла
            if start.elapsed() > timeout {
                break;
            }

            // Получаем текущую позицию курсора
            let mut point = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut point).is_err() {
                // Не удалось получить позицию — выходим
                break;
            }

            let cursor_x = point.x;
            let cursor_y = point.y;

            // Проверяем, находится ли курсор в пределах элемента
            let is_inside = cursor_x >= x && cursor_x <= (x + width) &&
                            cursor_y >= y && cursor_y <= (y + height);

            if is_inside {
                // Курсор на элементе — рисуем рамку, если ещё не нарисована
                if !is_drawn {
                    let hdc_screen = GetDC(None);
                    if !hdc_screen.is_invalid() {
                        let old_pen = SelectObject(hdc_screen, pen.into());
                        let old_brush = SelectObject(hdc_screen, GetStockObject(NULL_BRUSH).into());

                        let left = x - (BORDER_THICKNESS / 2);
                        let top = y - (BORDER_THICKNESS / 2);
                        let right = x + width + (BORDER_THICKNESS / 2);
                        let bottom = y + height + (BORDER_THICKNESS / 2);

                        let _ = Rectangle(hdc_screen, left, top, right, bottom);

                        // Сохраняем для последующей очистки
                        last_hdc = Some(hdc_screen);
                        last_old_pen = Some(old_pen);
                        last_old_brush = Some(old_brush);

                        is_drawn = true;
                    }
                }
            } else {
                // Курсор ушёл с элемента — стираем рамку и выходим
                if is_drawn {
                    if let Some(hdc_screen) = last_hdc {
                        if let Some(old_pen) = last_old_pen {
                            SelectObject(hdc_screen, old_pen);
                        }
                        if let Some(old_brush) = last_old_brush {
                            SelectObject(hdc_screen, old_brush);
                        }

                        // Инвалидируем область для перерисовки
                        let left = x - (BORDER_THICKNESS / 2) - 2;
                        let top = y - (BORDER_THICKNESS / 2) - 2;
                        let right = x + width + (BORDER_THICKNESS / 2) + 2;
                        let bottom = y + height + (BORDER_THICKNESS / 2) + 2;

                        let rect = RECT {
                            left,
                            top,
                            right,
                            bottom,
                        };

                        let _ = InvalidateRect(None, Some(&rect), false);
                        ReleaseDC(None, hdc_screen);
                    }

                    // Выходим из функции
                    break;
                }
            }

            // Короткая пауза перед следующей проверкой
            thread::sleep(Duration::from_millis(30));
        }

        // Освобождаем перо в любом случае
        let _ = DeleteObject(pen.into());
    }
}
