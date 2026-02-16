use super::types::type_to_ts;
use super::TsCodegen;
use crate::codegen::{self, Target};
use crate::driver::Config;
use crate::hir::*;
use crate::lexer::{Interner, Span};
use crate::semantic::{FuncSig, ParamMode, ParamType, Primitive, Type, TypeTable};
use crate::{Compiler, Output};

fn span() -> Span {
    Span::default()
}

fn render_ts(program: &HirProgram, types: &TypeTable, interner: &Interner) -> String {
    let output = codegen::generate(Target::TypeScript, program, types, interner).expect("ts codegen");
    let Output::TypeScript(ts) = output else {
        panic!("expected TypeScript output");
    };
    ts.code
}

#[test]
fn maps_primitive_types_to_typescript() {
    let interner = Interner::new();
    let program = HirProgram { items: Vec::new(), entry: None };
    let codegen = TsCodegen::new(&program, &interner);
    let types = TypeTable::new();

    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Textus), &types), "string");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Numerus), &types), "number");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Fractus), &types), "number");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Bivalens), &types), "boolean");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Nihil), &types), "null");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Vacuum), &types), "void");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Numquam), &types), "never");
    assert_eq!(type_to_ts(&codegen, types.primitive(Primitive::Ignotum), &types), "unknown");
}

#[test]
fn maps_collection_and_option_types_to_typescript() {
    let interner = Interner::new();
    let program = HirProgram { items: Vec::new(), entry: None };
    let codegen = TsCodegen::new(&program, &interner);
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let numerus = types.primitive(Primitive::Numerus);

    let arr = types.array(textus);
    let map = types.map(textus, numerus);
    let set = types.set(numerus);
    let opt = types.option(numerus);

    assert_eq!(type_to_ts(&codegen, arr, &types), "Array<string>");
    assert_eq!(type_to_ts(&codegen, map, &types), "Record<string, number>");
    assert_eq!(type_to_ts(&codegen, set, &types), "Set<number>");
    assert_eq!(type_to_ts(&codegen, opt, &types), "number | null");
}

#[test]
fn maps_function_and_generic_application_types() {
    let mut interner = Interner::new();
    let t_sym = interner.intern("T");
    let box_sym = interner.intern("Box");
    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(1),
            def_id: DefId(99),
            kind: HirItemKind::Struct(HirStruct {
                name: box_sym,
                type_params: vec![HirTypeParam {
                    def_id: DefId(100),
                    name: t_sym,
                    span: span(),
                }],
                fields: Vec::new(),
                methods: Vec::new(),
                extends: None,
                implements: Vec::new(),
            }),
            span: span(),
        }],
        entry: None,
    };

    let codegen = TsCodegen::new(&program, &interner);
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let fn_ty = types.function(FuncSig {
        params: vec![
            ParamType { ty: numerus, mode: ParamMode::Owned, optional: false },
            ParamType { ty: textus, mode: ParamMode::Owned, optional: true },
        ],
        ret: numerus,
        is_async: false,
        is_generator: false,
    });
    let generic_base = types.intern(Type::Struct(DefId(99)));
    let generic = types.intern(Type::Applied(generic_base, vec![textus]));

    assert_eq!(type_to_ts(&codegen, fn_ty, &types), "(p1: number, p2?: string) => number");
    assert_eq!(type_to_ts(&codegen, generic, &types), "Box<string>");
}

#[test]
fn resolves_named_user_types() {
    let mut interner = Interner::new();
    let persona = interner.intern("Persona");
    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(1),
            def_id: DefId(7),
            kind: HirItemKind::Struct(HirStruct {
                name: persona,
                type_params: Vec::new(),
                fields: Vec::new(),
                methods: Vec::new(),
                extends: None,
                implements: Vec::new(),
            }),
            span: span(),
        }],
        entry: None,
    };
    let codegen = TsCodegen::new(&program, &interner);
    let mut types = TypeTable::new();
    let named = types.intern(Type::Struct(DefId(7)));
    assert_eq!(type_to_ts(&codegen, named, &types), "Persona");
}

#[test]
fn emit_target_ts_smoke_test() {
    let source = "functio id(numerus x) → numerus { redde x } incipit { fixum numerus y ← id(1) }";
    let compiler = Compiler::new(Config::default().with_target(Target::TypeScript));
    let result = compiler.compile_str("<test>", source);
    assert!(
        result.diagnostics.iter().all(|d| !d.is_error()),
        "unexpected diagnostics: {}",
        result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let Some(Output::TypeScript(out)) = result.output else {
        panic!("expected TypeScript output");
    };
    assert!(out.code.contains("Generated by radix"));
}

#[test]
fn ts_codegen_collects_function_param_names() {
    let mut interner = Interner::new();
    let f_name = interner.intern("f");
    let x_name = interner.intern("x");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(1),
            def_id: DefId(11),
            kind: HirItemKind::Function(HirFunction {
                name: f_name,
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(12),
                    name: x_name,
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    optional: false,
                    span: span(),
                }],
                ret_ty: Some(numerus),
                body: None,
                is_async: false,
                is_generator: false,
            }),
            span: span(),
        }],
        entry: None,
    };
    let codegen = TsCodegen::new(&program, &interner);
    assert_eq!(type_to_ts(&codegen, types.intern(Type::Struct(DefId(11))), &types), "f");
}

#[test]
fn emits_function_and_entry_iife() {
    let mut interner = Interner::new();
    let f_name = interner.intern("id");
    let x_name = interner.intern("x");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(1),
            def_id: DefId(1),
            kind: HirItemKind::Function(HirFunction {
                name: f_name,
                type_params: Vec::new(),
                params: vec![HirParam {
                    def_id: DefId(2),
                    name: x_name,
                    ty: numerus,
                    mode: HirParamMode::Owned,
                    optional: false,
                    span: span(),
                }],
                ret_ty: Some(numerus),
                body: Some(HirBlock {
                    stmts: vec![HirStmt {
                        id: HirId(3),
                        kind: HirStmtKind::Redde(Some(HirExpr {
                            id: HirId(4),
                            kind: HirExprKind::Path(DefId(2)),
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

    let code = render_ts(&program, &types, &interner);
    assert!(code.contains("function id(x: number): number"));
    assert!(code.contains("(() => {"));
}

#[test]
fn emits_class_interface_import_and_variable_statements() {
    let mut interner = Interner::new();
    let person = interner.intern("Persona");
    let greeter = interner.intern("Greeter");
    let field_name = interner.intern("nomen");
    let method_name = interner.intern("salve");
    let param_name = interner.intern("prefix");
    let import_path = interner.intern("./norma");
    let import_item = interner.intern("loge");
    let local_name = interner.intern("x");
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let numerus = types.primitive(Primitive::Numerus);

    let program = HirProgram {
        items: vec![
            HirItem {
                id: HirId(1),
                def_id: DefId(1),
                kind: HirItemKind::Import(HirImport {
                    path: import_path,
                    items: vec![HirImportItem {
                        def_id: DefId(2),
                        name: import_item,
                        alias: None,
                    }],
                }),
                span: span(),
            },
            HirItem {
                id: HirId(3),
                def_id: DefId(3),
                kind: HirItemKind::Interface(HirInterface {
                    name: greeter,
                    type_params: Vec::new(),
                    methods: vec![HirInterfaceMethod {
                        name: method_name,
                        params: vec![HirParam {
                            def_id: DefId(4),
                            name: param_name,
                            ty: textus,
                            mode: HirParamMode::Owned,
                            optional: false,
                            span: span(),
                        }],
                        ret_ty: Some(textus),
                        span: span(),
                    }],
                }),
                span: span(),
            },
            HirItem {
                id: HirId(5),
                def_id: DefId(5),
                kind: HirItemKind::Struct(HirStruct {
                    name: person,
                    type_params: Vec::new(),
                    fields: vec![HirField {
                        def_id: DefId(6),
                        name: field_name,
                        ty: textus,
                        is_static: false,
                        init: None,
                        span: span(),
                    }],
                    methods: vec![HirMethod {
                        def_id: DefId(7),
                        func: HirFunction {
                            name: method_name,
                            type_params: Vec::new(),
                            params: vec![HirParam {
                                def_id: DefId(8),
                                name: param_name,
                                ty: textus,
                                mode: HirParamMode::Owned,
                                optional: false,
                                span: span(),
                            }],
                            ret_ty: Some(textus),
                            body: Some(HirBlock {
                                stmts: Vec::new(),
                                expr: Some(Box::new(HirExpr {
                                    id: HirId(9),
                                    kind: HirExprKind::Path(DefId(8)),
                                    ty: Some(textus),
                                    span: span(),
                                })),
                                span: span(),
                            }),
                            is_async: false,
                            is_generator: false,
                        },
                        receiver: HirReceiver::Ref,
                        span: span(),
                    }],
                    extends: None,
                    implements: vec![DefId(3)],
                }),
                span: span(),
            },
        ],
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(10),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(10),
                        name: local_name,
                        ty: Some(numerus),
                        init: Some(HirExpr {
                            id: HirId(11),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        mutable: true,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(12),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(12),
                        name: interner.intern("y"),
                        ty: Some(numerus),
                        init: Some(HirExpr {
                            id: HirId(13),
                            kind: HirExprKind::Literal(HirLiteral::Int(2)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let code = render_ts(&program, &types, &interner);
    assert!(code.contains("import { loge } from \"./norma\";"));
    assert!(code.contains("interface Greeter"));
    assert!(code.contains("class Persona implements Greeter"));
    assert!(code.contains("let x: number = 1;"));
    assert!(code.contains("const y: number = 2;"));
}

#[test]
fn lowers_logical_and_comparison_operators() {
    let mut interner = Interner::new();
    let x = interner.intern("x");
    let y = interner.intern("y");
    let z = interner.intern("z");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let bivalens = types.primitive(Primitive::Bivalens);
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(20),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(10),
                        name: x,
                        ty: Some(numerus),
                        init: Some(HirExpr {
                            id: HirId(21),
                            kind: HirExprKind::Literal(HirLiteral::Int(1)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(22),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(11),
                        name: y,
                        ty: Some(numerus),
                        init: Some(HirExpr {
                            id: HirId(23),
                            kind: HirExprKind::Literal(HirLiteral::Int(2)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(24),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(12),
                        name: z,
                        ty: Some(numerus),
                        init: Some(HirExpr {
                            id: HirId(25),
                            kind: HirExprKind::Literal(HirLiteral::Int(3)),
                            ty: Some(numerus),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(1),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(2),
                        kind: HirExprKind::Binary(
                            HirBinOp::Or,
                            Box::new(HirExpr {
                                id: HirId(3),
                                kind: HirExprKind::Binary(
                                    HirBinOp::And,
                                    Box::new(HirExpr {
                                        id: HirId(4),
                                        kind: HirExprKind::Binary(
                                            HirBinOp::Eq,
                                            Box::new(HirExpr {
                                                id: HirId(5),
                                                kind: HirExprKind::Path(DefId(10)),
                                                ty: Some(numerus),
                                                span: span(),
                                            }),
                                            Box::new(HirExpr {
                                                id: HirId(6),
                                                kind: HirExprKind::Path(DefId(11)),
                                                ty: Some(numerus),
                                                span: span(),
                                            }),
                                        ),
                                        ty: Some(bivalens),
                                        span: span(),
                                    }),
                                    Box::new(HirExpr {
                                        id: HirId(7),
                                        kind: HirExprKind::Binary(
                                            HirBinOp::LtEq,
                                            Box::new(HirExpr {
                                                id: HirId(8),
                                                kind: HirExprKind::Path(DefId(12)),
                                                ty: Some(numerus),
                                                span: span(),
                                            }),
                                            Box::new(HirExpr {
                                                id: HirId(9),
                                                kind: HirExprKind::Literal(HirLiteral::Int(10)),
                                                ty: Some(numerus),
                                                span: span(),
                                            }),
                                        ),
                                        ty: Some(bivalens),
                                        span: span(),
                                    }),
                                ),
                                ty: Some(bivalens),
                                span: span(),
                            }),
                            Box::new(HirExpr {
                                id: HirId(13),
                                kind: HirExprKind::Binary(
                                    HirBinOp::Coalesce,
                                    Box::new(HirExpr {
                                        id: HirId(14),
                                        kind: HirExprKind::Path(DefId(10)),
                                        ty: Some(numerus),
                                        span: span(),
                                    }),
                                    Box::new(HirExpr {
                                        id: HirId(15),
                                        kind: HirExprKind::Path(DefId(11)),
                                        ty: Some(numerus),
                                        span: span(),
                                    }),
                                ),
                                ty: Some(numerus),
                                span: span(),
                            }),
                        ),
                        ty: Some(bivalens),
                        span: span(),
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };
    let code = render_ts(&program, &types, &interner);
    assert!(code.contains("==="));
    assert!(code.contains("<="));
    assert!(code.contains("&&"));
    assert!(code.contains("||"));
    assert!(code.contains("??"));
}

#[test]
fn emits_optional_chain_closure_template_and_await() {
    let mut interner = Interner::new();
    let obj = interner.intern("obj");
    let field = interner.intern("nomen");
    let arg = interner.intern("x");
    let tmpl = interner.intern("salve §1");
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let numerus = types.primitive(Primitive::Numerus);
    let option_text = types.option(textus);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(100),
            def_id: DefId(3),
            kind: HirItemKind::Function(HirFunction {
                name: interner.intern("fetch"),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(textus),
                body: Some(HirBlock {
                    stmts: Vec::new(),
                    expr: Some(Box::new(HirExpr {
                        id: HirId(101),
                        kind: HirExprKind::Literal(HirLiteral::String(interner.intern("ok"))),
                        ty: Some(textus),
                        span: span(),
                    })),
                    span: span(),
                }),
                is_async: true,
                is_generator: false,
            }),
            span: span(),
        }],
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(200),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(1),
                        name: obj,
                        ty: Some(option_text),
                        init: Some(HirExpr {
                            id: HirId(201),
                            kind: HirExprKind::Literal(HirLiteral::Nil),
                            ty: Some(option_text),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(1),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(2),
                        kind: HirExprKind::OptionalChain(
                            Box::new(HirExpr {
                                id: HirId(3),
                                kind: HirExprKind::Path(DefId(1)),
                                ty: Some(option_text),
                                span: span(),
                            }),
                            HirOptionalChainKind::Member(field),
                        ),
                        ty: Some(option_text),
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(4),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(5),
                        kind: HirExprKind::Clausura(
                            vec![HirParam {
                                def_id: DefId(2),
                                name: arg,
                                ty: numerus,
                                mode: HirParamMode::Owned,
                                optional: false,
                                span: span(),
                            }],
                            Some(numerus),
                            Box::new(HirExpr {
                                id: HirId(6),
                                kind: HirExprKind::Path(DefId(2)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                        ),
                        ty: None,
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(7),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(8),
                        kind: HirExprKind::Scriptum(
                            tmpl,
                            vec![HirExpr {
                                id: HirId(9),
                                kind: HirExprKind::Literal(HirLiteral::String(arg)),
                                ty: Some(textus),
                                span: span(),
                            }],
                        ),
                        ty: Some(textus),
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(10),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(11),
                        kind: HirExprKind::Cede(Box::new(HirExpr {
                            id: HirId(12),
                            kind: HirExprKind::Call(
                                Box::new(HirExpr {
                                    id: HirId(13),
                                    kind: HirExprKind::Path(DefId(3)),
                                    ty: None,
                                    span: span(),
                                }),
                                vec![],
                            ),
                            ty: None,
                            span: span(),
                        })),
                        ty: None,
                        span: span(),
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let code = render_ts(&program, &types, &interner);
    assert!(code.contains("obj?.nomen"));
    assert!(code.contains("(x: number): number => x"));
    assert!(code.contains("`salve ${0}`"));
    assert!(code.contains("(async () => {"));
    assert!(code.contains("await "));
}

#[test]
fn emits_collection_pipeline_ab_transforms() {
    let mut interner = Interner::new();
    let items = interner.intern("items");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let lista = types.array(numerus);
    let bivalens = types.primitive(Primitive::Bivalens);
    let program = HirProgram {
        items: Vec::new(),
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(10),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(1),
                        name: items,
                        ty: Some(lista),
                        init: Some(HirExpr {
                            id: HirId(11),
                            kind: HirExprKind::Array(vec![]),
                            ty: Some(lista),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(1),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(2),
                        kind: HirExprKind::Ab {
                            source: Box::new(HirExpr {
                                id: HirId(3),
                                kind: HirExprKind::Path(DefId(1)),
                                ty: Some(lista),
                                span: span(),
                            }),
                            filter: Some(HirCollectionFilter {
                                negated: false,
                                kind: HirCollectionFilterKind::Condition(Box::new(HirExpr {
                                    id: HirId(4),
                                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                                    ty: Some(bivalens),
                                    span: span(),
                                })),
                            }),
                            transforms: vec![
                                HirCollectionTransform {
                                    kind: HirTransformKind::First,
                                    arg: Some(Box::new(HirExpr {
                                        id: HirId(5),
                                        kind: HirExprKind::Literal(HirLiteral::Int(5)),
                                        ty: Some(numerus),
                                        span: span(),
                                    })),
                                },
                                HirCollectionTransform { kind: HirTransformKind::Sum, arg: None },
                            ],
                        },
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let code = render_ts(&program, &types, &interner);
    assert!(code.contains(".filter"));
    assert!(code.contains(".slice(0, 5)"));
    assert!(code.contains(".reduce((acc, value) => acc + value, 0)"));
}

#[test]
fn translates_norma_methods_and_intrinsics() {
    let mut interner = Interner::new();
    let items = interner.intern("items");
    let text = interner.intern("text");
    let table = interner.intern("table");
    let appende = interner.intern("appende");
    let longitudo = interner.intern("longitudo");
    let pone = interner.intern("pone");
    let pavimentum = interner.intern("pavimentum");
    let mut types = TypeTable::new();
    let numerus = types.primitive(Primitive::Numerus);
    let textus = types.primitive(Primitive::Textus);
    let lista = types.array(numerus);
    let tabula = types.map(textus, numerus);

    let program = HirProgram {
        items: vec![HirItem {
            id: HirId(1),
            def_id: DefId(50),
            kind: HirItemKind::Import(HirImport {
                path: interner.intern("norma/mathesis"),
                items: vec![HirImportItem {
                    def_id: DefId(51),
                    name: pavimentum,
                    alias: None,
                }],
            }),
            span: span(),
        }],
        entry: Some(HirBlock {
            stmts: vec![
                HirStmt {
                    id: HirId(2),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(2),
                        name: items,
                        ty: Some(lista),
                        init: Some(HirExpr {
                            id: HirId(3),
                            kind: HirExprKind::Array(vec![]),
                            ty: Some(lista),
                            span: span(),
                        }),
                        mutable: true,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(4),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(4),
                        name: text,
                        ty: Some(textus),
                        init: Some(HirExpr {
                            id: HirId(5),
                            kind: HirExprKind::Literal(HirLiteral::String(interner.intern("salve"))),
                            ty: Some(textus),
                            span: span(),
                        }),
                        mutable: false,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(6),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(6),
                        name: table,
                        ty: Some(tabula),
                        init: Some(HirExpr {
                            id: HirId(7),
                            kind: HirExprKind::Struct(DefId(0), vec![]),
                            ty: Some(tabula),
                            span: span(),
                        }),
                        mutable: true,
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(8),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(9),
                        kind: HirExprKind::MethodCall(
                            Box::new(HirExpr {
                                id: HirId(10),
                                kind: HirExprKind::Path(DefId(2)),
                                ty: Some(lista),
                                span: span(),
                            }),
                            appende,
                            vec![HirExpr {
                                id: HirId(11),
                                kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                ty: Some(numerus),
                                span: span(),
                            }],
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(12),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(13),
                        kind: HirExprKind::MethodCall(
                            Box::new(HirExpr {
                                id: HirId(14),
                                kind: HirExprKind::Path(DefId(4)),
                                ty: Some(textus),
                                span: span(),
                            }),
                            longitudo,
                            vec![],
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(15),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(16),
                        kind: HirExprKind::MethodCall(
                            Box::new(HirExpr {
                                id: HirId(17),
                                kind: HirExprKind::Path(DefId(6)),
                                ty: Some(tabula),
                                span: span(),
                            }),
                            pone,
                            vec![
                                HirExpr {
                                    id: HirId(18),
                                    kind: HirExprKind::Literal(HirLiteral::String(interner.intern("k"))),
                                    ty: Some(textus),
                                    span: span(),
                                },
                                HirExpr {
                                    id: HirId(19),
                                    kind: HirExprKind::Literal(HirLiteral::Int(9)),
                                    ty: Some(numerus),
                                    span: span(),
                                },
                            ],
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                },
                HirStmt {
                    id: HirId(20),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: HirId(21),
                        kind: HirExprKind::Call(
                            Box::new(HirExpr {
                                id: HirId(22),
                                kind: HirExprKind::Path(DefId(51)),
                                ty: None,
                                span: span(),
                            }),
                            vec![HirExpr {
                                id: HirId(23),
                                kind: HirExprKind::Literal(HirLiteral::Float(3.9)),
                                ty: Some(types.primitive(Primitive::Fractus)),
                                span: span(),
                            }],
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    };

    let code = render_ts(&program, &types, &interner);
    assert!(!code.contains("import { pavimentum } from"));
    assert!(code.contains("items.push(1)"));
    assert!(code.contains("text.length"));
    assert!(code.contains("table[\"k\"] = 9"));
    assert!(code.contains("Math.floor(3.9)"));
}
