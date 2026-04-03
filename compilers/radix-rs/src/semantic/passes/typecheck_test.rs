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

#[test]
fn infers_function_return_type() {
    let mut types = TypeTable::new();
    let mut program = HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(0),
            def_id: DefId(0),
            kind: HirItemKind::Function(HirFunction {
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: None,
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
    assert_eq!(func.ret_ty, Some(types.primitive(Primitive::Numerus)));
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
                        name: method_name,
                        type_params: Vec::new(),
                        params: Vec::new(),
                        ret_ty: Some(numerus),
                        body: None,
                        is_async: false,
                        is_generator: false,
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
