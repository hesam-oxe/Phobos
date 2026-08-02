// Phobos IDE - Advanced JavaScript Language Support
// Complete implementation with syntax patterns, keywords, and highlighting

use crate::languages::LanguageSupport;

pub struct JavaScriptLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    builtins: Vec<&'static str>,
    operators: Vec<&'static str>,
}

impl JavaScriptLang {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "var", "let", "const", "function", "return", "if", "else", "switch", "case",
                "default", "break", "continue", "for", "while", "do", "try", "catch", "finally",
                "throw", "new", "this", "class", "extends", "constructor", "super", "static",
                "async", "await", "yield", "import", "export", "from", "as", "typeof", "instanceof",
                "in", "of", "delete", "void", "debugger", "with",
            ],
            types: vec![
                "Object", "Array", "String", "Number", "Boolean", "Function", "Symbol", "BigInt",
                "Map", "Set", "WeakMap", "WeakSet", "Promise", "Proxy", "Reflect", "Error",
                "TypeError", "ReferenceError", "SyntaxError", "RangeError",
            ],
            builtins: vec![
                "console", "document", "window", "navigator", "location", "history", "screen",
                "localStorage", "sessionStorage", "fetch", "setTimeout", "setInterval",
                "clearTimeout", "clearInterval", "requestAnimationFrame", "alert", "confirm",
                "prompt", "JSON", "Math", "Date", "RegExp", "ArrayBuffer", "Blob", "File",
                "FormData", "URL", "URLSearchParams", "Headers", "Request", "Response",
            ],
            operators: vec![
                "==", "===", "!=", "!==", "<", ">", "<=", ">=", "&&", "||", "!", "??",
                "+", "-", "*", "/", "%", "**", "++", "--", "=", "+=", "-=", "*=", "/=",
            ],
        }
    }
}

impl LanguageSupport for JavaScriptLang {
    fn name(&self) -> &str {
        "JavaScript"
    }

    fn extensions(&self) -> Vec<&str> {
        vec!["js", "mjs", "cjs"]
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

impl JavaScriptLang {
    fn highlight_line(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut in_template = false;
        let mut in_comment = false;
        let mut in_regex = false;
        let mut current_word = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            // Handle single-line comments
            if !in_string && !in_template && !in_regex && i + 1 < chars.len() 
               && &chars[i..=i+1] == &['/', '/'] {
                result.push_str(&format!("<span class=\"comment\">{}</span>", &line[i..]));
                break;
            }
            
            // Handle multi-line comments start
            if !in_string && !in_template && !in_regex && i + 1 < chars.len() 
               && &chars[i..=i+1] == &['/', '*'] {
                in_comment = true;
            }
            // Handle multi-line comments end
            if in_comment && i + 1 < chars.len() && &chars[i..=i+1] == &['*', '/'] {
                in_comment = false;
                result.push_str("<span class=\"comment\">*/</span>");
                i += 2;
                continue;
            }
            if in_comment {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle template literals
            if c == '`' && !in_string {
                in_template = !in_template;
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle regular strings
            if (c == '"' || c == '\'') && !in_template && (i == 0 || chars[i-1] != '\\') {
                in_string = !in_string;
                result.push(c);
                i += 1;
                continue;
            }
            
            if in_string || in_template {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle regex literals (simplified detection)
            if c == '/' && !in_regex && (i == 0 || " \t\n\r=([,!&|?:;{".contains(chars[i-1])) {
                in_regex = true;
                result.push(c);
                i += 1;
                continue;
            }
            if in_regex {
                if c == '/' && chars[i-1] != '\\' {
                    in_regex = false;
                }
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle identifiers and keywords
            if c.is_alphabetic() || c == '_' || c == '$' {
                current_word.push(c);
                i += 1;
                continue;
            }
            
            // Handle numbers
            if c.is_numeric() || (c == '.' && i + 1 < chars.len() && chars[i+1].is_numeric()) {
                let mut num = String::new();
                while i < chars.len() && (chars[i].is_numeric() || chars[i] == '.' || chars[i] == 'e' || chars[i] == 'E' || chars[i] == 'x') {
                    num.push(chars[i]);
                    i += 1;
                }
                result.push_str(&format!("<span class=\"number\">{}</span>", num));
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
