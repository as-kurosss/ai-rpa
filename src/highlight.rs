// highlight.rs

use anyhow::{anyhow, Result};
use uiautomation::UIElement;
use crate::highlight_overlay::{draw_highlight_rect_async, draw_highlight_rect_animated, draw_highlight_rect_blocking};

/// Конфигурация подсветки
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Длительность подсветки в миллисекундах
    pub duration_ms: u64,
    /// Количество миганий (для анимации)
    pub flashes: u32,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            duration_ms: 2000,
            flashes: 3,
        }
    }
}

/// Подсвечивает UI элемент на заданное время (в отдельном потоке)
pub fn highlight_element(element: &UIElement, config: Option<HighlightConfig>) -> Result<()> {
    let config = config.unwrap_or_default();

    let rect = element.get_bounding_rectangle()
        .map_err(|e| anyhow!("Не удалось получить границы элемента: {}", e))?;

    let left = rect.get_left();
    let top = rect.get_top();
    let width = rect.get_width();
    let height = rect.get_height();

    println!("🔲 Подсветка элемента: x={}, y={}, w={}, h={}", left, top, width, height);

    draw_highlight_rect_async(left, top, width, height, config.duration_ms);

    Ok(())
}

/// Анимированная подсветка с миганием (в отдельном потоке)
pub fn highlight_element_animated(element: &UIElement, flashes: u32) -> Result<()> {
    let rect = element.get_bounding_rectangle()
        .map_err(|e| anyhow!("Не удалось получить границы элемента: {}", e))?;

    let left = rect.get_left();
    let top = rect.get_top();
    let width = rect.get_width();
    let height = rect.get_height();

    println!("✨ Анимированная подсветка: {} миганий", flashes);
    println!("   Границы: x={}, y={}, w={}, h={}", left, top, width, height);

    draw_highlight_rect_animated(left, top, width, height, flashes);

    Ok(())
}

/// Подсвечивает все шаги в дереве селектора (от корня к элементу)
/// Блокирует поток, т.к. это демонстрационная функция
pub fn highlight_selector_tree(automation: &uiautomation::UIAutomation, steps: &[crate::SelectorStep]) -> Result<()> {
    println!("🌳 Подсветка дерева селектора ({} шагов)...", steps.len());

    for (i, step) in steps.iter().enumerate() {
        println!("  Шаг {}/{}", i + 1, steps.len());

        if let Some(selector) = step.to_selector() {
            let root = automation.get_root_element()?;
            if let Ok(element) = selector.find(automation, &root) {
                let rect = element.get_bounding_rectangle().map_err(|e| anyhow!("{}", e))?;
                draw_highlight_rect_blocking(rect.get_left(), rect.get_top(), rect.get_width(), rect.get_height(), 500);
                std::thread::sleep(std::time::Duration::from_millis(300));
            } else {
                println!("    ⚠️  Не удалось найти элемент для шага {}", i);
            }
        }
    }

    println!("✅ Подсветка дерева завершена");
    Ok(())
}
