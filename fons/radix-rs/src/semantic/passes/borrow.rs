//! Pass 4: Borrow checking
//!
//! Validates ownership and borrowing rules for Rust target.
//! Only runs when targeting Rust.

use crate::hir::HirProgram;
use crate::semantic::{Resolver, SemanticError, TypeTable};

/// Analyze borrowing and ownership
pub fn analyze(
    _hir: &HirProgram,
    _resolver: &Resolver,
    _types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
    // TODO: Implement borrow checking
    //
    // This is a simplified version of Rust's borrow checker,
    // tailored to Faber's `de`/`in`/`ex` annotations.
    //
    // Rules to enforce:
    // 1. A value can have ONE mutable reference OR multiple immutable references
    // 2. References cannot outlive the value they refer to
    // 3. Values marked `ex` are moved and cannot be used after
    // 4. `de` creates an immutable borrow
    // 5. `in` creates a mutable borrow
    //
    // Implementation approach:
    // 1. Track "places" (paths to values: x, x.y, x[i])
    // 2. Track borrows (which places are borrowed, mutably or not)
    // 3. Track moves (which places have been moved from)
    // 4. At each use, verify:
    //    - Not moved
    //    - Not borrowed mutably (if reading)
    //    - Not borrowed at all (if writing or moving)

    Ok(())
}
