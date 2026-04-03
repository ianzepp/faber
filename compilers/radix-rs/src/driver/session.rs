//! Compilation Session Management
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Manages compiler configuration and session state. The Config struct holds
//! user-specified compilation options (target language, strict mode, stdlib path),
//! and the Session struct tracks state during compilation.
//!
//! COMPILER PHASE: Driver (infrastructure)
//! INPUT: User-specified configuration options
//! OUTPUT: Config and Session structs for use by compilation pipeline
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Builder pattern for Config: Chainable methods for ergonomic configuration.
//!   WHY: Config::new().with_target(Rust).strict() is more readable than constructors.
//! - Minimal session state: Session currently holds only Config.
//!   WHY: Future expansion may add caching, incremental state, or diagnostics aggregation.

use crate::codegen::Target;
use std::path::PathBuf;

/// Compilation configuration.
///
/// WHY: Centralized configuration enables future expansion (optimization levels,
/// feature flags) without changing function signatures throughout the compiler.
#[derive(Debug, Clone)]
pub struct Config {
    /// Target language
    pub target: Target,

    /// Emit comments in output
    pub emit_comments: bool,

    /// Path to stdlib
    pub stdlib_path: Option<PathBuf>,

    /// Enable strict mode (all warnings are errors)
    pub strict: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { target: Target::Rust, emit_comments: true, stdlib_path: None, strict: false }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn with_stdlib(mut self, path: PathBuf) -> Self {
        self.stdlib_path = Some(path);
        self
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }
}

/// Compilation session state.
///
/// WHY: Encapsulates mutable state that may grow in the future (cached modules,
/// incremental compilation data, diagnostics buffer).
pub struct Session {
    pub config: Config,
}

impl Session {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
