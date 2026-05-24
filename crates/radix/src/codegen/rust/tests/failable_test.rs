use super::*;

#[test]
fn emits_result_and_err_for_direct_iace() {
    let mut interner = Interner::new();
    let boom = interner.intern("boom");
    let oops = interner.intern("oops");
    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(900),
            def_id: DefId(900),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: boom,
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(numerus),
                err_ty: None,
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(901),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: HirId(902),
                            kind: HirExprKind::Throw(Box::new(HirExpr {
                                id: HirId(903),
                                kind: HirExprKind::Literal(HirLiteral::String(oops)),
                                ty: Some(types.primitive(Primitive::Textus)),
                                span: span(),
                            })),
                            ty: None,
                            span: span(),
                        }),
                        span: span(),
                    }],
                    expr: None,
                    span: span(),
                }),
                is_async: false,
                is_generator: false,
                test: None,
            }),
            span: span(),
        }],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("fn boom() -> Result<i64, String>"));
    assert!(rust
        .code
        .contains("return Err(String::from(\"oops\".to_string()));"));
}

#[test]
fn propagates_failable_calls_with_question_mark() {
    let mut interner = Interner::new();
    let callee = interner.intern("callee");
    let caller = interner.intern("caller");
    let oops = interner.intern("oops");
    let types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(910),
                def_id: DefId(910),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: callee,
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(numerus),
                    err_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![HirStmt {
                            id: HirId(911),
                            kind: HirStmtKind::Expr(HirExpr {
                                id: HirId(912),
                                kind: HirExprKind::Throw(Box::new(HirExpr {
                                    id: HirId(913),
                                    kind: HirExprKind::Literal(HirLiteral::String(oops)),
                                    ty: Some(textus),
                                    span: span(),
                                })),
                                ty: None,
                                span: span(),
                            }),
                            span: span(),
                        }],
                        expr: None,
                        span: span(),
                    }),
                    is_async: false,
                    is_generator: false,
                    test: None,
                }),
                span: span(),
            },
            HirItem {
                id: HirId(920),
                def_id: DefId(920),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: caller,
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(numerus),
                    err_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![HirStmt {
                            id: HirId(921),
                            kind: HirStmtKind::Redde(Some(HirExpr {
                                id: HirId(922),
                                kind: HirExprKind::Call(
                                    Box::new(HirExpr {
                                        id: HirId(923),
                                        kind: HirExprKind::Path(DefId(910)),
                                        ty: None,
                                        span: span(),
                                    }),
                                    Vec::new(),
                                ),
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
                    test: None,
                }),
                span: span(),
            },
        ],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("fn callee() -> Result<i64, String>"));
    assert!(rust.code.contains("fn caller() -> Result<i64, String>"));
    assert!(rust.code.contains("return Ok(callee()?);"));
}
