pub mod assembly;
pub mod c_family;
pub mod csharp;
pub mod rust;
pub mod go;
pub mod python;
pub mod java;
pub mod javascript;
pub mod php;
pub mod ruby;
pub mod sql;
pub mod pascal;
pub mod docker;
pub mod data_langs;
pub mod markdown;
pub mod shell;
pub mod web;
pub mod swift;
pub mod kotlin;

pub trait LanguageSupport {
    fn name(&self) -> &str;
    fn extensions(&self) -> Vec<&str>;
    fn syntax_highlight(&self, code: &str) -> String;
}

pub struct LanguageRegistry {
    pub languages: Vec<Box<dyn LanguageSupport>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self { languages: Vec::new() }
    }
    
    pub fn detect(&self, filename: &str) -> Option<&str> {
        for lang in &self.languages {
            for ext in lang.extensions() {
                if filename.ends_with(ext) {
                    return Some(lang.name());
                }
            }
        }
        None
    }
}
