//! Phobos IDE - Terminal Service Module

pub mod shell;

pub use shell::{create_terminal, AnsiParser, TerminalOutput, TerminalShell};
