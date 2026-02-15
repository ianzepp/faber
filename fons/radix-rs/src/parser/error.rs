//! Parser Error Types and Classification
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module defines the error types emitted during parsing. Error classification
//! enables downstream tools (IDE integration, error reporting) to provide context-
//! appropriate diagnostics and recovery suggestions.
//!
//! COMPILER PHASE: Parsing
//! INPUT: N/A (error type definitions only)
//! OUTPUT: ParseError instances created throughout parser
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Granular error kinds: Specific error types allow targeted error messages
//! - Lexer compatibility: LexError passthrough maintains single error type across phases
//! - Span tracking: Every error includes source location for IDE integration
//!
//! TRADE-OFFS
//! ==========
//! - Many error kinds vs generic errors: We prefer specific kinds for better UX,
//!   accepting the maintenance cost of updating this enum when grammar changes
//! - String messages vs structured data: Messages are strings for simplicity,
//!   sacrificing type-safe error construction for easier error creation

use crate::lexer::Span;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// A parse error with location and diagnostic information.
///
/// WHY: Bundles error classification (kind), human-readable message, and source
/// location into a single structure for error reporting and IDE integration.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub span: Span,
}

/// Classification of parse errors by syntactic category.
///
/// WHY: Specific error kinds enable:
/// - Targeted error messages in diagnostic output
/// - IDE features like quick-fixes specific to error type
/// - Statistical analysis of common syntax errors
///
/// TRADE-OFF: Requires updating when grammar evolves, but provides better UX
/// than generic "parse error" messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    // Lexer passthrough - allows treating lex/parse errors uniformly
    LexError,

    // General errors - fallback for uncommon cases
    Expected,
    Unexpected,

    // Declaration errors - structural issues in top-level constructs
    InvalidDeclaration,
    MissingFunctionName,
    MissingFunctionBody,
    MissingClassName,
    MissingClassBody,
    InvalidParameter,
    InvalidTypeParameter,
    InvalidModifier,
    InvalidAnnotation,
    DuplicateModifier,

    // Statement errors - control flow and block structure issues
    InvalidStatement,
    MissingCondition,
    MissingBlock,
    InvalidPattern,
    InvalidCasuArm,
    InvalidCasuValue,

    // Expression errors - malformed expressions and operators
    InvalidExpression,
    InvalidLiteral,
    InvalidOperator,
    UnterminatedGroup,
    InvalidCallArgument,
    InvalidMemberAccess,
    InvalidAssignmentTarget,

    // Type annotation errors
    InvalidType,
    InvalidTypeAnnotation,
    UnterminatedTypeParams,

    // Import and directive errors
    InvalidImport,
    InvalidDirective,
}

// =============================================================================
// STANDARD TRAIT IMPLEMENTATIONS
// =============================================================================

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}
