//! Pass 6: Linting
//!
//! Produces warnings for common issues.

use crate::hir::HirProgram;
use crate::semantic::{Resolver, TypeTable, SemanticError, SemanticErrorKind, error::WarningKind};
use crate::lexer::Span;

/// Run lint checks
pub fn lint(
    _hir: &HirProgram,
    _resolver: &Resolver,
    _types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut warnings = Vec::new();

    // TODO: Implement lint checks
    //
    // 1. Unused variables (except those prefixed with _)
    // 2. Unused imports
    // 3. Unused functions (private only)
    // 4. Unreachable code after return/throw/panic
    // 5. Unnecessary qua casts (casting to same type)
    // 6. Deprecated features (e.g., `pro` in patterns)
    // 7. Shadowed variables (optional, configurable)

    // Convert warnings to errors (with warning kind)
    let errors: Vec<SemanticError> = warnings
        .into_iter()
        .map(|(kind, msg, span): (WarningKind, String, Span)| {
            SemanticError::new(SemanticErrorKind::Warning(kind), msg, span)
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
