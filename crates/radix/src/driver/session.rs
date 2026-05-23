//! Driver configuration and per-compilation session state.
//!
//! The driver accepts a [`Session`] rather than a loose set of options so phase
//! entrypoints have one stable place to read target policy and future
//! compilation state. Today the session is intentionally small: [`Config`]
//! captures user-facing choices, while mutable caches and incremental state have
//! not yet earned a place in the contract.
//!
//! BOUNDARY
//! ========
//! `Config` describes compilation policy supplied by callers. `Session` is the
//! object passed through the driver and later phases. Keeping those concepts
//! distinct lets future work add cached module state, diagnostics sinks, or
//! package context without changing every public driver function signature.
//!
//! INVARIANTS
//! ==========
//! - Rust is the default backend because it is the primary executable target.
//! - Comments are emitted by default so generated code remains inspectable.
//! - `stdlib_path` is optional; package-mode library resolution owns the richer
//!   stdlib provider story.

use crate::codegen::Target;
use std::path::PathBuf;

/// User-facing compilation policy consumed by the driver.
///
/// The builder methods keep call sites readable while preserving a single
/// structure that can grow with optimization levels, feature gates, or target
/// capabilities.
#[derive(Debug, Clone)]
pub struct Config {
    /// Backend language selected for code generation and target policy checks.
    pub target: Target,

    /// Whether generated output should preserve explanatory comments.
    pub emit_comments: bool,

    /// Optional explicit stdlib path for callers that do not use package mode.
    pub stdlib_path: Option<PathBuf>,

    /// Treat warnings as errors when semantic phases honor strict mode.
    pub strict: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { target: Target::Rust, emit_comments: true, stdlib_path: None, strict: false }
    }
}

impl Config {
    /// Create a default configuration for ordinary Rust-target compilation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Select the backend target for this configuration.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Override the stdlib root used by callers outside package resolution.
    pub fn with_stdlib(mut self, path: PathBuf) -> Self {
        self.stdlib_path = Some(path);
        self
    }

    /// Enable strict diagnostic policy for phases that support it.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }
}

/// Per-compilation driver state.
///
/// The session currently wraps immutable config only. It exists as a deliberate
/// extension point so future cached or incremental state can travel with the
/// same driver API instead of widening phase signatures.
pub struct Session {
    /// Configuration shared by all phases of this compilation.
    pub config: Config,
}

impl Session {
    /// Create a session from caller-supplied configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
