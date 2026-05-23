//! Cross-phase diagnostic contract for the compiler.
//!
//! Diagnostics are the user-facing boundary where lexer, parser, semantic, I/O,
//! and backend failures become one stable report shape. The compiler phases own
//! the domain-specific error enums; this module owns the common vocabulary for
//! severity, codes, help text, file/span context, and terminal rendering.
//!
//! Keeping the catalog, data model, and renderer split is deliberate. The
//! catalog defines the public code/help taxonomy, `diagnostic` normalizes phase
//! errors into one transport type, and `render` is the only layer coupled to
//! ariadne's terminal output model.
//!
//! INVARIANTS
//! ==========
//! - Error codes are stable external handles, not incidental strings.
//! - Phase-specific errors are converted before rendering so output policy stays
//!   centralized.
//! - Diagnostics tolerate missing spans and source text; that degradation must
//!   produce less context, not a crash.

mod catalog;
mod diagnostic;
mod render;

pub use catalog::{lex_spec, parse_spec, semantic_spec, DiagnosticSpec};
pub use diagnostic::{Diagnostic, Severity};
pub use render::render_diagnostics;
