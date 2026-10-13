use eframe::egui;
use egui::{Color32, Frame, Margin, Rounding, Stroke};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::editor::code_editor::CodeEditor;
use crate::editor::syntax::SyntaxHighlighter;
use crate::editor::minimap::Minimap;
use crate::editor::completion::CompletionProvider;
use crate::core::{CompletionItem as CoreCompletionItem, CompletionItemKind as CoreKind};
use crate::ai::ghost_typer::{Suggestion, SuggestionKind};
use crate::services::lsp::{LspManager, LspEvent};
use crate::ui::file_explorer::FileExplorer;
use crate::ui::tabs::TabManager;
use crate::ui::theme::Theme;
use crate::ui::status_bar::StatusBar;
use crate::ui::search::SearchPanel;
use crate::ui::terminal::TerminalPanel;
use crate::ui::welcome::WelcomePage;
use crate::ui::zen_mode::ZenMode;
use crate::ui::ai_panel::AiPanel;
use crate::ui::build_panel::BuildPanel;
use crate::ui::git_panel::GitPanel;
use crate::ui::debug_panel::DebugPanel;
use crate::ui::pty_terminal::PtyTerminal;
use crate::low_level::hex_editor::HexEditor;
use crate::low_level::disassembler::Disassembler;
use crate::low_level::memory_viewer::MemoryViewer;
use crate::low_level::register_viewer::RegisterViewer;
use crate::services::plugins::PluginManager;
use crate::utils::file_io::FileIO;
use crate::utils::config::PhobosConfig;

pub struct PhobosApp {
    editor: CodeEditor,
    editors: HashMap<String, String>,
    file_explorer: FileExplorer,
    tabs: TabManager,
    theme: Theme,
    status_bar: StatusBar,
    search_panel: SearchPanel,
    terminal: TerminalPanel,
    pty: PtyTerminal,
    welcome_page: WelcomePage,
    zen_mode: ZenMode,
    minimap: Minimap,
    syntax_highlighter: SyntaxHighlighter,
    ai_panel: AiPanel,
    build_panel: BuildPanel,
    git_panel: GitPanel,
    debug_panel: DebugPanel,
    hex_editor: HexEditor,
    disassembler: Disassembler,
    mem_viewer: MemoryViewer,
    reg_viewer: RegisterViewer,
    plugins: PluginManager,
    config: PhobosConfig,
    show_file_explorer: bool,
    show_terminal: bool,
    show_pty: bool,
    show_ai_panel: bool,
    show_build: bool,
    show_git: bool,
    show_debug: bool,
    show_hex: bool,
    show_disasm: bool,
    show_plugins: bool,
    show_command_palette: bool,
    command_palette_input: String,
    notification: Option<(String, f64)>,
    current_file_path: Option<String>,
    split_view: bool,
    auto_save: bool,
    last_save: std::time::Instant,
    watcher: Option<notify::RecommendedWatcher>,
    watch_rx: Option<std::sync::mpsc::Receiver<crate::services::watcher::WatchEvent>>,
    lsp: LspManager,
    completion: CompletionProvider,
    lsp_hover: Option<String>,
    pending_completion: u64,
    pending_hover: u64,
}

impl PhobosApp {
    pub fn new() -> Self {
        let config = PhobosConfig::load();
        let mut plugins = PluginManager::new();
        plugins.discover();
        let (watcher, watch_rx) = crate::services::watcher::create_watcher().ok()
            .map(|(w, r)| (Some(w), Some(r))).unwrap_or((None, None));
        let mut app = Self {
            editor: CodeEditor::new(),
            editors: HashMap::new(),
            file_explorer: FileExplorer::new(),
            tabs: TabManager::new(),
            theme: Theme::dark(),
            status_bar: StatusBar::new(),
            search_panel: SearchPanel::new(),
            terminal: TerminalPanel::new(),
            pty: PtyTerminal::new(),
            welcome_page: WelcomePage::new(),
            zen_mode: ZenMode::new(),
            minimap: Minimap::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            ai_panel: AiPanel::new(),
            build_panel: BuildPanel::new(),
            git_panel: GitPanel::new(),
            debug_panel: DebugPanel::new(),
            hex_editor: HexEditor::new(),
            disassembler: Disassembler::new(),
            mem_viewer: MemoryViewer::new(),
            reg_viewer: RegisterViewer::new(),
            plugins,
            config: config.clone(),
            show_file_explorer: true,
            show_terminal: false,
            show_pty: false,
            show_ai_panel: false,
            show_build: false,
            show_git: false,
            show_debug: false,
            show_hex: false,
            show_disasm: false,
            show_plugins: false,
            show_command_palette: false,
            command_palette_input: String::new(),
            notification: Some(("💀 Phobos IDE v1.0 — God Mode".into(), 5.0)),
            current_file_path: None,
            split_view: false,
            auto_save: config.auto_save,
            last_save: std::time::Instant::now(),
            watcher,
            watch_rx,
            lsp: LspManager::new(),
            completion: CompletionProvider::new("rust"),
            lsp_hover: None,
            pending_completion: 0,
            pending_hover: 0,
        };
        app.editor.config.show_line_numbers = config.show_line_numbers;
        app.editor.config.minimap_enabled = config.show_minimap;
        app.welcome_page.recent_files = config.recent_files.clone();
        app
    }

    fn persist_config(&mut self) {
        self.config.auto_save = self.auto_save;
        self.config.show_line_numbers = self.editor.config.show_line_numbers;
        self.config.show_minimap = self.minimap.visible;
        self.config.recent_files = self.welcome_page.recent_files.clone();
        self.config.workspace_root = self.current_file_path.clone();
        let _ = self.config.save();
    }

    fn stash_current(&mut self) {
        if let Some(p) = self.current_file_path.clone() {
            self.editors.insert(p, self.editor.content.clone());
        }
    }

    fn load_path(&mut self, path: String) {
        self.stash_current();
        if let Some(content) = FileIO::read_file(&path) {
            if let Some(cached) = self.editors.get(&path) {
                self.editor.content = cached.clone();
            } else {
                self.editor.content = content;
                self.editors.insert(path.clone(), self.editor.content.clone());
            }
            self.current_file_path = Some(path.clone());
            if let Some(ext) = std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
                let lang = self.syntax_highlighter.get_extension(ext).to_string();
                self.editor.set_language(&lang);
                self.ai_panel.set_language(&lang);
                self.completion.set_language(&lang);
            }
            let root = self.workspace_root_for(&path);
            self.lsp.open_file(&path, &self.editor.content, root);
            let name = std::path::Path::new(&path).file_name()
                .map(|n| n.to_string_lossy().to_string()).unwrap_or(path.clone());
            self.tabs.open_tab(&name);
            self.welcome_page.visible = false;
            if !self.welcome_page.recent_files.contains(&path) {
                self.welcome_page.recent_files.insert(0, path.clone());
                self.welcome_page.recent_files.truncate(20);
            }
            self.git_panel.path = std::path::Path::new(&path).parent()
                .map(|p| p.to_string_lossy().to_string()).unwrap_or(".".into());
            // watch parent dir
            if let Some(w) = self.watcher.as_mut() {
                crate::services::watcher::watch_path(w, &self.git_panel.path);
            }
            self.notification = Some((format!("📂 {}", name), 2.5));
            self.persist_config();
        }
    }

    fn open_file(&mut self) {
        if let Some((path, content)) = FileIO::open_file_dialog() {
            let path_str = path.to_string_lossy().to_string();
            self.stash_current();
            self.editor.content = content.clone();
            self.editors.insert(path_str.clone(), content);
            self.current_file_path = Some(path_str.clone());
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lang = self.syntax_highlighter.get_extension(ext).to_string();
                self.editor.set_language(&lang);
                self.ai_panel.set_language(&lang);
                self.completion.set_language(&lang);
            }
            let root = self.workspace_root_for(&path_str);
            self.lsp.open_file(&path_str, &self.editor.content, root);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("Unknown".into());
            self.tabs.open_tab(&name);
            self.welcome_page.visible = false;
            self.welcome_page.recent_files.insert(0, path_str.clone());
            self.welcome_page.recent_files.truncate(20);
            self.notification = Some((format!("📂 Opened: {}", name), 3.0));
            self.persist_config();
        }
    }

    fn open_folder(&mut self) {
        if let Some(p) = FileIO::open_folder_dialog() {
            let s = p.to_string_lossy().to_string();
            self.file_explorer.current_dir = s.clone();
            self.file_explorer.refresh();
            self.git_panel.path = s.clone();
            if let Some(w) = self.watcher.as_mut() {
                crate::services::watcher::watch_path(w, &s);
            }
            self.welcome_page.visible = false;
            self.notification = Some((format!("📁 Folder: {}", s), 3.0));
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = self.current_file_path.clone() {
            if FileIO::write_file(&path, &self.editor.content) {
                self.editors.insert(path.clone(), self.editor.content.clone());
                self.lsp.change(&path, &self.editor.content);
                self.notification = Some(("💾 Saved".into(), 2.0));
                self.last_save = std::time::Instant::now();
            }
        } else { self.save_file_as(); }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = FileIO::save_file_dialog(&self.editor.content) {
            let s = path.to_string_lossy().to_string();
            self.current_file_path = Some(s.clone());
            self.editors.insert(s, self.editor.content.clone());
            self.notification = Some(("💾 Saved".into(), 2.0));
            self.last_save = std::time::Instant::now();
        }
    }

    fn new_file(&mut self) {
        self.stash_current();
        self.editor.content = String::new();
        self.current_file_path = None;
        self.tabs.open_tab("Untitled");
        self.welcome_page.visible = false;
    }

    fn format_document(&mut self) {
        let lang = self.editor.language.clone();
        if lang == "rs" || lang == "rust" {
            if let Some(path) = self.current_file_path.clone() {
                // try rustfmt on temp copy
                let tmp = format!("{}.phobos_fmt", path);
                if FileIO::write_file(&tmp, &self.editor.content) {
                    let out = std::process::Command::new("rustfmt")
                        .arg(&tmp).arg("--emit").arg("stdout").output();
                    let _ = std::fs::remove_file(&tmp);
                    if let Ok(o) = out {
                        if o.status.success() {
                            self.editor.content = String::from_utf8_lossy(&o.stdout).to_string();
                            self.notification = Some(("✨ Formatted (rustfmt)".into(), 2.0));
                            return;
                        }
                    }
                }
            }
            // fallback: trim trailing whitespace
            self.editor.content = self.editor.content.lines()
                .map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
            self.notification = Some(("✨ Formatted (basic)".into(), 2.0));
        } else {
            self.editor.content = self.editor.content.lines()
                .map(|l| l.trim_end()).collect::<Vec<_>>().join("\n");
            // tab → spaces normalize
            self.notification = Some(("✨ Formatted (whitespace)".into(), 2.0));
        }
    }

    fn poll_watcher(&mut self) {
        if let Some(rx) = self.watch_rx.as_ref() {
            while let Ok(ev) = rx.try_recv() {
                use crate::services::watcher::WatchEvent as W;
                match ev {
                    W::Changed(p) | W::Created(p) => {
                        self.file_explorer.refresh();
                        // hot-reload current file if changed externally
                        if let Some(cur) = self.current_file_path.clone() {
                            if p.to_string_lossy() == cur {
                                if let Some(content) = FileIO::read_file(&cur) {
                                    if content != self.editor.content {
                                        self.editor.content = content;
                                        self.notification = Some(("🔄 Reloaded (external change)".into(), 2.0));
                                    }
                                }
                            }
                        }
                    }
                    W::Removed(_) => { self.file_explorer.refresh(); }
                }
            }
        }
    }

    fn poll_lsp(&mut self) {
        for ev in self.lsp.poll() {
            match ev {
                LspEvent::Diagnostics { path, diagnostics } => {
                    if self.current_file_path.as_deref() == Some(path.as_str()) {
                        self.editor.set_diagnostics(diagnostics);
                    }
                }
                LspEvent::Completions { id, items } => {
                    if id == self.pending_completion {
                        let suggestions =
                            items.iter().map(lsp_to_suggestion).collect::<Vec<_>>();
                        if !suggestions.is_empty() {
                            self.completion.set_suggestions(suggestions);
                        }
                    }
                }
                LspEvent::Hover { id, text } => {
                    if id == self.pending_hover && !text.trim().is_empty() {
                        self.lsp_hover = Some(text);
                    }
                }
                LspEvent::Status(msg) => {
                    self.notification = Some((msg, 2.5));
                }
            }
        }
    }

    fn workspace_root_for(&self, path: &str) -> PathBuf {
        if !self.file_explorer.current_dir.is_empty() {
            return PathBuf::from(&self.file_explorer.current_dir);
        }
        std::path::Path::new(path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn request_lsp_completion(&mut self) {
        let pos = self.editor.cursor_pos;
        let code = self.editor.content.clone();
        self.completion.update_suggestions(&code, pos);
        if let Some(path) = self.current_file_path.clone() {
            if LspManager::supports(&path) {
                let (line, col) = self.editor.cursor_lsp_pos();
                if let Some(id) = self.lsp.request_completion(&path, line, col) {
                    self.pending_completion = id;
                }
            }
        }
    }

    fn request_lsp_hover(&mut self) {
        if let Some(path) = self.current_file_path.clone() {
            if LspManager::supports(&path) {
                let (line, col) = self.editor.cursor_lsp_pos();
                if let Some(id) = self.lsp.request_hover(&path, line, col) {
                    self.pending_hover = id;
                }
            } else {
                self.notification =
                    Some(("No language server for this file".into(), 2.0));
            }
        }
    }

    fn exec_palette(&mut self, cmd: &str) {
        match cmd {
            c if c.starts_with("📂 File: Open") => self.open_file(),
            c if c.starts_with("📁 File: Open Folder") => self.open_folder(),
            c if c.starts_with("💾 File: Save") => self.save_file(),
            c if c.starts_with("📄 File: New") => self.new_file(),
            c if c.starts_with("🔍 Edit: Find") => self.search_panel.visible = true,
            c if c.starts_with("📋 Edit: Format") => self.format_document(),
            c if c.starts_with("📁 View: Toggle Explorer") => self.show_file_explorer = !self.show_file_explorer,
            c if c.starts_with("💻 View: Toggle Terminal") => self.show_pty = !self.show_pty,
            c if c.starts_with("🧠 AI") => self.show_ai_panel = !self.show_ai_panel,
            c if c.starts_with("🔨 Build") => { self.show_build = true; if let Some(p) = self.current_file_path.clone() { let l = self.editor.language.clone(); self.build_panel.build(&p, &l); } }
            c if c.starts_with("▶ Run") => { self.show_build = true; if let Some(p) = self.current_file_path.clone() { let l = self.editor.language.clone(); self.build_panel.run(&p, &l); } }
            c if c.starts_with("🐞 Debug") => self.show_debug = !self.show_debug,
            c if c.starts_with("🌿 Git") => self.show_git = !self.show_git,
            c if c.starts_with("🔧 Hex") => { self.show_hex = true; if let Some(p) = self.current_file_path.clone() { self.hex_editor.open_file(&p); } }
            c if c.starts_with("⚙️ Disasm") => { self.show_disasm = true; if let Some(p) = self.current_file_path.clone() { self.disassembler.disassemble_file(&p); } }
            c if c.starts_with("✨ LSP") => self.request_lsp_completion(),
            c if c.starts_with("🔍 LSP") => self.request_lsp_hover(),
            c if c.starts_with("🧘") => self.zen_mode.toggle(),
            c if c.starts_with("📊") => self.split_view = !self.split_view,
            _ => {}
        }
        self.show_command_palette = false;
        self.command_palette_input.clear();
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        if self.zen_mode.active { return; }
        egui::TopBottomPanel::top("menu_bar")
            .frame(Frame { fill: self.theme.bg_secondary, inner_margin: Margin::symmetric(0.0, 2.0), ..Default::default() })
            .show(ctx, |ui| {
                ui.set_height(32.0);
                egui::menu::bar(ui, |ui| {
                    ui.set_min_width(ctx.screen_rect().width());
                    ui.label(egui::RichText::new("💀 Phobos").color(self.theme.accent).size(16.0).strong());
                    ui.separator();
                    ui.menu_button("📁 File", |ui| {
                        if ui.button("📄 New File").clicked() { self.new_file(); }
                        if ui.button("📂 Open File...").clicked() { self.open_file(); }
                        if ui.button("📁 Open Folder...").clicked() { self.open_folder(); }
                        ui.separator();
                        if ui.button("💾 Save (Ctrl+S)").clicked() { self.save_file(); }
                        if ui.button("💾 Save As...").clicked() { self.save_file_as(); }
                        ui.checkbox(&mut self.auto_save, "🔄 Auto Save");
                        ui.separator();
                        if ui.button("❌ Exit").clicked() { self.persist_config(); std::process::exit(0); }
                    });
                    ui.menu_button("✏️ Edit", |ui| {
                        if ui.button("📋 Format Document (Shift+Alt+F)").clicked() { self.format_document(); }
                        ui.separator();
                        if ui.button("🔍 Find (Ctrl+F)").clicked() { self.search_panel.visible = true; }
                        if ui.button("🔄 Replace (Ctrl+H)").clicked() { self.search_panel.visible = true; }
                    });
                    ui.menu_button("👁️ View", |ui| {
                        ui.checkbox(&mut self.show_file_explorer, "📁 Explorer (Ctrl+B)");
                        ui.checkbox(&mut self.show_pty, "💻 PTY Terminal (Ctrl+J)");
                        ui.checkbox(&mut self.show_terminal, "📟 Fake Term");
                        ui.checkbox(&mut self.show_ai_panel, "🧠 AI Panel");
                        ui.checkbox(&mut self.show_build, "🔨 Build");
                        ui.checkbox(&mut self.show_git, "🌿 Git");
                        ui.checkbox(&mut self.show_debug, "🐞 Debug");
                        ui.checkbox(&mut self.show_hex, "🔧 Hex");
                        ui.checkbox(&mut self.show_disasm, "⚙️ Disasm");
                        ui.checkbox(&mut self.show_plugins, "🔌 Plugins");
                        ui.separator();
                        ui.checkbox(&mut self.editor.config.show_line_numbers, "🔢 Line Numbers");
                        ui.checkbox(&mut self.minimap.visible, "🗺️ Minimap");
                        ui.checkbox(&mut self.editor.config.bracket_matching, "🔗 Bracket Matching");
                        ui.checkbox(&mut self.split_view, "📊 Split View (Ctrl+\\)");
                        ui.separator();
                        if ui.button("🧘 Zen Mode").clicked() { self.zen_mode.toggle(); }
                        if ui.button("🔍 Command Palette... (Ctrl+P)").clicked() { self.show_command_palette = true; }
                    });
                    ui.menu_button("🔨 Build", |ui| {
                        if ui.button("🔨 Build (Ctrl+B)").clicked() {
                            self.show_build = true;
                            if let Some(p) = self.current_file_path.clone() { let l = self.editor.language.clone(); self.build_panel.build(&p, &l); }
                        }
                        if ui.button("▶ Run (F5)").clicked() {
                            self.show_build = true;
                            if let Some(p) = self.current_file_path.clone() { let l = self.editor.language.clone(); self.build_panel.run(&p, &l); }
                        }
                        if ui.button("🐞 Debug (Shift+F5)").clicked() { self.show_debug = true; }
                    });
                    ui.menu_button("🌐 Language", |ui| {
                        for (name, ext) in SyntaxHighlighter::language_list() {
                            if ui.button(name).clicked() {
                                self.editor.set_language(ext);
                                self.ai_panel.set_language(ext);
                            }
                        }
                    });
                    ui.menu_button("🔌 Plugins", |ui| {
                        if ui.button("Manage...").clicked() { self.show_plugins = true; }
                        for (id, p) in self.plugins.plugins.iter() {
                            ui.label(format!("{} {} {}", if p.enabled {"🟢"} else {"⚪"}, p.name, id));
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("🔥 v1.0 GOD").color(self.theme.text_secondary).size(11.0));
                    });
                });
            });
    }

    fn render_command_palette(&mut self, ctx: &egui::Context) {
        if !self.show_command_palette { return; }
        egui::Window::new("🔍 Command Palette")
            .collapsible(false).resizable(false).title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([540.0, 420.0])
            .frame(Frame { fill: self.theme.bg_secondary, stroke: Stroke::new(2.0, self.theme.accent),
                rounding: Rounding::same(12.0), inner_margin: Margin::same(16.0), ..Default::default() })
            .show(ctx, |ui| {
                ui.heading(egui::RichText::new("🔍 Command Palette").color(self.theme.accent).size(18.0));
                ui.add_space(8.0);
                ui.add(egui::TextEdit::singleline(&mut self.command_palette_input)
                    .hint_text("Type a command...").desired_width(f32::INFINITY));
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) { self.show_command_palette = false; }
                ui.add_space(12.0); ui.separator();
                let commands = [
                    ("📂 File: Open", "Ctrl+O"), ("📁 File: Open Folder", "Ctrl+K O"),
                    ("💾 File: Save", "Ctrl+S"), ("📄 File: New", "Ctrl+N"),
                    ("🔍 Edit: Find", "Ctrl+F"), ("📋 Edit: Format Document", "Shift+Alt+F"),
                    ("📁 View: Toggle Explorer", "Ctrl+B"), ("💻 View: Toggle Terminal", "Ctrl+J"),
                    ("🧠 AI: Toggle Panel", "Ctrl+I"), ("🔨 Build: Build", "Ctrl+B"),
                    ("▶ Run: Run", "F5"), ("🐞 Debug: Panel", "Shift+F5"),
                    ("✨ LSP: Complete", "Ctrl+Space"), ("🔍 LSP: Hover", "Ctrl+K"),
                    ("🌿 Git: Panel", ""), ("🔧 Hex: Open", ""), ("⚙️ Disasm: Open", ""),
                    ("🧘 View: Zen Mode", ""), ("📊 View: Split Editor", "Ctrl+\\"),
                ];
                for (cmd, shortcut) in commands {
                    if self.command_palette_input.is_empty() || cmd.to_lowercase().contains(&self.command_palette_input.to_lowercase()) {
                        ui.horizontal(|ui| {
                            if ui.button(cmd).clicked() { self.exec_palette(cmd); }
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
        self.poll_watcher();
        self.poll_lsp();
        self.render_menu_bar(ctx);

        if ctx.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) { self.open_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) { self.save_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) { self.new_file(); }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) { self.search_panel.visible = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) { self.search_panel.visible = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl) { self.show_command_palette = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) { self.show_file_explorer = !self.show_file_explorer; }
        if ctx.input(|i| i.key_pressed(egui::Key::J) && i.modifiers.ctrl) { self.show_pty = !self.show_pty; }
        if ctx.input(|i| i.key_pressed(egui::Key::Backslash) && i.modifiers.ctrl) { self.split_view = !self.split_view; }
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.show_build = true;
            if let Some(p) = self.current_file_path.clone() { let l = self.editor.language.clone(); self.build_panel.run(&p, &l); }
        }
        if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::F5)) { self.show_debug = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) {
            // close tab = stash + clear
            self.stash_current();
            self.editor.content.clear();
            self.current_file_path = None;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::I) && i.modifiers.ctrl) { self.show_ai_panel = !self.show_ai_panel; }
        // LSP completion (Ctrl+Space): instant local AI + async server enrich
        if ctx.input(|i| i.key_pressed(egui::Key::Space) && i.modifiers.ctrl) { self.request_lsp_completion(); }
        // Accept selected completion with Tab (consume so editor doesn't indent)
        if self.completion.is_visible() && ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            if let Some(text) = self.completion.accept_completion() {
                self.editor.insert_text(&text);
            }
            ctx.input_mut(|i| { i.consume_key(egui::Modifiers::NONE, egui::Key::Tab); });
        }
        if self.completion.is_visible() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.completion.dismiss();
        }
        if self.completion.is_visible() && ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.completion.navigate_up();
            ctx.input_mut(|i| { i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp); });
        }
        if self.completion.is_visible() && ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.completion.navigate_down();
            ctx.input_mut(|i| { i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown); });
        }
        // LSP hover at cursor (Ctrl+K)
        if ctx.input(|i| i.key_pressed(egui::Key::K) && i.modifiers.ctrl) { self.request_lsp_hover(); }

        if self.auto_save && self.current_file_path.is_some() && self.last_save.elapsed().as_secs() > 5 {
            self.save_file();
        }

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
            } else { self.notification = None; }
        }

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

        if self.welcome_page.visible {
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.welcome_page.show(ui).is_some() { self.open_file(); }
            });
        } else {
            if self.show_file_explorer && !self.zen_mode.active {
                egui::SidePanel::left("file_panel").resizable(true).default_width(260.0)
                    .show(ctx, |ui| {
                        self.file_explorer.show(ui);
                        if let Some(path) = self.file_explorer.on_file_open.take() {
                            self.load_path(path);
                        }
                    });
            }
            // right AI panel
            if self.show_ai_panel && !self.zen_mode.active {
                egui::SidePanel::right("ai_panel").resizable(true).default_width(340.0)
                    .show(ctx, |ui| {
                        let cursor = self.editor.cursor_pos;
                        let code = self.editor.content.clone();
                        self.ai_panel.show(ui, &code, cursor);
                    });
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.split_view {
                    ui.columns(2, |columns| {
                        self.tabs.show(&mut columns[0]);
                        columns[0].add(egui::Separator::default().spacing(0.0));
                        self.editor.show(&mut columns[0]);
                        self.tabs.show(&mut columns[1]);
                        columns[1].add(egui::Separator::default().spacing(0.0));
                        // second pane shows same buffer (true split needs 2 editors; share content)
                        self.editor.show(&mut columns[1]);
                    });
                } else {
                    self.tabs.show(ui);
                    ui.add(egui::Separator::default().spacing(0.0));
                    if self.minimap.visible {
                        let cursor_line =
                            self.editor.get_cursor_position().0.saturating_sub(1);
                        let content = self.editor.content.clone();
                        ui.columns(2, |columns| {
                            self.editor.show(&mut columns[0]);
                            if let Some(line) =
                                self.minimap.show(&mut columns[1], &content, cursor_line)
                            {
                                self.editor.jump_to_line(line);
                            }
                        });
                    } else { self.editor.show(ui); }
                }
                render_completion_popup(&mut self.completion, &mut self.editor, ui);
            });
        }

        // bottom panels
        if self.show_pty && !self.zen_mode.active {
            egui::TopBottomPanel::bottom("pty").resizable(true).default_height(260.0)
                .show(ctx, |ui| { self.pty.show(ui); });
        } else if self.show_terminal && !self.zen_mode.active {
            egui::TopBottomPanel::bottom("terminal").resizable(true).default_height(220.0)
                .show(ctx, |ui| { self.terminal.show(ui); });
        }
        if self.show_build {
            egui::TopBottomPanel::bottom("build").resizable(true).default_height(200.0)
                .show(ctx, |ui| { self.build_panel.show(ui); });
        }
        if self.show_git && !self.zen_mode.active {
            egui::SidePanel::left("git_panel").resizable(true).default_width(300.0)
                .show(ctx, |ui| { self.git_panel.show(ui); });
        }
        if self.show_debug {
            egui::TopBottomPanel::bottom("debug").resizable(true).default_height(240.0)
                .show(ctx, |ui| { self.debug_panel.show(ui); });
        }
        if self.show_hex {
            egui::Window::new("🔧 Hex").resizable(true).default_size([700.0, 450.0])
                .show(ctx, |ui| { self.hex_editor.show(ui); });
        }
        if self.show_disasm {
            egui::Window::new("⚙️ Disasm").resizable(true).default_size([700.0, 450.0])
                .show(ctx, |ui| { self.disassembler.show(ui); });
        }
        if self.mem_viewer.visible {
            egui::Window::new("🧠 Memory").show(ctx, |ui| { self.mem_viewer.show(ui); });
        }
        if self.reg_viewer.visible {
            egui::Window::new("📟 Regs").show(ctx, |ui| { self.reg_viewer.show(ui); });
        }
        if self.show_plugins {
            egui::Window::new("🔌 Plugins").show(ctx, |ui| {
                ui.heading("Plugins");
                let mut toggle: Option<String> = None;
                for (id, p) in self.plugins.plugins.iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {} v{} - {}", if p.enabled {"🟢"} else {"⚪"}, p.name, p.version, p.description));
                        if ui.button("toggle").clicked() { toggle = Some(id.clone()); }
                    });
                }
                if let Some(id) = toggle { self.plugins.toggle(&id); }
                if ui.button("Close").clicked() { self.show_plugins = false; }
            });
        }

        if self.lsp_hover.is_some() {
            let mut close = false;
            egui::Window::new("🔍 LSP Hover")
                .resizable(true)
                .default_size([420.0, 200.0])
                .show(ctx, |ui| {
                    if let Some(text) = &self.lsp_hover {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(egui::RichText::new(text).monospace().size(12.0));
                        });
                    }
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            if close {
                self.lsp_hover = None;
            }
        }
        if !self.zen_mode.active {
            egui::TopBottomPanel::bottom("status_bar")
                .frame(Frame { fill: Color32::from_rgb(0, 122, 204), inner_margin: Margin::symmetric(12.0, 4.0), ..Default::default() })
                .show(ctx, |ui| {
                    let lsp_on = self
                        .current_file_path
                        .as_deref()
                        .map(LspManager::supports)
                        .unwrap_or(false);
                    self.status_bar.show(ui, &self.editor, lsp_on);
                });
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

fn lsp_to_suggestion(item: &CoreCompletionItem) -> Suggestion {
    let kind = match item.kind {
        CoreKind::Method => SuggestionKind::Method,
        CoreKind::Function | CoreKind::Constructor => SuggestionKind::Function,
        CoreKind::Field
        | CoreKind::Variable
        | CoreKind::Property
        | CoreKind::Value
        | CoreKind::Reference => SuggestionKind::Variable,
        CoreKind::Class
        | CoreKind::Interface
        | CoreKind::Module
        | CoreKind::Enum
        | CoreKind::Color => SuggestionKind::Type,
        CoreKind::Keyword => SuggestionKind::Keyword,
        CoreKind::Snippet => SuggestionKind::Snippet,
        CoreKind::File => SuggestionKind::Import,
        CoreKind::Text | CoreKind::Unit => SuggestionKind::Keyword,
    };
    let mut desc = item.detail.clone().unwrap_or_default();
    if let Some(doc) = &item.documentation {
        let first: String = doc.chars().take(120).collect();
        if !desc.is_empty() {
            desc.push_str(" — ");
        }
        desc.push_str(&first);
    }
    Suggestion {
        text: item.insert_text.clone().unwrap_or_else(|| item.label.clone()),
        kind,
        confidence: 0.85,
        description: desc,
    }
}

fn render_completion_popup(
    completion: &mut CompletionProvider,
    editor: &mut CodeEditor,
    ui: &mut egui::Ui,
) {
    if completion.is_visible() && completion.render_popup(ui).is_some() {
        if let Some(text) = completion.accept_completion() {
            editor.insert_text(&text);
        }
    }
}
