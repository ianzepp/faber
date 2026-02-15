//! Span utilities for AST nodes
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Provides the Spanned trait for uniform access to source positions across
//! different AST node types. This enables generic diagnostic reporting without
//! type-specific position extraction.
//!
//! COMPILER PHASE: All phases (utility)
//! INPUT: AST nodes implementing Spanned
//! OUTPUT: Span values for error reporting and source mapping

use crate::lexer::Span;

/// Trait for AST nodes that have source locations.
///
/// WHY: Enables writing generic diagnostic functions that work on any AST node
/// without needing to pattern match on node types to extract spans. Also makes
/// it explicit which types are located in source.
pub trait Spanned {
    fn span(&self) -> Span;
}
