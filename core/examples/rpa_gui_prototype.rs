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

// ─── Соединение между блоками ─────────────────────────────────

#[derive(Clone, Debug)]
struct Connection {
    from_id: u64,
    to_id: u64,
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
    connections: Vec<Connection>,
    next_block_id: u64,
    selected_block_id: Option<u64>,
    dragging_block_id: Option<u64>,
    drag_offset: egui::Vec2,
    canvas_offset: egui::Vec2,
    execution_log: Vec<String>,
    is_running: bool,
    search_query: String,
    log_receiver: mpsc::Receiver<String>,
    /// Блок, который перетаскивают из палитры на canvas
    drag_payload_block_type: Option<BlockType>,
    /// Позиция указателя при перетаскивании из палитры (для floating preview)
    drag_pointer_pos: egui::Pos2,
    /// Создание соединения: блок, из которого тянем
    connecting_from: Option<u64>,
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
            connections: vec![
                Connection { from_id: 1, to_id: 2 },
                Connection { from_id: 2, to_id: 3 },
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
            drag_payload_block_type: None,
            drag_pointer_pos: egui::Pos2::ZERO,
            connecting_from: None,
        }
    }

    /// Топологический обход по графу соединений.
    /// Если есть соединения — следует по ним, иначе — по Y-порядку.
    fn order_blocks_by_connections(&self) -> Vec<FlowBlock> {
        if self.connections.is_empty() {
            // Нет соединений — старый способ (по Y)
            let mut sorted = self.blocks.clone();
            sorted.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
            return sorted;
        }

        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Нахожу начальные блоки (без входящих соединений)
        let has_incoming: std::collections::HashSet<u64> = self.connections.iter().map(|c| c.to_id).collect();
        let mut start_ids: Vec<u64> = self.blocks.iter()
            .filter(|b| !has_incoming.contains(&b.id))
            .map(|b| b.id)
            .collect();
        // Если нет начальных (цикл) — беру блок с минимальным Y
        if start_ids.is_empty() {
            let mut by_y = self.blocks.clone();
            by_y.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
            if let Some(first) = by_y.first() {
                start_ids.push(first.id);
            }
        }
        // Сортирую стартовые по Y
        start_ids.sort_by_key(|id| {
            self.blocks.iter().find(|b| b.id == *id).map(|b| b.position.y as i32).unwrap_or(0)
        });

        for start_id in start_ids {
            self._traverse(start_id, &mut result, &mut visited);
        }

        // Добавляю недостижимые блоки (по Y)
        let mut unreachable: Vec<_> = self.blocks.iter()
            .filter(|b| !visited.contains(&b.id))
            .cloned()
            .collect();
        unreachable.sort_by(|a, b| a.position.y.partial_cmp(&b.position.y).unwrap());
        result.extend(unreachable);

        result
    }

    fn _traverse(&self, id: u64, result: &mut Vec<FlowBlock>, visited: &mut std::collections::HashSet<u64>) {
        if visited.contains(&id) {
            return;
        }
        visited.insert(id);
        if let Some(block) = self.blocks.iter().find(|b| b.id == id) {
            result.push(block.clone());
        }
        // Рекурсия по исходящим соединениям
        let outgoing: Vec<u64> = self.connections.iter()
            .filter(|c| c.from_id == id)
            .map(|c| c.to_id)
            .collect();
        for next_id in outgoing {
            self._traverse(next_id, result, visited);
        }
    }

    fn execute_scenario(&mut self) {
        self.is_running = true;
        self.execution_log.clear();
        self.execution_log.push("▶ Запуск сценария...".to_string());

        // Определяю порядок выполнения через соединения (топологический обход)
        let blocks_data: Vec<(String, HashMap<String, String>)> = self.order_blocks_by_connections()
            .into_iter()
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

            for (i, (type_name, config)) in blocks_data.iter().enumerate() {
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
                                let mut type_config = std::collections::HashMap::new();
                                type_config.insert("text".to_string(), text.to_string());
                                match registry.execute_tool_with_config("Type", sel, &type_config, &automation, &mut ctx) {
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
                        // Кастомный widget с Sense::drag для поддержки перетаскивания
                        let desired_size = egui::vec2(180.0, 30.0);
                        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

                        // Фон
                        let is_hovered = response.hovered();
                        let is_dragging = response.is_pointer_button_down_on();
                        let fill = if is_dragging {
                            egui::Color32::from_rgb(220, 220, 220)
                        } else if is_hovered {
                            egui::Color32::from_rgb(230, 230, 230)
                        } else {
                            egui::Color32::from_rgb(240, 240, 240)
                        };
                        let stroke_color = if is_hovered {
                            egui::Color32::from_rgb(160, 160, 160)
                        } else {
                            egui::Color32::from_rgb(200, 200, 200)
                        };
                        ui.painter_at(rect).rect_filled(rect, 4.0, fill);
                        ui.painter_at(rect).rect_stroke(rect, 4.0, egui::Stroke::new(1.0, stroke_color), egui::StrokeKind::Outside);

                        // Текст
                        let label = format!("{}  {}", block_type.icon(), block_type.name());
                        ui.painter_at(rect).text(
                            rect.left_center() + egui::vec2(8.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            label,
                            egui::FontId::new(12.0, egui::FontFamily::Proportional),
                            egui::Color32::from_rgb(40, 40, 40),
                        );

                        // Начало перетаскивания — запоминаем блок и позицию мыши
                        if response.drag_started() {
                            self.drag_payload_block_type = Some(block_type.clone());
                            let pointer = ctx.input(|i| i.pointer.latest_pos()).unwrap_or(egui::Pos2::ZERO);
                            self.drag_pointer_pos = pointer;
                        }

                        // Курсор
                        if is_hovered {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
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

                // Блоки в порядке добавления (не сортировка по Y — порядок теперь через соединения)
                let blocks = self.blocks.clone();

                // ─── Соединения ────────────────────────────────
                let block_rect_fn = |block: &FlowBlock| -> egui::Rect {
                    egui::Rect::from_min_size(
                        egui::Pos2::new(
                            rect.min.x + block.position.x + self.canvas_offset.x,
                            rect.min.y + block.position.y + self.canvas_offset.y,
                        ),
                        egui::Vec2::new(220.0, 70.0),
                    )
                };

                let find_block = |id: u64| -> Option<FlowBlock> {
                    self.blocks.iter().find(|b| b.id == id).cloned()
                };

                // Рисую существующие соединения
                for conn in &self.connections {
                    if let (Some(from_block), Some(to_block)) = (find_block(conn.from_id), find_block(conn.to_id)) {
                        let from_rect = block_rect_fn(&from_block);
                        let to_rect = block_rect_fn(&to_block);

                        let start = egui::Pos2::new(from_rect.center().x, from_rect.max.y); // output
                        let end = egui::Pos2::new(to_rect.center().x, to_rect.min.y); // input

                        let dx = (end.x - start.x).abs();
                        let dy = (end.y - start.y).abs();
                        let cp_offset = (dy.max(dx * 0.5)).min(80.0);

                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            [
                                start,
                                egui::Pos2::new(start.x, start.y + cp_offset),
                                egui::Pos2::new(end.x, end.y - cp_offset),
                                end,
                            ],
                            false,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(2.5, egui::Color32::from_rgb(90, 140, 200)),
                        ));

                        // Стрелка
                        let arrow_size = 7.0;
                        let arrow_tip = end;
                        let dir = (end - egui::Pos2::new(end.x, end.y - cp_offset)).normalized();
                        let perp = egui::vec2(-dir.y, dir.x);
                        let arrow_left = arrow_tip - dir * arrow_size - perp * (arrow_size * 0.6);
                        let arrow_right = arrow_tip - dir * arrow_size + perp * (arrow_size * 0.6);
                        painter.add(egui::epaint::Shape::convex_polygon(
                            vec![arrow_tip, arrow_left, arrow_right],
                            egui::Color32::from_rgb(90, 140, 200),
                            egui::Stroke::NONE,
                        ));
                    }
                }

                // Временная линия при создании соединения
                if self.connecting_from.is_some() {
                    if let Some(from_block) = find_block(self.connecting_from.unwrap()) {
                        let from_rect = block_rect_fn(&from_block);
                        let start = egui::Pos2::new(from_rect.center().x, from_rect.max.y);
                        let pointer = ctx.input(|i| i.pointer.latest_pos()).unwrap_or(start);
                        let in_canvas = rect.contains(pointer);

                        if in_canvas {
                            let end = pointer;
                            let dx = (end.x - start.x).abs();
                            let dy = (end.y - start.y).abs();
                            let cp_offset = (dy.max(dx * 0.5)).min(80.0);

                            painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                                [
                                    start,
                                    egui::Pos2::new(start.x, start.y + cp_offset),
                                    egui::Pos2::new(end.x, end.y - cp_offset),
                                    end,
                                ],
                                false,
                                egui::Color32::TRANSPARENT,
                                egui::Stroke::new(2.5, egui::Color32::from_rgba_unmultiplied(90, 140, 200, 150)),
                            ));

                            // Подсветка целевого input dot
                            for block in &blocks {
                                let br = block_rect_fn(block);
                                let input_dot = egui::Pos2::new(br.center().x, br.min.y);
                                if input_dot.distance(pointer) < 20.0 {
                                    painter.circle_filled(input_dot, 12.0, egui::Color32::from_rgba_unmultiplied(90, 140, 200, 80));
                                    painter.circle_stroke(input_dot, 12.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(90, 140, 200)));
                                }
                            }
                        }
                    }
                }

                // ─── Блоки ─────────────────────────────────────
                // Собираю информацию о hover на input dots для подсветки
                let mut hovered_input_dot: Option<u64> = None;
                let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
                if let Some(pp) = pointer_pos {
                    for block in &blocks {
                        let br = block_rect_fn(block);
                        let input_dot = egui::Pos2::new(br.center().x, br.min.y);
                        if input_dot.distance(pp) < 15.0 {
                            hovered_input_dot = Some(block.id);
                        }
                    }
                }

                for block in &blocks {
                    let block_rect = block_rect_fn(block);

                    let accent = block.block_type.accent_color();
                    let is_selected = self.selected_block_id == Some(block.id);
                    let is_dragging = self.dragging_block_id == Some(block.id);
                    let is_connecting_target = hovered_input_dot == Some(block.id);

                    // Тень при выделении / перетаскивании
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

                    // Подсветка если цель соединения
                    if is_connecting_target {
                        painter.rect_filled(block_rect.expand(2.0), 8.0, egui::Color32::from_rgba_unmultiplied(90, 140, 200, 30));
                    }

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

                    // ─── Точки соединения ──────────────────────
                    let dot_radius = 5.0;
                    let in_dot = egui::Pos2::new(block_rect.center().x, block_rect.min.y);
                    let out_dot = egui::Pos2::new(block_rect.center().x, block_rect.max.y);

                    // Input dot — подсвечивается при наведении
                    let in_dot_highlight = is_connecting_target;
                    let in_dot_r = if in_dot_highlight { dot_radius + 4.0 } else { dot_radius };
                    painter.circle_filled(in_dot, in_dot_r, egui::Color32::from_rgb(90, 140, 200));
                    painter.circle_stroke(in_dot, in_dot_r + 1.0, egui::Stroke::new(1.5, egui::Color32::WHITE));

                    // Output dot — зелёная для drag
                    let out_dot_highlight = self.connecting_from == Some(block.id);
                    let out_dot_r = if out_dot_highlight { dot_radius + 3.0 } else { dot_radius };
                    let out_color = if out_dot_highlight {
                        egui::Color32::from_rgb(60, 180, 75)
                    } else {
                        egui::Color32::from_rgb(90, 140, 200)
                    };
                    painter.circle_filled(out_dot, out_dot_r, out_color);
                    painter.circle_stroke(out_dot, out_dot_r + 1.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
                }

                // ─── Взаимодействие ────────────────────────────
                let response = ui.interact(
                    rect,
                    ui.id().with("canvas"),
                    egui::Sense::click_and_drag(),
                );

                // Определяю canvas-пози указателя
                let canvas_pos = response.interact_pointer_pos().map(|pos| egui::Pos2::new(
                    pos.x - rect.min.x - self.canvas_offset.x,
                    pos.y - rect.min.y - self.canvas_offset.y,
                ));

                // 1) Клик — выбор блока или снятие выделения
                if response.clicked() {
                    if let Some(cp) = canvas_pos {
                        let mut found = false;
                        for block in self.blocks.iter().rev() {
                            let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                            if br.contains(cp) {
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

                // 2) Начало перетаскивания блока на canvas
                if response.drag_started() {
                    if let Some(cp) = canvas_pos {
                        // Проверяю, попал ли в output dot (начало соединения)
                        for block in &self.blocks {
                            let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                            let out_dot = egui::Pos2::new(br.center().x, br.max.y);
                            if out_dot.distance(egui::Pos2::new(cp.x, cp.y)) < 15.0 {
                                // Начало создания соединения
                                self.connecting_from = Some(block.id);
                                break;
                            }
                        }

                        // Если не соединение — проверяю попал ли в блок
                        if self.connecting_from.is_none() {
                            for block in &self.blocks {
                                let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                                if br.contains(cp) {
                                    self.dragging_block_id = Some(block.id);
                                    self.drag_offset = egui::vec2(
                                        cp.x - block.position.x,
                                        cp.y - block.position.y,
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }

                // 3) Перетаскивание существующего блока на canvas
                if let Some(dragging_id) = self.dragging_block_id {
                    if let Some(cp) = canvas_pos {
                        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == dragging_id) {
                            // Привязка к сетке
                            let raw_x = cp.x - self.drag_offset.x;
                            let raw_y = cp.y - self.drag_offset.y;
                            block.position.x = (raw_x / 10.0).round() * 10.0;
                            block.position.y = (raw_y / 10.0).round() * 10.0;
                        }
                    }
                }

                // 4) Создание соединения — завершение при отпускании
                if self.connecting_from.is_some() && response.drag_stopped() {
                    if let Some(cp) = canvas_pos {
                        // Проверяю, отпустил ли рядом с input dot какого-то блока
                        for block in &self.blocks {
                            let br = egui::Rect::from_min_size(block.position, egui::Vec2::new(220.0, 70.0));
                            let in_dot = egui::Pos2::new(br.center().x, br.min.y);
                            let from_id = self.connecting_from.unwrap();
                            if in_dot.distance(egui::Pos2::new(cp.x, cp.y)) < 25.0 && block.id != from_id {
                                // Проверяю, нет ли уже такого соединения
                                let exists = self.connections.iter().any(|c| c.from_id == from_id && c.to_id == block.id);
                                if !exists {
                                    self.connections.push(Connection {
                                        from_id,
                                        to_id: block.id,
                                    });
                                    self.execution_log.push(format!("🔗 Соединение: #{} → #{}", from_id, block.id));
                                }
                                break;
                            }
                        }
                    }
                    self.connecting_from = None;
                }

                // 5) Pan canvas — только если не тащим блок и не создаём соединение
                if response.dragged() && self.dragging_block_id.is_none() && self.connecting_from.is_none() {
                    self.canvas_offset += response.drag_delta();
                }

                // 6) Отпускание перетаскивания блока
                if response.drag_stopped() && self.connecting_from.is_none() {
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

        // Обновляю позицию pointer при перетаскивании из палитры (глобально)
        if self.drag_payload_block_type.is_some() {
            let pointer_down = ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
            if pointer_down {
                if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                    self.drag_pointer_pos = pos;
                }
            }
        }

        // Floating preview блока из палитры
        if let Some(ref block_type) = self.drag_payload_block_type {
            let preview_pos = self.drag_pointer_pos;
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drag_preview"),
            ));
            let w = 220.0;
            let h = 70.0;
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(preview_pos.x - w / 2.0, preview_pos.y - h / 2.0),
                egui::Vec2::new(w, h),
            );
            // Тень
            painter.rect_filled(rect.expand(3.0), 6.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50));
            // Фон
            painter.rect_filled(rect, 6.0, egui::Color32::WHITE);
            // Бордер
            painter.rect_stroke(rect, 6.0, egui::Stroke::new(2.0, block_type.accent_color()), egui::StrokeKind::Outside);
            // Полоска
            let strip = egui::Rect::from_min_max(
                egui::Pos2::new(rect.min.x, rect.min.y),
                egui::Pos2::new(rect.min.x + 4.0, rect.max.y),
            );
            painter.rect_filled(strip, 2.0, block_type.accent_color());
            // Текст
            let label = format!("{} {}", block_type.icon(), block_type.name());
            painter.text(
                egui::Pos2::new(rect.min.x + 14.0, rect.min.y + 28.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
                egui::Color32::from_rgb(40, 40, 40),
            );
            // Курсор
            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }

        // Drop блока из палитры — проверяю глобально при отпускании кнопки
        if self.drag_payload_block_type.is_some() {
            let just_released = ctx.input(|i| i.pointer.any_released());
            if just_released {
                if let Some(block_type) = self.drag_payload_block_type.take() {
                    let pointer_pos = ctx.input(|i| i.pointer.latest_pos()).unwrap_or(egui::Pos2::ZERO);
                    // Проверяю, что pointer не над левой или правой панелью
                    let left_panel_width = 220.0;
                    let right_panel_width = 260.0;
                    let screen_width = ctx.input(|i| i.screen_rect().width());
                    let screen_height = ctx.input(|i| i.screen_rect().height());
                    let in_canvas_x = pointer_pos.x > left_panel_width && pointer_pos.x < (screen_width - right_panel_width);
                    let in_canvas_y = pointer_pos.y > 40.0 && pointer_pos.y < screen_height; // top_bar ~40px
                    if in_canvas_x && in_canvas_y {
                        // Добавляю блок в canvas
                        let top_bar_height = 40.0;
                        let canvas_pos = egui::Pos2::new(
                            pointer_pos.x - left_panel_width - self.canvas_offset.x - 110.0,
                            pointer_pos.y - top_bar_height - self.canvas_offset.y - 35.0,
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
                        let name = block_type.name();
                        self.blocks.push(FlowBlock {
                            id: self.next_block_id,
                            block_type,
                            position: canvas_pos,
                            config,
                        });
                        self.next_block_id += 1;
                        self.execution_log.push(format!("+ Добавлен: {}", name));
                    }
                }
            }
        }

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
