//! Pass 1: Collect declarations
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! First pass of semantic analysis that scans the AST and registers all
//! top-level declarations in the symbol table. Creates DefIds for named items
//! without analyzing bodies or resolving references.
//!
//! COMPILER PHASE: Semantic (Pass 1)
//! INPUT: AST (syntax::Program) from parser
//! OUTPUT: Populated Resolver symbol table with DefIds; errors for duplicates
//!
//! WHY: Forward references are allowed in Faber (unlike C), so declarations
//! must be collected before resolution. This enables `functio a() { b() }` to
//! reference `functio b()` defined later in the file.
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Shallow Scan: Only processes top-level declarations; function bodies are
//!   analyzed in the resolve pass after all names are available
//! - Duplicate Detection: Immediately detects duplicate definitions at the
//!   global scope, preventing ambiguous references in later passes
//! - Variant Registration: Enum and union variants are registered as separate
//!   definitions, enabling pattern matching resolution
//!
//! EDGE CASES
//! ==========
//! - Top-level variables: Must have identifier bindings (not patterns) since
//!   they become module-level constants
//! - Imports: Register the bound name (alias if provided, otherwise original
//!   name) for subsequent resolution

use crate::semantic::{Resolver, SemanticError, SemanticErrorKind, Symbol, SymbolKind, TypeTable};
use crate::syntax::{Program, StmtKind};

/// Collect all declarations from the program
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
                    define_symbol(resolver, &mut errors, binding.name, binding.span, SymbolKind::Module);
                }
                crate::syntax::ImportKind::Wildcard { alias } => {
                    define_symbol(resolver, &mut errors, alias.name, alias.span, SymbolKind::Module);
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
