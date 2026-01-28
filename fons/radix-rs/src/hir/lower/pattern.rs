//! Pattern lowering
//!
//! Lowers AST patterns to HIR patterns.

use super::Lowerer;
use crate::hir::{HirLiteral, HirPattern};
use crate::lexer::Span;
use crate::syntax::{Literal, Pattern};

/// Lower a pattern
pub fn lower_pattern(lowerer: &mut Lowerer, pattern: &Pattern) -> HirPattern {
    match pattern {
        Pattern::Wildcard(span) => {
            lowerer.current_span = *span;
            HirPattern::Wildcard
        }
        Pattern::Ident(ident, bind) => {
            lowerer.current_span = ident.span;
            if bind.is_some() {
                lowerer.error("pattern bindings are not lowered yet");
            }
            let def_id = lowerer.def_id_for(ident.name);
            HirPattern::Binding(def_id, ident.name)
        }
        Pattern::Literal(lit, span) => lower_literal(lowerer, lit, *span),
        Pattern::Path(path) => {
            lowerer.current_span = path.span;
            if path.bind.is_some() {
                lowerer.error("path pattern bindings are not lowered yet");
            }
            let Some(last) = path.segments.last() else {
                lowerer.error("empty path pattern");
                return HirPattern::Wildcard;
            };
            let def_id = lowerer.def_id_for(last.name);
            HirPattern::Variant(def_id, Vec::new())
        }
    }
}

pub fn lower_literal(lowerer: &mut Lowerer, lit: &Literal, span: Span) -> HirPattern {
    lowerer.current_span = span;
    let literal = match lit {
        Literal::Integer(value) => HirLiteral::Int(*value),
        Literal::Float(value) => HirLiteral::Float(*value),
        Literal::String(value) => HirLiteral::String(*value),
        Literal::Bool(value) => HirLiteral::Bool(*value),
        Literal::Nil => HirLiteral::Nil,
        _ => {
            lowerer.error("unsupported literal pattern");
            return HirPattern::Wildcard;
        }
    };

    HirPattern::Literal(literal)
}

impl<'a> Lowerer<'a> {
    /// Lower omissis (wildcard) pattern
    pub fn lower_omissis(&mut self) -> HirPattern {
        HirPattern::Wildcard
    }

    /// Lower nomen (identifier) pattern
    pub fn lower_nomen_pattern(&mut self, ident: &crate::syntax::Ident) -> HirPattern {
        let def_id = self.def_id_for(ident.name);
        HirPattern::Binding(def_id, ident.name)
    }
}
