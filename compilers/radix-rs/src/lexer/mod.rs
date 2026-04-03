//! Lexer module for Faber source code
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Transforms raw UTF-8 source text into a flat token stream with string
//! interning and error collection. The lexer is a single-pass, error-resilient
//! scanner that never panics on malformed input.
//!
//! COMPILER PHASE: Lexing (first phase)
//! INPUT: UTF-8 source text (`&str`)
//! OUTPUT: Token stream + interned strings + lexer errors
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Error collection: Malformed tokens produce Error tokens and continue
//!   scanning, enabling maximum error reporting in a single pass
//! - String interning: All identifiers and string literals are deduplicated
//!   via NFC-normalized Unicode to ensure semantic equivalence
//! - Zero allocations in hot path: Token stream is pre-allocated, cursor
//!   uses iterator cloning for lookahead

mod cursor;
mod scan;
mod token;

pub use cursor::Cursor;
pub use scan::{Interner, Lexer};
pub use token::{Span, Symbol, Token, TokenKind};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Lex source code into tokens.
///
/// WHY: Convenience wrapper for the common case where you want to lex an
/// entire file in one call. Returns ownership of the interner because the
/// parser needs it to resolve symbols.
pub fn lex(source: &str) -> LexResult {
    Lexer::new(source).lex()
}

// =============================================================================
// TYPES
// =============================================================================

/// Result of lexing, containing tokens, errors, and the string interner.
///
/// WHY: Bundles all lexer outputs together. Even when there are errors, the
/// token stream is complete (with Error tokens) so the parser can attempt
/// recovery.
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
    pub interner: Interner,
}

impl LexResult {
    /// Check if lexing completed without errors.
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

// =============================================================================
// ERRORS
// =============================================================================

/// Lexer error with location and message.
///
/// WHY: Errors are collected during lexing rather than immediately reported,
/// allowing the compiler to show all lex errors in a single pass. Each error
/// includes a span for precise diagnostics.
#[derive(Debug, Clone)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
    pub message: String,
}

/// Categories of lexer errors.
///
/// WHY: Structured error kinds enable different recovery strategies and help
/// distinguish syntactic errors (our problem) from encoding errors (user's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    /// String literal missing closing quote
    UnterminatedString,
    /// Block comment missing closing `*/`
    UnterminatedComment,
    /// Malformed numeric literal (e.g., `0x`, `1e`)
    InvalidNumber,
    /// Invalid escape sequence in string
    InvalidEscape,
    /// Character that cannot start a token
    UnexpectedCharacter,
}
