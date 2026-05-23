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

use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{DefId, HirCasuArm, HirExpr, HirExprKind, HirItemKind, HirPattern, HirProgram};
use crate::semantic::{SemanticError, SemanticErrorKind, Type, TypeId, TypeTable};
use rustc_hash::{FxHashMap, FxHashSet};

/// Check pattern match exhaustiveness
pub fn check(hir: &HirProgram, types: &TypeTable) -> Result<(), Vec<SemanticError>> {
    let enum_variants = collect_enum_variants(hir);
    let mut checker = ExhaustiveChecker { types, enum_variants, errors: Vec::new() };
    checker.visit_program(hir);

    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
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

struct ExhaustiveChecker<'a> {
    types: &'a TypeTable,
    enum_variants: FxHashMap<DefId, Vec<DefId>>,
    errors: Vec<SemanticError>,
}

impl HirVisitor for ExhaustiveChecker<'_> {
    fn visit_expr(&mut self, expr: &HirExpr) {
        if let HirExprKind::Discerne(scrutinees, arms) = &expr.kind {
            check_match(scrutinees, arms, self.types, &self.enum_variants, &mut self.errors);
        }
        walk_expr(self, expr);
    }
}

fn check_match(
    scrutinees: &[HirExpr],
    arms: &[HirCasuArm],
    types: &TypeTable,
    enum_variants: &FxHashMap<DefId, Vec<DefId>>,
    errors: &mut Vec<SemanticError>,
) {
    fn pattern_variant_id(pattern: &HirPattern) -> Option<DefId> {
        match pattern {
            HirPattern::Variant(def_id, _) => Some(*def_id),
            HirPattern::Alias(_, _, inner) => pattern_variant_id(inner),
            _ => None,
        }
    }

    fn is_catchall_pattern(pattern: &HirPattern) -> bool {
        match pattern {
            HirPattern::Wildcard | HirPattern::Binding(_, _) => true,
            HirPattern::Alias(_, _, inner) => is_catchall_pattern(inner),
            HirPattern::Variant(_, _) | HirPattern::Literal(_) => false,
        }
    }

    if scrutinees.len() != 1 {
        return;
    }

    let Some(scrutinee_ty) = scrutinees[0].ty else {
        return;
    };

    let enum_def = enum_def_from_type(scrutinee_ty, types);
    let expected_variants = enum_def.and_then(|def_id| enum_variants.get(&def_id));

    let mut covered = FxHashSet::default();
    let mut has_catchall = false;

    for arm in arms {
        let is_guarded = arm.guard.is_some();
        let _is_catchall = arm.patterns.iter().all(is_catchall_pattern);

        if has_catchall {
            errors.push(SemanticError::new(
                SemanticErrorKind::UnreachablePattern,
                "unreachable pattern",
                arm.span,
            ));
            continue;
        }

        let Some(pattern) = arm.patterns.first() else {
            continue;
        };

        if let Some(def_id) = pattern_variant_id(pattern) {
            if !is_guarded && !covered.insert(def_id) {
                errors.push(SemanticError::new(
                    SemanticErrorKind::DuplicatePattern,
                    "duplicate pattern",
                    arm.span,
                ));
            }
        } else if is_catchall_pattern(pattern) && !is_guarded {
            has_catchall = true;
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
                    scrutinees[0].span,
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
