//! Pass 5: Exhaustiveness checking
//!
//! Verifies that pattern matches cover all cases.

use crate::hir::{
    DefId, HirBlock, HirCasuArm, HirExpr, HirExprKind, HirItem, HirItemKind, HirPattern,
    HirProgram, HirStmt, HirStmtKind,
};
use crate::semantic::{SemanticError, SemanticErrorKind, Type, TypeId, TypeTable};
use rustc_hash::{FxHashMap, FxHashSet};

/// Check pattern match exhaustiveness
pub fn check(hir: &HirProgram, types: &TypeTable) -> Result<(), Vec<SemanticError>> {
    let mut errors = Vec::new();
    let enum_variants = collect_enum_variants(hir);

    for item in &hir.items {
        match &item.kind {
            HirItemKind::Function(func) => {
                if let Some(body) = &func.body {
                    check_block(body, types, &enum_variants, &mut errors);
                }
            }
            HirItemKind::Struct(strukt) => {
                for method in &strukt.methods {
                    if let Some(body) = &method.func.body {
                        check_block(body, types, &enum_variants, &mut errors);
                    }
                }
            }
            HirItemKind::Const(const_item) => {
                check_expr(&const_item.value, types, &enum_variants, &mut errors);
            }
            _ => {}
        }
    }

    if let Some(entry) = &hir.entry {
        check_block(entry, types, &enum_variants, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_enum_variants(hir: &HirProgram) -> FxHashMap<DefId, Vec<DefId>> {
    let mut map = FxHashMap::default();
    for item in &hir.items {
        if let HirItemKind::Enum(enum_item) = &item.kind {
            let variants = enum_item.variants.iter().map(|v| v.def_id).collect();
            map.insert(item.def_id, variants);
        }
    }
    map
}

fn check_block(
    block: &HirBlock,
    types: &TypeTable,
    enum_variants: &FxHashMap<DefId, Vec<DefId>>,
    errors: &mut Vec<SemanticError>,
) {
    for stmt in &block.stmts {
        check_stmt(stmt, types, enum_variants, errors);
    }
    if let Some(expr) = &block.expr {
        check_expr(expr, types, enum_variants, errors);
    }
}

fn check_stmt(
    stmt: &HirStmt,
    types: &TypeTable,
    enum_variants: &FxHashMap<DefId, Vec<DefId>>,
    errors: &mut Vec<SemanticError>,
) {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            if let Some(init) = &local.init {
                check_expr(init, types, enum_variants, errors);
            }
        }
        HirStmtKind::Expr(expr) => check_expr(expr, types, enum_variants, errors),
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                check_expr(expr, types, enum_variants, errors);
            }
        }
        HirStmtKind::Rumpe | HirStmtKind::Perge => {}
    }
}

fn check_expr(
    expr: &HirExpr,
    types: &TypeTable,
    enum_variants: &FxHashMap<DefId, Vec<DefId>>,
    errors: &mut Vec<SemanticError>,
) {
    match &expr.kind {
        HirExprKind::Binary(_, lhs, rhs) => {
            check_expr(lhs, types, enum_variants, errors);
            check_expr(rhs, types, enum_variants, errors);
        }
        HirExprKind::Unary(_, operand) => check_expr(operand, types, enum_variants, errors),
        HirExprKind::Call(callee, args) => {
            check_expr(callee, types, enum_variants, errors);
            for arg in args {
                check_expr(arg, types, enum_variants, errors);
            }
        }
        HirExprKind::MethodCall(receiver, _name, args) => {
            check_expr(receiver, types, enum_variants, errors);
            for arg in args {
                check_expr(arg, types, enum_variants, errors);
            }
        }
        HirExprKind::Field(object, _) => check_expr(object, types, enum_variants, errors),
        HirExprKind::Index(object, index) => {
            check_expr(object, types, enum_variants, errors);
            check_expr(index, types, enum_variants, errors);
        }
        HirExprKind::Block(block) => check_block(block, types, enum_variants, errors),
        HirExprKind::Si(cond, then_block, else_block) => {
            check_expr(cond, types, enum_variants, errors);
            check_block(then_block, types, enum_variants, errors);
            if let Some(block) = else_block {
                check_block(block, types, enum_variants, errors);
            }
        }
        HirExprKind::Discerne(scrutinee, arms) => {
            check_expr(scrutinee, types, enum_variants, errors);
            check_match(scrutinee, arms, types, enum_variants, errors);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    check_expr(guard, types, enum_variants, errors);
                }
                check_expr(&arm.body, types, enum_variants, errors);
            }
        }
        HirExprKind::Loop(block) => check_block(block, types, enum_variants, errors),
        HirExprKind::Dum(cond, block) => {
            check_expr(cond, types, enum_variants, errors);
            check_block(block, types, enum_variants, errors);
        }
        HirExprKind::Itera(_, iter, block) => {
            check_expr(iter, types, enum_variants, errors);
            check_block(block, types, enum_variants, errors);
        }
        HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            check_expr(lhs, types, enum_variants, errors);
            check_expr(rhs, types, enum_variants, errors);
        }
        HirExprKind::Array(elements) => {
            for element in elements {
                check_expr(element, types, enum_variants, errors);
            }
        }
        HirExprKind::Struct(_, fields) => {
            for (_, value) in fields {
                check_expr(value, types, enum_variants, errors);
            }
        }
        HirExprKind::Tuple(elements) => {
            for element in elements {
                check_expr(element, types, enum_variants, errors);
            }
        }
        HirExprKind::Clausura(_, _, body) => check_expr(body, types, enum_variants, errors),
        HirExprKind::Cede(expr)
        | HirExprKind::Qua(expr, _)
        | HirExprKind::Ref(_, expr)
        | HirExprKind::Deref(expr) => check_expr(expr, types, enum_variants, errors),
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
    }
}

fn check_match(
    scrutinee: &HirExpr,
    arms: &[HirCasuArm],
    types: &TypeTable,
    enum_variants: &FxHashMap<DefId, Vec<DefId>>,
    errors: &mut Vec<SemanticError>,
) {
    let Some(scrutinee_ty) = scrutinee.ty else {
        return;
    };

    let enum_def = enum_def_from_type(scrutinee_ty, types);
    let expected_variants = enum_def.and_then(|def_id| enum_variants.get(&def_id));

    let mut covered = FxHashSet::default();
    let mut has_catchall = false;

    for arm in arms {
        let is_guarded = arm.guard.is_some();
        let is_catchall = matches!(
            arm.pattern,
            HirPattern::Wildcard | HirPattern::Binding(_, _)
        );

        if has_catchall {
            errors.push(SemanticError::new(
                SemanticErrorKind::UnreachablePattern,
                "unreachable pattern",
                arm.span,
            ));
            continue;
        }

        match &arm.pattern {
            HirPattern::Variant(def_id, _) => {
                if !is_guarded {
                    if !covered.insert(*def_id) {
                        errors.push(SemanticError::new(
                            SemanticErrorKind::DuplicatePattern,
                            "duplicate pattern",
                            arm.span,
                        ));
                    }
                }
            }
            HirPattern::Wildcard | HirPattern::Binding(_, _) => {
                if !is_guarded {
                    has_catchall = true;
                }
            }
            HirPattern::Literal(_) => {}
        }
    }

    if let Some(expected) = expected_variants {
        if !has_catchall {
            let missing: Vec<_> = expected
                .iter()
                .filter(|def_id| !covered.contains(def_id))
                .collect();
            if !missing.is_empty() {
                errors.push(SemanticError::new(
                    SemanticErrorKind::NonExhaustiveMatch,
                    "non-exhaustive match",
                    scrutinee.span,
                ));
            }
        }
    }
}

fn enum_def_from_type(ty: TypeId, types: &TypeTable) -> Option<DefId> {
    match types.get(ty) {
        Type::Enum(def_id) => Some(*def_id),
        Type::Applied(base, _) => enum_def_from_type(*base, types),
        Type::Alias(_, resolved) => enum_def_from_type(*resolved, types),
        Type::Ref(_, inner) => enum_def_from_type(*inner, types),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{
        HirBlock, HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt, HirStmtKind, HirVariant,
        HirVariantField,
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
        let scrutinee = HirExpr {
            id: crate::hir::HirId(1),
            kind: HirExprKind::Path(DefId(100)),
            ty: Some(scrutinee_ty),
            span: span(),
        };
        let arms = patterns
            .into_iter()
            .map(|pattern| HirCasuArm {
                pattern,
                guard: None,
                body: lit_expr(2),
                span: span(),
            })
            .collect();
        HirExpr {
            id: crate::hir::HirId(3),
            kind: HirExprKind::Discerne(Box::new(scrutinee), arms),
            ty: None,
            span: span(),
        }
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
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(20),
                    kind: HirStmtKind::Expr(match_expr),
                    span: span(),
                }],
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
        let match_expr = match_expr(
            enum_ty,
            vec![
                HirPattern::Wildcard,
                HirPattern::Variant(DefId(2), Vec::new()),
            ],
        );
        let program = program_with_match(enum_def, match_expr);

        let result = check(&program, &types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::UnreachablePattern));
    }
}
