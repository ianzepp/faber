//! Diagnostic reporting

mod catalog;
mod diagnostic;
mod render;

pub use catalog::{lex_spec, parse_spec, semantic_spec, DiagnosticSpec};
pub use diagnostic::{Diagnostic, Severity};
pub use render::render_diagnostics;
