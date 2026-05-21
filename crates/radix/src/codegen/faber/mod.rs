//! Faber Canonical Code Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module implements a canonical pretty-printer for Faber source code. It
//! converts HIR back into well-formatted Faber source text, enabling formatting,
//! normalization, round-trip compilation validation, and AST visualization.
//!
//! COMPILER PHASE: Codegen
//! INPUT: HirProgram (fully-analyzed HIR), TypeTable, Interner
//! OUTPUT: FaberOutput (formatted Faber source text)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Canonical form: Generate a single normalized representation for any given HIR.
//!   WHY: Consistent formatting across all Faber code, enables reliable diff comparisons.
//! - Round-trip fidelity: Preserve all semantic information from the HIR.
//!   WHY: Parser(Codegen(HIR)) should produce equivalent HIR for validation testing.
//! - Minimal whitespace: Generate readable but compact output.
//!   WHY: Balance human readability with efficient storage and transmission.
//!
//! TRADE-OFFS
//! ==========
//! - Comments and original formatting are lost (HIR doesn't preserve them).
//! - Generates Latin keywords only; no target-language interop in this backend.
//! - DefId resolution requires building a names map upfront (single-pass generation).

use super::{CodeWriter, Codegen, CodegenError};
use crate::hir::HirProgram;
use crate::lexer::Interner;
use crate::semantic::TypeTable;
use crate::FaberOutput;

#[cfg(test)]
use crate::hir::DefId;
#[cfg(test)]
use crate::semantic::{Primitive, Type};

// =============================================================================
// CORE
// =============================================================================
//
// The FaberCodegen struct is stateless; all name resolution is performed during
// generation by building a DefId -> Symbol map from the HIR. This enables
// parallel code generation if needed in the future.

/// Faber canonical code generator.
///
/// WHY: Separate from RustCodegen to maintain clean separation between Faber
/// pretty-printing (for tooling/formatting) and target code generation.
pub struct FaberCodegen;

impl FaberCodegen {
    pub fn new() -> Self {
        Self
    }
}

mod decl;
mod expr;
mod literal;
mod names;
mod ops;
mod pattern;
mod stmt;
mod types;

impl Default for FaberCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen for FaberCodegen {
    type Output = FaberOutput;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<FaberOutput, CodegenError> {
        super::reject_hir_errors(hir)?;

        let mut w = CodeWriter::new();
        let names = self.collect_names(hir);

        for item in &hir.items {
            self.generate_item(item, types, &names, interner, &mut w)?;
            w.newline();
        }

        if let Some(entry) = &hir.entry {
            w.writeln("incipit {");
            w.indented(|w| self.write_block(entry, types, &names, interner, w));
            w.writeln("}");
        }

        Ok(FaberOutput { code: w.finish() })
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
