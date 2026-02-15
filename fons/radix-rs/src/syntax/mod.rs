//! Syntax tree definitions and AST traversal utilities
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Provides the Abstract Syntax Tree node definitions, span utilities, and
//! visitor pattern implementation for AST traversal. This module serves as
//! the primary interface between parser and semantic analysis.
//!
//! COMPILER PHASE: Parsing / Semantic Analysis boundary
//! INPUT: Parser constructs these types from tokens
//! OUTPUT: Semantic analysis and HIR lowering consume these types
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Clean abstraction: Parser produces AST, semantic analysis consumes it
//! - Visitor pattern: walk_* functions provide structured traversal
//! - Span tracking: Spanned trait unifies position queries across node types

mod ast;
mod span;
mod visit;

pub use ast::*;
pub use span::Spanned;
pub use visit::{walk_expr, walk_stmt, Visitor};
