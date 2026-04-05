// highlight_overlay.rs

use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;

/// Толщина рамки подсветки в пикселях
const BORDER_THICKNESS: i32 = 5;

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
