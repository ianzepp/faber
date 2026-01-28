//! Lexer state machine

use super::cursor::Cursor;
use super::token::{Span, Symbol, Token, TokenKind};
use super::{LexError, LexErrorKind, LexResult};
use rustc_hash::FxHashMap;

/// Lexer mode - determines which keyword table is active
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
    /// Check if mode should reset on newline
    fn is_line_based(self) -> bool {
        matches!(self, Self::Annotation | Self::Section)
    }
}

/// String interner for symbols
pub struct Interner {
    map: FxHashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
            strings: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }
        let sym = Symbol(self.strings.len() as u32);
        self.strings.push(s.to_owned());
        self.map.insert(s.to_owned(), sym);
        sym
    }

    pub fn resolve(&self, sym: Symbol) -> &str {
        &self.strings[sym.0 as usize]
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

/// Lexer for Faber source
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
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

    pub fn lex(mut self) -> LexResult {
        while !self.cursor.is_eof() {
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.cursor.pos(), self.cursor.pos()),
        ));
        LexResult {
            tokens: self.tokens,
            errors: self.errors,
            interner: self.interner,
        }
    }

    fn scan_token(&mut self) {
        let start = self.cursor.pos();

        let Some(c) = self.cursor.advance() else {
            return;
        };

        // Check for newline and reset mode if needed
        if c == '\n' {
            self.current_line += 1;
            if self.mode.is_line_based() {
                self.mode = LexerMode::Normal;
            }
            return;
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
                self.mode = LexerMode::Annotation;
                TokenKind::At
            }
            '§' => {
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

            // Logical and bitwise
            '&' => {
                if self.cursor.eat('&') {
                    TokenKind::AmpAmp
                } else if self.cursor.eat('=') {
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp
                }
            }

            '|' => {
                if self.cursor.eat('|') {
                    TokenKind::PipePipe
                } else if self.cursor.eat('=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }

            // Optional chaining
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

            // String literals (double or single quoted)
            '"' | '\'' => return self.scan_string(start, c),

            // Template literals (backtick)
            '`' => return self.scan_template(start),

            // Numbers
            '0'..='9' => return self.scan_number(start, c),

            // Identifiers and keywords
            c if is_ident_start(c) => return self.scan_identifier(start),

            _ => {
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

    fn scan_hash_comment(&mut self, start: u32) {
        self.cursor.eat_while(|c| c != '\n');

        let text = self.cursor.slice(start + 1, self.cursor.pos());
        let sym = self.interner.intern(text.trim());

        self.tokens.push(Token::new(
            TokenKind::LineComment(sym),
            Span::new(start, self.cursor.pos()),
        ));
    }

    fn scan_block_comment(&mut self, start: u32) {
        let mut depth = 1;

        while depth > 0 && !self.cursor.is_eof() {
            match self.cursor.advance() {
                Some('/') if self.cursor.eat('*') => depth += 1,
                Some('*') if self.cursor.eat('/') => depth -= 1,
                _ => {}
            }
        }

        if depth > 0 {
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
        self.tokens.push(Token::new(
            TokenKind::BlockComment(sym),
            Span::new(start, self.cursor.pos()),
        ));
    }

    fn scan_string(&mut self, start: u32, quote: char) {
        // Check for triple-quoted string (only for double quotes)
        // Use peek to avoid consuming quotes if not a triple
        let is_triple =
            quote == '"' && self.cursor.peek() == Some('"') && self.cursor.peek_next() == Some('"');
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
                    self.errors.push(LexError {
                        kind: LexErrorKind::UnterminatedString,
                        span: Span::new(start, self.cursor.pos()),
                        message: "unterminated string literal (newline in string)".to_owned(),
                    });
                    break;
                }
                Some('\\') => {
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some(c) if c == quote => {
                    self.cursor.advance();
                    if is_triple {
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

        self.tokens.push(Token::new(
            TokenKind::String(sym),
            Span::new(start, self.cursor.pos()),
        ));
    }

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

        self.tokens.push(Token::new(
            TokenKind::TemplateString(sym),
            Span::new(start, self.cursor.pos()),
        ));
    }

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
                            self.tokens.push(Token::new(
                                TokenKind::Integer(n),
                                Span::new(start, self.cursor.pos()),
                            ));
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
                            self.tokens.push(Token::new(
                                TokenKind::Integer(n),
                                Span::new(start, self.cursor.pos()),
                            ));
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
                            self.tokens.push(Token::new(
                                TokenKind::Integer(n),
                                Span::new(start, self.cursor.pos()),
                            ));
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

        let is_float = if self.cursor.peek() == Some('.')
            && self.cursor.peek_next().is_some_and(|c| c.is_ascii_digit())
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
            self.cursor.eat('+') || self.cursor.eat('-');
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
                    self.tokens.push(Token::new(
                        TokenKind::Float(n),
                        Span::new(start, self.cursor.pos()),
                    ));
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
                    self.tokens.push(Token::new(
                        TokenKind::Integer(n),
                        Span::new(start, self.cursor.pos()),
                    ));
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

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Normal mode keywords - statements and expressions
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

/// Annotation mode keywords - words after @
fn annotation_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}

/// Section mode keywords - words after § (no reserved keywords)
fn section_keyword_or_ident(text: &str, interner: &mut Interner) -> TokenKind {
    // Section keywords are not reserved - they become Ident tokens
    if text == "_" {
        TokenKind::Underscore(interner.intern(text))
    } else {
        TokenKind::Ident(interner.intern(text))
    }
}
