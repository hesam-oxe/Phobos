// Phobos IDE - Config Module: persistent settings (TOML)
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhobosConfig {
    pub theme: String,
    pub font_size: u16,
    pub tab_size: usize,
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub show_line_numbers: bool,
    pub show_minimap: bool,
    pub word_wrap: bool,
    pub bracket_matching: bool,
    pub lsp_enabled: bool,
    pub recent_files: Vec<String>,
    pub workspace_root: Option<String>,
}

impl Default for PhobosConfig {
    fn default() -> Self {
        Self {
            theme: "dark".into(), font_size: 14, tab_size: 4,
            auto_save: true, auto_save_delay_ms: 1000,
            show_line_numbers: true, show_minimap: true,
            word_wrap: true, bracket_matching: true, lsp_enabled: true,
            recent_files: Vec::new(), workspace_root: None,
        }
    }
}

impl PhobosConfig {
    pub fn path() -> PathBuf {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
            .join("phobos").join("config.toml")
    }
    pub fn load() -> Self {
        let p = Self::path();
        if let Ok(s) = std::fs::read_to_string(&p) {
            if let Ok(c) = toml::from_str(&s) { return c; }
        }
        Self::default()
    }
    pub fn save(&self) -> Result<()> {
        let p = Self::path();
        if let Some(parent) = p.parent() { std::fs::create_dir_all(parent)?; }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(&p, s)?;
        Ok(())
    }
    pub fn push_recent(&mut self, file: String) {
        self.recent_files.retain(|f| f != &file);
        self.recent_files.insert(0, file);
        self.recent_files.truncate(20);
    }
}
