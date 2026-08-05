//! Phobos IDE - Syntax Highlighting Module
//! Tree-sitter based syntax highlighting with theme support

use egui::Color32;

/// Color theme for syntax highlighting
#[derive(Debug, Clone)]
pub struct HighlightTheme {
    pub name: String,
    pub keyword: Color32,
    pub function: Color32,
    pub string: Color32,
    pub comment: Color32,
    pub number: Color32,
    pub type_name: Color32,
    pub variable: Color32,
    pub operator: Color32,
    pub punctuation: Color32,
}

impl HighlightTheme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            keyword: Color32::from_rgb(86, 156, 214),
            function: Color32::from_rgb(220, 220, 170),
            string: Color32::from_rgb(206, 145, 120),
            comment: Color32::GRAY,
            number: Color32::from_rgb(181, 206, 168),
            type_name: Color32::from_rgb(78, 201, 176),
            variable: Color32::WHITE,
            operator: Color32::from_rgb(212, 212, 212),
            punctuation: Color32::from_rgb(212, 212, 212),
        }
    }
    
    /// Light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            keyword: Color32::from_rgb(0, 0, 193),
            function: Color32::from_rgb(100, 100, 50),
            string: Color32::from_rgb(193, 100, 50),
            comment: Color32::from_rgb(128, 128, 128),
            number: Color32::from_rgb(0, 128, 0),
            type_name: Color32::from_rgb(0, 128, 128),
            variable: Color32::BLACK,
            operator: Color32::from_rgb(50, 50, 50),
            punctuation: Color32::from_rgb(50, 50, 50),
        }
    }
    
    /// Monokai theme
    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            keyword: Color32::from_rgb(249, 38, 114),
            function: Color32::from_rgb(166, 226, 46),
            string: Color32::from_rgb(230, 219, 116),
            comment: Color32::from_rgb(117, 113, 94),
            number: Color32::from_rgb(174, 129, 255),
            type_name: Color32::from_rgb(102, 217, 239),
            variable: Color32::from_rgb(248, 248, 242),
            operator: Color32::from_rgb(249, 38, 114),
            punctuation: Color32::from_rgb(248, 248, 242),
        }
    }
}

/// Legacy SyntaxHighlighter for backward compatibility
pub struct SyntaxHighlighter {
    theme: HighlightTheme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            theme: HighlightTheme::dark(),
        }
    }
    
    pub fn get_extension(&self, lang: &str) -> &'static str {
        match lang.to_lowercase().as_str() {
            "asm" | "assembly" => "asm",
            "c" => "c",
            "cpp" | "c++" | "cxx" => "cpp",
            "cs" | "csharp" | "c#" => "cs",
            "rust" | "rs" => "rs",
            "go" => "go",
            "python" | "py" => "py",
            "java" => "java",
            "javascript" | "js" => "js",
            "typescript" | "ts" => "ts",
            "php" => "php",
            "ruby" | "rb" => "rb",
            "sql" => "sql",
            "pascal" | "pas" | "delphi" => "pas",
            "swift" => "swift",
            "kotlin" | "kt" => "kt",
            "shell" | "bash" | "sh" => "bash",
            "html" => "html",
            "css" => "css",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "markdown" | "md" => "md",
            "dockerfile" | "docker" => "dockerfile",
            "toml" => "toml",
            "xml" => "xml",
            _ => "txt",
        }
    }

    pub fn language_list() -> Vec<(&'static str, &'static str)> {
        vec![
            ("🟣 Assembly", "asm"), ("🔵 C", "c"), ("🔵 C++", "cpp"),
            ("🟣 C#", "cs"), ("🟠 Rust", "rust"), ("🟦 Go", "go"),
            ("🟡 Python", "py"), ("☕ Java", "java"),
            ("🟨 JavaScript", "js"), ("🟦 TypeScript", "ts"),
            ("🐘 PHP", "php"), ("💎 Ruby", "rb"),
            ("🗄️ SQL", "sql"), ("🏛️ Pascal", "pas"),
            ("🕊️ Swift", "swift"), ("🟣 Kotlin", "kt"),
            ("🐚 Shell", "sh"), ("🌐 HTML", "html"),
            ("🎨 CSS", "css"), ("📋 JSON", "json"),
            ("📝 YAML", "yaml"), ("📘 Markdown", "md"),
            ("🐳 Dockerfile", "dockerfile"), ("⚙️ TOML", "toml"),
        ]
    }
    
    pub fn set_theme(&mut self, theme: HighlightTheme) {
        self.theme = theme;
    }
    
    pub fn theme(&self) -> &HighlightTheme {
        &self.theme
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
