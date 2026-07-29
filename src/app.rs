use eframe::egui;
use egui::{Color32, Frame, Margin, Rounding, Stroke};
use crate::editor::code_editor::CodeEditor;
use crate::editor::syntax::SyntaxHighlighter;
use crate::editor::minimap::Minimap;
use crate::ui::file_explorer::FileExplorer;
use crate::ui::tabs::TabManager;
use crate::ui::theme::Theme;
use crate::ui::status_bar::StatusBar;
use crate::ui::search::SearchPanel;
use crate::ui::terminal::TerminalPanel;
use crate::ui::welcome::WelcomePage;
use crate::ui::zen_mode::ZenMode;
use crate::utils::file_io::FileIO;

pub struct PhobosApp {
    editor: CodeEditor,
    file_explorer: FileExplorer,
    tabs: TabManager,
    theme: Theme,
    status_bar: StatusBar,
    search_panel: SearchPanel,
    terminal: TerminalPanel,
    welcome_page: WelcomePage,
    zen_mode: ZenMode,
    minimap: Minimap,
    syntax_highlighter: SyntaxHighlighter,
    show_file_explorer: bool,
    show_terminal: bool,
    show_ai_panel: bool,
    show_command_palette: bool,
    command_palette_input: String,
    notification: Option<(String, f64)>,
    current_file_path: Option<String>,
    split_view: bool,
    auto_save: bool,
}

impl PhobosApp {
    pub fn new() -> Self {
        Self {
            editor: CodeEditor::new(),
            file_explorer: FileExplorer::new(),
            tabs: TabManager::new(),
            theme: Theme::dark(),
            status_bar: StatusBar::new(),
            search_panel: SearchPanel::new(),
            terminal: TerminalPanel::new(),
            welcome_page: WelcomePage::new(),
            zen_mode: ZenMode::new(),
            minimap: Minimap::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            show_file_explorer: true,
            show_terminal: false,
            show_ai_panel: false,
            show_command_palette: false,
            command_palette_input: String::new(),
            notification: Some(("💀 Phobos IDE v0.3.0 — The God of Fear rises".into(), 5.0)),
            current_file_path: None,
            split_view: false,
            auto_save: true,
        }
    }

    fn open_file(&mut self) {
        if let Some((path, content)) = FileIO::open_file_dialog() {
            self.editor.content = content;
            let path_str = path.to_string_lossy().to_string();
            self.current_file_path = Some(path_str.clone());
            
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                self.editor.language = self.syntax_highlighter.get_extension(ext).to_string();
            }
            
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("Unknown".into());
            self.tabs.open_tab(&name);
            self.welcome_page.visible = false;
            self.welcome_page.recent_files.push(path_str);
            self.notification = Some((format!("📂 Opened: {}", name), 3.0));
        }
    }
    
    fn save_file(&mut self) {
        if let Some(path) = &self.current_file_path {
            if FileIO::write_file(path, &self.editor.content) {
                self.notification = Some(("💾 File saved!".into(), 2.0));
            }
        } else {
            self.save_file_as();
        }
    }
    
    fn save_file_as(&mut self) {
        if let Some(path) = FileIO::save_file_dialog(&self.editor.content) {
            self.current_file_path = Some(path.to_string_lossy().to_string());
            self.notification = Some(("💾 File saved!".into(), 2.0));
        }
    }

    fn new_file(&mut self) {
        self.editor.content = String::new();
        self.current_file_path = None;
        self.tabs.open_tab("Untitled");
        self.welcome_page.visible = false;
    }
    
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        if self.zen_mode.active { return; }

        egui::TopBottomPanel::top("menu_bar")
            .frame(Frame {
                fill: self.theme.bg_secondary,
                inner_margin: Margin::symmetric(0.0, 2.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_height(32.0);
                
                egui::menu::bar(ui, |ui| {
                    ui.set_min_width(ctx.screen_rect().width());
                    
                    ui.label(egui::RichText::new("💀 Phobos").color(self.theme.accent).size(16.0).strong());
                    ui.separator();
                    
                    ui.menu_button("📁 File", |ui| {
                        if ui.button("📄 New File").clicked() { self.new_file(); }
                        if ui.button("📂 Open File...").clicked() { self.open_file(); }
                        if ui.button("📁 Open Folder...").clicked() { }
                        ui.separator();
                        if ui.button("💾 Save (Ctrl+S)").clicked() { self.save_file(); }
                        if ui.button("💾 Save As...").clicked() { self.save_file_as(); }
                        ui.checkbox(&mut self.auto_save, "🔄 Auto Save");
                        ui.separator();
                        if ui.button("❌ Exit").clicked() { std::process::exit(0); }
                    });
                    
                    ui.menu_button("✏️ Edit", |ui| {
                        if ui.button("↩️ Undo (Ctrl+Z)").clicked() { }
                        if ui.button("↪️ Redo (Ctrl+Y)").clicked() { }
                        ui.separator();
                        if ui.button("🔍 Find (Ctrl+F)").clicked() { self.search_panel.visible = true; }
                        if ui.button("🔄 Replace (Ctrl+H)").clicked() { self.search_panel.visible = true; }
                        ui.separator();
                        if ui.button("📋 Format Document").clicked() { }
                    });
                    
                    ui.menu_button("👁️ View", |ui| {
                        ui.checkbox(&mut self.show_file_explorer, "📁 Explorer (Ctrl+B)");
                        ui.checkbox(&mut self.show_terminal, "💻 Terminal (Ctrl+J)");
                        ui.checkbox(&mut self.show_ai_panel, "🧠 AI Panel");
                        ui.separator();
                        ui.checkbox(&mut self.editor.line_numbers, "🔢 Line Numbers");
                        ui.checkbox(&mut self.minimap.visible, "🗺️ Minimap");
                        ui.checkbox(&mut self.editor.bracket_matching, "🔗 Bracket Matching");
                        ui.checkbox(&mut self.split_view, "📊 Split View (Ctrl+\\)");
                        ui.separator();
                        if ui.button("🧘 Zen Mode (Ctrl+K Z)").clicked() { self.zen_mode.toggle(); }
                        if ui.button("🔍 Command Palette... (Ctrl+P)").clicked() { self.show_command_palette = true; }
                    });
                    
                    ui.menu_button("🌐 Language", |ui| {
                        for (name, ext) in SyntaxHighlighter::language_list() {
                            if ui.button(name).clicked() {
                                self.editor.set_language(ext);
                            }
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("🔥 v0.3.0").color(self.theme.text_secondary).size(11.0));
                    });
                });
            });
    }

    fn render_command_palette(&mut self, ctx: &egui::Context) {
        if !self.show_command_palette { return; }
        
        egui::Window::new("🔍 Command Palette")
            .collapsible(false).resizable(false).title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([500.0, 350.0])
            .frame(Frame {
                fill: self.theme.bg_secondary,
                stroke: Stroke::new(2.0, self.theme.accent),
                rounding: Rounding::same(12.0),
                inner_margin: Margin::same(16.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("🔍 Command Palette").color(self.theme.accent).size(18.0));
                ui.add_space(8.0);
                
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_palette_input)
                        .hint_text("Type a command...")
                        .font(egui::TextStyle::Body)
                        .desired_width(f32::INFINITY)
                );
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_command_palette = false;
                }
                
                ui.add_space(12.0);
                ui.separator();
                
                let commands = [
                    ("📂 File: Open", "Ctrl+O"),
                    ("💾 File: Save", "Ctrl+S"),
                    ("📄 File: New", "Ctrl+N"),
                    ("🔍 Edit: Find", "Ctrl+F"),
                    ("🔄 Edit: Replace", "Ctrl+H"),
                    ("📋 Edit: Format Document", "Shift+Alt+F"),
                    ("📁 View: Toggle Explorer", "Ctrl+B"),
                    ("💻 View: Toggle Terminal", "Ctrl+J"),
                    ("🧘 View: Zen Mode", "Ctrl+K Z"),
                    ("📊 View: Split Editor", "Ctrl+\\"),
                    ("❌ File: Exit", "Alt+F4"),
                ];
                
                for (cmd, shortcut) in commands {
                    if self.command_palette_input.is_empty() || cmd.to_lowercase().contains(&self.command_palette_input.to_lowercase()) {
                        ui.horizontal(|ui| {
                            if ui.button(cmd).clicked() {
                                self.show_command_palette = false;
                                self.command_palette_input.clear();
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(shortcut).color(self.theme.text_secondary).size(11.0));
                            });
                        });
                    }
                }
            });
    }
}

impl eframe::App for PhobosApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.apply(ctx);
        self.render_menu_bar(ctx);
        
        // ⚡ Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) { self.open_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) { self.save_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) { self.new_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) { self.search_panel.visible = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) { self.search_panel.visible = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl) { self.show_command_palette = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) { self.show_file_explorer = !self.show_file_explorer; }
        if ctx.input(|i| i.key_pressed(egui::Key::J) && i.modifiers.ctrl) { self.show_terminal = !self.show_terminal; }
        if ctx.input(|i| i.key_pressed(egui::Key::Backslash) && i.modifiers.ctrl) { self.split_view = !self.split_view; }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) { /* Close tab */ }
        
        // Zen Mode toggle
        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) {
            if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                self.zen_mode.toggle();
            }
        }
        
        // Auto-save
        if self.auto_save && self.current_file_path.is_some() {
            // Could implement periodic auto-save here
        }
        
        // Notification
        if let Some((msg, time)) = &self.notification {
            if *time > 0.0 {
                egui::TopBottomPanel::bottom("notification")
                    .frame(Frame { fill: Color32::from_rgb(40, 40, 40), ..Default::default() })
                    .show(ctx, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(msg.as_str()).color(self.theme.accent).size(12.0));
                        });
                    });
                self.notification = Some((msg.clone(), time - ctx.input(|i| i.unstable_dt) as f64));
            } else {
                self.notification = None;
            }
        }
        
        // Search panel
        if self.search_panel.visible {
            egui::TopBottomPanel::top("search_panel")
                .frame(Frame { fill: self.theme.bg_secondary, ..Default::default() })
                .show(ctx, |ui| {
                    if let Some(new_content) = self.search_panel.show(ui, &self.editor.content) {
                        self.editor.content = new_content;
                    }
                });
        }
        
        self.render_command_palette(ctx);
        
        // Welcome page or Editor
        if self.welcome_page.visible {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(_) = self.welcome_page.show(ui) {
                    self.open_file();
                }
            });
        } else {
            // File Explorer
            if self.show_file_explorer && !self.zen_mode.active {
                egui::SidePanel::left("file_panel")
                    .resizable(true).default_width(260.0)
                    .show(ctx, |ui| {
                        self.file_explorer.show(ui);
                        if let Some(path) = self.file_explorer.on_file_open.take() {
                            if let Some(content) = FileIO::read_file(&path) {
                                self.editor.content = content;
                                self.current_file_path = Some(path.clone());
                                if let Some(ext) = std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
                                    self.editor.language = self.syntax_highlighter.get_extension(ext).to_string();
                                }
                            }
                        }
                    });
            }
            
            // Main Editor
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.split_view {
                    // Split view — دو Editor کنار هم
                    ui.columns(2, |columns| {
                        self.tabs.show(&mut columns[0]);
                        columns[0].add(egui::Separator::default().spacing(0.0));
                        self.editor.show(&mut columns[0]);
                        
                        self.tabs.show(&mut columns[1]);
                        columns[1].add(egui::Separator::default().spacing(0.0));
                        self.editor.show(&mut columns[1]);
                    });
                } else {
                    self.tabs.show(ui);
                    ui.add(egui::Separator::default().spacing(0.0));
                    
                    // Editor with optional minimap
                    if self.minimap.visible {
                        ui.columns(2, |columns| {
                            self.editor.show(&mut columns[0]);
                            self.minimap.show(&mut columns[1], &self.editor.content);
                        });
                    } else {
                        self.editor.show(ui);
                    }
                }
            });
        }
        
        // Terminal
        if self.show_terminal && !self.zen_mode.active {
            egui::TopBottomPanel::bottom("terminal")
                .resizable(true).default_height(220.0)
                .show(ctx, |ui| {
                    self.terminal.show(ui);
                });
        }
        
        // Status Bar
        if !self.zen_mode.active {
            egui::TopBottomPanel::bottom("status_bar")
                .frame(Frame { fill: Color32::from_rgb(0, 122, 204), inner_margin: Margin::symmetric(12.0, 4.0), ..Default::default() })
                .show(ctx, |ui| {
                    self.status_bar.show(ui, &self.editor);
                });
        }
    }
}
