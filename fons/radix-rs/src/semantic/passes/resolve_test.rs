use super::{resolve, Resolver, SemanticErrorKind, TypeTable};
use crate::lexer::{Interner, Span, Symbol};
use crate::semantic::passes::collect;
use crate::semantic::Primitive;
use crate::syntax::{
    BlockStmt, CasuArm, DiscerneStmt, EnumDecl, EnumMember, Expr, ExprKind, ExprStmt, FuncDecl, PathPattern, Pattern,
    Program, ReddeStmt, Stmt, StmtKind, TypeAliasDecl, TypeExpr, TypeExprKind,
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

fn named_type(name: crate::syntax::Ident) -> TypeExpr {
    TypeExpr { nullable: false, mode: None, kind: TypeExprKind::Named(name, Vec::new()), span: Span::default() }
}

#[test]
fn reports_undefined_variable_in_expression() {
    let mut interner = Interner::new();
    let expr = Expr { id: 0, kind: ExprKind::Ident(ident(&mut interner, "missing")), span: Span::default() };
    let program = program(vec![stmt(StmtKind::Expr(ExprStmt { expr: Box::new(expr) }))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::UndefinedVariable));
}

#[test]
fn reports_return_outside_function() {
    let program = program(vec![stmt(StmtKind::Redde(ReddeStmt { value: None }))]);
    let mut interner = Interner::new();

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::ReturnOutsideFunction));
}

#[test]
fn resolves_type_alias_to_builtin() {
    let mut interner = Interner::new();
    let alias_sym = interner.intern("Numeri");
    let alias = TypeAliasDecl { name: ident_sym(alias_sym), ty: named_type(ident(&mut interner, "numerus")) };
    let program = program(vec![stmt(StmtKind::TypeAlias(alias))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_ok());

    let def_id = resolver.lookup(alias_sym).expect("alias def");
    let symbol = resolver.get_symbol(def_id).expect("alias symbol");
    assert_eq!(symbol.ty, Some(types.primitive(Primitive::Numerus)));
}

#[test]
fn reports_type_alias_cycle() {
    let mut interner = Interner::new();
    let a_sym = interner.intern("A");
    let b_sym = interner.intern("B");
    let alias_a = TypeAliasDecl { name: ident_sym(a_sym), ty: named_type(ident_sym(b_sym)) };
    let alias_b = TypeAliasDecl { name: ident_sym(b_sym), ty: named_type(ident_sym(a_sym)) };
    let program = program(vec![stmt(StmtKind::TypeAlias(alias_a)), stmt(StmtKind::TypeAlias(alias_b))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::CircularDependency));
}

#[test]
fn reports_non_variant_pattern() {
    let mut interner = Interner::new();
    let foo_sym = interner.intern("Foo");
    let func = FuncDecl {
        name: ident_sym(foo_sym),
        type_params: Vec::new(),
        params: Vec::new(),
        modifiers: Vec::new(),
        ret: None,
        body: Some(BlockStmt { stmts: Vec::new(), span: Span::default() }),
        annotations: Vec::new(),
    };
    let match_stmt = DiscerneStmt {
        exhaustive: false,
        subjects: vec![Expr {
            id: 1,
            kind: ExprKind::Literal(crate::syntax::Literal::Integer(1)),
            span: Span::default(),
        }],
        arms: vec![CasuArm {
            patterns: vec![Pattern::Path(PathPattern {
                segments: vec![ident_sym(foo_sym)],
                bind: None,
                span: Span::default(),
            })],
            body: crate::syntax::IfBody::Block(BlockStmt { stmts: Vec::new(), span: Span::default() }),
            span: Span::default(),
        }],
        default: None,
    };
    let program = program(vec![stmt(StmtKind::Func(func)), stmt(StmtKind::Discerne(match_stmt))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::UndefinedVariable));
}

#[test]
fn allows_variant_patterns_from_collected_enum() {
    let mut interner = Interner::new();
    let enum_name = ident(&mut interner, "Color");
    let red_sym = interner.intern("Red");
    let red = EnumMember { name: ident_sym(red_sym), value: None, span: Span::default() };
    let enum_decl = EnumDecl { name: enum_name, members: vec![red] };

    let match_stmt = DiscerneStmt {
        exhaustive: false,
        subjects: vec![Expr {
            id: 1,
            kind: ExprKind::Literal(crate::syntax::Literal::Integer(1)),
            span: Span::default(),
        }],
        arms: vec![CasuArm {
            patterns: vec![Pattern::Path(PathPattern {
                segments: vec![ident_sym(red_sym)],
                bind: None,
                span: Span::default(),
            })],
            body: crate::syntax::IfBody::Block(BlockStmt { stmts: Vec::new(), span: Span::default() }),
            span: Span::default(),
        }],
        default: None,
    };

    let program = program(vec![stmt(StmtKind::Enum(enum_decl)), stmt(StmtKind::Discerne(match_stmt))]);

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_ok());
}

#[test]
fn reports_break_outside_loop() {
    let program = program(vec![stmt(StmtKind::Rumpe(crate::syntax::RumpeStmt {
        span: Span::default(),
    }))]);
    let mut interner = Interner::new();

    let mut resolver = Resolver::new();
    let mut types = TypeTable::new();
    let _ = collect::collect(&program, &mut resolver, &mut types);
    let result = resolve(&program, &mut resolver, &interner, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::BreakOutsideLoop));
}
