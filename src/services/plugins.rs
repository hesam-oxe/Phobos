//! Phobos IDE - Plugin System (marketplace + local install)
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub description: String,
    pub external: bool,
}

pub struct PluginManager {
    pub plugins: HashMap<String, PluginInfo>,
}

impl PluginManager {
    pub fn new() -> Self { Self { plugins: HashMap::new() } }

    fn plugins_dir() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("phobos").join("plugins"))
    }

    fn state_path() -> Option<std::path::PathBuf> {
        Self::plugins_dir().map(|d| d.join("state.toml"))
    }

    /// Built-in + recommended catalog shown in the marketplace.
    pub fn catalog() -> Vec<PluginInfo> {
        let mut v = vec![
            ("phobos.git", "Git Integration", "1.0.0", "Branch, diff, log, blame"),
            ("phobos.lsp", "Language Servers", "1.0.0", "rust-analyzer, pylsp, gopls, clangd, tsserver"),
            ("phobos.ai", "AI Assistant", "1.0.0", "Ghost typer, bug finder, explainer"),
            ("phobos.hex", "Hex + Disasm", "1.0.0", "Hex editor, iced-x86 disassembler"),
            ("phobos.debug", "Debugger", "1.0.0", "GDB/MI, pdb, delve, DAP adapters"),
            ("phobos.term", "Terminal", "1.0.0", "Full PTY terminal"),
        ]
        .into_iter()
        .map(|(id, name, ver, desc)| PluginInfo {
            id: id.into(), name: name.into(), version: ver.into(),
            enabled: true, description: desc.into(), external: false,
        })
        .collect::<Vec<_>>();
        v.push(PluginInfo {
            id: "phobos.themes".into(), name: "Extra Themes".into(),
            version: "0.1.0".into(), enabled: false,
            description: "Community color themes (TOML)".into(), external: true,
        });
        v.push(PluginInfo {
            id: "phobos.snippets".into(), name: "Snippet Pack".into(),
            version: "0.1.0".into(), enabled: false,
            description: "Extra completion snippets (TOML)".into(), external: true,
        });
        v
    }

    pub fn discover(&mut self) {
        for p in Self::catalog().into_iter().filter(|p| !p.external) {
            self.plugins.insert(p.id.clone(), p);
        }
        // External: scan ~/.config/phobos/plugins/*.toml
        if let Some(dir) = Self::plugins_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.extension().map(|x| x == "toml").unwrap_or(false)
                        && p.file_name().map(|n| n != "state.toml").unwrap_or(true)
                    {
                        if let Ok(s) = std::fs::read_to_string(&p) {
                            if let Ok(v) = toml::from_str::<toml::Value>(&s) {
                                let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("unknown").to_string();
                                self.plugins.insert(id.clone(), PluginInfo {
                                    id,
                                    name: v.get("name").and_then(|x| x.as_str()).unwrap_or("ext").into(),
                                    version: v.get("version").and_then(|x| x.as_str()).unwrap_or("0.1").into(),
                                    enabled: v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true),
                                    description: v.get("description").and_then(|x| x.as_str()).unwrap_or("external").into(),
                                    external: true,
                                });
                            }
                        }
                    }
                }
            }
        }
        self.apply_state();
    }

    fn load_disabled(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        if let Some(p) = Self::state_path() {
            if let Ok(s) = std::fs::read_to_string(&p) {
                if let Ok(v) = toml::from_str::<toml::Value>(&s) {
                    if let Some(arr) = v.get("disabled").and_then(|x| x.as_array()) {
                        for x in arr.iter().filter_map(|x| x.as_str()) {
                            set.insert(x.to_string());
                        }
                    }
                }
            }
        }
        set
    }

    fn apply_state(&mut self) {
        let disabled = self.load_disabled();
        for id in &disabled {
            if let Some(p) = self.plugins.get_mut(id) {
                p.enabled = false;
            }
        }
    }

    fn save_state(&self) {
        let disabled: Vec<&str> = self.plugins.values()
            .filter(|p| !p.enabled)
            .map(|p| p.id.as_str())
            .collect();
        if let Some(p) = Self::state_path() {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let val = toml::Value::Table({
                let mut t = toml::map::Map::new();
                t.insert("disabled".into(), toml::Value::Array(
                    disabled.into_iter().map(|s| toml::Value::String(s.into())).collect(),
                ));
                t
            });
            if let Ok(s) = toml::to_string_pretty(&val) {
                let _ = std::fs::write(&p, s);
            }
        }
    }

    /// Install an external plugin from TOML text. Returns the plugin id.
    pub fn install_str(&mut self, name: &str, toml_text: &str) -> anyhow::Result<String> {
        let v: toml::Value = toml::from_str(toml_text)?;
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or(name).to_string();
        if id.is_empty() || id.contains(|c: char| c.is_whitespace() || c == '/') {
            anyhow::bail!("invalid plugin id");
        }
        let dir = Self::plugins_dir().ok_or_else(|| anyhow::anyhow!("no config dir"))?;
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join(format!("{}.toml", id)), toml_text)?;
        self.plugins.insert(id.clone(), PluginInfo {
            id: id.clone(),
            name: v.get("name").and_then(|x| x.as_str()).unwrap_or(&id).to_string(),
            version: v.get("version").and_then(|x| x.as_str()).unwrap_or("0.1").to_string(),
            enabled: v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true),
            description: v.get("description").and_then(|x| x.as_str()).unwrap_or("external").to_string(),
            external: true,
        });
        self.apply_state();
        Ok(id)
    }

    pub fn remove_external(&mut self, id: &str) -> bool {
        let is_ext = self.plugins.get(id).map(|p| p.external).unwrap_or(false);
        if !is_ext {
            return false;
        }
        self.plugins.remove(id);
        if let Some(dir) = Self::plugins_dir() {
            let _ = std::fs::remove_file(dir.join(format!("{}.toml", id)));
        }
        true
    }

    /// Installed plugins plus not-yet-installed catalog entries, filtered.
    pub fn search(&self, q: &str) -> Vec<PluginInfo> {
        let mut out: Vec<PluginInfo> =
            self.plugins.values().cloned().collect();
        for c in Self::catalog().into_iter().filter(|c| c.external) {
            if !self.plugins.contains_key(&c.id) {
                out.push(c);
            }
        }
        let q = q.to_lowercase();
        out.retain(|p| {
            q.is_empty()
                || p.id.to_lowercase().contains(&q)
                || p.name.to_lowercase().contains(&q)
                || p.description.to_lowercase().contains(&q)
        });
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(p) = self.plugins.get_mut(id) {
            p.enabled = !p.enabled;
        }
        self.save_state();
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(p) = self.plugins.get_mut(id) {
            p.enabled = enabled;
        }
        self.save_state();
    }

    pub fn enabled(&self, id: &str) -> bool {
        self.plugins.get(id).map(|p| p.enabled).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_core_plugins() {
        let ids: Vec<String> =
            PluginManager::catalog().into_iter().map(|p| p.id).collect();
        assert!(ids.contains(&"phobos.lsp".to_string()));
        assert!(ids.contains(&"phobos.debug".to_string()));
    }

    #[test]
    fn fresh_manager_search_is_empty() {
        let m = PluginManager::new();
        assert!(m.search("").is_empty());
        assert!(!m.enabled("phobos.git"));
    }

    #[test]
    fn toggle_missing_is_noop() {
        let mut m = PluginManager::new();
        m.toggle("nope");
        m.set_enabled("nope", true);
        assert!(m.remove_external("nope") == false);
    }
}
