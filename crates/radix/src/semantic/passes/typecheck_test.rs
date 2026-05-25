use super::{typecheck, DefId, Primitive, Resolver, SemanticErrorKind, Type, TypeTable};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirLocal, HirProgram, HirStmt,
    HirStmtKind, HirStruct, HirTypeParam,
};
use crate::lexer::Span;

fn span() -> Span {
    Span::default()
}

fn literal_int(id: u32, value: i64) -> HirExpr {
    HirExpr { id: crate::hir::HirId(id), kind: HirExprKind::Literal(HirLiteral::Int(value)), ty: None, span: span() }
}

fn literal_string(id: u32) -> HirExpr {
    HirExpr {
        id: crate::hir::HirId(id),
        kind: HirExprKind::Literal(HirLiteral::String(crate::lexer::Symbol(99))),
        ty: None,
        span: span(),
    }
}

#[test]
fn rejects_redde_without_explicit_return_type() {
    let mut types = TypeTable::new();
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: None,
                err_ty: None,
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Redde(Some(literal_int(2, 42))),
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

    let resolver = Resolver::new();
    let result = typecheck(&mut program, &resolver, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|err| {
        err.kind == SemanticErrorKind::MissingReturn
            && err.message.contains("redde requires an explicit normal return type")
    }));
}

#[test]
fn omitted_return_effect_function_finalizes_to_vacuum() {
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: None,
                err_ty: Some(textus),
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: crate::hir::HirId(2),
                            kind: HirExprKind::Throw(Box::new(literal_string(3))),
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

    let resolver = Resolver::new();
    let result = typecheck(&mut program, &resolver, &mut types);
    assert!(result.is_ok());

    let item = &program.items[0];
    let HirItemKind::Function(func) = &item.kind else {
        panic!("expected function item");
    };
    assert_eq!(func.ret_ty, Some(types.primitive(Primitive::Vacuum)));
}

#[test]
fn reports_initializer_type_mismatch() {
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let mut program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(0),
                kind: HirStmtKind::Local(HirLocal {
                    def_id: DefId(0),
                    name: crate::lexer::Symbol(1),
                    ty: Some(textus),
                    init: Some(literal_int(1, 7)),
                    mutable: false,
                }),
                span: span(),
            }],
            expr: None,
            span: span(),
        }),
    };

    let resolver = Resolver::new();
    let result = typecheck(&mut program, &resolver, &mut types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::TypeMismatch));
}

#[test]
fn typechecks_iace_against_declared_alternate_exit() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(numerus),
                err_ty: Some(textus),
                body: Some(HirBlock {
                    stmts: vec![
                        HirStmt {
                            id: crate::hir::HirId(1),
                            kind: HirStmtKind::Expr(HirExpr {
                                id: crate::hir::HirId(2),
                                kind: HirExprKind::Throw(Box::new(literal_string(3))),
                                ty: None,
                                span: span(),
                            }),
                            span: span(),
                        },
                        HirStmt {
                            id: crate::hir::HirId(4),
                            kind: HirStmtKind::Redde(Some(literal_int(5, 1))),
                            span: span(),
                        },
                    ],
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

    let result = typecheck(&mut program, &Resolver::new(), &mut types);
    assert!(result.is_ok(), "expected declared alternate exit to accept iace: {result:?}");
}

#[test]
fn rejects_iace_without_declared_alternate_exit() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(numerus),
                err_ty: None,
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: crate::hir::HirId(2),
                            kind: HirExprKind::Throw(Box::new(literal_string(3))),
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

    let result = typecheck(&mut program, &Resolver::new(), &mut types);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .iter()
        .any(|err| err.message.contains("iace requires an enclosing function")));
}

#[test]
fn rejects_iace_value_that_mismatches_alternate_exit() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(numerus),
                err_ty: Some(numerus),
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: crate::hir::HirId(2),
                            kind: HirExprKind::Throw(Box::new(literal_string(3))),
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

    let result = typecheck(&mut program, &Resolver::new(), &mut types);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .iter()
        .any(|err| err.message.contains("alternate exit value type mismatch")));
}

#[test]
fn redde_still_checks_against_normal_return_type() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                cli_args: None,
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(numerus),
                err_ty: Some(textus),
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Redde(Some(literal_string(2))),
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

    let result = typecheck(&mut program, &Resolver::new(), &mut types);
    assert!(result.is_err());
    assert!(result.unwrap_err().iter().any(|err| err
        .message
        .contains("return type does not match function signature")));
}

#[test]
fn rejects_failable_call_in_ordinary_expression_position() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let mut program = HirProgram {
        items: vec![
            HirItem {
                id: crate::hir::HirId(0),
                def_id: DefId(1),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: crate::lexer::Symbol(1),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(numerus),
                    err_ty: Some(textus),
                    body: None,
                    is_async: false,
                    is_generator: false,
                    test: None,
                }),
                span: span(),
            },
            HirItem {
                id: crate::hir::HirId(10),
                def_id: DefId(2),
                kind: HirItemKind::Function(HirFunction {
                    cli_args: None,
                    name: crate::lexer::Symbol(2),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(numerus),
                    err_ty: Some(textus),
                    body: Some(HirBlock {
                        stmts: vec![HirStmt {
                            id: crate::hir::HirId(11),
                            kind: HirStmtKind::Redde(Some(HirExpr {
                                id: crate::hir::HirId(12),
                                kind: HirExprKind::Call(
                                    Box::new(HirExpr {
                                        id: crate::hir::HirId(13),
                                        kind: HirExprKind::Path(DefId(1)),
                                        ty: None,
                                        span: span(),
                                    }),
                                    Vec::new(),
                                ),
                                ty: None,
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

    let result = typecheck(&mut program, &Resolver::new(), &mut types);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .iter()
        .any(|err| err.message.contains("failable call requires handling")));
}

#[test]
fn structured_cape_consumes_local_iace_without_function_alternate_exit() {
    let session =
        crate::driver::Session::new(crate::driver::Config::default().with_target(crate::codegen::Target::Faber));
    let result = crate::driver::analyze_source(
        &session,
        "<test>",
        r#"functio handled() → numerus { fac { iace "bad" } cape err { redde 0 } redde 1 }"#,
    );

    if let Err(errors) = result {
        panic!("expected local cape to consume iace: {errors:?}");
    }
}

#[test]
fn structured_cape_consumes_failable_direct_call() {
    let session =
        crate::driver::Session::new(crate::driver::Config::default().with_target(crate::codegen::Target::Faber));
    let result = crate::driver::analyze_source(
        &session,
        "<test>",
        r#"
functio fail() → numerus ⇥ textus { iace "bad" }
functio handled() → numerus { fac { redde fail() } cape err { redde 0 } }
"#,
    );

    if let Err(errors) = result {
        panic!("expected local cape to consume failable call: {errors:?}");
    }
}

#[test]
fn structured_cape_rejects_incompatible_handler_error_types() {
    let session =
        crate::driver::Session::new(crate::driver::Config::default().with_target(crate::codegen::Target::Faber));
    let result = crate::driver::analyze_source(
        &session,
        "<test>",
        r#"
functio fail_text() → numerus ⇥ textus { iace "bad" }
functio fail_num() → numerus ⇥ numerus { iace 1 }
functio handled() → numerus {
    fac {
        fail_text()
        fail_num()
    } cape err {
        redde 0
    }
}
"#,
    );

    let errors = match result {
        Ok(_) => panic!("expected mixed handled error types to be rejected"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|err| err
        .message
        .contains("handled failable call error type mismatch")));
}

#[test]
fn resolves_method_call_type() {
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let struct_def = DefId(1);
    let struct_ty = types.intern(Type::Struct(struct_def));
    let method_name = crate::lexer::Symbol(3);
    let local_name = crate::lexer::Symbol(2);

    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: struct_def,
            kind: HirItemKind::Struct(HirStruct {
                name: crate::lexer::Symbol(1),
                type_params: Vec::<HirTypeParam>::new(),
                fields: Vec::new(),
                methods: vec![crate::hir::HirMethod {
                    def_id: DefId(2),
                    func: HirFunction {
                        cli_args: None,
                        name: method_name,
                        type_params: Vec::new(),
                        params: Vec::new(),
                        ret_ty: Some(numerus),
                        err_ty: None,
                        body: None,
                        is_async: false,
                        is_generator: false,
                        test: None,
                    },
                    receiver: crate::hir::HirReceiver::None,
                    span: span(),
                }],
                extends: None,
                implements: Vec::new(),
            }),
            span: span(),
        }],
        entry: Some(HirBlock {
            stmts: vec![HirStmt {
                id: crate::hir::HirId(10),
                kind: HirStmtKind::Local(HirLocal {
                    def_id: DefId(10),
                    name: local_name,
                    ty: Some(struct_ty),
                    init: None,
                    mutable: false,
                }),
                span: span(),
            }],
            expr: Some(Box::new(HirExpr {
                id: crate::hir::HirId(11),
                kind: HirExprKind::MethodCall(
                    Box::new(HirExpr {
                        id: crate::hir::HirId(12),
                        kind: HirExprKind::Path(DefId(10)),
                        ty: None,
                        span: span(),
                    }),
                    method_name,
                    Vec::new(),
                ),
                ty: None,
                span: span(),
            })),
            span: span(),
        }),
    };

    let resolver = Resolver::new();
    let result = typecheck(&mut program, &resolver, &mut types);
    assert!(result.is_ok());

    let entry_expr = program
        .entry
        .as_ref()
        .and_then(|block| block.expr.as_ref())
        .expect("expected entry expr");
    assert_eq!(entry_expr.ty, Some(numerus));
}
