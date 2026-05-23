//! Stateful scanner for Faber tokens.
//!
//! This module turns the cursor and token definitions into the actual Phase 2
//! lexer. It owns the tokenization state machine, line-scoped lexer modes,
//! literal parsing, NFC string interning, and the lexical recovery boundary.
//! Parser recovery starts after this module emits tokens; scanner recovery is
//! limited to consuming enough source to produce the next plausible token.
//!
//! TOKENIZATION POLICY
//! ===================
//! - Normal mode reserves language keywords. Annotation and section modes turn
//!   words after `@` and `§` back into identifiers unless the scanner has an
//!   explicit tokenization reason to do otherwise.
//! - String, comment, and identifier payloads are interned after Unicode NFC
//!   normalization; spans still point at the original, unnormalized bytes.
//! - Numeric literals are parsed during lexing so later phases receive typed
//!   values, but invalid numbers remain lexical diagnostics instead of panics.
//! - Newlines are whitespace tokens only in the semantic sense: they reset
//!   line-scoped modes and update line state, but are not emitted.
//!
//! ERROR STRATEGY
//! ==============
//! A malformed token emits a structured [`LexError`](super::LexError), often
//! paired with `TokenKind::Error`, and scanning continues. The scanner does not
//! synthesize missing syntax for the parser or reinterpret later bytes to hide
//! an earlier lexical failure.

use super::cursor::Cursor;
use super::token::{Span, Symbol, Token, TokenKind};
use super::{LexError, LexErrorKind, LexResult};
use rustc_hash::FxHashMap;
use unicode_normalization::UnicodeNormalization;

/// Line-scoped keyword interpretation state.
///
/// Faber intentionally lets annotation and section metadata use words that
/// would be reserved in ordinary source. The scanner tracks that context here
/// instead of pushing it into the parser so tokenization remains deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerMode {
    /// Ordinary source: statement, expression, and declaration keywords apply.
    Normal,

    /// After `@` on the current line: annotation names are identifier-like.
    Annotation,

    /// After `§` on the current line: section names are identifier-like.
    Section,
}

impl LexerMode {
    /// Return whether this mode expires at the next newline.
    fn is_line_based(self) -> bool {
        matches!(self, Self::Annotation | Self::Section)
    }
}

/// String interner for lexer-produced symbols.
///
/// The interner is part of the lexer contract, not a global table. Symbol ids
/// are stable only inside one [`LexResult`](super::LexResult). All payload text
/// is normalized to Unicode NFC before lookup so visually equivalent Faber
/// identifiers and string payloads resolve to the same symbol.
///
/// TRADE-OFF: normalization adds scanner cost, but it prevents later semantic
/// phases from having to reason about canonically equivalent spellings.
pub struct Interner {
    map: FxHashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Self { map: FxHashMap::default(), strings: Vec::new() }
    }

    /// Intern source text and return its symbol handle.
    ///
    /// The returned handle resolves to the normalized spelling, not necessarily
    /// the byte-exact spelling in source. Use token spans when diagnostics need
    /// the original bytes.
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

    /// Resolve a symbol back to its normalized string content.
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

/// Scanner state for one Faber source buffer.
///
/// A lexer instance owns its cursor, token buffer, diagnostics, and interner so
/// the output cannot accidentally mix symbols from another source. Construct a
/// fresh lexer per source buffer.
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

    /// Consume the source and return all lexer products together.
    ///
    /// The method always emits EOF after scanning the source, even if earlier
    /// tokens produced diagnostics. This keeps parser entrypoints simple and
    /// makes lexical failure non-fatal to syntax recovery.
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

    fn scan_operator(&mut self, _start: u32, c: char) -> Option<TokenKind> {
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
            '∷' => TokenKind::Verte,
            '⇒' => TokenKind::Conversio,
            '∴' => TokenKind::Ergo,
            '∪' => TokenKind::Cup,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            '-' => TokenKind::Minus,
            '/' => TokenKind::Slash,
            '=' => TokenKind::Eq,
            '!' => {
                if self.cursor.eat('.') {
                    TokenKind::BangDot
                } else if self.cursor.eat('[') {
                    TokenKind::BangBracket
                } else if self.cursor.eat('(') {
                    TokenKind::BangParen
                } else {
                    TokenKind::Bang
                }
            }
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '←' => TokenKind::Assign,
            '≡' => TokenKind::EqEq,
            '≠' => TokenKind::BangEq,
            '≤' => TokenKind::LtEq,
            '≥' => TokenKind::GtEq,
            '→' => TokenKind::Arrow,
            '⇥' => TokenKind::ExitArrow,
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

    /// Scan one token or recoverable lexical error from the current position.
    ///
    /// This is the scanner's dispatch boundary. Specialized scanners own
    /// token-family policy, while this method owns mode reset, whitespace, and
    /// fallback behavior for characters with no token meaning.
    fn scan_token(&mut self) {
        let start = self.cursor.pos();

        let Some(c) = self.cursor.advance() else {
            return;
        };

        if c == '\n' {
            self.current_line += 1;
            if self.mode.is_line_based() {
                self.mode = LexerMode::Normal;
            }
            return;
        }

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
            '"' => return self.scan_string(start),
            '❝' => return self.scan_block_string(start),
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

    /// Scan a hash comment, the only source comment form currently accepted.
    fn scan_hash_comment(&mut self, start: u32) {
        self.cursor.eat_while(|c| c != '\n');

        let text = self.cursor.slice(start + 1, self.cursor.pos());
        let sym = self.interner.intern(text.trim());

        self.emit_token(TokenKind::LineComment(sym), start);
    }

    // =========================================================================
    // STRING SCANNERS
    // =========================================================================

    /// Scan a single-line string literal delimited by double quotes.
    ///
    /// Single-line strings recover at newline or EOF. Escape handling is
    /// intentionally shallow here: the scanner skips a backslash plus the next
    /// character so delimiter search stays correct, but it does not validate an
    /// escape vocabulary.
    fn scan_string(&mut self, start: u32) {
        if self.cursor.peek() == Some('"') && self.cursor.peek_next() == Some('"') {
            self.cursor.advance();
            self.cursor.advance();
            self.emit_error(
                LexErrorKind::UnexpectedCharacter,
                start,
                "triple-quote string literals are not supported; use ❝...❞ block strings",
            );
            self.emit_token(TokenKind::Error, start);
            return;
        }

        loop {
            match self.cursor.peek() {
                None => {
                    self.emit_error(LexErrorKind::UnterminatedString, start, "unterminated string literal");
                    break;
                }
                Some('\n') => {
                    self.emit_error(
                        LexErrorKind::UnterminatedString,
                        start,
                        "unterminated string literal (newline in string)",
                    );
                    break;
                }
                Some('\\') => {
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some('"') => {
                    self.cursor.advance();
                    break;
                }
                _ => {
                    self.cursor.advance();
                }
            }
        }

        let sym = self.extract_and_intern(start, 1, 1);
        self.emit_token(TokenKind::String(sym), start);
    }

    /// Scan a block string literal delimited by `❝` and `❞`.
    ///
    /// Block strings may contain newlines, so this scanner also preserves the
    /// lexer's line counter for downstream diagnostics.
    fn scan_block_string(&mut self, start: u32) {
        let mut terminated = false;

        loop {
            match self.cursor.peek() {
                None => {
                    self.emit_error(LexErrorKind::UnterminatedString, start, "unterminated block string literal");
                    break;
                }
                Some('❞') => {
                    self.cursor.advance();
                    terminated = true;
                    break;
                }
                Some('\n') => {
                    self.current_line += 1;
                    self.cursor.advance();
                }
                _ => {
                    self.cursor.advance();
                }
            }
        }

        let suffix_len = if terminated { '❞'.len_utf8() as u32 } else { 0 };
        let sym = self.extract_and_intern(start, '❝'.len_utf8() as u32, suffix_len);
        self.emit_token(TokenKind::String(sym), start);
    }

    // =========================================================================
    // NUMBER SCANNER
    // =========================================================================

    /// Scan a numeric literal and store its parsed value in the token.
    ///
    /// Faber accepts decimal integers/floats, exponent forms, radix-prefixed
    /// integers, and `_` separators. A `.` starts a float only when followed by
    /// a digit, preserving member access and range tokens after integers.
    fn scan_number(&mut self, start: u32, first: char) {
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

        self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');

        let is_float = if self.cursor.peek() == Some('.') && self.cursor.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            self.cursor.advance(); // consume '.'
            self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_');
            true
        } else {
            false
        };

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

    /// Scan a Unicode identifier and apply the active keyword policy.
    ///
    /// Identifier boundaries use Unicode XID rules plus `_`. The resulting text
    /// is either resolved through the mode-specific keyword table or interned as
    /// normalized identifier payload.
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

/// Return whether a character can start a Faber identifier.
///
/// Faber follows Unicode XID start rules and additionally permits `_` for
/// conventional unused bindings and internal-style names.
fn is_ident_start(c: char) -> bool {
    unicode_ident::is_xid_start(c) || c == '_'
}

/// Return whether a character can continue a Faber identifier.
fn is_ident_continue(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '_'
}

#[cfg(test)]
#[path = "scan_test.rs"]
mod tests;

// =============================================================================
// KEYWORD TABLES
// =============================================================================

/// Resolve ordinary-source keyword spellings, or intern identifier text.
///
/// This table is the live tokenization surface for current normal-mode
/// spelling. The richer registry in `keywords.rs` records ownership and alias
/// policy for tools that need to reason about current versus transitional
/// spelling.
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
        "sponte" => TokenKind::Sponte,
        "fixus" => TokenKind::Fixus,

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
        "rumpe" => TokenKind::Rumpe,
        "perge" => TokenKind::Perge,
        "tacet" => TokenKind::Tacet,

        // Error handling
        "tempta" => TokenKind::Tempta,
        "cape" => TokenKind::Cape,
        "demum" => TokenKind::Demum,
        "iace" => TokenKind::Iace,
        "mori" => TokenKind::Mori,
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

        // Type operations: only the ∷ glyph produces Verte (postfix static type ascription).
        // qua/innatum/novum are now ordinary identifiers (see verte-alias-clean-break).

        // Output
        "scribe" => TokenKind::Scribe,
        "vide" => TokenKind::Vide,
        "mone" => TokenKind::Mone,
        "nota" => TokenKind::Nota,

        // Entry points
        "incipit" => TokenKind::Incipit,
        "incipiet" => TokenKind::Incipiet,
        "argumenta" => TokenKind::Argumenta,
        "cura" => TokenKind::Cura,
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

/// Resolve words after `@` as annotation payload identifiers.
///
/// Annotation semantics are handled after lexing. Keeping annotation names as
/// identifiers lets the annotation layer own its taxonomy without expanding the
/// global reserved-word surface.
fn annotation_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}

/// Resolve words after `§` as section payload identifiers.
///
/// Section labels are metadata, not grammar keywords. This mirrors annotation
/// mode and avoids reserving ordinary language words solely for section usage.
fn section_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}
