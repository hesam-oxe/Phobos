//! Phobos IDE - LSP Service Module

pub mod client;

pub use client::{create_lsp_client, LspClient, LspServerConfig, LspState};
