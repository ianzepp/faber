//! Canonical Faber source emission for analyzed HIR.
//!
//! This backend is a source-preserving pretty-printer in the semantic sense: it
//! emits Faber source text from analyzed HIR while preserving names, types,
//! declarations, expressions, and control-flow structure that survived lowering.
//! Unlike the Rust, Go, and TypeScript backends, it does not translate Faber
//! into a host runtime model. Its output boundary is another Faber source string,
//! used by formatter-style tooling, canonicalization, and round-trip validation.
//!
//! BOUNDARY
//! ========
//! - Input: a fully analyzed [`HirProgram`], its [`TypeTable`], and the shared
//!   [`Interner`] that owns source spellings.
//! - Output: [`FaberOutput`], containing canonical Faber text only.
//! - HIR recovery markers are rejected at the backend boundary through
//!   `reject_hir_errors`; this printer should not invent syntax for malformed
//!   programs.
//! - Type syntax is derived from the supplied [`TypeTable`] instead of guessed
//!   from expression shape. Missing or degraded type information remains an
//!   upstream semantic issue, not a codegen policy choice.
//!
//! OUTPUT POLICY
//! =============
//! The printer aims for grammar-valid canonical spellings and layout, not the
//! original file. HIR does not retain comments or author formatting, so this
//! backend cannot provide byte-for-byte preservation. Parentheses, operator
//! spellings, type forms, and block layout are chosen to keep the generated
//! source parseable and semantically recognizable after the original surface
//! tokens have been normalized away. Known grammar gaps, such as the current
//! nullable type spelling in `types.rs`, are documented at the local policy
//! surface instead of hidden behind a stronger module-level promise.
//!
//! TRADE-OFFS
//! ==========
//! - Comments, blank-line intent, and original wrapping are not available in HIR.
//! - Some surface distinctions collapse during analysis; the printer chooses the
//!   canonical Faber spelling for the HIR variant it receives.
//! - DefId-based references require an upfront name collection pass before the
//!   single emission pass can write source spellings.

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
        // WHY: emitting fallback text for recovery HIR would make invalid input
        // look canonical. The frontend owns diagnostics; this boundary only
        // prints programs that analysis proved coherent enough to represent.
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
