//! Phobos IDE - AI Provider (OpenAI-compatible chat, blocking I/O for worker threads)
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChatConfig {
    /// Base URL, e.g. `https://api.openai.com/v1` (Ollama: `http://localhost:11434/v1`)
    pub endpoint: String,
    /// Kept in memory only — never written to disk by Phobos.
    pub api_key: String,
    pub model: String,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMsg>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMsg {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: RespMsg,
}

#[derive(Deserialize)]
struct RespMsg {
    content: String,
}

/// One-shot chat completion. Blocks the calling (worker) thread.
pub fn chat_complete(cfg: &ChatConfig, system: &str, user: &str) -> Result<String> {
    if cfg.api_key.trim().is_empty() {
        anyhow::bail!("API key is empty — set it in the AI panel (kept in memory only)");
    }
    let url = format!("{}/chat/completions", cfg.endpoint.trim_end_matches('/'));
    let req = ChatRequest {
        model: &cfg.model,
        messages: vec![
            ChatMsg { role: "system", content: system.to_string() },
            ChatMsg { role: "user", content: user.to_string() },
        ],
        temperature: 0.2,
        max_tokens: 800,
    };
    let resp: ChatResponse = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", cfg.api_key.trim()))
        .set("Content-Type", "application/json")
        .send_json(&req)
        .map_err(|e| anyhow::anyhow!("AI request failed: {}", e))?
        .into_json()
        .context("bad AI response JSON")?;
    resp.choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .context("empty AI response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_key_errors_without_network() {
        let cfg = ChatConfig::default();
        let r = chat_complete(&cfg, "sys", "hi");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("API key"));
    }
}
