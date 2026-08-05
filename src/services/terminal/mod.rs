//! Phobos IDE - Terminal Service Module

pub mod shell;

pub use shell::{TerminalShell, TerminalOutput, AnsiParser, create_terminal};
