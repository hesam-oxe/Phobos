//! Phobos IDE - Plugin System (god-mode extensibility)
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub description: String,
}

pub struct PluginManager {
    pub plugins: HashMap<String, PluginInfo>,
}

impl PluginManager {
    pub fn new() -> Self { Self { plugins: HashMap::new() } }

    pub fn discover(&mut self) {
        // Built-in plugins (god-mode core)
        for (id, name, desc) in [
            ("phobos.git", "Git Integration", "Branch, diff, log, blame"),
            ("phobos.lsp", "Language Servers", "rust-analyzer, pylsp, gopls, clangd, tsserver"),
            ("phobos.ai", "AI Assistant", "Ghost typer, bug finder, explainer"),
            ("phobos.hex", "Hex + Disasm", "Hex editor, iced-x86 disassembler"),
            ("phobos.debug", "Debugger", "GDB/MI, pdb, delve"),
            ("phobos.term", "Terminal", "Full PTY terminal"),
        ] {
            self.plugins.insert(id.into(), PluginInfo {
                id: id.into(), name: name.into(), version: "1.0.0".into(),
                enabled: true, description: desc.into(),
            });
        }
        // External: scan ~/.config/phobos/plugins/*.toml
        if let Some(dir) = dirs::config_dir().map(|d| d.join("phobos").join("plugins")) {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().map(|x| x=="toml").unwrap_or(false) {
                        if let Ok(s) = std::fs::read_to_string(&p) {
                            if let Ok(v) = toml::from_str::<toml::Value>(&s) {
                                let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("unknown").to_string();
                                self.plugins.insert(id.clone(), PluginInfo {
                                    id, name: v.get("name").and_then(|x| x.as_str()).unwrap_or("ext").into(),
                                    version: v.get("version").and_then(|x| x.as_str()).unwrap_or("0.1").into(),
                                    enabled: v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true),
                                    description: "external".into(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(p) = self.plugins.get_mut(id) { p.enabled = !p.enabled; }
    }
}
