use eframe::egui;
use std::path::Path;
use crate::ui::theme::ACCENT;
const DIR_COLOR: egui::Color32 = egui::Color32::from_rgb(240, 190, 90);

pub struct FileExplorer {
    pub current_dir: String,
    entries: Vec<FileEntry>,
    selected: Option<usize>,
    pub on_file_open: Option<String>,
}

#[derive(Clone)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    icon: String,
    size: String,
}

impl FileExplorer {
    pub fn new() -> Self {
        let mut explorer = Self {
            current_dir: String::from("."),
            entries: Vec::new(),
            selected: None,
            on_file_open: None,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let icon = if is_dir { "📁" } else { self.file_icon(&name) };
                let size = if !is_dir {
                    if let Ok(meta) = entry.metadata() {
                        self.format_size(meta.len())
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                self.entries.push(FileEntry {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir,
                    icon: icon.to_string(),
                    size,
                });
            }
        }
        self.entries.sort_by(|a, b| {
            b.is_dir.cmp(&a.is_dir)
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }

    fn file_icon(&self, name: &str) -> &str {
        match name.split('.').last().unwrap_or("") {
            "rs" => "🦀",
            "py" => "🐍",
            "js" => "🟨",
            "ts" => "🟦",
            "html" => "🌐",
            "css" => "🎨",
            "json" => "📋",
            "toml" => "⚙️",
            "md" => "📘",
            "c" | "h" => "🔵",
            "cpp" | "hpp" => "🔵",
            "java" => "☕",
            "go" => "🔷",
            "php" => "🐘",
            "sh" | "bash" => "🐚",
            "sql" => "🗄️",
            _ => "📄",
        }
    }

    fn format_size(&self, bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> &mut Self {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("EXPLORER")
                    .size(11.0)
                    .strong()
                    .color(egui::Color32::from_rgb(150, 150, 165)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("🔄").clicked() {
                    self.refresh();
                }
                if ui.small_button("⬆").clicked() {
                    if let Some(parent) = Path::new(&self.current_dir).parent() {
                        self.current_dir = parent.to_string_lossy().to_string();
                        self.refresh();
                    }
                }
            });
        });

        // breadcrumb path
        let crumbs: Vec<&str> = self.current_dir.split('/').filter(|c| !c.is_empty()).collect();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(
                    if crumbs.is_empty() { "/".to_string() } else { format!("/ {}", crumbs.join(" / ")) },
                )
                .size(10.0)
                .color(egui::Color32::from_rgb(110, 112, 130)),
            );
        });
        ui.add_space(2.0);
        ui.separator();

        let theme_ok = true;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, entry) in self.entries.clone().iter().enumerate() {
                let selected = self.selected == Some(i);
                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 22.0),
                    egui::Sense::click(),
                );
                let hovered = resp.hovered();

                // row background
                if selected {
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(5.0),
                        egui::Color32::from_rgb(45, 44, 60),
                    );
                    // accent left bar
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.left(), rect.top() + 2.0),
                            egui::vec2(3.0, rect.height() - 4.0),
                        ),
                        egui::Rounding::same(2.0),
                        ACCENT,
                    );
                } else if hovered {
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(5.0),
                        egui::Color32::from_rgb(38, 38, 50),
                    );
                }

                let icon_color = if entry.is_dir { DIR_COLOR } else { file_color(&entry.name) };
                // colored type dot
                ui.painter().circle_filled(
                    egui::pos2(rect.left() + 9.0, rect.center().y),
                    3.5,
                    icon_color,
                );
                // name
                let name_x = rect.left() + 18.0;
                ui.painter().text(
                    egui::pos2(name_x, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &entry.name,
                    egui::FontId::proportional(12.5),
                    if selected || hovered {
                        egui::Color32::from_rgb(235, 235, 245)
                    } else {
                        egui::Color32::from_rgb(190, 190, 205)
                    },
                );
                // size (right aligned)
                if !entry.size.is_empty() {
                    ui.painter().text(
                        egui::pos2(rect.right() - 8.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        &entry.size,
                        egui::FontId::monospace(9.5),
                        egui::Color32::from_rgb(105, 107, 125),
                    );
                }

                if resp.clicked() {
                    self.selected = Some(i);
                    if entry.is_dir {
                        self.current_dir = entry.path.clone();
                        self.refresh();
                    } else {
                        self.on_file_open = Some(entry.path.clone());
                    }
                }
            }
        });
        let _ = theme_ok;
        self
    }
}

fn file_color(name: &str) -> egui::Color32 {
    match name.rsplit('.').next().unwrap_or("") {
        "rs" => egui::Color32::from_rgb(222, 130, 70),
        "py" => egui::Color32::from_rgb(80, 170, 220),
        "js" | "mjs" | "cjs" => egui::Color32::from_rgb(230, 190, 70),
        "ts" | "tsx" => egui::Color32::from_rgb(90, 150, 250),
        "html" | "htm" | "vue" | "svelte" => egui::Color32::from_rgb(230, 110, 80),
        "css" | "scss" | "less" => egui::Color32::from_rgb(110, 130, 250),
        "json" => egui::Color32::from_rgb(230, 210, 110),
        "toml" | "yaml" | "yml" => egui::Color32::from_rgb(160, 160, 175),
        "md" | "markdown" => egui::Color32::from_rgb(150, 190, 240),
        "c" | "h" | "cpp" | "hpp" => egui::Color32::from_rgb(100, 170, 230),
        "java" => egui::Color32::from_rgb(220, 140, 90),
        "go" => egui::Color32::from_rgb(90, 190, 220),
        "php" => egui::Color32::from_rgb(150, 130, 210),
        "sh" | "bash" | "zsh" | "fish" => egui::Color32::from_rgb(140, 200, 120),
        "sql" => egui::Color32::from_rgb(200, 170, 120),
        "rb" => egui::Color32::from_rgb(230, 100, 100),
        _ => egui::Color32::from_rgb(130, 132, 150),
    }
}

