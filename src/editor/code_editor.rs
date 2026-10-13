//! Phobos IDE - Code Editor Widget
//! Custom editor wrapping egui::TextEdit with syntax highlighting, gutter, and LSP integration

use crate::core::{CompletionItem, LspDiagnostic};
use eframe::egui;
use egui::{Color32, FontId, Rect, Response, Sense, TextFormat};
use std::sync::Arc;
use tree_sitter::{Language, Parser, Query, QueryCursor};

/// Syntax-highlighted text format
#[derive(Debug, Clone)]
pub struct FormatRange {
    pub start: usize,
    pub end: usize,
    pub color: Color32,
    pub bold: bool,
    pub italic: bool,
}

/// Gutter marker types
#[derive(Debug, Clone)]
pub enum GutterMarker {
    /// Line number only
    Number(usize),
    /// Error indicator (red dot)
    Error,
    /// Warning indicator (yellow dot)
    Warning,
    /// Info indicator (blue dot)
    Info,
    /// Breakpoint (red circle)
    Breakpoint,
    /// Current execution line (arrow)
    ExecutionLine,
}

/// Editor configuration
#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub show_line_numbers: bool,
    pub tab_size: usize,
    pub font_size: f32,
    pub word_wrap: bool,
    pub bracket_matching: bool,
    pub auto_indent: bool,
    pub highlight_current_line: bool,
    pub render_whitespace: bool,
    pub minimap_enabled: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            tab_size: 4,
            font_size: 14.0,
            word_wrap: true,
            bracket_matching: true,
            auto_indent: true,
            highlight_current_line: true,
            render_whitespace: false,
            minimap_enabled: false,
        }
    }
}

/// Main Code Editor widget
pub struct CodeEditor {
    content: String,
    language: String,
    config: EditorConfig,

    // Tree-sitter parser and highlighter
    parser: Option<Parser>,
    language_grammar: Option<Language>,
    highlights_query: Option<Query>,

    // Cached syntax highlights (computed on background thread)
    cached_highlights: Vec<FormatRange>,
    highlights_version: u64,
    content_version: u64,

    // Gutter markers from diagnostics
    gutter_markers: Vec<Vec<GutterMarker>>, // per-line markers

    // Cursor and selection
    cursor_position: usize,
    selection_range: Option<(usize, usize)>,

    // Viewport state
    scroll_offset_y: f32,
    visible_lines: std::ops::Range<usize>,

    // LSP diagnostics for this file
    diagnostics: Vec<LspDiagnostic>,

    // Completions popup state
    show_completions: bool,
    completion_items: Vec<CompletionItem>,
    selected_completion: usize,

    // Set when the user hits the save shortcut; polled by app.rs
    save_requested: bool,
}

impl CodeEditor {
    /// Create a new code editor
    pub fn new() -> Self {
        Self {
            content: String::from("// 💀 Phobos IDE — The God of Fear\n// Type some code to see syntax highlighting...\n\nfn main() {\n    println!(\"Phobos crushes VS Code!\");\n}\n"),
            language: "rust".to_string(),
            config: EditorConfig::default(),
            parser: None,
            language_grammar: None,
            highlights_query: None,
            cached_highlights: Vec::new(),
            highlights_version: 0,
            content_version: 0,
            gutter_markers: Vec::new(),
            cursor_position: 0,
            selection_range: None,
            scroll_offset_y: 0.0,
            visible_lines: 0..0,
            diagnostics: Vec::new(),
            show_completions: false,
            completion_items: Vec::new(),
            selected_completion: 0,
            save_requested: false,
        }
    }

    /// Set the programming language for syntax highlighting
    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
        self.initialize_parser(language);
    }

    /// Initialize Tree-sitter parser for a language
    fn initialize_parser(&mut self, language: &str) {
        let mut parser = Parser::new();

        // Load language grammar based on language name
        let lang_result = match language {
            "rust" | "rs" => Some(tree_sitter_rust::language()),
            "python" | "py" => Some(tree_sitter_python::language()),
            "javascript" | "js" => Some(tree_sitter_javascript::language()),
            "typescript" | "ts" => Some(tree_sitter_typescript::language_tsx()),
            "cpp" | "c++" => Some(tree_sitter_cpp::language()),
            "java" => Some(tree_sitter_java::language()),
            "go" => Some(tree_sitter_go::language()),
            "ruby" | "rb" => Some(tree_sitter_ruby::language()),
            "php" => Some(tree_sitter_php::language()),
            "csharp" | "cs" | "c#" => Some(tree_sitter_c_sharp::language()),
            "bash" | "sh" => Some(tree_sitter_bash::language()),
            "json" => Some(tree_sitter_json::language()),
            // YAML disabled until tree-sitter stack upgrade (see Cargo.toml)
            "yaml" | "yml" => None,
            "html" => Some(tree_sitter_html::language()),
            "css" => Some(tree_sitter_css::language()),
            _ => None,
        };

        if let Some(lang) = lang_result {
            if parser.set_language(lang).is_ok() {
                self.parser = Some(parser);
                self.language_grammar = Some(lang);

                // TODO: Load highlights query from language-specific queries
                // For now, we'll use basic regex-based highlighting as fallback
            }
        }
    }

    /// Update content and trigger re-highlighting
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.content_version += 1;
        self.recompute_highlights();
    }

    /// Get current content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Current language id (e.g. "rust")
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Cursor byte offset (clamped to a char boundary by writers)
    pub fn cursor_offset(&self) -> usize {
        self.cursor_position
    }

    /// Mutable access to editor configuration
    pub fn config_mut(&mut self) -> &mut EditorConfig {
        &mut self.config
    }

    /// True if the user pressed the save shortcut inside the editor.
    /// app.rs polls this after show() and performs the actual save.
    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.save_requested)
    }

    /// Update diagnostics and gutter markers
    pub fn set_diagnostics(&mut self, diagnostics: Vec<LspDiagnostic>) {
        self.diagnostics = diagnostics;
        self.update_gutter_markers();
    }

    /// Update gutter markers based on diagnostics
    fn update_gutter_markers(&mut self) {
        let line_count = self.content.lines().count();
        self.gutter_markers = vec![Vec::new(); line_count];

        for diag in &self.diagnostics {
            let line_idx = diag.line as usize;
            if line_idx < self.gutter_markers.len() {
                let marker = match diag.severity {
                    crate::core::DiagnosticSeverity::Error => GutterMarker::Error,
                    crate::core::DiagnosticSeverity::Warning => GutterMarker::Warning,
                    crate::core::DiagnosticSeverity::Information
                    | crate::core::DiagnosticSeverity::Hint => GutterMarker::Info,
                };
                self.gutter_markers[line_idx].push(marker);
            }
        }
    }

    /// Trigger syntax highlighting recomputation
    fn recompute_highlights(&mut self) {
        self.highlights_version = self.content_version;

        // In production, this would be done on a background thread
        // and results sent back via channel
        let code = self.content.clone();
        self.cached_highlights = self.compute_syntax_highlights(&code);
    }

    /// Compute syntax highlights using Tree-sitter
    fn compute_syntax_highlights(&mut self, code: &str) -> Vec<FormatRange> {
        let mut highlights = Vec::new();

        if let (Some(parser), Some(_lang)) = (&mut self.parser, &self.language_grammar) {
            if let Some(tree) = parser.parse(code, None) {
                let root = tree.root_node();

                // Walk the tree and collect highlighted ranges
                // This is simplified - real implementation would use QueryCursor
                Self::collect_highlights_recursive(&root, code, &mut highlights);
            }
        } else {
            // Fallback: basic keyword highlighting
            highlights.extend(self.basic_keyword_highlighting(code));
        }

        highlights
    }

    /// Recursively collect syntax highlights from Tree-sitter nodes
    fn collect_highlights_recursive(
        node: &tree_sitter::Node,
        source: &str,
        highlights: &mut Vec<FormatRange>,
    ) {
        let node_type = node.kind();
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();

        // Map node types to colors
        let color = match node_type {
            "comment" | "line_comment" | "block_comment" => Color32::GRAY,
            "string" | "string_literal" | "raw_string" => Color32::from_rgb(206, 145, 120),
            "character" | "char_literal" => Color32::from_rgb(206, 145, 120),
            "number" | "integer_literal" | "float_literal" => Color32::from_rgb(181, 206, 168),
            "keyword" | "type_identifier" => Color32::from_rgb(86, 156, 214),
            "function" | "function_definition" | "call_expression" => {
                Color32::from_rgb(220, 220, 170)
            }
            "identifier" => Color32::WHITE,
            "operator" => Color32::from_rgb(212, 212, 212),
            "punctuation" | "delimiter" => Color32::from_rgb(212, 212, 212),
            _ => Color32::WHITE,
        };

        if start_byte < source.len() && end_byte <= source.len() {
            highlights.push(FormatRange {
                start: start_byte,
                end: end_byte,
                color,
                bold: matches!(node_type, "keyword"),
                italic: matches!(node_type, "comment" | "line_comment"),
            });
        }

        // Process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_highlights_recursive(&child, source, highlights);
        }
    }

    /// Fallback keyword-based highlighting
    fn basic_keyword_highlighting(&self, code: &str) -> Vec<FormatRange> {
        let mut highlights = Vec::new();

        // Simple regex-like matching for common keywords
        let keywords = [
            "fn", "let", "mut", "if", "else", "for", "while", "return", "struct", "enum", "impl",
            "trait", "use", "mod", "pub", "const", "static", "ref", "match", "async", "await",
        ];

        for keyword in &keywords {
            let mut start = 0;
            while let Some(pos) = code[start..].find(keyword) {
                let actual_pos = start + pos;

                // Check word boundaries
                let before = actual_pos.checked_sub(1).and_then(|i| code.chars().nth(i));
                let after = code[actual_pos + keyword.len()..].chars().next();

                if before.map_or(true, |c| !c.is_alphanumeric())
                    && after.map_or(true, |c| !c.is_alphanumeric())
                {
                    highlights.push(FormatRange {
                        start: actual_pos,
                        end: actual_pos + keyword.len(),
                        color: Color32::from_rgb(86, 156, 214),
                        bold: true,
                        italic: false,
                    });
                }

                start = actual_pos + 1;
            }
        }

        // Highlight strings
        let mut in_string = false;
        let mut string_start = 0;
        let mut escaped = false;

        for (i, c) in code.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            if c == '\\' && in_string {
                escaped = true;
                continue;
            }

            if c == '"' {
                if in_string {
                    highlights.push(FormatRange {
                        start: string_start,
                        end: i + 1,
                        color: Color32::from_rgb(206, 145, 120),
                        bold: false,
                        italic: false,
                    });
                    in_string = false;
                } else {
                    string_start = i;
                    in_string = true;
                }
            }
        }

        // Highlight comments
        for line in code.lines() {
            if let Some(comment_start) = line.find("//") {
                let line_start = code.as_ptr() as usize - line.as_ptr() as usize;
                highlights.push(FormatRange {
                    start: line_start + comment_start,
                    end: line_start + line.len(),
                    color: Color32::GRAY,
                    bold: false,
                    italic: true,
                });
            }
        }

        highlights
    }

    /// Render the editor widget
    pub fn show(&mut self, ui: &mut egui::Ui) -> Response {
        let font_id = FontId::monospace(self.config.font_size);
        let line_height = ui.fonts(|f| f.row_height(&font_id));
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, ' '));

        let available = ui.available_rect_before_wrap();
        let gutter_width = if self.config.show_line_numbers {
            let line_count = self.content.lines().count();
            let digits = line_count.max(1).to_string().len();
            (digits.max(3) as f32) * char_width + 20.0
        } else {
            0.0
        };

        let editor_rect =
            Rect::from_min_max(available.min + egui::vec2(gutter_width, 0.0), available.max);

        // Draw gutter
        if self.config.show_line_numbers {
            self.render_gutter(ui, gutter_width, available.height(), line_height);
        }

        // Draw editor area
        let response = ui
            .allocate_ui_at_rect(editor_rect, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_editor_content(ui, font_id, line_height, char_width);
                    });
            })
            .response;

        // Handle keyboard input
        self.handle_keyboard_input(ui, &response);

        response
    }

    /// Render the gutter with line numbers and markers
    fn render_gutter(&self, ui: &mut egui::Ui, width: f32, height: f32, line_height: f32) {
        let painter = ui.painter();
        let gutter_rect = Rect::from_min_size(
            ui.available_rect_before_wrap().min,
            egui::vec2(width, height),
        );

        // Background
        painter.rect_filled(
            gutter_rect,
            egui::Rounding::ZERO,
            Color32::from_rgb(30, 30, 30),
        );

        let line_count = self.content.lines().count().max(1);
        let digits = line_count.to_string().len();
        let char_width =
            ui.fonts(|f| f.glyph_width(&FontId::monospace(self.config.font_size), ' '));

        for line_idx in 0..line_count {
            let y = gutter_rect.min.y + line_idx as f32 * line_height;

            // Draw diagnostic markers
            if line_idx < self.gutter_markers.len() {
                let mut x_offset = (digits as f32) * char_width + 5.0;
                for marker in &self.gutter_markers[line_idx] {
                    let color = match marker {
                        GutterMarker::Error => Color32::RED,
                        GutterMarker::Warning => Color32::YELLOW,
                        GutterMarker::Info => Color32::BLUE,
                        _ => Color32::GRAY,
                    };

                    painter.circle_filled(
                        egui::pos2(gutter_rect.min.x + x_offset, y + line_height / 2.0),
                        3.0,
                        color,
                    );
                    x_offset += 8.0;
                }
            }

            // Draw line number
            let line_num = (line_idx + 1).to_string();
            painter.text(
                egui::pos2(gutter_rect.min.x + 5.0, y),
                egui::Align2::LEFT_TOP,
                line_num,
                FontId::monospace(self.config.font_size),
                Color32::from_rgb(100, 100, 100),
            );
        }
    }

    /// Render the main editor content
    fn render_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        font_id: FontId,
        line_height: f32,
        char_width: f32,
    ) {
        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let mut layout_job = egui::text::LayoutJob::default();

            // Apply syntax highlighting
            for highlight in &self.cached_highlights {
                if highlight.end <= text.len() {
                    let format = TextFormat {
                        font_id: font_id.clone(),
                        color: highlight.color,
                        italics: highlight.italic,
                        underline: egui::Stroke::NONE,
                        strikethrough: egui::Stroke::NONE,
                        ..Default::default()
                    };
                    layout_job.append(&text[highlight.start..highlight.end], 0.0, format);
                }
            }

            ui.fonts(|f| f.layout_job(layout_job))
        };

        let output = egui::TextEdit::multiline(&mut self.content)
            .font(font_id.clone())
            .code_editor()
            .desired_rows(30)
            .desired_width(f32::INFINITY)
            .layouter(&mut layouter)
            .show(ui);
        let response = output.response;

        if response.changed() {
            self.content_version += 1;
            self.recompute_highlights();
        }

        // Track cursor from the TextEdit output (char index -> byte offset)
        if let Some(range) = output.cursor_range {
            let target = range.primary.ccursor.index;
            let byte_offset = self
                .content
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(self.content.len()))
                .nth(target)
                .unwrap_or(self.content.len());
            self.cursor_position = byte_offset;
        }
    }

    /// Handle keyboard shortcuts
    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui, response: &Response) {
        if !response.has_focus() {
            return;
        }

        let input = ui.input(|i| i.clone());

        // Ctrl+S - Save (polled by app.rs via take_save_request)
        if input.key_pressed(egui::Key::S) && input.modifiers.ctrl {
            self.save_requested = true;
        }

        // Tab - Insert spaces
        if input.key_pressed(egui::Key::Tab) {
            let indent = " ".repeat(self.config.tab_size);
            self.content.insert_str(self.cursor_position, &indent);
            self.cursor_position += self.config.tab_size;
        }
    }

    /// Get current cursor line and column
    pub fn get_cursor_position(&self) -> (usize, usize) {
        let prefix = &self.content[..self.cursor_position.min(self.content.len())];
        let line = prefix.lines().count();
        let col = prefix.lines().last().map(|l| l.len()).unwrap_or(0);
        (line, col)
    }

    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        self.content_version += 1;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
            self.content_version += 1;
        }
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new()
    }
}
