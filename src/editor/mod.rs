//! Phobos IDE - Editor Module
//! Code editor widget with syntax highlighting, gutter, and LSP integration

pub mod code_editor;
pub mod syntax;
pub mod completion;
pub mod minimap;
pub mod hover;
pub mod diagnostics;

pub use code_editor::CodeEditor;
pub use syntax::{SyntaxHighlighter, HighlightTheme};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticsPanel};
