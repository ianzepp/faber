//! Code generation from HIR to target languages
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! The codegen module transforms HIR (High-level Intermediate Representation) into
//! executable source code for target languages. This is the final phase of the
//! compilation pipeline.
//!
//! COMPILER PHASE: Codegen
//! INPUT: HirProgram (semantic analysis output), TypeTable, Interner
//! OUTPUT: Target-specific source code (Rust, Faber pretty-print, or future targets)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Target abstraction: Each backend implements the `Codegen` trait, allowing
//!   uniform access regardless of target language. New backends can be added
//!   without modifying the driver logic.
//!
//! - Multi-target support: Different backends handle language-specific quirks
//!   (Rust's Result wrapping for failable functions, Faber's Latin keywords, etc.)
//!   through specialized transforms.
//!
//! - Error propagation: Failable functions (those that contain `iace` / throw
//!   expressions) are detected during Rust codegen and emit Result<T, String>
//!   signatures with automatic `?` operator insertion.
//!
//! BACKENDS
//! ========
//! - Rust: Full compilation to executable Rust code
//! - Faber: Canonical pretty-printing for formatting and round-tripping

pub mod faber;
pub mod rust;
mod writer;

pub use writer::CodeWriter;

use crate::hir::HirProgram;
use crate::lexer::Interner;
use crate::semantic::TypeTable;

// =============================================================================
// TYPES
// =============================================================================
//
// Core types for target specification and error handling.

/// Compilation target language.
///
/// WHY: Target enumeration allows the driver to select backends without
/// hardcoding backend names throughout the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
    Faber,
}

/// Code generation error.
///
/// WHY: Codegen errors are distinct from semantic errors because they occur
/// after all semantic analysis passes. They typically indicate unimplemented
/// features or internal compiler bugs rather than user code errors.
#[derive(Debug)]
pub struct CodegenError {
    pub message: String,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CodegenError {}

// =============================================================================
// CORE
// =============================================================================
//
// Main codegen trait and dispatch logic.

/// Code generation trait for different targets.
///
/// WHY: This trait enables target-agnostic driver code. Each backend provides
/// its own implementation with target-specific transforms and conventions.
pub trait Codegen {
    type Output;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<Self::Output, CodegenError>;
}

/// Generate code for the specified target.
///
/// WHY: This function provides a unified entry point for all codegen backends,
/// dispatching to the appropriate implementation based on the target enum.
///
/// TRANSFORMS: Delegates to target-specific generators that handle language
/// quirks (e.g., Rust Result wrapping, Faber Latin keywords).
pub fn generate(
    target: Target,
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
) -> Result<crate::Output, CodegenError> {
    match target {
        Target::Rust => {
            let gen = rust::RustCodegen::new(hir, interner);
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::Rust(output))
        }
        Target::Faber => {
            let gen = faber::FaberCodegen::new();
            let output = gen.generate(hir, types, interner)?;
            Ok(crate::Output::Faber(output))
        }
    }
}
