//! Pattern lowering
//!
//! Lowers AST patterns to HIR patterns.

use super::Lowerer;
use crate::hir::HirPattern;
use crate::syntax::Pattern;

/// Lower a pattern
pub fn lower_pattern(lowerer: &mut Lowerer, pattern: &Pattern) -> HirPattern {
    match pattern {
        Pattern::Wildcard(_) => HirPattern::Wildcard,
        Pattern::Ident(ident, bind) => {
            // TODO: Handle binding
            HirPattern::Wildcard
        }
        Pattern::Literal(lit, _) => {
            // TODO: Convert to HirLiteral
            HirPattern::Wildcard
        }
        Pattern::Path(path) => {
            // TODO: Resolve path and create variant pattern
            HirPattern::Wildcard
        }
    }
}

impl<'a> Lowerer<'a> {
    /// Lower omissis (wildcard) pattern
    pub fn lower_omissis(&mut self) -> HirPattern {
        HirPattern::Wildcard
    }

    /// Lower nomen (identifier) pattern
    pub fn lower_nomen_pattern(&mut self, ident: &crate::syntax::Ident) -> HirPattern {
        // TODO: Create binding pattern with DefId
        HirPattern::Wildcard
    }
}
