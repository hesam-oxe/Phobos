// Phobos IDE - Go Language Support
use crate::languages::LanguageSupport;

pub struct GoLang {
    keywords: Vec<&'static str>,
    types: Vec<&'static str>,
    builtins: Vec<&'static str>,
}

impl GoLang {
    pub fn new() -> Self {
        Self {
            keywords: vec!["package","import","func","var","const","type","struct","interface","map","chan","go","select","case","default","if","else","switch","for","range","return","break","continue","fallthrough","goto","defer","panic","recover","make","new","len","cap","append","copy","delete","nil","true","false","iota"],
            types: vec!["int","int8","int16","int32","int64","uint","uint8","uint16","uint32","uint64","uintptr","float32","float64","complex64","complex128","bool","string","byte","rune","error","any"],
            builtins: vec!["println","print","panic","recover","close","closed","len","cap","new","make","append","copy","delete","complex","real","imag","clear","min","max"],
        }
    }
    fn highlight_line(&self, line: &str) -> String {
        let mut out = String::new();
        let mut in_str = false; let mut cur = String::new();
        let chars: Vec<char> = line.chars().collect(); let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if !in_str && i+1 < chars.len() && chars[i]=='/' && chars[i+1]=='/' {
                out.push_str(&format!("<span class=\"comment\">{}</span>", &line[i..])); break;
            }
            if c=='"' && (i==0 || chars[i-1]!='\\') { in_str=!in_str; out.push(c); i+=1; continue; }
            if in_str { out.push(c); i+=1; continue; }
            if c.is_alphabetic()||c=='_' { cur.push(c); i+=1; continue; }
            if !cur.is_empty() {
                let w = cur.as_str();
                if self.keywords.contains(&w) { out.push_str(&format!("<span class=\"keyword\">{}</span>", w)); }
                else if self.types.contains(&w) { out.push_str(&format!("<span class=\"type\">{}</span>", w)); }
                else if self.builtins.contains(&w) { out.push_str(&format!("<span class=\"builtin\">{}</span>", w)); }
                else { out.push_str(w); }
                cur.clear();
            }
            out.push(c); i+=1;
        }
        if !cur.is_empty() {
            let w = cur.as_str();
            if self.keywords.contains(&w) { out.push_str(&format!("<span class=\"keyword\">{}</span>", w)); }
            else { out.push_str(w); }
        }
        out
    }
}

impl LanguageSupport for GoLang {
    fn name(&self) -> &str { "Go" }
    fn extensions(&self) -> Vec<&str> { vec!["go"] }
    fn syntax_highlight(&self, code: &str) -> String {
        code.lines().map(|l| self.highlight_line(l)).collect::<Vec<_>>().join("\n") + "\n"
    }
}
