//! Parsed Faber syntax tree surface.
//!
//! This module is the contract between token-level parsing and every later
//! compiler phase. The parser builds these nodes as a faithful, untyped record
//! of the source grammar; semantic analysis, lowering, diagnostics, and tooling
//! consume the same data without needing to know parser internals.
//!
//! The boundary is deliberately syntactic. A node may carry parse-time shape,
//! source spans, annotations, and stable [`NodeId`]s, but it does not assert
//! name resolution, inferred types, backend legality, or target meaning. Those
//! facts belong to later phase tables keyed by the syntax tree.
//!
//! INVARIANTS
//! ==========
//! - Statements and expressions have `NodeId`s suitable for phase-local side
//!   tables.
//! - Spans always describe source locations for diagnostics, not ownership or
//!   semantic extent.
//! - Annotation and directive syntax is preserved here even when its meaning is
//!   interpreted by lowering, package tooling, or backend codegen.
//! - Visitor helpers are traversal utilities, not a completeness guarantee for
//!   semantic analysis.

mod ast;
mod span;
mod visit;

pub use ast::*;
pub use span::Spanned;
pub use visit::{walk_expr, walk_stmt, Visitor};
