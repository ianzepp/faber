//! Diagnostic reporting

mod diagnostic;
mod render;

pub use diagnostic::{Diagnostic, Severity};
pub use render::render_diagnostics;
