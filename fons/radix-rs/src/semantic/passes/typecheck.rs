//! Pass 3: Type checking
//!
//! Bidirectional type inference and checking.

use crate::hir::HirProgram;
use crate::semantic::{Resolver, TypeTable, SemanticError};

/// Type check the HIR
pub fn typecheck(
    _hir: &HirProgram,
    _resolver: &Resolver,
    _types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    // TODO: Implement type checking
    //
    // Bidirectional approach:
    // - synthesize(expr) -> TypeId: infer type bottom-up
    // - check(expr, expected: TypeId): verify type top-down
    //
    // 1. For each function:
    //    - Synthesize parameter types
    //    - Check body against return type (if present)
    //    - Infer return type if not annotated
    //
    // 2. For expressions:
    //    - Literals: synthesize directly
    //    - Variables: look up type from symbol
    //    - Calls: check args against param types, synthesize return
    //    - Binary ops: check operand compatibility
    //    - etc.
    //
    // 3. Type inference variables:
    //    - Create fresh InferVar for unknown types
    //    - Unify during checking
    //    - Error if cannot resolve

    Ok(())
}
