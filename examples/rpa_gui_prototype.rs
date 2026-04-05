// rpa_gui_prototype.rs — RPA-платформа с реальным выполнением инструментов

use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use ai_rpa::tool::ExecutionContext;
use ai_rpa::tool_registry::ToolRegistry;
use ai_rpa::selector::Selector;
use ai_rpa::app_launcher::{find_executable, launch_app_and_wait};
use uiautomation::UIAutomation;

// ─── Типы блоков ───────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum BlockType {
    LaunchApp,
    Click,
    TypeText,
}

impl BlockType {
    fn all() -> Vec<Self> {
        vec![
            Self::LaunchApp,
            Self::Click,
            Self::TypeText,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::LaunchApp => "Запуск приложения",
            Self::Click => "Клик",
            Self::TypeText => "Ввод текста",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::LaunchApp => "🚀",
            Self::Click => "🖱",
            Self::TypeText => "⌨",
        }
    }

    fn accent_color(&self) -> egui::Color32 {
        match self {
            Self::LaunchApp => egui::Color32::from_rgb(130, 130, 130),
            Self::Click => egui::Color32::from_rgb(130, 130, 130),
            Self::TypeText => egui::Color32::from_rgb(130, 130, 130),
        }
    }
}

// ─── Блок на canvas ────────────────────────────────────────────

#[derive(Clone, Debug)]
struct FlowBlock {
    id: u64,
    block_type: BlockType,
    position: egui::Pos2,
    config: HashMap<String, String>,
}

// ─── Приложение ────────────────────────────────────────────────

struct RpaApp {
    blocks: Vec<FlowBlock>,
    next_block_id: u64,
    selected_block_id: Option<u64>,
    dragging_block_id: Option<u64>,
    drag_offset: egui::Vec2,
    canvas_offset: egui::Vec2,
    execution_log: Vec<String>,
    is_running: bool,
    search_query: String,
    log_receiver: mpsc::Receiver<String>,
}

impl RpaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Нейтральная тема — минимум цветов
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.visuals.window_fill = egui::Color32::from_rgb(36, 36, 36);
        style.visuals.panel_fill = egui::Color32::from_rgb(36, 36, 36);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(48, 48, 48);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(48, 48, 48);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(64, 64, 64);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 80, 80);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(70, 130, 180);
        style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 56, 56));
        cc.egui_ctx.set_style(style);

        Self {
            blocks: vec![
                FlowBlock {
                    id: 1,
                    block_type: BlockType::LaunchApp,
                    position: egui::Pos2::new(100.0, 80.0),
                    config: HashMap::from([
                        ("app".to_string(), "notepad".to_string()),
                    ]),
                },
                FlowBlock {
                    id: 2,
                    block_type: BlockType::Click,
                    position: egui::Pos2::new(100.0, 220.0),
                    config: HashMap::from([
                        ("selector".to_string(), "classname=Edit".to_string()),
                    ]),
                },
                FlowBlock {
                    id: 3,
                    block_type: BlockType::TypeText,
                    position: egui::Pos2::new(100.0, 360.0),
                    config: HashMap::from([
                        ("selector".to_string(), "classname=Edit".to_string()),
                        ("text".to_string(), "Привет из RPA Studio!".to_string()),
                    ]),
                },
            ],
            next_block_id: 4,
            selected_block_id: None,
            dragging_block_id: None,
            drag_offset: egui::Vec2::ZERO,
            canvas_offset: egui::Vec2::ZERO,
            execution_log: vec![],
            is_running: false,
            search_query: String::new(),
            log_receiver: {
                let (tx, rx) = mpsc::channel();
                // Тестовое сообщение, чтобы канал работал
                let _ = tx;
                rx
            },
        }
    }

    fn add_block(&mut self, block_type: BlockType) {
        let center = egui::Pos2::new(
            200.0 - self.canvas_offset.x,
            200.0 - self.canvas_offset.y,
        );
        let mut config = HashMap::new();
        match &block_type {
            BlockType::LaunchApp => {
                config.insert("app".to_string(), "notepad".to_string());
            }
            BlockType::Click => {
                config.insert("selector".to_string(), "classname=Edit".to_string());
            }
            BlockType::TypeText => {
                config.insert("selector".to_string(), "classname=Edit".to_string());
                config.insert("text".to_string(), "".to_string());
            }
        }
        self.blocks.push(FlowBlock {
            id: self.next_block_id,
            block_type,
            position: center,
            config,
        });
        self.next_block_id += 1;
    }

    fn execute_scenario(&mut self) {
        self.is_running = true;
        self.execution_log.clear();
        self.execution_log.push("▶ Запуск сценария...".to_string());

        // Собираем данные сценария
        let mut sorted_blocks: Vec<_> = self.blocks.clone();
        sorted_blocks.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());

        let scenario: Vec<(String, HashMap<String, String>)> = sorted_blocks
            .iter()
            .map(|b| {
                let type_name = match b.block_type {
                    BlockType::LaunchApp => "LaunchApp".to_string(),
                    BlockType::Click => "Click".to_string(),
                    BlockType::TypeText => "TypeText".to_string(),
                };
                (type_name, b.config.clone())
            })
            .collect();

        let (tx, rx) = mpsc::channel();
        self.log_receiver = rx;

        thread::spawn(move || {
            // В новом потоке COM не инициализирован.
            // UIAutomation::new() сам вызовет CoInitializeEx.
            let automation = match UIAutomation::new() {
                Ok(a) => a,
                Err(e) => {
                    let _ = tx.send(format!("❌ Ошибка UI Automation: {}", e));
                    return;
                }
            };

            let mut ctx = ExecutionContext::new();
            let registry = ToolRegistry::new();

            for (i, (type_name, config)) in scenario.iter().enumerate() {
                let step = i + 1;

                match type_name.as_str() {
                    "LaunchApp" => {
                        let app_name = config.get("app").map(|s| s.as_str()).unwrap_or("notepad");
                        let _ = tx.send(format!("  [{step}] 🚀 Запуск: {}", app_name));

                        match find_executable(app_name) {
                            Ok(app_path) => {
                                match launch_app_and_wait(&app_path, &[], 1000) {
                                    Ok(pid) => {
                                        let _ = tx.send(format!("      ✓ PID: {}", pid));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(format!("      ❌ {}", e));
                                        let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(format!("      ❌ {}", e));
                                let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                return;
                            }
                        }
                    }
                    "Click" => {
                        let selector_str = config.get("selector").map(|s| s.as_str()).unwrap_or("classname=Edit");
                        let _ = tx.send(format!("  [{step}] 🖱 Клик: {}", selector_str));

                        match parse_selector(selector_str) {
                            Ok(sel) => {
                                match registry.execute_tool("Click", sel, &automation, &mut ctx) {
                                    Ok(()) => {
                                        let _ = tx.send("      ✓ Клик выполнен".to_string());
                                    }
                                    Err(e) => {
                                        let _ = tx.send(format!("      ❌ {}", e));
                                        let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(format!("      ❌ {}", e));
                                let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                return;
                            }
                        }
                    }
                    "TypeText" => {
                        let selector_str = config.get("selector").map(|s| s.as_str()).unwrap_or("classname=Edit");
                        let text = config.get("text").map(|s| s.as_str()).unwrap_or("");
                        let _ = tx.send(format!("  [{step}] ⌨ Ввод: \"{}\"", text));

                        match parse_selector(selector_str) {
                            Ok(sel) => {
                                match registry.execute_tool_with_text("Type", sel, text, &automation, &mut ctx) {
                                    Ok(()) => {
                                        let _ = tx.send("      ✓ Ввод выполнен".to_string());
                                    }
                                    Err(e) => {
                                        let _ = tx.send(format!("      ❌ {}", e));
                                        let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(format!("      ❌ {}", e));
                                let _ = tx.send("⏹ Сценарий остановлен из-за ошибки".to_string());
                                return;
                            }
                        }
                    }
                    _ => {}
                }
            }

            let _ = tx.send("✓ Сценарий завершён успешно".to_string());
        });
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    ui.heading(egui::RichText::new("🤖 RPA Studio").size(16.0).strong());

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let run_btn = ui.add_sized(
                        [100.0, 30.0],
                        egui::Button::new("▶ Запуск")
                            .fill(egui::Color32::from_rgb(34, 139, 34))
                            .stroke(egui::Stroke::NONE),
                    );
                    if run_btn.clicked() && !self.is_running {
                        self.execute_scenario();
                    }

                    let stop_btn = ui.add_sized(
                        [90.0, 30.0],
                        egui::Button::new("⏹ Стоп")
                            .fill(egui::Color32::from_rgb(180, 50, 50))
                            .stroke(egui::Stroke::NONE),
                    );
                    if stop_btn.clicked() {
                        self.is_running = false;
                        self.execution_log.push("⏹ Остановлено пользователем".to_string());
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if ui.add_sized([100.0, 30.0], egui::Button::new("💾 Сохранить")).clicked() {
                        self.execution_log.push("💾 Сохранено".to_string());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("Блоков: {}", self.blocks.len())).size(11.0));
                    });
                });
            });
    }

    fn draw_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .exact_width(220.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.heading(egui::RichText::new("🧩 Блоки").size(14.0));
                });

                ui.add_space(8.0);

                // Поиск
                let search_box = egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("🔍 Поиск...")
                    .desired_width(190.0);
                ui.add(search_box);

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                // Список блоков
                ui.scope(|ui| {
                    ui.set_min_width(180.0);
                    let filtered: Vec<_> = BlockType::all()
                        .into_iter()
                        .filter(|b| {
                            self.search_query.is_empty()
                                || b.name().to_lowercase().contains(&self.search_query.to_lowercase())
                        })
                        .collect();

                    for block_type in filtered {
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{}  {}", block_type.icon(), block_type.name()))
                                .color(egui::Color32::from_rgb(40, 40, 40))
                                .size(12.0),
                        )
                        .fill(egui::Color32::from_rgb(240, 240, 240))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200)))
                        .corner_radius(4.0)
                        .min_size(egui::vec2(180.0, 30.0));

                        let response = ui.add(btn);
                        if response.clicked() {
                            self.add_block(block_type.clone());
                            self.execution_log.push(format!("+ Добавлен: {}", block_type.name()));
                        }

                        ui.add_space(4.0);
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(egui::RichText::new("Статистика:").size(10.0).color(egui::Color32::from_rgb(140, 140, 140)));
                ui.label(egui::RichText::new(format!("  Блоков: {}", self.blocks.len())).size(10.0).color(egui::Color32::from_rgb(140, 140, 140)));
            });
    }

    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_panel")
            .exact_width(260.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.heading(egui::RichText::new("⚙ Свойства").size(14.0));
                });

                ui.add_space(10.0);

                if let Some(selected_id) = self.selected_block_id {
                    if let Some(block) = self.blocks.iter_mut().find(|b| b.id == selected_id) {
                        // Карточка блока
                        let accent = block.block_type.accent_color();
                        ui.horizontal(|ui| {
                            ui.colored_label(accent, egui::RichText::new(block.block_type.icon()).size(20.0));
                            ui.vertical(|ui| {
                                ui.colored_label(accent, egui::RichText::new(block.block_type.name()).size(13.0).strong());
                                ui.label(egui::RichText::new(format!("ID: {}", block.id)).size(10.0).color(egui::Color32::from_rgb(140, 140, 140)));
                            });
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Позиция
                        ui.label(egui::RichText::new("Позиция:").size(11.0).color(egui::Color32::from_rgb(140, 140, 140)));
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut block.position.x).speed(1.0).range(0.0..=5000.0));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut block.position.y).speed(1.0).range(0.0..=5000.0));
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Параметры блока
                        ui.label(egui::RichText::new("Параметры:").size(11.0).color(egui::Color32::from_rgb(140, 140, 140)));
                        ui.add_space(4.0);

                        match block.block_type {
                            BlockType::LaunchApp => {
                                let mut app = block.config.get("app").cloned().unwrap_or_default();
                                ui.label(egui::RichText::new("Приложение:").size(10.0));
                                ui.text_edit_singleline(&mut app);
                                block.config.insert("app".to_string(), app);
                            }
                            BlockType::Click => {
                                let mut selector = block.config.get("selector").cloned().unwrap_or_default();
                                ui.label(egui::RichText::new("Селектор:").size(10.0));
                                ui.text_edit_singleline(&mut selector);
                                block.config.insert("selector".to_string(), selector);

                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("Формат: classname=Edit").size(9.0).color(egui::Color32::from_rgb(120, 120, 120)));
                            }
                            BlockType::TypeText => {
                                let mut selector = block.config.get("selector").cloned().unwrap_or_default();
                                ui.label(egui::RichText::new("Селектор:").size(10.0));
                                ui.text_edit_singleline(&mut selector);
                                block.config.insert("selector".to_string(), selector);

                                ui.add_space(6.0);
                                let mut text = block.config.get("text").cloned().unwrap_or_default();
                                ui.label(egui::RichText::new("Текст:").size(10.0));
                                ui.text_edit_multiline(&mut text);
                                block.config.insert("text".to_string(), text);
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            if ui.small_button("🗑 Удалить").clicked() {
                                self.blocks.retain(|b| b.id != selected_id);
                                self.selected_block_id = None;
                            }
                            if ui.small_button("📋 Дублировать").clicked() {
                                if let Some(src) = self.blocks.iter().find(|b| b.id == selected_id) {
                                    self.blocks.push(FlowBlock {
                                        id: self.next_block_id,
                                        block_type: src.block_type.clone(),
                                        position: egui::Pos2::new(src.position.x + 30.0, src.position.y + 30.0),
                                        config: src.config.clone(),
                                    });
                                    self.next_block_id += 1;
                                }
                            }
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(egui::RichText::new("Выберите блок на canvas\nдля редактирования")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 120, 120)));
                    });
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);

                // Лог выполнения
                ui.label(egui::RichText::new("📋 Лог:").size(11.0).color(egui::Color32::from_rgb(140, 140, 140)));
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for log_entry in &self.execution_log {
                            let color = if log_entry.starts_with("❌") || log_entry.starts_with("⏹") {
                                egui::Color32::from_rgb(220, 80, 80)
                            } else if log_entry.starts_with("✓") {
                                egui::Color32::from_rgb(80, 200, 80)
                            } else if log_entry.starts_with("▶") {
                                egui::Color32::from_rgb(100, 180, 255)
                            } else {
                                egui::Color32::from_rgb(180, 180, 180)
                            };
                            ui.label(egui::RichText::new(log_entry).size(10.0).color(color));
                        }
                        if self.execution_log.is_empty() {
                            ui.label(egui::RichText::new("Нет записей").size(10.0).color(egui::Color32::from_rgb(120, 120, 120)));
                        }
                    });
            });
    }

    fn draw_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::WHITE))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                let painter = ui.painter_at(rect);

                // Белый фон
                painter.rect_filled(rect, 0.0, egui::Color32::WHITE);

                // Зелёная клеточка — тетрадь
                let grid_spacing = 20.0;
                let grid_color = egui::Color32::from_rgba_unmultiplied(180, 220, 180, 100);

                let offset_x = self.canvas_offset.x.rem_euclid(grid_spacing);
                let offset_y = self.canvas_offset.y.rem_euclid(grid_spacing);

                for x in (offset_x as i32..rect.width() as i32).step_by(grid_spacing as usize) {
                    painter.line_segment(
                        [
                            egui::Pos2::new(x as f32, rect.min.y),
                            egui::Pos2::new(x as f32, rect.max.y),
                        ],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }
                for y in (offset_y as i32..rect.height() as i32).step_by(grid_spacing as usize) {
                    painter.line_segment(
                        [
                            egui::Pos2::new(rect.min.x, y as f32),
                            egui::Pos2::new(rect.max.x, y as f32),
                        ],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }

                // Сортируем блоки по Y для порядка выполнения
                let mut sorted_blocks: Vec<_> = self.blocks.clone();
                sorted_blocks.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());

                // Рисуем соединения
                for i in 0..sorted_blocks.len().saturating_sub(1) {
                    let current = &sorted_blocks[i];
                    let next = &sorted_blocks[i + 1];

                    let start = egui::Pos2::new(
                        rect.min.x + current.position.x + 110.0 + self.canvas_offset.x,
                        rect.min.y + current.position.y + 70.0 + self.canvas_offset.y,
                    );
                    let end = egui::Pos2::new(
                        rect.min.x + next.position.x + 110.0 + self.canvas_offset.x,
                        rect.min.y + next.position.y + self.canvas_offset.y,
                    );

                    let mid_y = (start.y + end.y) / 2.0;
                    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                        [start, egui::Pos2::new(start.x, mid_y), egui::Pos2::new(end.x, mid_y), end],
                        false,
                        egui::Color32::TRANSPARENT,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                    ));

                    // Стрелка
                    let arrow_size = 6.0;
                    let arrow_tip = end;
                    let arrow_left = egui::Pos2::new(end.x - arrow_size / 2.0, end.y - arrow_size);
                    let arrow_right = egui::Pos2::new(end.x + arrow_size / 2.0, end.y - arrow_size);
                    painter.add(egui::epaint::Shape::convex_polygon(
                        vec![arrow_tip, arrow_left, arrow_right],
                        egui::Color32::from_rgb(100, 100, 100),
                        egui::Stroke::NONE,
                    ));
                }

                // Рисуем блоки
                for block in &sorted_blocks {
                    let block_rect = egui::Rect::from_min_size(
                        egui::Pos2::new(
                            rect.min.x + block.position.x + self.canvas_offset.x,
                            rect.min.y + block.position.y + self.canvas_offset.y,
                        ),
                        egui::Vec2::new(220.0, 70.0),
                    );

                    let accent = block.block_type.accent_color();
                    let is_selected = self.selected_block_id == Some(block.id);
                    let is_dragging = self.dragging_block_id == Some(block.id);

                    // Тень при выделении
                    if is_selected || is_dragging {
                        painter.rect_filled(
                            block_rect.expand(3.0),
                            6.0,
                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
                        );
                    }

                    // Фон блока — белый
                    painter.rect_filled(block_rect, 6.0, egui::Color32::WHITE);

                    // Бордер
                    let border_color = if is_selected { accent } else { egui::Color32::from_rgb(200, 200, 200) };
                    painter.rect_stroke(block_rect, 6.0, egui::Stroke::new(if is_selected { 2.0 } else { 1.0 }, border_color), egui::StrokeKind::Outside);

                    // Цветная полоска слева
                    let strip_rect = egui::Rect::from_min_max(
                        egui::Pos2::new(block_rect.min.x, block_rect.min.y),
                        egui::Pos2::new(block_rect.min.x + 4.0, block_rect.max.y),
                    );
                    painter.rect_filled(strip_rect, 2.0, accent);

                    // Иконка и название
                    let label = format!("{} {}", block.block_type.icon(), block.block_type.name());
                    let galley = painter.layout_no_wrap(
                        label,
                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                        egui::Color32::from_rgb(40, 40, 40),
                    );
                    let text_pos = egui::Pos2::new(block_rect.min.x + 14.0, block_rect.min.y + 20.0);
                    painter.galley(text_pos, galley, egui::Color32::from_rgb(40, 40, 40));

                    // Подсказка с конфигом
                    if !block.config.is_empty() {
                        let hint = block.config.values().next().unwrap();
                        let display = if hint.len() > 35 { format!("{}...", &hint[..35]) } else { hint.clone() };
                        let hint_galley = painter.layout_no_wrap(
                            display,
                            egui::FontId::new(10.0, egui::FontFamily::Proportional),
                            egui::Color32::from_rgb(140, 140, 140),
                        );
                        let hint_pos = egui::Pos2::new(block_rect.min.x + 14.0, block_rect.min.y + 42.0);
                        painter.galley(hint_pos, hint_galley, egui::Color32::from_rgb(140, 140, 140));
                    }

                    // Точки соединения
                    let dot_radius = 5.0;
                    let in_dot = egui::Pos2::new(block_rect.center().x, block_rect.min.y);
                    painter.circle_filled(in_dot, dot_radius, egui::Color32::from_rgb(100, 100, 100));
                    painter.circle_stroke(in_dot, dot_radius + 1.0, egui::Stroke::new(1.5, egui::Color32::WHITE));

                    let out_dot = egui::Pos2::new(block_rect.center().x, block_rect.max.y);
                    painter.circle_filled(out_dot, dot_radius, egui::Color32::from_rgb(100, 100, 100));
                    painter.circle_stroke(out_dot, dot_radius + 1.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
                }

                // Взаимодействие
                let response = ui.interact(
                    rect,
                    ui.id().with("canvas"),
                    egui::Sense::click_and_drag(),
                );

                if response.dragged() {
                    self.canvas_offset += response.drag_delta();
                }

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = egui::Pos2::new(
                            pos.x - rect.min.x - self.canvas_offset.x,
                            pos.y - rect.min.y - self.canvas_offset.y,
                        );
                        let mut found = false;
                        for block in self.blocks.iter().rev() {
                            let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                            if br.contains(canvas_pos) {
                                self.selected_block_id = Some(block.id);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            self.selected_block_id = None;
                        }
                    }
                }

                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = egui::Pos2::new(
                            pos.x - rect.min.x - self.canvas_offset.x,
                            pos.y - rect.min.y - self.canvas_offset.y,
                        );
                        if self.dragging_block_id.is_none() {
                            for block in &self.blocks {
                                let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                                if br.contains(canvas_pos) {
                                    self.dragging_block_id = Some(block.id);
                                    self.drag_offset = egui::vec2(
                                        canvas_pos.x - block.position.x,
                                        canvas_pos.y - block.position.y,
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some(dragging_id) = self.dragging_block_id {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let canvas_pos = egui::Pos2::new(
                            pos.x - rect.min.x - self.canvas_offset.x,
                            pos.y - rect.min.y - self.canvas_offset.y,
                        );
                        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == dragging_id) {
                            block.position.x = canvas_pos.x - self.drag_offset.x;
                            block.position.y = canvas_pos.y - self.drag_offset.y;
                        }
                    }
                }

                if response.drag_stopped() {
                    self.dragging_block_id = None;
                }
            });
    }
}

impl eframe::App for RpaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Читаем логи из потока выполнения
        while let Ok(msg) = self.log_receiver.try_recv() {
            let done = msg.contains("завершён") || msg.contains("остановлен");
            self.execution_log.push(msg);
            if done {
                self.is_running = false;
            }
        }

        self.draw_top_bar(ctx);
        self.draw_left_panel(ctx);
        self.draw_right_panel(ctx);
        self.draw_canvas(ctx);

        ctx.request_repaint();
    }
}

/// Парсит строку селектора в Selector
fn parse_selector(s: &str) -> anyhow::Result<Selector> {
    if let Some(rest) = s.strip_prefix("classname=") {
        Ok(Selector::Classname(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name=") {
        Ok(Selector::Name(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("id=") {
        Ok(Selector::AutomationId(rest.to_string()))
    } else if let Some(rest) = s.strip_prefix("name_contains=") {
        Ok(Selector::NameContains(rest.to_string()))
    } else {
        Err(anyhow::anyhow!("Неизвестный формат селектора: '{}'. Используйте: classname=X, name=X, id=X, name_contains=X", s))
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🤖 RPA Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "🤖 RPA Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(RpaApp::new(cc)))),
    )
}
