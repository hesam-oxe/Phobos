use eframe::egui;
use crate::editor::code_editor::CodeEditor;
use crate::ui::file_explorer::FileExplorer;
use crate::ui::tabs::TabManager;
use crate::ui::theme::Theme;
use crate::ui::status_bar::StatusBar;

pub struct PhobosApp {
    editor: CodeEditor,
    file_explorer: FileExplorer,
    tabs: TabManager,
    theme: Theme,
    status_bar: StatusBar,
    show_file_explorer: bool,
    show_terminal: bool,
    show_ai_panel: bool,
}

impl PhobosApp {
    pub fn new() -> Self {
        Self {
            editor: CodeEditor::new(),
            file_explorer: FileExplorer::new(),
            tabs: TabManager::new(),
            theme: Theme::dark(),
            status_bar: StatusBar::new(),
            show_file_explorer: true,
            show_terminal: false,
            show_ai_panel: true,
        }
    }
}

impl eframe::App for PhobosApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.theme.apply(ctx);

        // Main menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("📁 File", |ui| {
                    if ui.button("📄 New").clicked() { }
                    if ui.button("📂 Open").clicked() { }
                    if ui.button("💾 Save").clicked() { }
                    ui.separator();
                    if ui.button("❌ Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("✏️ Edit", |ui| {
                    if ui.button("↩️ Undo").clicked() { }
                    if ui.button("↪️ Redo").clicked() { }
                    ui.separator();
                    if ui.button("🔍 Find").clicked() { }
                    if ui.button("🔄 Replace").clicked() { }
                });

                ui.menu_button("🔧 View", |ui| {
                    ui.checkbox(&mut self.show_file_explorer, "📁 File Explorer");
                    ui.checkbox(&mut self.show_terminal, "💻 Terminal");
                    ui.checkbox(&mut self.show_ai_panel, "🧠 AI Panel");
                });

                ui.menu_button("🌐 Language", |ui| {
                    let languages = [
                        "Assembly", "C", "C++", "C#", "Rust", "Go",
                        "Python", "Java", "JavaScript", "TypeScript",
                        "PHP", "Ruby", "SQL", "Pascal", "Swift", "Kotlin",
                        "Shell", "HTML", "CSS", "JSON", "YAML", "Markdown",
                    ];
                    for lang in languages {
                        if ui.button(lang).clicked() { }
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("💀 Phobos IDE v0.1.0");
                });
            });
        });

        // Left panel - File Explorer
        if self.show_file_explorer {
            egui::SidePanel::left("file_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.file_explorer.show(ui);
                });
        }

        // Right panel - AI Panel
        if self.show_ai_panel {
            egui::SidePanel::right("ai_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("🧠 Phobos AI");
                    ui.separator();
                    ui.label("🤖 Ghost Typing: Ready");
                    ui.label("🔍 Bug Finder: Active");
                    ui.label("📝 Code Explainer: Ready");
                    if ui.button("🤖 Explain Selected Code").clicked() { }
                    if ui.button("🔍 Find Bugs").clicked() { }
                    if ui.button("✨ Generate Code").clicked() { }
                });
        }

        // Bottom panel - Terminal
        if self.show_terminal {
            egui::TopBottomPanel::bottom("terminal")
                .resizable(true)
                .default_height(200.0)
                .show(ctx, |ui| {
                    ui.heading("💻 Terminal");
                    ui.separator();
                    ui.label("$ phobos-terminal v1.0");
                });
        }

        // Central panel - Editor with Tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tabs bar
            self.tabs.show(ui);
            ui.separator();

            // Code Editor
            self.editor.show(ui);
        });

        // Status Bar at bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar.show(ui, &self.editor);
        });
    }
}
