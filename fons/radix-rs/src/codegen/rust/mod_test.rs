use crate::codegen::{self, Target};
use crate::hir::{
    DefId, HirBlock, HirCasuArm, HirEnum, HirExpr, HirExprKind, HirFunction, HirId, HirInterface, HirItem, HirItemKind,
    HirLiteral, HirParam, HirParamMode, HirPattern, HirProgram, HirStmt, HirStmtKind, HirTypeAlias, HirVariant,
};
use crate::lexer::{Interner, Span};
use crate::semantic::{Primitive, Type, TypeTable};

fn span() -> Span {
    Span::default()
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
