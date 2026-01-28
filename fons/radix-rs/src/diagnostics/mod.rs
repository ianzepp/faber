//! Diagnostic reporting

mod diagnostic;
mod catalog;
mod render;

pub use diagnostic::{Diagnostic, Severity};
pub use catalog::DiagnosticSpec;
pub use render::render_diagnostics;
