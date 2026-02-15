//! radix - Recursive descent compiler for Faber
//!
//! Compiles Faber source code to Rust or canonical Faber output.

pub mod codegen;
pub mod diagnostics;
pub mod driver;
pub mod hir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod syntax;

use std::path::Path;

pub use codegen::Target;
pub use diagnostics::Diagnostic;
pub use driver::{Config, Session};

/// Primary compilation result
pub struct CompileResult {
    pub output: Option<Output>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileResult {
    pub fn success(&self) -> bool {
        self.output.is_some() && !self.diagnostics.iter().any(|d| d.is_error())
    }
}

/// Compiled output for a target
pub enum Output {
    Rust(RustOutput),
    Faber(FaberOutput),
}

/// Rust compilation output
pub struct RustOutput {
    pub code: String,
}

/// Faber canonical output
pub struct FaberOutput {
    pub code: String,
}

/// Main compiler entry point
pub struct Compiler {
    session: Session,
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Self { session: Session::new(config) }
    }

    /// Compile a file from disk
    pub fn compile(&self, path: &Path) -> CompileResult {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                return CompileResult { output: None, diagnostics: vec![Diagnostic::io_error(path, e)] };
            }
        };
        let name = path.display().to_string();
        self.compile_str(&name, &source)
    }

    /// Compile source code from a string
    pub fn compile_str(&self, name: &str, source: &str) -> CompileResult {
        driver::compile(&self.session, name, source)
    }
}
