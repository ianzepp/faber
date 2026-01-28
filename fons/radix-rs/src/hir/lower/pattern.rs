//! Pattern lowering
//!
//! Lowers AST patterns to HIR patterns.

use super::Lowerer;
use crate::hir::{HirLiteral, HirPattern};
use crate::lexer::Span;
use crate::syntax::{Literal, Mutability, PathPattern, Pattern, PatternBind};

/// Lower a pattern
pub fn lower_pattern(lowerer: &mut Lowerer, pattern: &Pattern) -> HirPattern {
    match pattern {
        Pattern::Wildcard(span) => {
            lowerer.current_span = *span;
            HirPattern::Wildcard
        }
        Pattern::Ident(ident, bind) => lower_ident_pattern(lowerer, ident, bind.as_ref()),
        Pattern::Literal(lit, span) => lower_literal(lowerer, lit, *span),
        Pattern::Path(path) => lower_path_pattern(lowerer, path),
    }
}

fn lower_ident_pattern(
    lowerer: &mut Lowerer,
    ident: &crate::syntax::Ident,
    bind: Option<&PatternBind>,
) -> HirPattern {
    lowerer.current_span = ident.span;

    match bind {
        None => {
            let def_id = lowerer.def_id_for(ident.name);
            HirPattern::Binding(def_id, ident.name)
        }
        Some(PatternBind::Alias(alias)) => {
            lowerer.current_span = alias.span;
            let def_id = lowerer.def_id_for(alias.name);
            HirPattern::Binding(def_id, alias.name)
        }
        Some(PatternBind::Bindings { mutability, names }) => {
            if *mutability == Mutability::Mutable {
                lowerer.error("mutable pattern bindings are not lowered yet");
            }
            let def_id = lowerer.def_id_for(ident.name);
            HirPattern::Variant(def_id, lower_bindings(lowerer, names))
        }
    }
}

fn lower_path_pattern(lowerer: &mut Lowerer, path: &PathPattern) -> HirPattern {
    lowerer.current_span = path.span;
    let Some(last) = path.segments.last() else {
        lowerer.error("empty path pattern");
        return HirPattern::Wildcard;
    };
    let def_id = lowerer.def_id_for(last.name);

    match path.bind.as_ref() {
        None => HirPattern::Variant(def_id, Vec::new()),
        Some(PatternBind::Alias(_)) => {
            lowerer.error("pattern alias bindings are not lowered yet");
            HirPattern::Variant(def_id, Vec::new())
        }
        Some(PatternBind::Bindings { mutability, names }) => {
            if *mutability == Mutability::Mutable {
                lowerer.error("mutable pattern bindings are not lowered yet");
            }
            HirPattern::Variant(def_id, lower_bindings(lowerer, names))
        }
    }
}

fn lower_bindings(lowerer: &mut Lowerer, names: &[crate::syntax::Ident]) -> Vec<HirPattern> {
    names
        .iter()
        .map(|ident| {
            let def_id = lowerer.def_id_for(ident.name);
            HirPattern::Binding(def_id, ident.name)
        })
        .collect()
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
