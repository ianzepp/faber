//! Parse-error data contract for the compiler front end.
//!
//! The parser reports syntax failures as lightweight diagnostics: a broad
//! category, human-readable message, and source span. The category is used by
//! tests and driver diagnostics; the message remains the precise user-facing
//! explanation chosen at the failure site.
//!
//! INVARIANTS
//! ==========
//! - Lexer failures are converted to `LexError` so callers can handle one parse
//!   result shape even when AST construction never begins.
//! - Every parser-produced error carries the current token span at the point the
//!   grammar contract failed.
//! - Error kinds should remain semantic enough for diagnostics, but not so
//!   granular that every grammar production needs a one-off enum variant.

use crate::lexer::Span;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Syntax diagnostic emitted while building an AST.
///
/// `message` is the display text selected by the parser branch that failed.
/// `kind` is the machine-readable bucket, and `span` points to the token where
/// parsing knew the construct could not satisfy the grammar.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Broad diagnostic category for reporting and tests.
    pub kind: ParseErrorKind,

    /// Human-readable diagnostic detail.
    pub message: String,

    /// Source location attached to the offending or missing construct.
    pub span: Span,
}

/// Parser diagnostic taxonomy.
///
/// Variants are intentionally grouped by the part of the grammar that failed.
/// Add new variants when a category unlocks clearer recovery, tests, or user
/// messaging; otherwise prefer an existing bucket with a precise message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    // Lexer passthrough - allows treating lex/parse errors uniformly
    LexError,

    // General errors - fallback for uncommon cases
    Expected,
    Unexpected,

    // Declaration errors - structural issues in top-level constructs
    DuplicateModifier,
    InvalidAnnotation,
    InvalidDeclaration,
    InvalidModifier,
    InvalidParameter,
    InvalidTypeParameter,
    MissingClassBody,
    MissingClassName,
    MissingFunctionBody,
    MissingFunctionName,

    // Statement errors - control flow and block structure issues
    InvalidCasuArm,
    InvalidCasuValue,
    InvalidPattern,
    InvalidStatement,
    MissingBlock,
    MissingCondition,

    // Expression errors - malformed expressions and operators
    InvalidAssignmentTarget,
    InvalidCallArgument,
    InvalidExpression,
    InvalidLiteral,
    InvalidMemberAccess,
    InvalidOperator,
    UnterminatedGroup,

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
