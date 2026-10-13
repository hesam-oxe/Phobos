pub mod assembly;
pub mod c_family;
pub mod csharp;
pub mod data_langs;
pub mod docker;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod markdown;
pub mod pascal;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod shell;
pub mod sql;
pub mod swift;
pub mod web;

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
        Self {
            languages: Vec::new(),
        }
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
