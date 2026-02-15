//! radix-rs - Production Faber compiler
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! radix-rs is a recursive descent compiler for the Faber programming language,
//! a Latin-based IR designed for LLM code generation. The compiler follows a
//! multi-phase pipeline:
//!
//! ```text
//! Source (.fab)
//!   → Lexer (tokens)
//!   → Parser (AST)
//!   → Semantic Analysis (HIR + type table)
//!   → Codegen (Rust or Faber pretty-print)
//! ```
//!
//! COMPILER PHASES
//! ===============
//! 1. **Lexing** (`lexer`): Tokenizes source into Latin keywords and symbols
//! 2. **Parsing** (`parser`): Builds abstract syntax tree (AST)
//! 3. **Semantic** (`semantic`): Name resolution, type checking, borrow analysis
//! 4. **HIR Lowering** (`hir`): Converts AST to high-level IR
//! 5. **Codegen** (`codegen`): Emits target source (Rust, Faber, etc.)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - **Never crash**: Collect errors, don't panic. The compiler must process
//!   malformed input gracefully and report actionable diagnostics.
//!
//! - **Multi-target**: Backends are pluggable. Rust is the primary target;
//!   Faber pretty-print enables formatting and round-tripping.
//!
//! - **Type-first syntax**: Faber uses `textus name` not `name: textus`, matching
//!   mathematical notation and reducing cognitive load for type inference.
//!
//! - **Latin vocabulary**: Keywords are Latin (functio, genus, discerne) to
//!   avoid conflicts with English reserved words in target languages.
//!
//! USAGE
//! =====
//! ```no_run
//! use radix::{Compiler, Config, Target};
//! use std::path::Path;
//!
//! let config = Config::default().with_target(Target::Rust);
//! let compiler = Compiler::new(config);
//! let result = compiler.compile(Path::new("example.fab"));
//!
//! if result.success() {
//!     // Use result.output
//! } else {
//!     for diag in &result.diagnostics {
//!         eprintln!("{:?}", diag);
//!     }
//! }
//! ```

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

// =============================================================================
// TYPES
// =============================================================================
//
// Public API types for compilation results and outputs.

/// Primary compilation result.
///
/// WHY: Bundles output and diagnostics together, allowing callers to inspect
/// warnings even when compilation succeeds.
pub struct CompileResult {
    pub output: Option<Output>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileResult {
    pub fn success(&self) -> bool {
        self.output.is_some() && !self.diagnostics.iter().any(|d| d.is_error())
    }
}

/// Compiled output for a target.
///
/// WHY: Enum allows type-safe dispatch to target-specific output formats
/// without runtime string matching or dynamic dispatch.
pub enum Output {
    Rust(RustOutput),
    Faber(FaberOutput),
}

/// Rust compilation output.
pub struct RustOutput {
    pub code: String,
}

/// Faber canonical output (pretty-printed Faber source).
pub struct FaberOutput {
    pub code: String,
}

// =============================================================================
// COMPILER API
// =============================================================================
//
// High-level API for invoking the compilation pipeline.

/// Main compiler entry point.
///
/// WHY: Encapsulates session state and provides a clean API for file or
/// string compilation. The session holds configuration that persists across
/// multiple compile calls.
pub struct Compiler {
    session: Session,
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Self { session: Session::new(config) }
    }

    /// Compile a file from disk.
    ///
    /// WHY: Convenience wrapper that handles file I/O and delegates to compile_str.
    /// I/O errors are converted to diagnostics rather than propagated as Results.
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

    /// Compile source code from a string.
    ///
    /// WHY: Core compilation path. File-based compilation delegates to this
    /// after reading the source, enabling in-memory compilation for tests.
    pub fn compile_str(&self, name: &str, source: &str) -> CompileResult {
        driver::compile(&self.session, name, source)
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "exempla_e2e_test.rs"]
mod exempla_e2e_tests;
