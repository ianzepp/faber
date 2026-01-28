//! Lexer for Faber source code

mod token;
mod cursor;
mod scan;

pub use token::{Token, TokenKind, Span, Symbol};
pub use cursor::Cursor;
pub use scan::{Lexer, Interner};

/// Lex source code into tokens
pub fn lex(source: &str) -> LexResult {
    Lexer::new(source).lex()
}

/// Result of lexing
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

impl LexResult {
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Lexer error
#[derive(Debug, Clone)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    UnterminatedString,
    UnterminatedComment,
    InvalidNumber,
    InvalidEscape,
    UnexpectedCharacter,
}
