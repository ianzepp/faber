//! Character stream with lookahead

use std::str::Chars;

/// Cursor for reading source characters
pub struct Cursor<'a> {
    source: &'a str,
    chars: Chars<'a>,
    pos: u32,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            pos: 0,
        }
    }

    /// Current byte position
    pub fn pos(&self) -> u32 {
        self.pos
    }

    /// Peek at the next character without consuming
    pub fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// Peek at the character after next
    pub fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next()
    }

    /// Consume and return the next character
    pub fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8() as u32;
        Some(c)
    }

    /// Consume if the next character matches
    pub fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume while predicate holds
    pub fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&predicate) {
            self.advance();
        }
    }

    /// Check if we've reached the end
    pub fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    /// Get a slice of the source
    pub fn slice(&self, start: u32, end: u32) -> &'a str {
        &self.source[start as usize..end as usize]
    }

    /// Remaining source from current position
    pub fn rest(&self) -> &'a str {
        &self.source[self.pos as usize..]
    }
}
