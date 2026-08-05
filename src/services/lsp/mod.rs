//! Phobos IDE - LSP Service Module

pub mod client;

pub use client::{LspClient, LspServerConfig, LspState, create_lsp_client};
