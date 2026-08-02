// Phobos IDE - Advanced Python Language Support
// Complete implementation with syntax patterns, keywords, and highlighting

use crate::languages::LanguageSupport;

pub struct PythonLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    builtins: Vec<&'static str>,
    decorators: Vec<&'static str>,
}

impl PythonLang {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "def", "class", "if", "elif", "else", "while", "for", "in", "return", "break",
                "continue", "pass", "try", "except", "finally", "raise", "with", "as", "import",
                "from", "global", "nonlocal", "lambda", "yield", "async", "await", "assert",
                "del", "is", "not", "and", "or", "True", "False", "None",
            ],
            types: vec![
                "int", "float", "str", "bool", "list", "tuple", "dict", "set", "frozenset",
                "bytes", "bytearray", "complex", "range", "slice", "type", "object",
            ],
            builtins: vec![
                "print", "len", "range", "enumerate", "zip", "map", "filter", "reduce",
                "sorted", "reversed", "sum", "min", "max", "abs", "round", "any", "all",
                "isinstance", "issubclass", "hasattr", "getattr", "setattr", "delattr",
                "open", "input", "format", "repr", "str", "int", "float", "list", "dict",
                "tuple", "set", "chr", "ord", "hex", "oct", "bin", "pow", "divmod",
            ],
            decorators: vec![
                "staticmethod", "classmethod", "property", "abstractmethod",
                "dataclass", "cached_property", "lru_cache",
            ],
        }
    }
}

impl LanguageSupport for PythonLang {
    fn name(&self) -> &str {
        "Python"
    }

    fn extensions(&self) -> Vec<&str> {
        vec!["py", "pyw", "pyi", "pyx"]
    }

    fn syntax_highlight(&self, code: &str) -> String {
        let mut result = String::new();
        
        for line in code.lines() {
            let highlighted_line = self.highlight_line(line);
            result.push_str(&highlighted_line);
            result.push('\n');
        }
        
        result
    }
}

impl PythonLang {
    fn highlight_line(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut in_triple_string = false;
        let mut in_comment = false;
        let mut current_word = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        // Check for triple quotes
        if line.contains("\"\"\"") || line.contains("'''") {
            in_triple_string = true;
        }
        
        while i < chars.len() {
            let c = chars[i];
            
            // Handle comments (outside strings)
            if !in_string && !in_triple_string && c == '#' {
                result.push_str(&format!("<span class=\"comment\">{}</span>", &line[i..]));
                break;
            }
            
            // Handle triple-quoted strings
            if i + 2 < chars.len() && &chars[i..=i+2] == &['"', '"', '"'] {
                in_triple_string = !in_triple_string;
                result.push_str("\"\"\"");
                i += 3;
                continue;
            }
            if i + 2 < chars.len() && &chars[i..=i+2] == &['\'', '\'', '\''] {
                in_triple_string = !in_triple_string;
                result.push_str("'''");
                i += 3;
                continue;
            }
            
            if in_triple_string {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle regular strings
            if c == '"' || c == '\'' {
                in_string = !in_string;
                result.push(c);
                i += 1;
                continue;
            }
            
            if in_string {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle identifiers and keywords
            if c.is_alphabetic() || c == '_' {
                current_word.push(c);
                i += 1;
                continue;
            }
            
            // Word boundary - check if we have a keyword/type/builtin
            if !current_word.is_empty() {
                let word = current_word.as_str();
                if self.keywords.contains(&word) {
                    result.push_str(&format!("<span class=\"keyword\">{}</span>", word));
                } else if self.types.contains(&word) {
                    result.push_str(&format!("<span class=\"type\">{}</span>", word));
                } else if self.builtins.contains(&word) {
                    result.push_str(&format!("<span class=\"builtin\">{}</span>", word));
                } else if self.decorators.contains(&word) {
                    result.push_str(&format!("<span class=\"decorator\">@{}</span>", word));
                } else {
                    result.push_str(word);
                }
                current_word.clear();
            }
            
            result.push(c);
            i += 1;
        }
        
        // Handle remaining word
        if !current_word.is_empty() {
            let word = current_word.as_str();
            if self.keywords.contains(&word) {
                result.push_str(&format!("<span class=\"keyword\">{}</span>", word));
            } else if self.types.contains(&word) {
                result.push_str(&format!("<span class=\"type\">{}</span>", word));
            } else if self.builtins.contains(&word) {
                result.push_str(&format!("<span class=\"builtin\">{}</span>", word));
            } else {
                result.push_str(word);
            }
        }
        
        result
    }
}
