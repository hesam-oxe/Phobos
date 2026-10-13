//! Phobos IDE - Editor Module
//! Code editor widget with syntax highlighting, gutter, and LSP integration

pub mod code_editor;
pub mod completion;
pub mod diagnostics;
pub mod hover;
pub mod minimap;
pub mod syntax;

pub use code_editor::CodeEditor;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticsPanel};
pub use syntax::{HighlightTheme, SyntaxHighlighter};
