//! Recursive descent parser for Faber

mod decl;
mod stmt;
mod expr;
mod pattern;
mod types;
mod error;

pub use error::{ParseError, ParseErrorKind};

use crate::lexer::{Token, TokenKind, Span, Symbol, LexResult};
use crate::syntax::*;

/// Parse result
pub struct ParseResult {
    pub program: Option<Program>,
    pub errors: Vec<ParseError>,
}

impl ParseResult {
    pub fn success(&self) -> bool {
        self.program.is_some() && self.errors.is_empty()
    }
}

/// Parse tokens into an AST
pub fn parse(lex_result: LexResult) -> ParseResult {
    if !lex_result.success() {
        return ParseResult {
            program: None,
            errors: lex_result
                .errors
                .into_iter()
                .map(|e| ParseError {
                    kind: ParseErrorKind::LexError,
                    message: e.message,
                    span: e.span,
                })
                .collect(),
        };
    }

    let mut parser = Parser::new(lex_result.tokens);
    parser.parse_program()
}

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    next_node_id: NodeId,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            next_node_id: 0,
        }
    }

    /// Parse the entire program
    pub fn parse_program(&mut self) -> ParseResult {
        let start = self.current_span();
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        let end = self.previous_span();
        let span = start.merge(end);

        ParseResult {
            program: Some(Program { stmts, span }),
            errors: std::mem::take(&mut self.errors),
        }
    }

    // =========================================================================
    // Token operations
    // =========================================================================

    /// Get a fresh node ID
    fn next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    /// Peek at current token (skipping comments)
    fn peek(&self) -> &Token {
        self.peek_at(0)
    }

    /// Peek at token at offset (skipping comments)
    fn peek_at(&self, offset: usize) -> &Token {
        let mut pos = self.pos;
        let mut skipped = 0;
        while pos < self.tokens.len() {
            if !self.tokens[pos].kind.is_comment() {
                if skipped == offset {
                    return &self.tokens[pos];
                }
                skipped += 1;
            }
            pos += 1;
        }
        self.tokens.last().unwrap() // EOF
    }

    /// Check if current token matches
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    /// Check if current token is a specific keyword
    fn check_keyword(&self, kw: TokenKind) -> bool {
        self.peek().kind == kw
    }

    /// Advance and return current token
    fn advance(&mut self) -> &Token {
        // Skip comments
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_comment() {
            self.pos += 1;
        }
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    /// Consume token if it matches, return true
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume token if it matches keyword, return true
    fn eat_keyword(&mut self, kw: TokenKind) -> bool {
        if self.check_keyword(kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect a specific token, error if not found
    fn expect(&mut self, kind: &TokenKind, msg: &str) -> Result<&Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(ParseErrorKind::Expected, msg))
        }
    }

    /// Expect a specific keyword
    fn expect_keyword(&mut self, kw: TokenKind, msg: &str) -> Result<(), ParseError> {
        if self.check_keyword(kw) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParseErrorKind::Expected, msg))
        }
    }

    /// Check if at EOF
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    /// Get span of current token
    fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Get span of previous token
    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::default()
        }
    }

    /// Check if this is a simple variable declaration without type annotation.
    /// Pattern: identifier followed by '='
    fn is_simple_var_decl(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
            && matches!(self.peek_at(1).kind, TokenKind::Eq)
    }

    /// Create an error at current position
    fn error(&self, kind: ParseErrorKind, message: &str) -> ParseError {
        ParseError {
            kind,
            message: message.to_owned(),
            span: self.current_span(),
        }
    }

    /// Synchronize after error - skip to next statement
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Stop at statement boundaries
            match self.peek().kind {
                TokenKind::Fixum
                | TokenKind::Varia
                | TokenKind::Functio
                | TokenKind::Genus
                | TokenKind::Pactum
                | TokenKind::Typus
                | TokenKind::Ordo
                | TokenKind::Discretio
                | TokenKind::Si
                | TokenKind::Dum
                | TokenKind::Itera
                | TokenKind::Elige
                | TokenKind::Discerne
                | TokenKind::Custodi
                | TokenKind::Redde
                | TokenKind::Tempta
                | TokenKind::Importa
                | TokenKind::Section
                | TokenKind::Incipit
                | TokenKind::Incipiet
                | TokenKind::Probandum => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parse an identifier
    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        let token = self.advance();
        match &token.kind {
            TokenKind::Ident(sym) => Ok(Ident {
                name: *sym,
                span: token.span,
            }),
            _ => Err(ParseError {
                kind: ParseErrorKind::Expected,
                message: "expected identifier".to_owned(),
                span: token.span,
            }),
        }
    }

    /// Try to parse an identifier if present
    fn try_parse_ident(&mut self) -> Option<Ident> {
        if let TokenKind::Ident(sym) = self.peek().kind {
            let span = self.peek().span;
            self.advance();
            Some(Ident { name: sym, span })
        } else {
            None
        }
    }

    /// Parse a string literal
    fn parse_string(&mut self) -> Result<Symbol, ParseError> {
        let token = self.advance();
        match &token.kind {
            TokenKind::String(sym) => Ok(*sym),
            _ => Err(ParseError {
                kind: ParseErrorKind::Expected,
                message: "expected string".to_owned(),
                span: token.span,
            }),
        }
    }
}
