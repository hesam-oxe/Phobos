// Phobos IDE - Ghost Typer AI Module
// Intelligent code completion and suggestion system

use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub kind: SuggestionKind,
    pub confidence: f32,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionKind {
    Keyword,
    Function,
    Variable,
    Type,
    Method,
    Snippet,
    Import,
}

pub struct GhostTyper {
    language: String,
    context_window: usize,
    suggestions_cache: HashMap<String, Vec<Suggestion>>,
    patterns: HashMap<String, Vec<String>>,
}

impl GhostTyper {
    pub fn new(language: &str) -> Self {
        let mut typer = Self {
            language: language.to_string(),
            context_window: 50,
            suggestions_cache: HashMap::new(),
            patterns: HashMap::new(),
        };
        typer.initialize_patterns();
        typer
    }

    fn initialize_patterns(&mut self) {
        match self.language.as_str() {
            "rust" => {
                self.patterns.insert("fn_".to_string(), vec![
                    "fn main() {\n    $0\n}".to_string(),
                    "fn $1($2) -> $3 {\n    $0\n}".to_string(),
                    "pub fn $1($2: $3) -> Result<$4, $5> {\n    $0\n}".to_string(),
                ]);
                self.patterns.insert("impl_".to_string(), vec![
                    "impl $1 {\n    fn $2($3) -> $4 {\n        $0\n    }\n}".to_string(),
                    "impl $1 for $2 {\n    fn $3($4) -> $5 {\n        $0\n    }\n}".to_string(),
                ]);
                self.patterns.insert("struct_".to_string(), vec![
                    "struct $1 {\n    $2: $3,\n}".to_string(),
                    "pub struct $1<'a> {\n    pub $2: &'a $3,\n}".to_string(),
                ]);
                self.patterns.insert("match_".to_string(), vec![
                    "match $1 {\n    $2 => $3,\n    _ => $0,\n}".to_string(),
                ]);
                self.patterns.insert("for_".to_string(), vec![
                    "for $1 in $2 {\n    $0\n}".to_string(),
                ]);
                self.patterns.insert("if_".to_string(), vec![
                    "if $1 {\n    $0\n}".to_string(),
                    "if let Some($1) = $2 {\n    $0\n}".to_string(),
                ]);
                self.patterns.insert("let_".to_string(), vec![
                    "let $1 = $2;".to_string(),
                    "let mut $1 = $2;".to_string(),
                    "let $1: $2 = $3;".to_string(),
                ]);
            }
            "python" => {
                self.patterns.insert("def_".to_string(), vec![
                    "def $1($2):\n    $0".to_string(),
                    "async def $1($2):\n    $0".to_string(),
                ]);
                self.patterns.insert("class_".to_string(), vec![
                    "class $1:\n    def __init__(self, $2):\n        $0".to_string(),
                ]);
                self.patterns.insert("for_".to_string(), vec![
                    "for $1 in $2:\n    $0".to_string(),
                ]);
                self.patterns.insert("if_".to_string(), vec![
                    "if $1:\n    $0".to_string(),
                ]);
                self.patterns.insert("try_".to_string(), vec![
                    "try:\n    $0\nexcept Exception as e:\n    print(e)".to_string(),
                ]);
            }
            "cpp" | "c" => {
                self.patterns.insert("main_".to_string(), vec![
                    "int main(int argc, char* argv[]) {\n    $0\n    return 0;\n}".to_string(),
                ]);
                self.patterns.insert("class_".to_string(), vec![
                    "class $1 {\npublic:\n    $1();\n    ~$1();\nprivate:\n    $0\n};".to_string(),
                ]);
                self.patterns.insert("for_".to_string(), vec![
                    "for (int $1 = 0; $1 < $2; ++$1) {\n    $0\n}".to_string(),
                ]);
                self.patterns.insert("if_".to_string(), vec![
                    "if ($1) {\n    $0\n}".to_string(),
                ]);
            }
            _ => {}
        }
    }

    pub fn get_suggestions(&mut self, code: &str, cursor_pos: usize) -> Vec<Suggestion> {
        let before_cursor = &code[..cursor_pos.min(code.len())];
        let current_line = before_cursor.lines().last().unwrap_or("");
        
        // Check cache first
        if let Some(cached) = self.suggestions_cache.get(current_line) {
            return cached.clone();
        }

        let mut suggestions = Vec::new();

        // Pattern-based suggestions
        for (pattern, snippets) in &self.patterns {
            if current_line.ends_with(pattern.as_str()) || 
               current_line.contains(pattern.as_str()) {
                for snippet in snippets {
                    suggestions.push(Suggestion {
                        text: snippet.replace("$0", "").replace("$1", "item").replace("$2", "arg"),
                        kind: SuggestionKind::Snippet,
                        confidence: 0.9,
                        description: format!("{} snippet", pattern),
                    });
                }
            }
        }

        // Context-aware keyword suggestions
        suggestions.extend(self.get_keyword_suggestions(current_line));
        
        // Smart completions based on code context
        suggestions.extend(self.get_context_completions(before_cursor));

        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Cache results
        self.suggestions_cache.insert(current_line.to_string(), suggestions.clone());

        suggestions
    }

    fn get_keyword_suggestions(&self, line: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let keywords = match self.language.as_str() {
            "rust" => vec!["fn", "let", "mut", "if", "else", "match", "for", "while", "loop", "return", "break", "continue", "struct", "enum", "trait", "impl", "use", "mod", "pub", "async", "await"],
            "python" => vec!["def", "class", "if", "elif", "else", "for", "while", "return", "break", "continue", "import", "from", "try", "except", "finally", "with", "as", "lambda", "yield"],
            "cpp" | "c" => vec!["if", "else", "for", "while", "do", "switch", "case", "break", "continue", "return", "struct", "class", "template", "namespace", "using", "typedef"],
            "javascript" | "typescript" => vec!["function", "const", "let", "var", "if", "else", "for", "while", "return", "class", "import", "export", "async", "await", "try", "catch", "finally"],
            "java" => vec!["public", "private", "protected", "static", "final", "class", "interface", "extends", "implements", "if", "else", "for", "while", "return", "try", "catch", "finally", "throw", "throws"],
            _ => vec![],
        };

        for keyword in keywords {
            suggestions.push(Suggestion {
                text: keyword.to_string(),
                kind: SuggestionKind::Keyword,
                confidence: 0.7,
                description: format!("Keyword: {}", keyword),
            });
        }

        suggestions
    }

    fn get_context_completions(&self, before_cursor: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // Detect function calls
        if let Some(last_word) = self.extract_last_identifier(before_cursor) {
            if before_cursor.ends_with(&format!("{}.", last_word)) {
                // Method completion
                suggestions.push(Suggestion {
                    text: format!("method($0)".to_string()),
                    kind: SuggestionKind::Method,
                    confidence: 0.6,
                    description: format!("Method of {}", last_word),
                });
            }
        }

        suggestions
    }

    fn extract_last_identifier(&self, text: &str) -> Option<String> {
        let re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)$").ok()?;
        re.captures(text).map(|caps| caps[1].to_string())
    }

    pub fn accept_suggestion(&mut self, code: &str, suggestion: &Suggestion, cursor_pos: usize) -> String {
        let mut result = code.to_string();
        result.insert_str(cursor_pos, &suggestion.text);
        result
    }

    pub fn clear_cache(&mut self) {
        self.suggestions_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_snippets() {
        let mut typer = GhostTyper::new("rust");
        let suggestions = typer.get_suggestions("fn_", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.kind == SuggestionKind::Snippet));
    }

    #[test]
    fn test_keyword_suggestions() {
        let typer = GhostTyper::new("rust");
        let suggestions = typer.get_suggestions("", 0);
        assert!(suggestions.iter().any(|s| s.kind == SuggestionKind::Keyword));
    }
}
