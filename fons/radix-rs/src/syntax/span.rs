//! Span utilities for AST nodes

use crate::lexer::Span;

/// Trait for AST nodes that have source locations
pub trait Spanned {
    fn span(&self) -> Span;
}
