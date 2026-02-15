//! Pass 6: Exhaustiveness checking
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Verifies that discerne (match) expressions cover all possible values of the
//! scrutinee type, preventing runtime match failures for enum variants.
//!
//! COMPILER PHASE: Semantic (Pass 6)
//! INPUT: Typed HIR with pattern match expressions
//! OUTPUT: Exhaustiveness errors for incomplete matches; unreachable pattern warnings
//!
//! WHY: Pattern matching on enums must be exhaustive to prevent runtime panics.
//! Catching missing cases at compile time provides safety and better error
//! messages than runtime failures.
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Conservative Checking: Requires wildcard or complete variant coverage;
//!   does not analyze guard conditions (assumes guards may fail)
//! - Duplicate Detection: Warns on unreachable patterns after catchall
//! - Guarded Patterns: Patterns with guards are not considered exhaustive
//!   (guard might fail), so a catchall is still required
//!
//! LIMITATIONS
//! ===========
//! - No constructor analysis: Does not check literal ranges or nested patterns
//! - Guard blindness: Treats guarded patterns as non-exhaustive
//! - Enum-only: Only checks enum variant coverage, not integers or strings

use crate::hir::{
    DefId, HirBlock, HirCasuArm, HirExpr, HirExprKind, HirItemKind, HirPattern, HirProgram, HirStmt, HirStmtKind,
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
        HirExprKind::OptionalChain(object, chain) => {
            check_expr(object, types, enum_variants, errors);
            match chain {
                crate::hir::HirOptionalChainKind::Member(_) => {}
                crate::hir::HirOptionalChainKind::Index(index) => check_expr(index, types, enum_variants, errors),
                crate::hir::HirOptionalChainKind::Call(args) => {
                    for arg in args {
                        check_expr(arg, types, enum_variants, errors);
                    }
                }
            }
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
        HirExprKind::Itera(_, _, iter, block) => {
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
        HirExprKind::Scribe(elements) => {
            for element in elements {
                check_expr(element, types, enum_variants, errors);
            }
        }
        HirExprKind::Scriptum(_, args) => {
            for arg in args {
                check_expr(arg, types, enum_variants, errors);
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            check_expr(cond, types, enum_variants, errors);
            if let Some(message) = message {
                check_expr(message, types, enum_variants, errors);
            }
        }
        HirExprKind::Panic(value) | HirExprKind::Throw(value) => check_expr(value, types, enum_variants, errors),
        HirExprKind::Tempta { body, catch, finally } => {
            check_block(body, types, enum_variants, errors);
            if let Some(catch) = catch {
                check_block(catch, types, enum_variants, errors);
            }
            if let Some(finally) = finally {
                check_block(finally, types, enum_variants, errors);
            }
        }
        HirExprKind::Clausura(_, _, body) => check_expr(body, types, enum_variants, errors),
        HirExprKind::Cede(expr) | HirExprKind::Qua(expr, _) | HirExprKind::Ref(_, expr) | HirExprKind::Deref(expr) => {
            check_expr(expr, types, enum_variants, errors)
        }
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
        let _is_catchall = matches!(arm.pattern, HirPattern::Wildcard | HirPattern::Binding(_, _));

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
                if !is_guarded && !covered.insert(*def_id) {
                    errors.push(SemanticError::new(
                        SemanticErrorKind::DuplicatePattern,
                        "duplicate pattern",
                        arm.span,
                    ));
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
#[path = "exhaustive_test.rs"]
mod tests;
