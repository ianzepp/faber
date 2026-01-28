//! Pass 1: Collect declarations
//!
//! Walks the AST and registers all top-level declarations,
//! creating DefIds for each named item. Does not look inside
//! function bodies.

use crate::semantic::{Resolver, SemanticError, SemanticErrorKind, Symbol, SymbolKind, TypeTable};
use crate::syntax::{Program, StmtKind};

/// Collect all declarations from the program
pub fn collect(
    program: &Program,
    resolver: &mut Resolver,
    _types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();

    for stmt in &program.stmts {
        match &stmt.kind {
            StmtKind::Var(decl) => {
                if let crate::syntax::BindingPattern::Ident(ident) = &decl.binding {
                    define_symbol(
                        resolver,
                        &mut errors,
                        ident.name,
                        ident.span,
                        SymbolKind::Local,
                    );
                } else {
                    errors.push(SemanticError::new(
                        SemanticErrorKind::LoweringError,
                        "top-level variable requires an identifier binding",
                        stmt.span,
                    ));
                }
            }
            StmtKind::Func(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Function,
                );
            }
            StmtKind::Class(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Struct,
                );
            }
            StmtKind::Interface(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Interface,
                );
            }
            StmtKind::TypeAlias(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::TypeAlias,
                );
            }
            StmtKind::Enum(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Enum,
                );
            }
            StmtKind::Union(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Enum,
                );
            }
            StmtKind::Import(decl) => match &decl.kind {
                crate::syntax::ImportKind::Named { name, alias } => {
                    let binding = alias.as_ref().unwrap_or(name);
                    define_symbol(
                        resolver,
                        &mut errors,
                        binding.name,
                        binding.span,
                        SymbolKind::Module,
                    );
                }
                crate::syntax::ImportKind::Wildcard { alias } => {
                    define_symbol(
                        resolver,
                        &mut errors,
                        alias.name,
                        alias.span,
                        SymbolKind::Module,
                    );
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

fn define_symbol(
    resolver: &mut Resolver,
    errors: &mut Vec<SemanticError>,
    name: crate::lexer::Symbol,
    span: crate::lexer::Span,
    kind: SymbolKind,
) {
    let def_id = resolver.fresh_def_id();
    let symbol = Symbol {
        def_id,
        name,
        kind,
        ty: None,
        mutable: false,
        span,
    };

    if resolver.define(symbol).is_err() {
        errors.push(SemanticError::new(
            SemanticErrorKind::DuplicateDefinition,
            "duplicate definition",
            span,
        ));
    }
}
