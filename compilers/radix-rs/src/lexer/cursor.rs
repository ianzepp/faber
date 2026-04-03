//! Character stream cursor with lookahead for lexical analysis
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Provides a thin wrapper around a string iterator that tracks byte positions
//! and provides lookahead capabilities. This abstraction isolates character
//! traversal concerns from token scanning logic.
//!
//! COMPILER PHASE: Lexing
//! INPUT: Raw UTF-8 source text (`&str`)
//! OUTPUT: Character-by-character access with position tracking
//!
//! DESIGN PHILOSOPHY
//! =================
//! - UTF-8 native: Tracks byte offsets, not character indices, for efficient
//!   slicing without re-encoding
//! - Zero-copy lookahead: Uses iterator cloning instead of buffering to peek
//!   ahead without consuming characters
//! - Stateless operations: All mutations go through `advance()` to maintain
//!   position invariants

use std::str::Chars;

// =============================================================================
// CORE TYPE
// =============================================================================

/// Cursor for reading source characters with position tracking.
///
/// WHY: The lexer needs to peek ahead (e.g., distinguish `/` from `//`) and
/// track byte positions for span construction. This type provides both without
/// requiring the lexer to manage raw iterators and offsets separately.
///
/// INVARIANTS:
/// -----------
/// INV-1: `pos` always represents the byte offset of the next character to read
/// INV-2: `chars` is always positioned at `source[pos..]`
/// INV-3: All byte offsets are valid UTF-8 boundaries (guaranteed by Chars)
pub struct Cursor<'a> {
    source: &'a str,
    chars: Chars<'a>,
    pos: u32,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor at the beginning of source text.
    pub fn new(source: &'a str) -> Self {
        Self { source, chars: source.chars(), pos: 0 }
    }

    /// Current byte position in the source.
    ///
    /// WHY: Returned as u32 instead of usize because source files are limited
    /// to 4GB (enforced elsewhere), and u32 makes Span half the size.
    pub fn pos(&self) -> u32 {
        self.pos
    }

    /// Peek at the next character without consuming it.
    ///
    /// WHY: Cloning the iterator is cheap (just copies a pointer and offset),
    /// and avoids buffering overhead for one-character lookahead.
    pub fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// Peek at the character after next.
    ///
    /// WHY: Required for disambiguating triple-quoted strings (""") and
    /// checking `.` followed by a digit in number parsing.
    pub fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
    }

    /// Consume and return the next character.
    ///
    /// WHY: Returns Option to signal EOF cleanly. Updates position by the
    /// UTF-8 byte length of the character to maintain byte-offset invariants.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8() as u32;
        Some(c)
    }

    /// Consume the next character if it matches the expected value.
    ///
    /// WHY: Common pattern for compound operators (e.g., `⊕`, `==`). Returns
    /// bool instead of Option to make conditional chains more ergonomic.
    pub fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume characters while predicate holds.
    ///
    /// WHY: Core primitive for scanning identifiers, numbers, and whitespace.
    /// Stops at first non-matching character or EOF.
    pub fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&predicate) {
            self.advance();
        }
    }

    /// Check if we've reached end of file.
    pub fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    /// Get a slice of the source between two byte positions.
    ///
    /// WHY: Zero-copy extraction of token text. Caller must ensure start and
    /// end are valid UTF-8 boundaries (guaranteed if they came from `pos()`).
    pub fn slice(&self, start: u32, end: u32) -> &'a str {
        &self.source[start as usize..end as usize]
    }

    /// Get the remaining source from current position.
    ///
    /// WHY: Used by error recovery to show context in diagnostic messages.
    pub fn rest(&self) -> &'a str {
        &self.source[self.pos as usize..]
    }
}
