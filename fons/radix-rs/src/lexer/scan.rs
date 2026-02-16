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
        self.emit_token(TokenKind::Eof, self.cursor.pos());
        LexResult { tokens: self.tokens, errors: self.errors, interner: self.interner }
    }

    fn emit_token(&mut self, kind: TokenKind, start: u32) {
        self.tokens
            .push(Token::new(kind, Span::new(start, self.cursor.pos())));
    }

    fn emit_error(&mut self, kind: LexErrorKind, start: u32, message: impl Into<String>) {
        self.errors
            .push(LexError { kind, span: Span::new(start, self.cursor.pos()), message: message.into() });
    }

    fn scan_operator(&mut self, start: u32, c: char) -> Option<TokenKind> {
        let kind = match c {
            '.' => TokenKind::Dot,
            '‥' => TokenKind::DotDot,
            '…' => TokenKind::Ellipsis,
            '∧' => TokenKind::Amp,
            '∨' => TokenKind::Pipe,
            '⊻' => TokenKind::Caret,
            '¬' => TokenKind::Tilde,
            '≪' => TokenKind::Sinistratum,
            '≫' => TokenKind::Dextratum,
            '⊕' => TokenKind::PlusEq,
            '⊖' => TokenKind::MinusEq,
            '⊛' => TokenKind::StarEq,
            '⊘' => TokenKind::SlashEq,
            '⊜' => TokenKind::AmpEq,
            '⊚' => TokenKind::PipeEq,
            '⇢' => TokenKind::Verte,
            '⇒' => TokenKind::Conversio,
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
            '-' => {
                if self.cursor.eat('>') {
                    TokenKind::Arrow
                } else if self.cursor.eat('=') {
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '/' => {
                if self.cursor.eat('/') {
                    self.scan_line_comment(start);
                    return None;
                } else if self.cursor.eat('*') {
                    self.scan_block_comment(start);
                    return None;
                } else if self.cursor.eat('=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '=' => {
                if self.cursor.eat('=') {
                    if self.cursor.eat('=') {
                        TokenKind::EqEqEq
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
                        TokenKind::BangEqEq
                    } else {
                        TokenKind::BangEq
                    }
                } else if self.cursor.eat('.') {
                    TokenKind::BangDot
                } else if self.cursor.eat('[') {
                    TokenKind::BangBracket
                } else if self.cursor.eat('(') {
                    TokenKind::BangParen
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
            '←' => TokenKind::Eq,
            '≡' => TokenKind::EqEq,
            '≠' => TokenKind::BangEq,
            '≤' => TokenKind::LtEq,
            '≥' => TokenKind::GtEq,
            '→' => TokenKind::Arrow,
            '?' => {
                if self.cursor.eat('.') {
                    TokenKind::QuestionDot
                } else if self.cursor.eat('[') {
                    TokenKind::QuestionBracket
                } else if self.cursor.eat('(') {
                    TokenKind::QuestionParen
                } else {
                    TokenKind::Question
                }
            }
            _ => return None,
        };

        Some(kind)
    }

    fn clean_numeric_text(text: &str) -> String {
        text.chars().filter(|&c| c != '_').collect()
    }

    fn scan_radix_int<P>(&mut self, start: u32, radix: u32, predicate: P, error_message: &'static str)
    where
        P: Fn(char) -> bool,
    {
        self.cursor.eat_while(|c| predicate(c) || c == '_');
        let text = self.cursor.slice(start + 2, self.cursor.pos());
        let clean = Self::clean_numeric_text(text);

        match i64::from_str_radix(&clean, radix) {
            Ok(n) => self.emit_token(TokenKind::Integer(n), start),
            Err(_) => self.emit_error(LexErrorKind::InvalidNumber, start, error_message),
        }
    }

    fn extract_and_intern(&mut self, start: u32, prefix_len: u32, suffix_len: u32) -> Symbol {
        let content_start = start + prefix_len;
        let content_end = self.cursor.pos().saturating_sub(suffix_len);
        let text = self.cursor.slice(content_start, content_end);
        self.interner.intern(text)
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

        // Whitespace - skip (except newline handled above)
        if matches!(c, ' ' | '\t' | '\r') {
            return;
        }

        let kind = match c {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '@' => {
                self.mode = LexerMode::Annotation;
                TokenKind::At
            }
            '§' => {
                self.mode = LexerMode::Section;
                TokenKind::Section
            }
            '#' => return self.scan_hash_comment(start),
            '"' | '\'' => return self.scan_string(start, c),
            '`' => return self.scan_template(start),
            '0'..='9' => return self.scan_number(start, c),
            c if is_ident_start(c) => return self.scan_identifier(start),
            c => {
                if let Some(kind) = self.scan_operator(start, c) {
                    kind
                } else {
                    self.emit_error(
                        LexErrorKind::UnexpectedCharacter,
                        start,
                        format!("unexpected character: '{}'", c),
                    );
                    TokenKind::Error
                }
            }
        };

        self.emit_token(kind, start);
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

        self.emit_token(kind, start);
    }

    /// Scan a hash comment starting with `#`.
    ///
    /// WHY: Supports shebang lines (`#!/usr/bin/env faber`) and Python-style
    /// comments for familiarity.
    fn scan_hash_comment(&mut self, start: u32) {
        self.cursor.eat_while(|c| c != '\n');

        let text = self.cursor.slice(start + 1, self.cursor.pos());
        let sym = self.interner.intern(text.trim());

        self.emit_token(TokenKind::LineComment(sym), start);
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
            self.emit_error(LexErrorKind::UnterminatedComment, start, "unterminated block comment");
        }

        let sym = self.extract_and_intern(start, 2, 2);
        self.emit_token(TokenKind::BlockComment(sym), start);
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
                    self.emit_error(LexErrorKind::UnterminatedString, start, "unterminated string literal");
                    break;
                }
                Some('\n') if !is_triple => {
                    // WHY: Single-line strings can't contain newlines
                    self.emit_error(
                        LexErrorKind::UnterminatedString,
                        start,
                        "unterminated string literal (newline in string)",
                    );
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

        let sym = if is_triple {
            self.extract_and_intern(start, 3, 3)
        } else {
            self.extract_and_intern(start, 1, 1)
        };
        self.emit_token(TokenKind::String(sym), start);
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
                    self.emit_error(LexErrorKind::UnterminatedString, start, "unterminated template literal");
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

        let sym = if terminated {
            self.extract_and_intern(start, 1, 1)
        } else {
            self.extract_and_intern(start, 1, 0)
        };
        self.emit_token(TokenKind::TemplateString(sym), start);
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
                    self.scan_radix_int(start, 16, |c| c.is_ascii_hexdigit(), "invalid hexadecimal number");
                    return;
                }
                Some('b') | Some('B') => {
                    self.cursor.advance();
                    self.scan_radix_int(start, 2, |c| c == '0' || c == '1', "invalid binary number");
                    return;
                }
                Some('o') | Some('O') => {
                    self.cursor.advance();
                    self.scan_radix_int(start, 8, |c| ('0'..='7').contains(&c), "invalid octal number");
                    return;
                }
                _ => {}
            }
        }

        // Decimal number
        self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');

        // WHY: Check for `.` followed by digit to distinguish floats from member access.
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
        let clean = Self::clean_numeric_text(text);

        if is_float || has_exp {
            match clean.parse::<f64>() {
                Ok(n) => self.emit_token(TokenKind::Float(n), start),
                Err(_) => self.emit_error(LexErrorKind::InvalidNumber, start, "invalid floating-point number"),
            }
        } else {
            match clean.parse::<i64>() {
                Ok(n) => self.emit_token(TokenKind::Integer(n), start),
                Err(_) => self.emit_error(LexErrorKind::InvalidNumber, start, "invalid integer"),
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
        self.emit_token(kind, start);
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
        "finge" => TokenKind::Finge,
        "sub" => TokenKind::Sub,
        "implet" => TokenKind::Implet,

        // Type operations (qua/innatum/novum are keyword aliases for ⇢)
        "qua" => TokenKind::Verte,
        "innatum" => TokenKind::Verte,
        "novum" => TokenKind::Verte,
        "numeratum" => TokenKind::Numeratum,
        "fractatum" => TokenKind::Fractatum,
        "textatum" => TokenKind::Textatum,
        "bivalentum" => TokenKind::Bivalentum,

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
