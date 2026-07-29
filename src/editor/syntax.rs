use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
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

    pub fn highlight_html(&self, code: &str, lang: &str) -> String {
        let ext = self.get_extension(lang);
        let syntax = self.syntax_set
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        
        syntect::html::highlighted_html_for_string(code, &self.syntax_set, syntax, theme)
            .unwrap_or_else(|_| code.to_string())
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
}
