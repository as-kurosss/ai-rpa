// highlight_overlay.rs

use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// Толщина рамки подсветки в пикселях
const BORDER_THICKNESS: i32 = 3;

/// Цвет рамки (COLORREF: 0x00BBGGRR)
const GREEN_COLOR: COLORREF = COLORREF(0x00FF00);

/// Рисует зелёную рамку поверх элемента прямо на экране
/// Блокирует вызывающий поток на duration_ms миллисекунд
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
        for _i in 0..flashes {
            draw_highlight_rect_blocking(x, y, width, height, 300);
            thread::sleep(Duration::from_millis(200));
        }
    });
}

/// Рисует рамку на элементе и отслеживает курсор мыши.
/// Рамка остаётся, пока курсор находится в пределах элемента.
/// Как только курсор уходит — рамка стирается.
/// Блокирует вызывающий поток до тех пор, пока курсор не покинет элемент.
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

        loop {
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

        // Освобождаем перо
        let _ = DeleteObject(pen.into());
    }
}
