use super::{lint, Resolver, SemanticErrorKind, TypeTable, WarningKind};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirImport, HirImportItem, HirItem, HirItemKind, HirLiteral, HirLocal, HirProgram,
    HirStmt, HirStmtKind,
};
use crate::lexer::Span;
use crate::semantic::Primitive;

fn span() -> Span {
    Span::default()
}

fn lit_expr() -> HirExpr {
    HirExpr { id: crate::hir::HirId(0), kind: HirExprKind::Literal(HirLiteral::Int(1)), ty: None, span: span() }
}

#[test]
fn warns_on_unused_local() {
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(1),
                kind: HirStmtKind::Local(HirLocal {
                    def_id: crate::hir::DefId(1),
                    name: crate::lexer::Symbol(1),
                    ty: None,
                    init: Some(lit_expr()),
                    mutable: false,
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let result = lint(&program, &Resolver::new(), &TypeTable::new());
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnusedVariable)));
}

#[test]
fn warns_on_unreachable_code() {
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt { id: crate::hir::HirId(1), kind: HirStmtKind::Redde(Some(lit_expr())), span: span() },
                HirStmt { id: crate::hir::HirId(2), kind: HirStmtKind::Expr(lit_expr()), span: span() },
            ],
            expr: None,
            span: span(),
        }),
    };

    let result = lint(&program, &Resolver::new(), &TypeTable::new());
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnreachableCode)));
}

#[test]
fn warns_on_unused_import() {
    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: crate::hir::DefId(10),
            kind: HirItemKind::Import(HirImport {
                path: crate::lexer::Symbol(1),
                items: vec![HirImportItem {
                    def_id: crate::hir::DefId(11),
                    name: crate::lexer::Symbol(2),
                    alias: None,
                }],
            }),
            span: span(),
        }],
        entry: None,
    };

    let result = lint(&program, &Resolver::new(), &TypeTable::new());
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnusedImport)));
}

#[test]
fn warns_on_shadowed_variable() {
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: crate::hir::HirId(1),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: crate::hir::DefId(1),
                        name: crate::lexer::Symbol(1),
                        ty: None,
                        init: Some(lit_expr()),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: crate::hir::HirId(2),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: crate::hir::DefId(2),
                        name: crate::lexer::Symbol(1),
                        ty: None,
                        init: Some(lit_expr()),
                        mutable: false,
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let result = lint(&program, &Resolver::new(), &TypeTable::new());
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::ShadowedVariable));
}

#[test]
fn warns_on_unnecessary_cast() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(1),
                kind: HirStmtKind::Expr(HirExpr {
                    id: crate::hir::HirId(2),
                    kind: HirExprKind::Qua(
                        Box::new(HirExpr {
                            id: crate::hir::HirId(3),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        numerus,
                    ),
                    ty: Some(numerus),
                    span: span(),
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let result = lint(&program, &Resolver::new(), &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnnecessaryCast)));
}

#[test]
fn warns_on_unreachable_after_break() {
    let loop_block = HirBlock {
        stmts: vec![
            HirStmt { id: crate::hir::HirId(1), kind: HirStmtKind::Rumpe, span: span() },
            HirStmt { id: crate::hir::HirId(2), kind: HirStmtKind::Expr(lit_expr()), span: span() },
        ],
        expr: None,
        span: span(),
    };
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(3),
                kind: HirStmtKind::Expr(HirExpr {
                    id: crate::hir::HirId(4),
                    kind: HirExprKind::Loop(loop_block),
                    ty: None,
                    span: span(),
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let result = lint(&program, &Resolver::new(), &TypeTable::new());
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnreachableCode)));
}
