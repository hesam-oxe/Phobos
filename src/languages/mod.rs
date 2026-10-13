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
        Self {
            languages: vec![
                Box::new(rust::RustLang::new()),
                Box::new(python::PythonLang::new()),
                Box::new(javascript::JavaScriptLang::new()),
                Box::new(go::GoLang::new()),
                Box::new(java::JavaLang::new()),
                Box::new(c_family::CFamilyLang::new()),
                Box::new(csharp::CSharpLang::new()),
                Box::new(assembly::AssemblyLang::new()),
                Box::new(ruby::RubyLang::new()),
                Box::new(php::PhpLang::new()),
                Box::new(sql::SqlLang::new()),
                Box::new(pascal::PascalLang::new()),
                Box::new(shell::ShellLang::new()),
                Box::new(web::WebLang::new()),
                Box::new(swift::SwiftLang::new()),
                Box::new(kotlin::KotlinLang::new()),
                Box::new(markdown::MarkdownLang::new()),
                Box::new(docker::DockerLang::new()),
                Box::new(data_langs::DataLang::new()),
            ],
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
