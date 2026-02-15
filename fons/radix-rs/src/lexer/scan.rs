//! Lexer state machine and token scanner
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Implements the core lexing state machine with mode-based keyword resolution.
//! The scanner uses a cursor to read characters and builds tokens one at a time,
//! collecting errors without panicking.
//!
//! COMPILER PHASE: Lexing
//! INPUT: Source string via Cursor
//! OUTPUT: Token stream, string interner, error list
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Mode-based scanning: Different keyword tables activate after `@` and `§`
//!   to avoid treating annotation/section identifiers as reserved keywords
//! - NFC normalization: All interned strings are Unicode NFC-normalized to
//!   ensure `résumé` and `re\u0301sume\u0301` are the same symbol
//! - Error resilience: Invalid characters become Error tokens, scanning continues
//! - Nested comments: Block comments nest like /* /* */ */ for robustness

use super::cursor::Cursor;
use super::token::{Span, Symbol, Token, TokenKind};
use super::{LexError, LexErrorKind, LexResult};
use rustc_hash::FxHashMap;
use unicode_normalization::UnicodeNormalization;

// =============================================================================
// LEXER MODE
// =============================================================================
//
// WHY: Faber has context-sensitive keywords. After `@`, identifiers like
// `verte` are not reserved keywords but annotation names. After `§`, section
// names are treated as identifiers. Modes track this context per-line.

/// Lexer mode - determines which keyword table is active.
///
/// WHY: Prevents annotation and section names from conflicting with language
/// keywords, allowing `@ verte` and `§ functio` to use otherwise-reserved words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerMode {
    /// Normal mode - statement keywords active
    Normal,
    /// Annotation mode - after `@` on current line
    Annotation,
    /// Section mode - after `§` on current line
    Section,
}

impl LexerMode {
    /// Check if mode should reset on newline.
    ///
    /// WHY: Annotation and section modes are line-scoped to avoid requiring
    /// explicit mode resets and to make lexing order-independent.
    fn is_line_based(self) -> bool {
        matches!(self, Self::Annotation | Self::Section)
    }
}

// =============================================================================
// STRING INTERNER
// =============================================================================
//
// WHY: Identifiers and string literals are duplicated frequently. Interning
// reduces memory (one copy per unique string) and makes equality checks O(1).

/// String interner for symbols with NFC normalization.
///
/// WHY: Unicode allows multiple representations of the same visual text (e.g.,
/// é can be U+00E9 or U+0065 U+0301). NFC normalization ensures identifiers
/// with the same semantic content get the same symbol, preventing confusing
/// "different identifier" errors for visually identical names.
///
/// TRADE-OFF: Normalization costs CPU during lexing, but prevents subtle bugs
/// and makes symbol equality checks correct. This is the right trade-off for
/// a production compiler.
pub struct Interner {
    map: FxHashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Self { map: FxHashMap::default(), strings: Vec::new() }
    }

    /// Intern a string, returning its symbol handle.
    ///
    /// WHY: Normalizes to NFC before interning to ensure semantic equivalence.
    /// Uses FxHashMap for speed (strings are untrusted, but we're not defending
    /// against DoS attacks in a compiler).
    pub fn intern(&mut self, s: &str) -> Symbol {
        let normalized: String = s.nfc().collect();
        if let Some(&sym) = self.map.get(&normalized) {
            return sym;
        }
        let sym = Symbol(self.strings.len() as u32);
        self.strings.push(normalized.clone());
        self.map.insert(normalized, sym);
        sym
    }

    /// Resolve a symbol to its string content.
    pub fn resolve(&self, sym: Symbol) -> &str {
        &self.strings[sym.0 as usize]
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// LEXER STATE
// =============================================================================

/// Lexer for Faber source with mode tracking and error collection.
///
/// WHY: Maintains all mutable state for lexing in one place. Mode tracking
/// enables context-sensitive keyword resolution. Error collection allows
/// showing all lex errors in one pass.
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    #[allow(dead_code)]
    source: &'a str,
    interner: Interner,
    tokens: Vec<Token>,
    errors: Vec<LexError>,
    mode: LexerMode,
    current_line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
            source,
            interner: Interner::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
            mode: LexerMode::Normal,
            current_line: 1,
        }
    }

    /// Lex the entire source into tokens.
    ///
    /// WHY: Consumes self and returns all results together. This ensures the
    /// interner and tokens stay synchronized (you can't use symbols from a
    /// different lexer instance).
    pub fn lex(mut self) -> LexResult {
        while !self.cursor.is_eof() {
            self.scan_token();
        }
        // Always emit EOF token for parser convenience
        self.tokens
            .push(Token::new(TokenKind::Eof, Span::new(self.cursor.pos(), self.cursor.pos())));
        LexResult { tokens: self.tokens, errors: self.errors, interner: self.interner }
    }

    /// Scan a single token from the current cursor position.
    ///
    /// WHY: Core dispatch function. Reads one character, determines token kind,
    /// delegates to specialized scanners for complex tokens (strings, numbers,
    /// identifiers), and emits the token.
    fn scan_token(&mut self) {
        let start = self.cursor.pos();

        let Some(c) = self.cursor.advance() else {
            return;
        };

        // Reset mode on newline for line-scoped modes
        if c == '\n' {
            self.current_line += 1;
            if self.mode.is_line_based() {
                self.mode = LexerMode::Normal;
            }
            return; // WHY: Newlines are whitespace in Faber, not tokens
        }

        let kind = match c {
            // Whitespace - skip (except newline handled above)
            ' ' | '\t' | '\r' => return,

            // Single-character punctuation
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '~' => TokenKind::Tilde,
            '@' => {
                // WHY: Switch to annotation mode until end of line
                self.mode = LexerMode::Annotation;
                TokenKind::At
            }
            '§' => {
                // WHY: Switch to section mode until end of line
                self.mode = LexerMode::Section;
                TokenKind::Section
            }

            // Dot or range
            '.' => {
                if self.cursor.eat('.') {
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }

            // Operators with possible compound forms
            '+' => {
                if self.cursor.eat('=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '*' => {
                if self.cursor.eat('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,

            // Minus or arrow
            '-' => {
                if self.cursor.eat('>') {
                    TokenKind::Arrow
                } else if self.cursor.eat('=') {
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }

            // Hash comment (# ... newline)
            '#' => return self.scan_hash_comment(start),

            // Slash or comment
            '/' => {
                if self.cursor.eat('/') {
                    return self.scan_line_comment(start);
                } else if self.cursor.eat('*') {
                    return self.scan_block_comment(start);
                } else if self.cursor.eat('=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }

            // Comparison and assignment
            '=' => {
                if self.cursor.eat('=') {
                    if self.cursor.eat('=') {
                        TokenKind::EqEqEq // WHY: Strict equality like JavaScript
                    } else {
                        TokenKind::EqEq
                    }
                } else {
                    TokenKind::Eq
                }
            }

            '!' => {
                if self.cursor.eat('=') {
                    if self.cursor.eat('=') {
                        TokenKind::BangEqEq // WHY: Strict inequality
                    } else {
                        TokenKind::BangEq
                    }
                } else if self.cursor.eat('.') {
                    TokenKind::BangDot // WHY: Non-null assertion x!.y
                } else if self.cursor.eat('[') {
                    TokenKind::BangBracket // WHY: Non-null assertion x![i]
                } else if self.cursor.eat('(') {
                    TokenKind::BangParen // WHY: Non-null assertion x!()
                } else {
                    TokenKind::Bang
                }
            }

            '<' => {
                if self.cursor.eat('=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }

            '>' => {
                if self.cursor.eat('=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }

            // Logical and bitwise
            '&' => {
                if self.cursor.eat('&') {
                    TokenKind::AmpAmp // WHY: Logical AND (short-circuits)
                } else if self.cursor.eat('=') {
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp // WHY: Bitwise AND
                }
            }

            '|' => {
                if self.cursor.eat('|') {
                    TokenKind::PipePipe // WHY: Logical OR (short-circuits)
                } else if self.cursor.eat('=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe // WHY: Bitwise OR
                }
            }

            // Optional chaining
            '?' => {
                if self.cursor.eat('.') {
                    TokenKind::QuestionDot // WHY: Optional chaining x?.y
                } else if self.cursor.eat('[') {
                    TokenKind::QuestionBracket // WHY: Optional indexing x?[i]
                } else if self.cursor.eat('(') {
                    TokenKind::QuestionParen // WHY: Optional call x?()
                } else {
                    TokenKind::Question // WHY: Ternary conditional
                }
            }

            // String literals (double or single quoted)
            '"' | '\'' => return self.scan_string(start, c),

            // Template literals (backtick)
            '`' => return self.scan_template(start),

            // Numbers
            '0'..='9' => return self.scan_number(start, c),

            // Identifiers and keywords
            c if is_ident_start(c) => return self.scan_identifier(start),

            _ => {
                // WHY: Emit error but continue scanning for maximum error reporting
                self.errors.push(LexError {
                    kind: LexErrorKind::UnexpectedCharacter,
                    span: Span::new(start, self.cursor.pos()),
                    message: format!("unexpected character: '{}'", c),
                });
                TokenKind::Error
            }
        };

        self.tokens
            .push(Token::new(kind, Span::new(start, self.cursor.pos())));
    }

    // =========================================================================
    // COMMENT SCANNERS
    // =========================================================================

    /// Scan a line comment starting with `//`.
    ///
    /// WHY: Doc comments (`///`) are preserved for documentation generation.
    /// Regular comments are kept in the token stream for formatting tools.
    fn scan_line_comment(&mut self, start: u32) {
        // Check for doc comment (///)
        let is_doc = self.cursor.eat('/');

        self.cursor.eat_while(|c| c != '\n');

        let content_start = if is_doc { start + 3 } else { start + 2 };
        let text = self.cursor.slice(content_start, self.cursor.pos());
        let sym = self.interner.intern(text.trim());

        let kind = if is_doc {
            TokenKind::DocComment(sym)
        } else {
            TokenKind::LineComment(sym)
        };

        self.tokens
            .push(Token::new(kind, Span::new(start, self.cursor.pos())));
    }

    /// Scan a hash comment starting with `#`.
    ///
    /// WHY: Supports shebang lines (`#!/usr/bin/env faber`) and Python-style
    /// comments for familiarity.
    fn scan_hash_comment(&mut self, start: u32) {
        self.cursor.eat_while(|c| c != '\n');

        let text = self.cursor.slice(start + 1, self.cursor.pos());
        let sym = self.interner.intern(text.trim());

        self.tokens
            .push(Token::new(TokenKind::LineComment(sym), Span::new(start, self.cursor.pos())));
    }

    /// Scan a block comment with nesting support.
    ///
    /// WHY: Nested comments allow commenting out code that already contains
    /// comments without breaking. Unlike C, `/* /* */ */` is valid.
    fn scan_block_comment(&mut self, start: u32) {
        let mut depth = 1;

        while depth > 0 && !self.cursor.is_eof() {
            match self.cursor.advance() {
                Some('/') if self.cursor.eat('*') => depth += 1, // WHY: Nested comment opens
                Some('*') if self.cursor.eat('/') => depth -= 1, // WHY: Comment closes
                _ => {}
            }
        }

        if depth > 0 {
            // WHY: Emit error but still create a token for recovery
            self.errors.push(LexError {
                kind: LexErrorKind::UnterminatedComment,
                span: Span::new(start, self.cursor.pos()),
                message: "unterminated block comment".to_owned(),
            });
        }

        let text = self
            .cursor
            .slice(start + 2, self.cursor.pos().saturating_sub(2));
        let sym = self.interner.intern(text);
        self.tokens
            .push(Token::new(TokenKind::BlockComment(sym), Span::new(start, self.cursor.pos())));
    }

    // =========================================================================
    // STRING SCANNERS
    // =========================================================================

    /// Scan a string literal (single, double, or triple-quoted).
    ///
    /// WHY: Triple-quoted strings allow multi-line literals without escapes.
    /// Only double quotes support triple-quoting to avoid ambiguity with
    /// single-character literals in other languages.
    fn scan_string(&mut self, start: u32, quote: char) {
        // Check for triple-quoted string (only for double quotes)
        let is_triple = quote == '"' && self.cursor.peek() == Some('"') && self.cursor.peek_next() == Some('"');
        if is_triple {
            self.cursor.advance();
            self.cursor.advance();
        }

        loop {
            match self.cursor.peek() {
                None => {
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedString,
                        span: Span::new(start, self.cursor.pos()),
                        message: "unterminated string literal".to_owned(),
                    });
                    break;
                }
                Some('\n') if !is_triple => {
                    // WHY: Single-line strings can't contain newlines
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedString,
                        span: Span::new(start, self.cursor.pos()),
                        message: "unterminated string literal (newline in string)".to_owned(),
                    });
                    break;
                }
                Some('\\') => {
                    // WHY: Skip escaped character (escape processing happens in parser)
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some(c) if c == quote => {
                    self.cursor.advance();
                    if is_triple {
                        // WHY: Need three consecutive quotes to close
                        if self.cursor.eat('"') && self.cursor.eat('"') {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ => {
                    self.cursor.advance();
                }
            }
        }

        let content_start = if is_triple { start + 3 } else { start + 1 };
        let content_end = if is_triple {
            self.cursor.pos().saturating_sub(3)
        } else {
            self.cursor.pos().saturating_sub(1)
        };
        let text = self.cursor.slice(content_start, content_end);
        let sym = self.interner.intern(text);

        self.tokens
            .push(Token::new(TokenKind::String(sym), Span::new(start, self.cursor.pos())));
    }

    /// Scan a template literal with embedded expressions.
    ///
    /// WHY: Template strings like `result: ${x + y}` are parsed as a single
    /// token here, with interpolation handling deferred to the parser. This
    /// keeps the lexer simple and avoids tokenizing embedded code twice.
    fn scan_template(&mut self, start: u32) {
        let mut terminated = false;

        loop {
            match self.cursor.peek() {
                None => {
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedString,
                        span: Span::new(start, self.cursor.pos()),
                        message: "unterminated template literal".to_owned(),
                    });
                    break;
                }
                Some('\\') => {
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some('$') if self.cursor.peek_next() == Some('{') => {
                    self.cursor.advance();
                    self.cursor.advance();
                    // WHY: Track brace depth to find matching `}`
                    let mut depth = 1;
                    while depth > 0 && !self.cursor.is_eof() {
                        match self.cursor.advance() {
                            Some('{') => depth += 1,
                            Some('}') => depth -= 1,
                            _ => {}
                        }
                    }
                }
                Some('`') => {
                    self.cursor.advance();
                    terminated = true;
                    break;
                }
                _ => {
                    self.cursor.advance();
                }
            }
        }

        let content_start = start + 1;
        let content_end = if terminated {
            self.cursor.pos().saturating_sub(1)
        } else {
            self.cursor.pos()
        };
        let text = if content_end > content_start {
            self.cursor.slice(content_start, content_end)
        } else {
            ""
        };
        let sym = self.interner.intern(text);

        self.tokens
            .push(Token::new(TokenKind::TemplateString(sym), Span::new(start, self.cursor.pos())));
    }

    // =========================================================================
    // NUMBER SCANNER
    // =========================================================================

    /// Scan a numeric literal (integer or float, various radixes).
    ///
    /// WHY: Supports hex (0x), binary (0b), octal (0o) prefixes, underscores
    /// for readability (1_000_000), floats with exponents (1.5e10), and stores
    /// parsed values in the token to avoid re-parsing in later phases.
    ///
    /// EDGE: Invalid numbers (e.g., `0x`, `1e`) emit errors but still create
    /// tokens to avoid cascading parser errors.
    fn scan_number(&mut self, start: u32, first: char) {
        // Check for radix prefix
        if first == '0' {
            match self.cursor.peek() {
                Some('x') | Some('X') => {
                    self.cursor.advance();
                    self.cursor.eat_while(|c| c.is_ascii_hexdigit() || c == '_');
                    let text = self.cursor.slice(start + 2, self.cursor.pos());
                    let clean: String = text.chars().filter(|&c| c != '_').collect();
                    match i64::from_str_radix(&clean, 16) {
                        Ok(n) => {
                            self.tokens
                                .push(Token::new(TokenKind::Integer(n), Span::new(start, self.cursor.pos())));
                        }
                        Err(_) => {
                            self.errors.push(LexError {
                                kind: LexErrorKind::InvalidNumber,
                                span: Span::new(start, self.cursor.pos()),
                                message: "invalid hexadecimal number".to_owned(),
                            });
                        }
                    }
                    return;
                }
                Some('b') | Some('B') => {
                    self.cursor.advance();
                    self.cursor.eat_while(|c| c == '0' || c == '1' || c == '_');
                    let text = self.cursor.slice(start + 2, self.cursor.pos());
                    let clean: String = text.chars().filter(|&c| c != '_').collect();
                    match i64::from_str_radix(&clean, 2) {
                        Ok(n) => {
                            self.tokens
                                .push(Token::new(TokenKind::Integer(n), Span::new(start, self.cursor.pos())));
                        }
                        Err(_) => {
                            self.errors.push(LexError {
                                kind: LexErrorKind::InvalidNumber,
                                span: Span::new(start, self.cursor.pos()),
                                message: "invalid binary number".to_owned(),
                            });
                        }
                    }
                    return;
                }
                Some('o') | Some('O') => {
                    self.cursor.advance();
                    self.cursor
                        .eat_while(|c| ('0'..='7').contains(&c) || c == '_');
                    let text = self.cursor.slice(start + 2, self.cursor.pos());
                    let clean: String = text.chars().filter(|&c| c != '_').collect();
                    match i64::from_str_radix(&clean, 8) {
                        Ok(n) => {
                            self.tokens
                                .push(Token::new(TokenKind::Integer(n), Span::new(start, self.cursor.pos())));
                        }
                        Err(_) => {
                            self.errors.push(LexError {
                                kind: LexErrorKind::InvalidNumber,
                                span: Span::new(start, self.cursor.pos()),
                                message: "invalid octal number".to_owned(),
                            });
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        // Decimal number
        self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');

        // WHY: Check for `.` followed by digit to distinguish floats from ranges (1..10)
        let is_float = if self.cursor.peek() == Some('.') && self.cursor.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            self.cursor.advance(); // consume '.'
            self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');
            true
        } else {
            false
        };

        // Exponent
        let has_exp = if self.cursor.peek() == Some('e') || self.cursor.peek() == Some('E') {
            self.cursor.advance();
            if !self.cursor.eat('+') {
                self.cursor.eat('-');
            }
            self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');
            true
        } else {
            false
        };

        let text = self.cursor.slice(start, self.cursor.pos());
        let clean: String = text.chars().filter(|&c| c != '_').collect();

        if is_float || has_exp {
            match clean.parse::<f64>() {
                Ok(n) => {
                    self.tokens
                        .push(Token::new(TokenKind::Float(n), Span::new(start, self.cursor.pos())));
                }
                Err(_) => {
                    self.errors.push(LexError {
                        kind: LexErrorKind::InvalidNumber,
                        span: Span::new(start, self.cursor.pos()),
                        message: "invalid floating-point number".to_owned(),
                    });
                }
            }
        } else {
            match clean.parse::<i64>() {
                Ok(n) => {
                    self.tokens
                        .push(Token::new(TokenKind::Integer(n), Span::new(start, self.cursor.pos())));
                }
                Err(_) => {
                    self.errors.push(LexError {
                        kind: LexErrorKind::InvalidNumber,
                        span: Span::new(start, self.cursor.pos()),
                        message: "invalid integer".to_owned(),
                    });
                }
            }
        }
    }

    // =========================================================================
    // IDENTIFIER AND KEYWORD SCANNER
    // =========================================================================

    /// Scan an identifier or keyword.
    ///
    /// WHY: Uses Unicode XID identifiers (allowing non-ASCII names like `ελληνικά`)
    /// and dispatches to mode-specific keyword tables to handle context-sensitive
    /// reserved words.
    fn scan_identifier(&mut self, start: u32) {
        self.cursor.eat_while(is_ident_continue);

        let text = self.cursor.slice(start, self.cursor.pos());
        let kind = match self.mode {
            LexerMode::Normal => keyword_or_ident(text, &mut self.interner),
            LexerMode::Annotation => annotation_keyword_or_ident(text, &mut self.interner),
            LexerMode::Section => section_keyword_or_ident(text, &mut self.interner),
        };
        self.tokens
            .push(Token::new(kind, Span::new(start, self.cursor.pos())));
    }
}

// =============================================================================
// IDENTIFIER PREDICATES
// =============================================================================

/// Check if character can start an identifier.
///
/// WHY: Uses Unicode XID_Start (allowing Greek, Cyrillic, CJK, etc.) instead
/// of ASCII-only to support international codebases. Underscore is allowed for
/// convention (e.g., `_unused`).
fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

/// Check if character can continue an identifier.
///
/// WHY: XID_Continue includes digits (so `x1` is valid) and combining marks.
fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

#[cfg(test)]
#[path = "scan_test.rs"]
mod tests;

// =============================================================================
// KEYWORD TABLES
// =============================================================================

/// Normal mode keywords - statements and expressions.
///
/// WHY: Main keyword table for Faber language. Returns keyword token for
/// reserved words, otherwise interns the identifier.
fn keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    match text {
        "_" => TokenKind::Underscore(interner.intern(text)),
        // Declarations
        "fixum" => TokenKind::Fixum,
        "varia" => TokenKind::Varia,
        "functio" => TokenKind::Functio,
        "genus" => TokenKind::Genus,
        "pactum" => TokenKind::Pactum,
        "typus" => TokenKind::Typus,
        "ordo" => TokenKind::Ordo,
        "discretio" => TokenKind::Discretio,
        "importa" => TokenKind::Importa,
        "probandum" => TokenKind::Probandum,
        "proba" => TokenKind::Proba,

        // Modifiers
        "abstractus" => TokenKind::Abstractus,
        "generis" => TokenKind::Generis,
        "nexum" => TokenKind::Nexum,
        "publica" => TokenKind::Publica,
        "privata" => TokenKind::Privata,
        "protecta" => TokenKind::Protecta,
        "prae" => TokenKind::Prae,
        "ceteri" => TokenKind::Ceteri,
        "immutata" => TokenKind::Immutata,
        "iacit" => TokenKind::Iacit,
        "curata" => TokenKind::Curata,
        "errata" => TokenKind::Errata,
        "exitus" => TokenKind::Exitus,
        "optiones" => TokenKind::Optiones,

        // Control flow
        "si" => TokenKind::Si,
        "sic" => TokenKind::Sic,
        "sin" => TokenKind::Sin,
        "secus" => TokenKind::Secus,
        "dum" => TokenKind::Dum,
        "itera" => TokenKind::Itera,
        "elige" => TokenKind::Elige,
        "casu" => TokenKind::Casu,
        "ceterum" => TokenKind::Ceterum,
        "discerne" => TokenKind::Discerne,
        "custodi" => TokenKind::Custodi,
        "fac" => TokenKind::Fac,
        "ergo" => TokenKind::Ergo,

        // Transfer
        "redde" => TokenKind::Redde,
        "reddit" => TokenKind::Reddit,
        "rumpe" => TokenKind::Rumpe,
        "perge" => TokenKind::Perge,
        "tacet" => TokenKind::Tacet,

        // Error handling
        "tempta" => TokenKind::Tempta,
        "cape" => TokenKind::Cape,
        "demum" => TokenKind::Demum,
        "iace" => TokenKind::Iace,
        "mori" => TokenKind::Mori,
        "moritor" => TokenKind::Moritor,
        "adfirma" => TokenKind::Adfirma,

        // Closures
        "clausura" => TokenKind::Clausura,
        "cede" => TokenKind::Cede,

        // Boolean/null
        "verum" => TokenKind::Verum,
        "falsum" => TokenKind::Falsum,
        "nihil" => TokenKind::Nihil,

        // Logical
        "et" => TokenKind::Et,
        "aut" => TokenKind::Aut,
        "non" => TokenKind::Non,
        "vel" => TokenKind::Vel,
        "est" => TokenKind::Est,

        // Objects
        "ego" => TokenKind::Ego,
        "novum" => TokenKind::Novum,
        "finge" => TokenKind::Finge,
        "sub" => TokenKind::Sub,
        "implet" => TokenKind::Implet,

        // Type operations
        "qua" => TokenKind::Qua,
        "innatum" => TokenKind::Innatum,
        "numeratum" => TokenKind::Numeratum,
        "fractatum" => TokenKind::Fractatum,
        "textatum" => TokenKind::Textatum,
        "bivalentum" => TokenKind::Bivalentum,

        // Bitwise
        "sinistratum" => TokenKind::Sinistratum,
        "dextratum" => TokenKind::Dextratum,

        // Output
        "scribe" => TokenKind::Scribe,
        "vide" => TokenKind::Vide,
        "mone" => TokenKind::Mone,

        // Entry points
        "incipit" => TokenKind::Incipit,
        "incipiet" => TokenKind::Incipiet,
        "argumenta" => TokenKind::Argumenta,
        "cura" => TokenKind::Cura,
        "arena" => TokenKind::Arena,
        "ad" => TokenKind::Ad,

        // Misc keywords
        "ex" => TokenKind::Ex,
        "de" => TokenKind::De,
        "in" => TokenKind::In,
        "ut" => TokenKind::Ut,
        "pro" => TokenKind::Pro,
        "omnia" => TokenKind::Omnia,
        "sparge" => TokenKind::Sparge,
        "praefixum" => TokenKind::Praefixum,
        "scriptum" => TokenKind::Scriptum,
        "lege" => TokenKind::Lege,
        "lineam" => TokenKind::Lineam,
        "sed" => TokenKind::Sed,

        // Ranges
        "ante" => TokenKind::Ante,
        "usque" => TokenKind::Usque,
        "per" => TokenKind::Per,
        "intra" => TokenKind::Intra,
        "inter" => TokenKind::Inter,

        // Collection DSL
        "ab" => TokenKind::Ab,
        "ubi" => TokenKind::Ubi,
        "prima" => TokenKind::Prima,
        "ultima" => TokenKind::Ultima,
        "summa" => TokenKind::Summa,

        // Testing
        "praepara" => TokenKind::Praepara,
        "praeparabit" => TokenKind::Praeparabit,
        "postpara" => TokenKind::Postpara,
        "postparabit" => TokenKind::Postparabit,
        "omitte" => TokenKind::Omitte,
        "futurum" => TokenKind::Futurum,
        "solum" => TokenKind::Solum,
        "tag" => TokenKind::Tag,
        "temporis" => TokenKind::Temporis,
        "metior" => TokenKind::Metior,
        "repete" => TokenKind::Repete,
        "fragilis" => TokenKind::Fragilis,
        "requirit" => TokenKind::Requirit,
        "solum_in" => TokenKind::SolumIn,

        // Nullability
        "nulla" => TokenKind::Nulla,
        "nonnulla" => TokenKind::Nonnulla,
        "nonnihil" => TokenKind::Nonnihil,
        "negativum" => TokenKind::Negativum,
        "positivum" => TokenKind::Positivum,

        // Identifier
        _ => TokenKind::Ident(interner.intern(text)),
    }
}

/// Annotation mode keywords - words after `@`.
///
/// WHY: In annotation context, all identifiers are treated as names, not
/// reserved keywords. This allows `@ verte` and `@ functio` without conflicts.
fn annotation_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}

/// Section mode keywords - words after `§`.
///
/// WHY: Section names (like `§ functio`) are identifiers, not keywords.
fn section_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}
