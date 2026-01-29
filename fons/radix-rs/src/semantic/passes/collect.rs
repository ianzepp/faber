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
                for member in &decl.members {
                    define_symbol(
                        resolver,
                        &mut errors,
                        member.name.name,
                        member.name.span,
                        SymbolKind::Variant,
                    );
                }
            }
            StmtKind::Union(decl) => {
                define_symbol(
                    resolver,
                    &mut errors,
                    decl.name.name,
                    decl.name.span,
                    SymbolKind::Enum,
                );
                for variant in &decl.variants {
                    define_symbol(
                        resolver,
                        &mut errors,
                        variant.name.name,
                        variant.name.span,
                        SymbolKind::Variant,
                    );
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Interner, Span, Symbol};
    use crate::syntax::{
        BlockStmt, EnumDecl, EnumMember, FuncDecl, Mutability, Program, Stmt, StmtKind, TypeExpr,
        TypeExprKind, VarDecl,
    };

    fn ident(interner: &mut Interner, name: &str) -> crate::syntax::Ident {
        crate::syntax::Ident {
            name: interner.intern(name),
            span: Span::default(),
        }
    }

    fn ident_sym(sym: Symbol) -> crate::syntax::Ident {
        crate::syntax::Ident {
            name: sym,
            span: Span::default(),
        }
    }

    fn stmt(kind: StmtKind) -> Stmt {
        Stmt {
            id: 0,
            kind,
            span: Span::default(),
            annotations: Vec::new(),
        }
    }

    fn program(stmts: Vec<Stmt>) -> Program {
        Program {
            directives: Vec::new(),
            stmts,
            span: Span::default(),
        }
    }

    #[test]
    fn reports_duplicate_definitions() {
        let mut interner = Interner::new();
        let sym = interner.intern("x");
        let var = VarDecl {
            mutability: Mutability::Immutable,
            is_await: false,
            ty: None,
            binding: crate::syntax::BindingPattern::Ident(ident_sym(sym)),
            init: None,
        };
        let var2 = VarDecl {
            mutability: Mutability::Immutable,
            is_await: false,
            ty: None,
            binding: crate::syntax::BindingPattern::Ident(ident_sym(sym)),
            init: None,
        };
        let program = program(vec![stmt(StmtKind::Var(var)), stmt(StmtKind::Var(var2))]);

        let mut resolver = Resolver::new();
        let mut types = TypeTable::new();
        let result = collect(&program, &mut resolver, &mut types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::DuplicateDefinition));
    }

    #[test]
    fn top_level_var_requires_identifier_binding() {
        let program = program(vec![stmt(StmtKind::Var(VarDecl {
            mutability: Mutability::Immutable,
            is_await: false,
            ty: None,
            binding: crate::syntax::BindingPattern::Wildcard(Span::default()),
            init: None,
        }))]);

        let mut resolver = Resolver::new();
        let mut types = TypeTable::new();
        let result = collect(&program, &mut resolver, &mut types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::LoweringError));
    }

    #[test]
    fn collects_enum_variants() {
        let mut interner = Interner::new();
        let enum_name = ident(&mut interner, "Color");
        let red = EnumMember {
            name: ident(&mut interner, "Red"),
            value: None,
            span: Span::default(),
        };
        let green = EnumMember {
            name: ident(&mut interner, "Green"),
            value: None,
            span: Span::default(),
        };
        let program = program(vec![stmt(StmtKind::Enum(EnumDecl {
            name: enum_name,
            members: vec![red, green],
        }))]);

        let mut resolver = Resolver::new();
        let mut types = TypeTable::new();
        let result = collect(&program, &mut resolver, &mut types);
        assert!(result.is_ok());

        let red_def = resolver.lookup(interner.intern("Red")).expect("red def");
        let symbol = resolver.get_symbol(red_def).expect("red symbol");
        assert_eq!(symbol.kind, SymbolKind::Variant);
    }

    #[test]
    fn collects_function_definitions() {
        let mut interner = Interner::new();
        let name = ident(&mut interner, "greet");
        let func = FuncDecl {
            name,
            type_params: Vec::new(),
            params: Vec::new(),
            modifiers: Vec::new(),
            ret: Some(TypeExpr {
                nullable: false,
                mode: None,
                kind: TypeExprKind::Named(ident(&mut interner, "vacuum"), Vec::new()),
                span: Span::default(),
            }),
            body: Some(BlockStmt {
                stmts: Vec::new(),
                span: Span::default(),
            }),
            annotations: Vec::new(),
        };
        let program = program(vec![stmt(StmtKind::Func(func))]);

        let mut resolver = Resolver::new();
        let mut types = TypeTable::new();
        let result = collect(&program, &mut resolver, &mut types);
        assert!(result.is_ok());

        let def_id = resolver
            .lookup(interner.intern("greet"))
            .expect("function def");
        let symbol = resolver.get_symbol(def_id).expect("function symbol");
        assert_eq!(symbol.kind, SymbolKind::Function);
    }
}
