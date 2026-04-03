use super::{collect, Resolver, SemanticErrorKind, SymbolKind, TypeTable};
use crate::lexer::{Interner, Span, Symbol};
use crate::syntax::{
    BlockStmt, EnumDecl, EnumMember, FuncDecl, ImportDecl, ImportKind, Mutability, Program, Stmt, StmtKind, TypeExpr,
    TypeExprKind, VarDecl, Visibility,
};

fn ident(interner: &mut Interner, name: &str) -> crate::syntax::Ident {
    crate::syntax::Ident { name: interner.intern(name), span: Span::default() }
}

fn ident_sym(sym: Symbol) -> crate::syntax::Ident {
    crate::syntax::Ident { name: sym, span: Span::default() }
}

fn stmt(kind: StmtKind) -> Stmt {
    Stmt { id: 0, kind, span: Span::default(), annotations: Vec::new() }
}

fn program(stmts: Vec<Stmt>) -> Program {
    Program { directives: Vec::new(), stmts, span: Span::default() }
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
    let red = EnumMember { name: ident(&mut interner, "Red"), value: None, span: Span::default() };
    let green = EnumMember { name: ident(&mut interner, "Green"), value: None, span: Span::default() };
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
        body: Some(BlockStmt { stmts: Vec::new(), span: Span::default() }),
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

#[test]
fn allows_duplicate_import_module_bindings() {
    let mut interner = Interner::new();
    let path = interner.intern("helpers");
    let import1 = ImportDecl {
        path,
        visibility: Visibility::Private,
        kind: ImportKind::Named { name: ident(&mut interner, "item"), alias: None },
        span: Span::default(),
    };
    let import2 = ImportDecl {
        path,
        visibility: Visibility::Private,
        kind: ImportKind::Named { name: ident(&mut interner, "item"), alias: None },
        span: Span::default(),
    };
    let program = program(vec![stmt(StmtKind::Import(import1)), stmt(StmtKind::Import(import2))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let result = collect(&program, &mut resolver, &mut types);
    assert!(result.is_ok());
}
