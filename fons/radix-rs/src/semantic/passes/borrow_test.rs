use super::{analyze, DefId, ParamMode, Resolver, SemanticErrorKind, TypeTable};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirParam, HirParamMode, HirProgram,
    HirStmt, HirStmtKind,
};
use crate::lexer::Span;
use crate::semantic::{FuncSig, ParamType, Primitive};

fn span() -> Span {
    Span::default()
}

fn lit_expr(id: u32) -> HirExpr {
    HirExpr { id: crate::hir::HirId(id), kind: HirExprKind::Literal(HirLiteral::Int(1)), ty: None, span: span() }
}

#[test]
fn reports_use_after_move() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let func_ty = types.function(FuncSig {
        params: vec![ParamType { ty: numerus, mode: ParamMode::Move, optional: false }],
        ret: numerus,
        is_async: false,
        is_generator: false,
    });

    let call = HirExpr {
        id: crate::hir::HirId(3),
        kind: HirExprKind::Call(
            Box::new(HirExpr {
                id: crate::hir::HirId(2),
                kind: HirExprKind::Path(DefId(20)),
                ty: Some(func_ty),
                span: span(),
            }),
            vec![HirExpr { id: crate::hir::HirId(4), kind: HirExprKind::Path(DefId(1)), ty: None, span: span() }],
        ),
        ty: None,
        span: span(),
    };

    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(1),
                    name: crate::lexer::Symbol(2),
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    span: span(),
                }],
                ret_ty: None,
                body: Some(HirBlock {
                    stmts: vec![
                        HirStmt { id: crate::hir::HirId(1), kind: HirStmtKind::Expr(call), span: span() },
                        HirStmt {
                            id: crate::hir::HirId(5),
                            kind: HirStmtKind::Expr(HirExpr {
                                id: crate::hir::HirId(6),
                                kind: HirExprKind::Path(DefId(1)),
                                ty: None,
                                span: span(),
                            }),
                            span: span(),
                        },
                    ],
                    expr: None,
                    span: span(),
                }),
                is_async: false,
                is_generator: false,
            }),
            span: span(),
        }],
        entry: None,
    };

    let result = analyze(&program, &Resolver::new(), &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::UseAfterMove));
}

#[test]
fn reports_mutable_borrow_conflict() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let shared = HirExpr {
        id: crate::hir::HirId(1),
        kind: HirExprKind::Ref(
            crate::hir::HirRefKind::Shared,
            Box::new(HirExpr { id: crate::hir::HirId(2), kind: HirExprKind::Path(DefId(1)), ty: None, span: span() }),
        ),
        ty: None,
        span: span(),
    };
    let mutable = HirExpr {
        id: crate::hir::HirId(3),
        kind: HirExprKind::Ref(
            crate::hir::HirRefKind::Mutable,
            Box::new(HirExpr { id: crate::hir::HirId(4), kind: HirExprKind::Path(DefId(1)), ty: None, span: span() }),
        ),
        ty: None,
        span: span(),
    };

    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(1),
                    name: crate::lexer::Symbol(2),
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    span: span(),
                }],
                ret_ty: None,
                body: Some(HirBlock {
                    stmts: vec![
                        HirStmt { id: crate::hir::HirId(5), kind: HirStmtKind::Expr(shared), span: span() },
                        HirStmt { id: crate::hir::HirId(6), kind: HirStmtKind::Expr(mutable), span: span() },
                    ],
                    expr: None,
                    span: span(),
                }),
                is_async: false,
                is_generator: false,
            }),
            span: span(),
        }],
        entry: None,
    };

    let result = analyze(&program, &Resolver::new(), &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::MutableBorrowConflict));
}
