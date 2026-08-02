// Phobos IDE - Advanced Rust Language Support
// Complete implementation with syntax patterns, keywords, and highlighting

use crate::languages::LanguageSupport;

pub struct RustLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    macros: Vec<&'static str>,
}

impl RustLang {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "fn", "let", "mut", "const", "static", "if", "else", "match", "while", "loop",
                "for", "in", "return", "break", "continue", "struct", "enum", "union", "trait",
                "impl", "where", "type", "mod", "pub", "crate", "super", "self", "Self",
                "use", "as", "extern", "unsafe", "ref", "box", "move", "async", "await",
                "dyn", "macro", "rule", "meta", "tt", "ident", "path", "ty", "pat", "expr",
                "stmt", "item", "block", "vis", "lit", "lifetime", "label",
            ],
            types: vec![
                "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize",
                "f32", "f64", "bool", "char", "str", "String",
                "Vec", "Option", "Result", "Box", "Rc", "Arc", "RefCell",
                "HashMap", "HashSet", "BTreeMap", "BTreeSet", "LinkedList",
                "Cow", "Slice", "Array", "Tuple", "Never", "Unit",
            ],
            macros: vec![
                "println", "print", "eprintln", "eprint", "format", "formatln",
                "panic", "assert", "assert_eq", "assert_ne", "debug_assert",
                "vec", "hashmap", "todo", "unimplemented", "unreachable",
                "compile_error", "stringify", "concat", "include", "include_str",
                "include_bytes", "env", "option_env", "cfg", "cfg_attr",
                "derive", "allow", "warn", "deny", "forbid",
            ],
        }
    }
}

impl LanguageSupport for RustLang {
    fn name(&self) -> &str {
        "Rust"
    }

    fn extensions(&self) -> Vec<&str> {
        vec!["rs", "rlib"]
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

impl RustLang {
    fn highlight_line(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut in_char = false;
        let mut in_comment = false;
        let mut current_word = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            // Handle comments
            if !in_string && !in_char && i + 1 < chars.len() && &chars[i..=i+1] == &['/', '/'] {
                result.push_str(&format!("<span class=\"comment\">{}</span>", &line[i..]));
                break;
            }
            
            // Handle block comments
            if !in_string && !in_char && i + 1 < chars.len() && &chars[i..=i+1] == &['/', '*'] {
                in_comment = true;
            }
            if in_comment && i + 1 < chars.len() && &chars[i..=i+1] == &['*', '/'] {
                in_comment = false;
                result.push_str(&format!("<span class=\"comment\">*/</span>"));
                i += 2;
                continue;
            }
            if in_comment {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle strings
            if c == '"' && !in_char {
                in_string = !in_string;
                result.push(c);
                i += 1;
                continue;
            }
            
            // Handle char literals
            if c == '\'' && !in_string && (i == 0 || chars[i-1] != '\\') {
                in_char = !in_char;
                result.push(c);
                i += 1;
                continue;
            }
            
            if in_string || in_char {
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
            
            // Word boundary - check if we have a keyword/type/macro
            if !current_word.is_empty() {
                let word = current_word.as_str();
                if self.keywords.contains(&word) {
                    result.push_str(&format!("<span class=\"keyword\">{}</span>", word));
                } else if self.types.contains(&word) {
                    result.push_str(&format!("<span class=\"type\">{}</span>", word));
                } else if self.macros.contains(&word) {
                    result.push_str(&format!("<span class=\"macro\">{}!</span>", word));
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
            } else if self.macros.contains(&word) {
                result.push_str(&format!("<span class=\"macro\">{}!</span>", word));
            } else {
                result.push_str(word);
            }
        }
        
        result
    }
}
