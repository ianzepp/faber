//! Code generation
//!
//! Generates target code from HIR. Currently supports:
//! - Rust: Full compilation to Rust source
//! - Faber: Canonical pretty-printing

pub mod rust;
pub mod faber;
mod writer;

pub use writer::CodeWriter;

use crate::hir::HirProgram;
use crate::semantic::TypeTable;
use crate::{RustOutput, FaberOutput, CrateDep};

/// Compilation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Rust,
    Faber,
}

/// Codegen error
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

/// Codegen trait for different targets
pub trait Codegen {
    type Output;

    fn generate(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
    ) -> Result<Self::Output, CodegenError>;
}

/// Generate code for the specified target
pub fn generate(
    target: Target,
    hir: &HirProgram,
    types: &TypeTable,
    crate_name: &str,
) -> Result<crate::Output, CodegenError> {
    match target {
        Target::Rust => {
            let gen = rust::RustCodegen::new(crate_name.to_owned());
            let output = gen.generate(hir, types)?;
            Ok(crate::Output::Rust(output))
        }
        Target::Faber => {
            let gen = faber::FaberCodegen::new();
            let output = gen.generate(hir, types)?;
            Ok(crate::Output::Faber(output))
        }
    }
}
