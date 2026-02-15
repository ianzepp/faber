//! Diagnostic reporting system
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! The diagnostics module provides error and warning reporting with rich
//! context: severity levels, error codes, help text, source spans, and
//! pretty-printed output using ariadne.
//!
//! COMPILER PHASE: Cross-cutting (used by all phases)
//! INPUT: Error types from lexer, parser, semantic analyzer
//! OUTPUT: Diagnostic messages with spans and help text
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Actionable errors: Every diagnostic has a help message suggesting how to
//!   fix the problem. Error codes (e.g., SEM001, PARSE012) enable documentation
//!   lookups and suppressions.
//!
//! - Rich context: Diagnostics include source spans, file names, and source
//!   line content for precise error location.
//!
//! - Pretty printing: Uses ariadne for colored, underlined error output with
//!   caret positioning (^^^^^) under the offending span.

mod catalog;
mod diagnostic;
mod render;

pub use catalog::{lex_spec, parse_spec, semantic_spec, DiagnosticSpec};
pub use diagnostic::{Diagnostic, Severity};
pub use render::render_diagnostics;
