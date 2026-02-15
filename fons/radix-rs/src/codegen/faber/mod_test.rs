use super::{Codegen, DefId, FaberCodegen, Interner, Primitive, TypeTable};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirParam, HirParamMode, HirProgram,
    HirStmt, HirStmtKind,
};
use crate::lexer::Span;

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
