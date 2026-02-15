use crate::codegen::{self, Target};
use crate::hir::{
    DefId, HirBlock, HirCasuArm, HirEnum, HirExpr, HirExprKind, HirFunction, HirId, HirInterface, HirItem, HirItemKind,
    HirLiteral, HirParam, HirParamMode, HirPattern, HirProgram, HirStmt, HirStmtKind, HirTypeAlias, HirVariant,
};
use crate::lexer::{Interner, Span};
use crate::semantic::{FuncSig, InferVar, Mutability, ParamMode, ParamType, Primitive, Type, TypeTable};

fn span() -> Span {
    Span::default()
}

fn empty_program() -> HirProgram {
    HirProgram { items: Vec::new(), entry: None }
}

#[test]
fn emits_rust_function_and_entry_via_codegen_dispatch() {
    let mut interner = Interner::new();
    let name_f = interner.intern("f");
    let name_x = interner.intern("x");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(0),
            def_id: DefId(1),
            kind: HirItemKind::Function(HirFunction {
                name: name_f,
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(2),
                    name: name_x,
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    span: span(),
                }],
                ret_ty: Some(numerus),
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(3),
                        kind: HirStmtKind::Redde(Some(HirExpr {
                            id: HirId(4),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
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
            }),
            span: span(),
        }],
        entry: Some(HirBlock { stmts: Vec::new(), expr: None, span: span() }),
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("fn todo_func_name"));
    assert!(rust.code.contains("fn main() {"));
}

#[test]
fn traverses_match_patterns_and_closure_params_in_name_collection() {
    let mut interner = Interner::new();
    let enum_name = interner.intern("Res");
    let variant_name = interner.intern("Ok");
    let local_name = interner.intern("value");
    let bind_name = interner.intern("bound");
    let closure_name = interner.intern("p");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);

    let match_expr = HirExpr {
        id: HirId(20),
        kind: HirExprKind::Discerne(
            Box::new(HirExpr {
                id: HirId(21),
                kind: HirExprKind::Path(DefId(40)),
                ty: Some(numerus),
                span: span(),
            }),
            vec![HirCasuArm {
                pattern: HirPattern::Variant(DefId(30), vec![HirPattern::Binding(DefId(41), bind_name)]),
                guard: Some(HirExpr {
                    id: HirId(22),
                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                    ty: Some(types.primitive(Primitive::Bivalens)),
                    span: span(),
                }),
                body: HirExpr {
                    id: HirId(23),
                    kind: HirExprKind::Literal(HirLiteral::Int(7)),
                    ty: Some(numerus),
                    span: span(),
                },
                span: span(),
            }],
        ),
        ty: Some(numerus),
        span: span(),
    };

    let closure_expr = HirExpr {
        id: HirId(24),
        kind: HirExprKind::Clausura(
            vec![HirParam {
                def_id: DefId(42),
                name: closure_name,
                ty: numerus,
                mode: HirParamMode::Owned,
                span: span(),
            }],
            None,
            Box::new(HirExpr {
                id: HirId(25),
                kind: HirExprKind::Path(DefId(42)),
                ty: Some(numerus),
                span: span(),
            }),
        ),
        ty: None,
        span: span(),
    };

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(10),
                def_id: DefId(29),
                kind: HirItemKind::Enum(HirEnum {
                    name: enum_name,
                    type_params: Vec::new(),
                    variants: vec![HirVariant {
                        def_id: DefId(30),
                        name: variant_name,
                        fields: Vec::new(),
                        span: span(),
                    }],
                }),
                span: span(),
            },
            HirItem {
                id: HirId(15),
                def_id: DefId(31),
                kind: HirItemKind::Function(HirFunction {
                    name: interner.intern("collector"),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![
                            HirStmt {
                                id: HirId(11),
                                kind: HirStmtKind::Local(crate::hir::HirLocal {
                                    def_id: DefId(40),
                                    name: local_name,
                                    ty: Some(numerus),
                                    init: Some(HirExpr {
                                        id: HirId(12),
                                        kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                        ty: Some(numerus),
                                        span: span(),
                                    }),
                                    mutable: false,
                                }),
                                span: span(),
                            },
                            HirStmt {
                                id: HirId(13),
                                kind: HirStmtKind::Expr(match_expr),
                                span: span(),
                            },
                            HirStmt {
                                id: HirId(14),
                                kind: HirStmtKind::Expr(closure_expr),
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
            },
        ],
        entry: None,
    };

    let output = codegen::generate(Target::Rust, &program, &types, &interner).expect("rust codegen");
    let crate::Output::Rust(rust) = output else {
        panic!("expected rust output");
    };

    assert!(rust.code.contains("match "));
    assert!(rust.code.contains("Variant"));
    assert!(rust.code.contains("|p|"));
}

#[test]
fn keeps_placeholder_type_names_for_named_defs() {
    let mut interner = Interner::new();
    let iface_name = interner.intern("Servitium");
    let alias_name = interner.intern("Alias");
    let mut types = TypeTable::new();
    let iface_ty = types.intern(Type::Interface(DefId(70)));
    let struct_ty = types.intern(Type::Struct(DefId(71)));
    let enum_ty = types.intern(Type::Enum(DefId(72)));

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(60),
                def_id: DefId(70),
                kind: HirItemKind::Interface(HirInterface {
                    name: iface_name,
                    type_params: Vec::new(),
                    methods: Vec::new(),
                }),
                span: span(),
            },
            HirItem {
                id: HirId(61),
                def_id: DefId(73),
                kind: HirItemKind::TypeAlias(HirTypeAlias {
                    name: alias_name,
                    ty: struct_ty,
                }),
                span: span(),
            },
            HirItem {
                id: HirId(62),
                def_id: DefId(74),
                kind: HirItemKind::Const(crate::hir::HirConst {
                    name: interner.intern("C"),
                    ty: Some(enum_ty),
                    value: HirExpr {
                        id: HirId(63),
                        kind: HirExprKind::Literal(HirLiteral::Int(0)),
                        ty: Some(types.primitive(Primitive::Numerus)),
                        span: span(),
                    },
                }),
                span: span(),
            },
            HirItem {
                id: HirId(67),
                def_id: DefId(75),
                kind: HirItemKind::Function(HirFunction {
                    name: interner.intern("iface_ret"),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: Some(iface_ty),
                    body: None,
                    is_async: false,
                    is_generator: false,
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

    assert!(rust.code.contains("TodoStruct"));
    assert!(rust.code.contains("TodoEnum"));
    assert!(rust.code.contains("dyn TodoTrait"));
}

#[test]
fn expr_codegen_handles_control_flow_and_operators() {
    let mut interner = Interner::new();
    let method = interner.intern("met");
    let field = interner.intern("fld");
    let numerus_name = interner.intern("N");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let bivalens = types.primitive(Primitive::Bivalens);
    let err_ty = types.intern(Type::Error);

    let codegen = super::RustCodegen::new(&empty_program(), &interner);
    let mut w = codegen::CodeWriter::new();

    let expr = HirExpr {
        id: HirId(300),
        kind: HirExprKind::Tuple(vec![
            HirExpr {
                id: HirId(301),
                kind: HirExprKind::Binary(
                    crate::hir::HirBinOp::Add,
                    Box::new(HirExpr {
                        id: HirId(302),
                        kind: HirExprKind::Unary(
                            crate::hir::HirUnOp::Neg,
                            Box::new(HirExpr {
                                id: HirId(303),
                                kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(304),
                        kind: HirExprKind::Literal(HirLiteral::Int(2)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(305),
                kind: HirExprKind::Call(
                    Box::new(HirExpr {
                        id: HirId(306),
                        kind: HirExprKind::Path(DefId(1)),
                        ty: None,
                        span: span(),
                    }),
                    vec![HirExpr {
                        id: HirId(307),
                        kind: HirExprKind::Literal(HirLiteral::String(numerus_name)),
                        ty: Some(types.primitive(Primitive::Textus)),
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(308),
                kind: HirExprKind::MethodCall(
                    Box::new(HirExpr {
                        id: HirId(309),
                        kind: HirExprKind::Path(DefId(2)),
                        ty: None,
                        span: span(),
                    }),
                    method,
                    vec![HirExpr {
                        id: HirId(310),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(311),
                kind: HirExprKind::Index(
                    Box::new(HirExpr {
                        id: HirId(312),
                        kind: HirExprKind::Field(
                            Box::new(HirExpr {
                                id: HirId(313),
                                kind: HirExprKind::Path(DefId(3)),
                                ty: None,
                                span: span(),
                            }),
                            field,
                        ),
                        ty: None,
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(314),
                        kind: HirExprKind::Literal(HirLiteral::Int(0)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(315),
                kind: HirExprKind::Si(
                    Box::new(HirExpr {
                        id: HirId(316),
                        kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    HirBlock {
                        stmts: vec![HirStmt {
                            id: HirId(317),
                            kind: HirStmtKind::Expr(HirExpr {
                                id: HirId(318),
                                kind: HirExprKind::Literal(HirLiteral::Int(9)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                            span: span(),
                        }],
                        expr: None,
                        span: span(),
                    },
                    Some(HirBlock {
                        stmts: Vec::new(),
                        expr: Some(Box::new(HirExpr {
                            id: HirId(319),
                            kind: HirExprKind::Literal(HirLiteral::Int(10)),
                            ty: Some(numerus),
                            span: span(),
                        })),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(320),
                kind: HirExprKind::Discerne(
                    Box::new(HirExpr {
                        id: HirId(321),
                        kind: HirExprKind::Path(DefId(4)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    vec![HirCasuArm {
                        pattern: HirPattern::Variant(DefId(5), vec![HirPattern::Binding(DefId(6), method)]),
                        guard: Some(HirExpr {
                            id: HirId(322),
                            kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                            ty: Some(bivalens),
                            span: span(),
                        }),
                        body: HirExpr {
                            id: HirId(323),
                            kind: HirExprKind::Literal(HirLiteral::Float(1.5)),
                            ty: Some(types.primitive(Primitive::Fractus)),
                            span: span(),
                        },
                        span: span(),
                    }],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(324),
                kind: HirExprKind::Loop(HirBlock { stmts: Vec::new(), expr: None, span: span() }),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(325),
                kind: HirExprKind::Dum(
                    Box::new(HirExpr {
                        id: HirId(326),
                        kind: HirExprKind::Literal(HirLiteral::Bool(false)),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    HirBlock { stmts: Vec::new(), expr: None, span: span() },
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(327),
                kind: HirExprKind::Itera(
                    DefId(7),
                    Box::new(HirExpr {
                        id: HirId(328),
                        kind: HirExprKind::Array(vec![HirExpr {
                            id: HirId(329),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        }]),
                        ty: None,
                        span: span(),
                    }),
                    HirBlock { stmts: Vec::new(), expr: None, span: span() },
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(330),
                kind: HirExprKind::Assign(
                    Box::new(HirExpr {
                        id: HirId(331),
                        kind: HirExprKind::Path(DefId(8)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(332),
                        kind: HirExprKind::Literal(HirLiteral::Int(11)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(333),
                kind: HirExprKind::AssignOp(
                    crate::hir::HirBinOp::Sub,
                    Box::new(HirExpr {
                        id: HirId(334),
                        kind: HirExprKind::Path(DefId(9)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    Box::new(HirExpr {
                        id: HirId(335),
                        kind: HirExprKind::Literal(HirLiteral::Int(4)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(336),
                kind: HirExprKind::Struct(
                    DefId(10),
                    vec![(
                        field,
                        HirExpr {
                            id: HirId(337),
                            kind: HirExprKind::Literal(HirLiteral::Nil),
                            ty: Some(types.primitive(Primitive::Nihil)),
                            span: span(),
                        },
                    )],
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(338),
                kind: HirExprKind::Clausura(
                    vec![HirParam {
                        def_id: DefId(11),
                        name: method,
                        ty: numerus,
                        mode: HirParamMode::Owned,
                        span: span(),
                    }],
                    None,
                    Box::new(HirExpr {
                        id: HirId(339),
                        kind: HirExprKind::Path(DefId(11)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(340),
                kind: HirExprKind::Cede(Box::new(HirExpr {
                    id: HirId(341),
                    kind: HirExprKind::Path(DefId(12)),
                    ty: None,
                    span: span(),
                })),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(342),
                kind: HirExprKind::Qua(
                    Box::new(HirExpr {
                        id: HirId(343),
                        kind: HirExprKind::Literal(HirLiteral::Int(5)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    numerus,
                ),
                ty: Some(numerus),
                span: span(),
            },
            HirExpr {
                id: HirId(344),
                kind: HirExprKind::Ref(
                    crate::hir::HirRefKind::Mutable,
                    Box::new(HirExpr {
                        id: HirId(345),
                        kind: HirExprKind::Path(DefId(13)),
                        ty: Some(numerus),
                        span: span(),
                    }),
                ),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(346),
                kind: HirExprKind::Deref(Box::new(HirExpr {
                    id: HirId(347),
                    kind: HirExprKind::Path(DefId(14)),
                    ty: None,
                    span: span(),
                })),
                ty: None,
                span: span(),
            },
            HirExpr {
                id: HirId(348),
                kind: HirExprKind::Block(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(349),
                        kind: HirStmtKind::Local(crate::hir::HirLocal {
                            def_id: DefId(15),
                            name: field,
                            ty: Some(err_ty),
                            init: None,
                            mutable: true,
                        }),
                        span: span(),
                    }],
                    expr: Some(Box::new(HirExpr {
                        id: HirId(350),
                        kind: HirExprKind::Error,
                        ty: Some(err_ty),
                        span: span(),
                    })),
                    span: span(),
                }),
                ty: None,
                span: span(),
            },
        ]),
        ty: None,
        span: span(),
    };

    super::expr::generate_expr(&codegen, &expr, &types, &mut w).expect("expr codegen");
    let code = w.finish();

    assert!(code.contains("todo_var"));
    assert!(code.contains("method("));
    assert!(code.contains("match "));
    assert!(code.contains("loop "));
    assert!(code.contains("while "));
    assert!(code.contains("for var in "));
    assert!(code.contains("Struct"));
    assert!(code.contains("|p|"));
    assert!(code.contains(".await"));
    assert!(code.contains(" as i64"));
    assert!(code.contains("&mut "));
    assert!(code.contains("*todo_var"));
    assert!(code.contains("todo!(\"error\")"));
}

#[test]
fn type_to_rust_covers_composite_and_special_cases() {
    let mut interner = Interner::new();
    let sym_t = interner.intern("T");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let fractus = types.primitive(Primitive::Fractus);

    let struct_ty = types.intern(Type::Struct(DefId(100)));
    let enum_ty = types.intern(Type::Enum(DefId(101)));
    let iface_ty = types.intern(Type::Interface(DefId(102)));
    let alias_ty = types.intern(Type::Alias(DefId(103), numerus));
    let array_ty = types.array(numerus);
    let map_ty = types.map(textus, numerus);
    let set_ty = types.set(fractus);
    let option_ty = types.option(numerus);
    let ref_ty = types.reference(Mutability::Immutable, numerus);
    let mut_ref_ty = types.reference(Mutability::Mutable, numerus);
    let param_ty = types.intern(Type::Param(sym_t));
    let applied_ty = types.intern(Type::Applied(struct_ty, vec![numerus]));
    let infer_ty = types.intern(Type::Infer(InferVar(1)));
    let union_empty_ty = types.intern(Type::Union(Vec::new()));
    let union_ty = types.intern(Type::Union(vec![numerus, textus]));
    let error_ty = types.intern(Type::Error);
    let sync_fn_ty = types.function(FuncSig {
        params: vec![ParamType {
            ty: numerus,
            mode: ParamMode::Owned,
            optional: false,
        }],
        ret: textus,
        is_async: false,
        is_generator: false,
    });
    let async_fn_ty = types.function(FuncSig {
        params: vec![ParamType {
            ty: numerus,
            mode: ParamMode::Owned,
            optional: false,
        }],
        ret: textus,
        is_async: true,
        is_generator: false,
    });

    let codegen = super::RustCodegen::new(&empty_program(), &interner);

    assert_eq!(super::types::type_to_rust(&codegen, numerus, &types), "i64");
    assert_eq!(super::types::type_to_rust(&codegen, array_ty, &types), "Vec<i64>");
    assert_eq!(super::types::type_to_rust(&codegen, map_ty, &types), "HashMap<String, i64>");
    assert_eq!(super::types::type_to_rust(&codegen, set_ty, &types), "HashSet<f64>");
    assert_eq!(super::types::type_to_rust(&codegen, option_ty, &types), "Option<i64>");
    assert_eq!(super::types::type_to_rust(&codegen, ref_ty, &types), "&i64");
    assert_eq!(super::types::type_to_rust(&codegen, mut_ref_ty, &types), "&mut i64");
    assert_eq!(super::types::type_to_rust(&codegen, struct_ty, &types), "TodoStruct");
    assert_eq!(super::types::type_to_rust(&codegen, enum_ty, &types), "TodoEnum");
    assert_eq!(super::types::type_to_rust(&codegen, iface_ty, &types), "dyn TodoTrait");
    assert_eq!(super::types::type_to_rust(&codegen, alias_ty, &types), "i64");
    assert_eq!(super::types::type_to_rust(&codegen, sync_fn_ty, &types), "fn(i64) -> String");
    assert_eq!(
        super::types::type_to_rust(&codegen, async_fn_ty, &types),
        "impl Future<Output = String>"
    );
    assert_eq!(super::types::type_to_rust(&codegen, param_ty, &types), "T");
    assert_eq!(super::types::type_to_rust(&codegen, applied_ty, &types), "TodoStruct<i64>");
    assert_eq!(super::types::type_to_rust(&codegen, infer_ty, &types), "_");
    assert_eq!(super::types::type_to_rust(&codegen, union_empty_ty, &types), "!");
    assert_eq!(super::types::type_to_rust(&codegen, union_ty, &types), "TodoUnion");
    assert_eq!(super::types::type_to_rust(&codegen, error_ty, &types), "/* error */");
}
