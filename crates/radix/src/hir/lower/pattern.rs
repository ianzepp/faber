//! Pattern lowering for `discerne` and other binding-sensitive forms.
//!
//! Patterns sit on a different boundary than ordinary expressions: an
//! identifier may either refer to an existing variant or introduce a fresh local
//! binding. This module makes that distinction explicit by resolving known
//! variants through the resolver while allocating new `DefId`s for names that
//! the pattern binds.
//!
//! INVARIANTS
//! ==========
//! - Fresh pattern bindings are immediately added to the lowering scope so guard
//!   and arm bodies can resolve them like ordinary locals.
//! - Variant patterns preserve the resolved variant `DefId`; constructor arity
//!   and payload typing remain typecheck responsibilities.
//! - Literal patterns lower through the same HIR literal taxonomy used by
//!   expressions, keeping match analysis and expression analysis aligned.
//! - Unsupported pattern policies still emit diagnostics and choose a recoverable
//!   HIR shape instead of aborting lowering.
//!
//! ERROR STRATEGY
//! ==============
//! Mutable pattern bindings are parsed but not semantically implemented here.
//! Lowering records the diagnostic and keeps the binding structure so later
//! phases can still report useful errors in the rest of the arm.

use super::Lowerer;
use crate::hir::{HirLiteral, HirPattern};
use crate::lexer::Span;
use crate::syntax::{Literal, Mutability, PathPattern, Pattern, PatternBind};

/// Lower a parser pattern into HIR while establishing any new bindings.
///
/// This entry point preserves the central pattern distinction: variant names
/// refer to existing definitions, while binding names allocate fresh `DefId`s
/// in the current scope. It intentionally does not validate constructor payload
/// types or exhaustiveness; those remain later analysis responsibilities.
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

fn lower_ident_pattern(lowerer: &mut Lowerer, ident: &crate::syntax::Ident, bind: Option<&PatternBind>) -> HirPattern {
    lowerer.current_span = ident.span;
    // Ident patterns are ambiguous until resolver state is consulted: a known
    // variant is a constructor pattern, otherwise the name introduces a binding.
    if let Some(def_id) = lowerer.resolver.lookup(ident.name) {
        if matches!(
            lowerer
                .resolver
                .get_symbol(def_id)
                .map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Variant)
        ) {
            return match bind {
                None => HirPattern::Variant(def_id, Vec::new()),
                Some(PatternBind::Alias(alias)) => {
                    lower_alias_pattern(lowerer, alias, HirPattern::Variant(def_id, Vec::new()))
                }
                Some(PatternBind::Bindings { mutability, names }) => {
                    if *mutability == Mutability::Mutable {
                        lowerer.error("mutable pattern bindings are not lowered yet");
                    }
                    HirPattern::Variant(def_id, lower_bindings(lowerer, names))
                }
            };
        }
    }

    let def_id = lowerer.next_def_id();
    lowerer.bind_local(ident.name, def_id);

    match bind {
        None => HirPattern::Binding(def_id, ident.name),
        Some(PatternBind::Alias(alias)) => lower_alias_pattern(lowerer, alias, HirPattern::Binding(def_id, ident.name)),
        Some(PatternBind::Bindings { mutability, names }) => {
            if *mutability == Mutability::Mutable {
                lowerer.error("mutable pattern bindings are not lowered yet");
            }
            HirPattern::Variant(def_id, lower_bindings(lowerer, names))
        }
    }
}

/// Lower a path pattern as an explicit variant reference.
///
/// Path syntax is already a reference form, so this does not allocate a binding
/// for the terminal segment. Payload names still allocate fresh locals.
fn lower_path_pattern(lowerer: &mut Lowerer, path: &PathPattern) -> HirPattern {
    lowerer.current_span = path.span;
    let Some(last) = path.segments.last() else {
        lowerer.error("empty path pattern");
        return HirPattern::Wildcard;
    };
    let def_id = lowerer.def_id_for(last.name);

    match path.bind.as_ref() {
        None => HirPattern::Variant(def_id, Vec::new()),
        Some(PatternBind::Alias(alias)) => lower_alias_pattern(lowerer, alias, HirPattern::Variant(def_id, Vec::new())),
        Some(PatternBind::Bindings { mutability, names }) => {
            if *mutability == Mutability::Mutable {
                lowerer.error("mutable pattern bindings are not lowered yet");
            }
            HirPattern::Variant(def_id, lower_bindings(lowerer, names))
        }
    }
}

/// Allocate fresh local bindings for a variant payload list.
fn lower_bindings(lowerer: &mut Lowerer, names: &[crate::syntax::Ident]) -> Vec<HirPattern> {
    names
        .iter()
        .map(|ident| {
            let def_id = lowerer.next_def_id();
            lowerer.bind_local(ident.name, def_id);
            HirPattern::Binding(def_id, ident.name)
        })
        .collect()
}

/// Bind an alias name to the whole lowered pattern shape.
///
/// The alias is a fresh local even when the wrapped pattern resolves to a
/// variant. This lets arm bodies refer to both destructured payloads and the
/// original matched value when later phases support that distinction.
fn lower_alias_pattern(lowerer: &mut Lowerer, alias: &crate::syntax::Ident, pattern: HirPattern) -> HirPattern {
    lowerer.current_span = alias.span;
    let alias_def_id = lowerer.next_def_id();
    lowerer.bind_local(alias.name, alias_def_id);
    HirPattern::Alias(alias_def_id, alias.name, Box::new(pattern))
}

/// Lower a literal pattern through the expression literal taxonomy.
pub fn lower_literal(lowerer: &mut Lowerer, lit: &Literal, span: Span) -> HirPattern {
    lowerer.current_span = span;
    let literal = match lit {
        Literal::Integer(value) => HirLiteral::Int(*value),
        Literal::Float(value) => HirLiteral::Float(*value),
        Literal::String(value) => HirLiteral::String(*value),
        Literal::Bool(value) => HirLiteral::Bool(*value),
        Literal::Nil => HirLiteral::Nil,
    };

    HirPattern::Literal(literal)
}

impl<'a> Lowerer<'a> {
    /// Lower omissis (wildcard) pattern
    #[allow(dead_code)]
    pub fn lower_omissis(&mut self) -> HirPattern {
        HirPattern::Wildcard
    }

    /// Lower nomen (identifier) pattern
    #[allow(dead_code)]
    pub fn lower_nomen_pattern(&mut self, ident: &crate::syntax::Ident) -> HirPattern {
        let def_id = self.next_def_id();
        self.bind_local(ident.name, def_id);
        HirPattern::Binding(def_id, ident.name)
    }
}
