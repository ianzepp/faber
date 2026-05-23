//! Lexical front end for Faber source text.
//!
//! This module is the compiler boundary between raw UTF-8 source and the token
//! stream consumed by the parser. It owns source-position spans, token and
//! literal representation, keyword spelling policy, Unicode identifier
//! normalization, and lexical diagnostics. Later phases should not re-interpret
//! raw source spelling when a token, span, or interned symbol already carries
//! the contract they need.
//!
//! TOKENIZATION POLICY
//! ===================
//! - Lexing is single-pass and error-resilient: malformed input records a
//!   diagnostic and scanning continues where the cursor can recover.
//! - Spans are byte offsets into the original source; they preserve exact
//!   source locations even when identifiers and strings are NFC-normalized for
//!   symbol identity.
//! - Keyword recognition is mode-aware. Normal source keywords are reserved,
//!   while annotation and section names remain identifier-like after `@` and
//!   `§` so metadata can use language-looking words deliberately.
//! - The lexer does not own grammar recovery. It emits `Error` tokens and
//!   structured [`LexError`] values; parser and diagnostic layers decide how to
//!   present or recover from the surrounding syntax.
//!
//! INVARIANTS
//! ==========
//! - Every successful lex emits a terminal EOF token, even when errors were
//!   collected earlier in the stream.
//! - Tokens and symbols must travel with the [`Interner`] returned in the same
//!   [`LexResult`]; symbol ids are local to one lexer instance.
//! - Lexer errors are diagnostics, not panics. Invalid user source should not
//!   crash this phase.

mod cursor;
mod keywords;
mod scan;
mod token;

pub use cursor::Cursor;
pub use keywords::{keyword_specs, lookup_keyword_spec, KeywordCategory, KeywordOwner, KeywordScope, KeywordSpec};
pub use scan::{Interner, Lexer};
pub use token::{Span, Symbol, Token, TokenKind};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Lex one Faber source buffer into parser-ready tokens.
///
/// This is the convenience boundary for callers that do not need to configure
/// scanner state directly. The returned [`LexResult`] keeps the token stream,
/// collected diagnostics, and symbol table together because interned symbols
/// are meaningful only against the interner that created them.
pub fn lex(source: &str) -> LexResult {
    Lexer::new(source).lex()
}

// =============================================================================
// TYPES
// =============================================================================

/// Complete output of one lexer run.
///
/// A result may contain errors and still carry a full token stream. That split
/// is intentional: downstream phases can attempt syntax recovery while the
/// diagnostics layer still reports precise lexical failures.
pub struct LexResult {
    /// Token stream in source order, always terminated by [`TokenKind::Eof`].
    pub tokens: Vec<Token>,

    /// Lexical diagnostics collected while scanning.
    pub errors: Vec<LexError>,

    /// Symbol table for identifiers, strings, and comments emitted above.
    pub interner: Interner,
}

impl LexResult {
    /// Return whether lexing produced no diagnostics.
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

// =============================================================================
// ERRORS
// =============================================================================

/// One lexical diagnostic with a source span and display-ready message.
///
/// The lexer records errors at the point where tokenization policy is violated,
/// but it does not format multi-line reports or decide parse recovery. Those
/// responsibilities belong to higher diagnostic layers.
#[derive(Debug, Clone)]
pub struct LexError {
    /// Machine-readable class for recovery and diagnostics.
    pub kind: LexErrorKind,

    /// Byte span in the original source that triggered the diagnostic.
    pub span: Span,

    /// Human-readable diagnostic text owned by the lexer.
    pub message: String,
}

/// Lexical failure classes produced before parsing begins.
///
/// These variants describe tokenization failures only. Syntax-level decisions
/// such as "unexpected token" or "missing expression" are intentionally not
/// modeled here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    /// String literal reached newline or EOF before its closing delimiter.
    UnterminatedString,

    /// Numeric literal text could not be parsed under its declared radix/form.
    InvalidNumber,

    /// Escape processing rejected a string escape sequence.
    InvalidEscape,

    /// Character had no token meaning in the current lexer mode.
    UnexpectedCharacter,
}
