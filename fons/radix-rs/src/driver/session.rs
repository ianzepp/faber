//! Compilation session

use crate::codegen::Target;
use std::path::PathBuf;

/// Compilation configuration
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

/// Compilation session state
pub struct Session {
    pub config: Config,
}

impl Session {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
