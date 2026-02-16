use super::{Codegen, DefId, FaberCodegen, Interner, Primitive, Type, TypeTable};
use crate::hir::{
    HirBlock, HirCasuArm, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirParam, HirParamMode,
    HirPattern, HirProgram, HirStmt, HirStmtKind,
};
use crate::lexer::Span;
use crate::semantic::InferVar;

fn span() -> Span {
    Span::default()
}

#[test]
fn emits_basic_function_and_entry() {
    let mut interner = Interner::new();
    let name_greet = interner.intern("greet");
    let name_param = interner.intern("name");
    let name_text = interner.intern("salve");

    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);

    let function = HirFunction {
        name: name_greet,
        type_params: Vec::new(),
        params: vec![HirParam {
            def_id: DefId(1),
            name: name_param,
            ty: textus,
            mode: HirParamMode::Owned,
            optional: false,
            span: span(),
        }],
        ret_ty: Some(textus),
        body: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(2),
                kind: HirStmtKind::Redde(Some(HirExpr {
                    id: crate::hir::HirId(3),
                    kind: HirExprKind::Literal(HirLiteral::String(name_text)),
                    ty: Some(textus),
                    span: span(),
                })),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
        is_async: false,
        is_generator: false,
    };

    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(function),
            span: span(),
        }],
        entry: Some(HirBlock { stmts: Vec::new(), expr: None, span: span() }),
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("functio greet"));
    assert!(output.code.contains("incipit"));
    assert!(output.code.contains("redde \"salve\""));
}

#[test]
fn renders_unresolved_infer_as_comment_marker() {
    let mut interner = Interner::new();
    let name = interner.intern("x");
    let mut types = TypeTable::new();
    let unresolved = types.intern(Type::Infer(InferVar(99)));

    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(1),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id: DefId(1),
                    name,
                    ty: Some(unresolved),
                    init: None,
                    mutable: false,
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("/* unresolved */ x"));
}

#[test]
fn rejects_hir_error_nodes_in_direct_faber_codegen() {
    let interner = Interner::new();
    let types = TypeTable::new();
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(1),
                kind: HirStmtKind::Expr(HirExpr {
                    id: crate::hir::HirId(2),
                    kind: HirExprKind::Error,
                    ty: None,
                    span: span(),
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let gen = FaberCodegen::new();
    let error = match gen.generate(&program, &types, &interner) {
        Ok(_) => panic!("expected faber codegen error"),
        Err(error) => error,
    };
    assert!(error.message.contains("HIR containing error expressions"));
}

#[test]
fn emits_parameter_references_with_original_names() {
    let mut interner = Interner::new();
    let name_identity = interner.intern("identity");
    let name_x = interner.intern("x");

    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let function = HirFunction {
        name: name_identity,
        type_params: Vec::new(),
        params: vec![HirParam {
            def_id: DefId(100),
            name: name_x,
            ty: numerus,
            mode: HirParamMode::Owned,
            optional: false,
            span: span(),
        }],
        ret_ty: Some(numerus),
        body: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(2),
                kind: HirStmtKind::Redde(Some(HirExpr {
                    id: crate::hir::HirId(3),
                    kind: HirExprKind::Path(DefId(100)),
                    ty: Some(numerus),
                    span: span(),
                })),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
        is_async: false,
        is_generator: false,
    };

    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(function),
            span: span(),
        }],
        entry: None,
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("redde x"));
    assert!(!output.code.contains("def_100"));
}

#[test]
fn emits_si_sin_secus_chain_with_reddit_shorthand() {
    let mut interner = Interner::new();
    let name_grade = interner.intern("grade");
    let text_a = interner.intern("A");
    let text_b = interner.intern("B");
    let text_f = interner.intern("F");

    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let bivalens = types.primitive(Primitive::Bivalens);

    let expr = HirExpr {
        id: crate::hir::HirId(10),
        kind: HirExprKind::Si(
            Box::new(HirExpr {
                id: crate::hir::HirId(11),
                kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                ty: Some(bivalens),
                span: span(),
            }),
            HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(12),
                    kind: HirStmtKind::Redde(Some(HirExpr {
                        id: crate::hir::HirId(13),
                        kind: HirExprKind::Literal(HirLiteral::String(text_a)),
                        ty: Some(textus),
                        span: span(),
                    })),
                    span: span(),
                }],
                expr: None,
                span: span(),
            },
            Some(HirBlock {
                stmts: Vec::new(),
                expr: Some(Box::new(HirExpr {
                    id: crate::hir::HirId(14),
                    kind: HirExprKind::Si(
                        Box::new(HirExpr {
                            id: crate::hir::HirId(15),
                            kind: HirExprKind::Literal(HirLiteral::Bool(false)),
                            ty: Some(bivalens),
                            span: span(),
                        }),
                        HirBlock {
                            stmts: vec![HirStmt {
                                id: crate::hir::HirId(16),
                                kind: HirStmtKind::Redde(Some(HirExpr {
                                    id: crate::hir::HirId(17),
                                    kind: HirExprKind::Literal(HirLiteral::String(text_b)),
                                    ty: Some(textus),
                                    span: span(),
                                })),
                                span: span(),
                            }],
                            expr: None,
                            span: span(),
                        },
                        Some(HirBlock {
                            stmts: vec![HirStmt {
                                id: crate::hir::HirId(18),
                                kind: HirStmtKind::Redde(Some(HirExpr {
                                    id: crate::hir::HirId(19),
                                    kind: HirExprKind::Literal(HirLiteral::String(text_f)),
                                    ty: Some(textus),
                                    span: span(),
                                })),
                                span: span(),
                            }],
                            expr: None,
                            span: span(),
                        }),
                    ),
                    ty: Some(textus),
                    span: span(),
                })),
                span: span(),
            }),
        ),
        ty: Some(textus),
        span: span(),
    };

    let function = HirFunction {
        name: name_grade,
        type_params: Vec::new(),
        params: Vec::new(),
        ret_ty: Some(textus),
        body: Some(HirBlock {
            stmts: vec![HirStmt { id: crate::hir::HirId(20), kind: HirStmtKind::Expr(expr), span: span() }],
            expr: None,
            span: span(),
        }),
        is_async: false,
        is_generator: false,
    };

    let program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(function),
            span: span(),
        }],
        entry: None,
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("si verum reddit \"A\""));
    assert!(output.code.contains("sin falsum reddit \"B\""));
    assert!(output.code.contains("secus reddit \"F\""));
    assert!(!output.code.contains("aliter"));
}

#[test]
fn discerne_cases_do_not_emit_nested_blocks() {
    let mut interner = Interner::new();
    let ok = interner.intern("OK");

    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);

    let arm_body = HirExpr {
        id: crate::hir::HirId(3),
        kind: HirExprKind::Block(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(4),
                kind: HirStmtKind::Redde(Some(HirExpr {
                    id: crate::hir::HirId(5),
                    kind: HirExprKind::Literal(HirLiteral::String(ok)),
                    ty: Some(textus),
                    span: span(),
                })),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
        ty: Some(textus),
        span: span(),
    };

    let discerne = HirExpr {
        id: crate::hir::HirId(1),
        kind: HirExprKind::Discerne(
            Box::new(HirExpr {
                id: crate::hir::HirId(2),
                kind: HirExprKind::Literal(HirLiteral::Int(200)),
                ty: Some(numerus),
                span: span(),
            }),
            vec![HirCasuArm {
                pattern: HirPattern::Literal(HirLiteral::Int(200)),
                guard: None,
                body: arm_body,
                span: span(),
            }],
        ),
        ty: Some(textus),
        span: span(),
    };

    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt { id: crate::hir::HirId(6), kind: HirStmtKind::Expr(discerne), span: span() }],
            expr: None,
            span: span(),
        }),
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("casu 200 {\n            redde \"OK\""));
    assert!(!output.code.contains("casu 200 {\n            {"));
}

#[test]
fn emits_object_literal_fields_from_innatum_entries() {
    let mut interner = Interner::new();
    let nomen = interner.intern("nomen");
    let activus = interner.intern("activus");
    let marcus = interner.intern("Marcus");

    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let bivalens = types.primitive(Primitive::Bivalens);
    let union_ty = types.intern(Type::Union(vec![textus, bivalens]));
    let map_ty = types.map(textus, union_ty);

    let expr = HirExpr {
        id: crate::hir::HirId(1),
        kind: HirExprKind::Verte {
            source: Box::new(HirExpr {
                id: crate::hir::HirId(2),
                kind: HirExprKind::Tuple(Vec::new()),
                ty: None,
                span: span(),
            }),
            target: map_ty,
            entries: Some(vec![
                (
                    nomen,
                    HirExpr {
                        id: crate::hir::HirId(3),
                        kind: HirExprKind::Literal(HirLiteral::String(marcus)),
                        ty: Some(textus),
                        span: span(),
                    },
                ),
                (
                    activus,
                    HirExpr {
                        id: crate::hir::HirId(4),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    },
                ),
            ]),
        },
        ty: Some(map_ty),
        span: span(),
    };

    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt { id: crate::hir::HirId(5), kind: HirStmtKind::Expr(expr), span: span() }],
            expr: None,
            span: span(),
        }),
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("nomen: \"Marcus\""));
    assert!(output.code.contains("activus: verum"));
    // With unified Verte, map construction now emits `{...} innatum Type`
    assert!(output.code.contains("innatum"));
}

#[test]
fn emits_regex_literals_with_flags() {
    let mut interner = Interner::new();
    let pattern = interner.intern("\\d+");
    let flags = interner.intern("g");

    let mut types = TypeTable::new();
    let regex = types.primitive(Primitive::Regex);

    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(1),
                kind: HirStmtKind::Expr(HirExpr {
                    id: crate::hir::HirId(2),
                    kind: HirExprKind::Literal(HirLiteral::Regex(pattern, Some(flags))),
                    ty: Some(regex),
                    span: span(),
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let gen = FaberCodegen::new();
    let output = gen.generate(&program, &types, &interner).expect("codegen");
    assert!(output.code.contains("sed \"\\d+\" g"));
}
