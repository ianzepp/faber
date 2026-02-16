use super::{Codegen, DefId, FaberCodegen, Interner, Primitive, Type, TypeTable};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirParam, HirParamMode, HirProgram,
    HirStmt, HirStmtKind,
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
