//! AST declaration collection for the semantic front end.
//!
//! `collect` is the first semantic pass after parsing. It performs a shallow
//! scan of top-level AST statements and reserves [`DefId`](crate::hir::DefId)
//! entries in the resolver for declarations that later phases must be able to
//! reference. It intentionally does not inspect function bodies, lower type
//! expressions, or infer values; its job is to make forward references possible
//! and make duplicate top-level names unambiguous before resolution starts.
//!
//! INVARIANTS
//! ==========
//! - The input is still parser AST; HIR does not exist yet.
//! - Every accepted top-level declaration receives exactly one resolver symbol.
//! - Duplicate names in the current resolver scope are hard semantic errors.
//! - Enum and union variants are registered as named symbols so pattern
//!   resolution can validate variant spelling without typechecking.
//! - Imported names bind the user-visible name: an alias when present, otherwise
//!   the imported name itself.
//!
//! ERROR STRATEGY
//! ==============
//! The pass accumulates duplicate-definition and invalid-top-level-binding
//! diagnostics instead of stopping at the first failure. Later semantic phases
//! should not run after collect errors because ambiguous or missing `DefId`s
//! would make name resolution unreliable.

use crate::semantic::{Resolver, SemanticError, SemanticErrorKind, Symbol, SymbolKind, TypeTable};
use crate::syntax::{Program, StmtKind};

/// Register top-level AST declarations in the resolver before name resolution.
///
/// This pass establishes the global symbol set that makes same-file forward
/// references legal. The `TypeTable` parameter is intentionally unused here:
/// collect creates identities, while resolve/typecheck are responsible for
/// lowering type expressions and assigning type information to symbols.
pub fn collect(program: &Program, resolver: &mut Resolver, _types: &mut TypeTable) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();

    for stmt in &program.stmts {
        match &stmt.kind {
            StmtKind::Var(decl) => {
                if let crate::syntax::BindingPattern::Ident(ident) = &decl.binding {
                    define_symbol(resolver, &mut errors, ident.name, ident.span, SymbolKind::Local);
                } else {
                    errors.push(SemanticError::new(
                        SemanticErrorKind::LoweringError,
                        "top-level variable requires an identifier binding",
                        stmt.span,
                    ));
                }
            }
            StmtKind::Func(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::Function);
            }
            StmtKind::Class(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::Struct);
            }
            StmtKind::Interface(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::Interface);
            }
            StmtKind::TypeAlias(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::TypeAlias);
            }
            StmtKind::Enum(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::Enum);
                for member in &decl.members {
                    define_symbol(resolver, &mut errors, member.name.name, member.name.span, SymbolKind::Variant);
                }
            }
            StmtKind::Union(decl) => {
                define_symbol(resolver, &mut errors, decl.name.name, decl.name.span, SymbolKind::Enum);
                for variant in &decl.variants {
                    define_symbol(resolver, &mut errors, variant.name.name, variant.name.span, SymbolKind::Variant);
                }
            }
            StmtKind::Import(decl) => match &decl.kind {
                crate::syntax::ImportKind::Named { name, alias } => {
                    let binding = alias.as_ref().unwrap_or(name);
                    define_import_symbol(resolver, &mut errors, binding.name, binding.span);
                }
                crate::syntax::ImportKind::Wildcard { alias } => {
                    define_import_symbol(resolver, &mut errors, alias.name, alias.span);
                }
            },
            _ => {}
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn define_import_symbol(
    resolver: &mut Resolver,
    errors: &mut Vec<SemanticError>,
    name: crate::lexer::Symbol,
    span: crate::lexer::Span,
) {
    // Package assembly can pre-seed module imports. Re-collecting the same
    // module binding should remain idempotent, while user declarations still
    // collide normally through `define_symbol`.
    if let Some(existing) = resolver.lookup(name) {
        if matches!(resolver.get_symbol(existing), Some(symbol) if symbol.kind == SymbolKind::Module) {
            return;
        }
    }

    define_symbol(resolver, errors, name, span, SymbolKind::Module);
}

fn define_symbol(
    resolver: &mut Resolver,
    errors: &mut Vec<SemanticError>,
    name: crate::lexer::Symbol,
    span: crate::lexer::Span,
    kind: SymbolKind,
) {
    let def_id = resolver.fresh_def_id();
    let symbol = Symbol { def_id, name, kind, ty: None, mutable: false, span };

    if resolver.define(symbol).is_err() {
        errors.push(SemanticError::new(
            SemanticErrorKind::DuplicateDefinition,
            "duplicate definition",
            span,
        ));
    }
}

#[cfg(test)]
#[path = "collect_test.rs"]
mod tests;
