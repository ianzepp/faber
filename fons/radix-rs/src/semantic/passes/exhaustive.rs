//! Pass 5: Exhaustiveness checking
//!
//! Verifies that pattern matches cover all cases.

use crate::hir::HirProgram;
use crate::semantic::{TypeTable, SemanticError};

/// Check pattern match exhaustiveness
pub fn check(
    _hir: &HirProgram,
    _types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
    // TODO: Implement exhaustiveness checking
    //
    // For each `discerne` (match) expression:
    // 1. Determine the type being matched
    // 2. If it's a discretio (enum):
    //    - Collect all variant patterns
    //    - Check that all variants are covered
    //    - Account for wildcards
    // 3. If `omnia` is specified, require full exhaustiveness
    // 4. Otherwise, require a default case
    //
    // Also check for:
    // - Redundant patterns (patterns that can never match)
    // - Unreachable arms (arms after a catch-all)

    Ok(())
}
