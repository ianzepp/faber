use super::{check, DefId, SemanticErrorKind, Type, TypeId, TypeTable};
use crate::hir::{
    HirBlock, HirCasuArm, HirExpr, HirExprKind, HirItem, HirItemKind, HirLiteral, HirPattern, HirProgram, HirStmt,
    HirStmtKind, HirVariant, HirVariantField,
};
use crate::lexer::Span;

fn span() -> Span {
    Span::default()
}

fn lit_expr(id: u32) -> HirExpr {
    HirExpr {
        id: crate::hir::HirId(id),
        kind: HirExprKind::Literal(HirLiteral::Int(1)),
        ty: Some(TypeId(0)),
        span: span(),
    }
}

fn match_expr(scrutinee_ty: TypeId, patterns: Vec<HirPattern>) -> HirExpr {
    let scrutinee =
        HirExpr { id: crate::hir::HirId(1), kind: HirExprKind::Path(DefId(100)), ty: Some(scrutinee_ty), span: span() };
    let arms = patterns
        .into_iter()
        .map(|pattern| HirCasuArm { pattern, guard: None, body: lit_expr(2), span: span() })
        .collect();
    HirExpr { id: crate::hir::HirId(3), kind: HirExprKind::Discerne(Box::new(scrutinee), arms), ty: None, span: span() }
}

fn program_with_match(enum_def: DefId, match_expr: HirExpr) -> HirProgram {
    HirProgram {
        items: vec![HirItem {
            id: crate::hir::HirId(10),
            def_id: enum_def,
            kind: HirItemKind::Enum(crate::hir::HirEnum {
                name: crate::lexer::Symbol(1),
                type_params: Vec::new(),
                variants: vec![
                    HirVariant {
                        def_id: DefId(2),
                        name: crate::lexer::Symbol(2),
                        fields: Vec::<HirVariantField>::new(),
                        span: span(),
                    },
                    HirVariant {
                        def_id: DefId(3),
                        name: crate::lexer::Symbol(3),
                        fields: Vec::<HirVariantField>::new(),
                        span: span(),
                    },
                ],
            }),
            span: span(),
        }],
        entry: Some(HirBlock {
            stmts: vec![HirStmt { id: crate::hir::HirId(20), kind: HirStmtKind::Expr(match_expr), span: span() }],
            expr: None,
            span: span(),
        }),
    }
}

#[test]
fn reports_non_exhaustive_match() {
    let mut types = TypeTable::new();
    let enum_def = DefId(1);
    let enum_ty = types.intern(Type::Enum(enum_def));
    let match_expr = match_expr(enum_ty, vec![HirPattern::Variant(DefId(2), Vec::new())]);
    let program = program_with_match(enum_def, match_expr);

    let result = check(&program, &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::NonExhaustiveMatch));
}

#[test]
fn reports_duplicate_variant_pattern() {
    let mut types = TypeTable::new();
    let enum_def = DefId(1);
    let enum_ty = types.intern(Type::Enum(enum_def));
    let match_expr = match_expr(
        enum_ty,
        vec![
            HirPattern::Variant(DefId(2), Vec::new()),
            HirPattern::Variant(DefId(2), Vec::new()),
        ],
    );
    let program = program_with_match(enum_def, match_expr);

    let result = check(&program, &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::DuplicatePattern));
}

#[test]
fn reports_unreachable_after_wildcard() {
    let mut types = TypeTable::new();
    let enum_def = DefId(1);
    let enum_ty = types.intern(Type::Enum(enum_def));
    let match_expr = match_expr(enum_ty, vec![HirPattern::Wildcard, HirPattern::Variant(DefId(2), Vec::new())]);
    let program = program_with_match(enum_def, match_expr);

    let result = check(&program, &types);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|err| err.kind == SemanticErrorKind::UnreachablePattern));
}
