//! Recursive-descent front end for Faber source.
//!
//! This module is the boundary between lexer output and the syntax tree consumed
//! by collection, resolution, lowering, typechecking, and backend analysis. It
//! owns token navigation, node-id allocation, parse-error accumulation, and the
//! recovery policy shared by the declaration, statement, expression, type, and
//! pattern subparsers.
//!
//! DESIGN PHILOSOPHY
//! =================
//! The parser is hand-written and predictive because the language grammar has
//! several contextual forms where good diagnostics and controlled recovery matter
//! more than minimizing parser code. Submodules may use bounded lookahead or
//! rollback for local ambiguities, but the public phase contract is still one
//! forward pass from token stream to AST plus diagnostics.
//!
//! INVARIANTS
//! ==========
//! - The token stream is expected to end in EOF and parser position never moves
//!   beyond the stream.
//! - Comments are syntactically transparent; navigation helpers skip them before
//!   grammar code sees a current token.
//! - Faber declaration syntax remains type-first (`textus nomen`), with `_`
//!   used where inference is explicitly requested.
//! - This phase only accepts syntax represented by the lexer and grammar. Legacy
//!   or tempting surface forms should become diagnostics, not hidden aliases.
//! - Node IDs are allocated monotonically in parse order for later compiler
//!   phases; they are not stable source identifiers.
//!
//! ERROR HANDLING
//! ==============
//! Lexer errors are terminal because there is no trustworthy token stream to
//! consume. Parser errors are collected and recovery resumes at statement or
//! block boundaries so one malformed construct does not erase the rest of the
//! file from diagnostics. Recovery is deliberately coarse: it preserves useful
//! following structure rather than pretending malformed subtrees are valid.

mod decl;
mod error;
mod expr;
mod pattern;
mod stmt;
mod types;

pub use error::{ParseError, ParseErrorKind};

use crate::lexer::{Interner, LexResult, Span, Symbol, Token, TokenKind};
use crate::syntax::*;

// =============================================================================
// TYPES
// =============================================================================

/// Result of the parsing phase for one compilation unit.
///
/// The interner is returned with the AST because every identifier and string
/// literal in syntax refers to interned symbols. Callers should treat `program`
/// as usable only when [`ParseResult::success`] is true; an AST may still be
/// present when recoverable syntax errors were collected.
pub struct ParseResult {
    /// Syntax tree for the file when lexing produced a token stream.
    pub program: Option<Program>,

    /// Lexing or parsing diagnostics accumulated for this phase.
    pub errors: Vec<ParseError>,

    /// String table shared by syntax nodes and later compiler phases.
    pub interner: Interner,
}

impl ParseResult {
    /// Check if parsing succeeded without errors.
    ///
    /// WHY: Provides a convenient boolean check for whether compilation can proceed.
    /// A program is only valid if we produced an AST *and* encountered no errors.
    pub fn success(&self) -> bool {
        self.program.is_some() && self.errors.is_empty()
    }
}

/// Mutable state for one parser invocation.
///
/// Subparser modules extend this type instead of each owning token state. That
/// keeps recovery, comment skipping, spans, identifiers, and node allocation
/// consistent across the grammar.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    next_node_id: NodeId,
    interner: Interner,
}

// =============================================================================
// ENTRY POINT
// =============================================================================

/// Parse lexer output into a syntax tree and parser diagnostics.
///
/// Lexing failure is represented as parser-phase output so the driver can report
/// one diagnostic stream. Unlike ordinary parse errors, lexer errors stop AST
/// construction because token recovery has already failed upstream.
pub fn parse(lex_result: LexResult) -> ParseResult {
    // EDGE: Lexer errors are terminal - we can't parse without valid tokens
    if !lex_result.success() {
        return ParseResult {
            program: None,
            errors: lex_result
                .errors
                .into_iter()
                .map(|e| ParseError { kind: ParseErrorKind::LexError, message: e.message, span: e.span })
                .collect(),
            interner: lex_result.interner,
        };
    }

    let mut parser = Parser::new(lex_result.tokens, lex_result.interner);
    parser.parse_program()
}

// =============================================================================
// CORE PARSER IMPLEMENTATION
// =============================================================================

impl Parser {
    /// Create a new parser from tokens and string interner.
    pub fn new(tokens: Vec<Token>, interner: Interner) -> Self {
        Self { tokens, pos: 0, errors: Vec::new(), next_node_id: 0, interner }
    }

    /// Parse a complete Faber source file.
    ///
    /// GRAMMAR:
    ///   program := directive* statement*
    ///
    /// File directives are only accepted before ordinary statements. Once the
    /// parser enters the statement stream, a later `§` token is reported through
    /// statement parsing rather than silently treated as a directive.
    pub fn parse_program(&mut self) -> ParseResult {
        let start = self.current_span();
        let mut directives = Vec::new();
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            if self.check(&TokenKind::Section) {
                match self.parse_directive_decl() {
                    Ok(directive) => directives.push(directive),
                    Err(e) => {
                        self.errors.push(e);
                        self.synchronize(false);
                    }
                }
            } else {
                match self.parse_statement() {
                    Ok(stmt) => stmts.push(stmt),
                    Err(e) => {
                        self.errors.push(e);
                        self.synchronize(false);
                    }
                }
            }
        }

        let end = self.previous_span();
        let span = start.merge(end);

        ParseResult {
            program: Some(Program { directives, stmts, span }),
            errors: std::mem::take(&mut self.errors),
            interner: std::mem::take(&mut self.interner),
        }
    }

    // =============================================================================
    // TOKEN NAVIGATION
    // =============================================================================
    //
    // These helpers abstract token stream navigation with automatic comment skipping.
    // WHY: Comments are syntactically insignificant in Faber, so parser logic
    // should never need to explicitly handle them.

    /// Allocate a fresh node ID for AST construction.
    ///
    /// Later compiler phases use node IDs to attach semantic facts to syntax.
    /// Allocation is parse-order-local, so callers must not persist these IDs as
    /// cross-run source identity.
    fn next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    /// Peek at current token (skipping comments).
    fn peek(&self) -> &Token {
        self.peek_at(0)
    }

    /// Peek at token at offset (skipping comments).
    ///
    /// WHY: Needed for lookahead decisions like distinguishing `fixum _ name =` from
    /// `fixum type name =` without consuming tokens.
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
        debug_assert!(!self.tokens.is_empty(), "token stream always ends with EOF");
        &self.tokens[self.tokens.len() - 1]
    }

    /// Check if current token matches kind (discriminant comparison only).
    ///
    /// WHY: Uses discriminant matching to check token type without comparing
    /// associated data, allowing patterns like `check(&TokenKind::Ident(_))`.
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    /// Check if current token is a specific keyword (exact match).
    fn check_keyword(&self, kw: TokenKind) -> bool {
        self.peek().kind == kw
    }

    /// Advance and return current token.
    ///
    /// WHY: Consuming tokens moves the parser state forward. Automatic comment
    /// skipping ensures parser logic never sees comment tokens.
    fn advance(&mut self) -> &Token {
        // Skip comments automatically
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_comment() {
            self.pos += 1;
        }
        if !self.is_at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    /// Consume token if it matches, return true.
    ///
    /// WHY: Optional token consumption for optional syntax elements like commas.
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume token if it matches keyword, return true.
    fn eat_keyword(&mut self, kw: TokenKind) -> bool {
        if self.check_keyword(kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume an identifier with exact source text.
    ///
    /// WHY: Some source words are contextual rather than globally reserved. This
    /// keeps spellings like `ab` available as ordinary identifiers except where
    /// a parser-owned grammar slot claims them.
    fn eat_identifier_text(&mut self, text: &str) -> bool {
        let TokenKind::Ident(sym) = self.peek().kind else {
            return false;
        };
        if self.interner.resolve(sym) != text {
            return false;
        }
        self.advance();
        true
    }

    /// Expect a specific token, error if not found.
    ///
    /// WHY: Required syntax elements produce errors when missing, allowing the
    /// caller to handle error recovery.
    fn expect(&mut self, kind: &TokenKind, msg: &str) -> Result<&Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(ParseErrorKind::Expected, msg))
        }
    }

    /// Expect a specific keyword.
    fn expect_keyword(&mut self, kw: TokenKind, msg: &str) -> Result<(), ParseError> {
        if self.check_keyword(kw) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(ParseErrorKind::Expected, msg))
        }
    }

    /// Check if at EOF.
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    /// Get span of current token.
    fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Get span of previous token.
    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::default()
        }
    }

    // =============================================================================
    // PARSING HELPERS
    // =============================================================================

    /// Create an error at current position.
    ///
    /// WHY: Centralizes error construction with automatic span capture.
    fn error(&self, kind: ParseErrorKind, message: &str) -> ParseError {
        ParseError { kind, message: message.to_owned(), span: self.current_span() }
    }

    /// Check whether a token can safely restart statement parsing after an error.
    ///
    /// This list is intentionally coupled to `parse_statement`: recovery should
    /// stop at the same keyword-led forms that can legally begin a statement,
    /// plus block closure and EOF. Drift here changes diagnostic reach.
    fn is_recovery_boundary(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Fixum
                | TokenKind::Varia
                | TokenKind::Functio
                | TokenKind::Genus
                | TokenKind::Pactum
                | TokenKind::Typus
                | TokenKind::Ordo
                | TokenKind::Discretio
                | TokenKind::Importa
                | TokenKind::Section
                | TokenKind::Ex
                | TokenKind::Probandum
                | TokenKind::Proba
                | TokenKind::Abstractus
                | TokenKind::Si
                | TokenKind::Dum
                | TokenKind::Itera
                | TokenKind::Elige
                | TokenKind::Discerne
                | TokenKind::Custodi
                | TokenKind::Fac
                | TokenKind::Redde
                | TokenKind::Rumpe
                | TokenKind::Perge
                | TokenKind::Iace
                | TokenKind::Mori
                | TokenKind::Tacet
                | TokenKind::Tempta
                | TokenKind::Adfirma
                | TokenKind::Scribe
                | TokenKind::Vide
                | TokenKind::Mone
                | TokenKind::Nota
                | TokenKind::Incipit
                | TokenKind::Incipiet
                | TokenKind::Cura
                | TokenKind::Ad
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::At
                | TokenKind::Eof
        )
    }

    /// Synchronize after an error by skipping to the next statement boundary.
    ///
    /// Top-level recovery may consume `}` because there is no enclosing block to
    /// protect. Block recovery can stop before `}` so the caller can close the
    /// block normally and preserve the surrounding AST shape.
    fn synchronize(&mut self, stop_at_rbrace: bool) {
        if Self::is_recovery_boundary(&self.peek().kind)
            && (stop_at_rbrace || !matches!(self.peek().kind, TokenKind::RBrace))
        {
            return;
        }

        self.advance();

        while !self.is_at_end() {
            if Self::is_recovery_boundary(&self.peek().kind)
                && (stop_at_rbrace || !matches!(self.peek().kind, TokenKind::RBrace))
            {
                return;
            }

            self.advance();
        }
    }

    // =============================================================================
    // IDENTIFIER PARSING
    // =============================================================================

    /// Parse an identifier.
    ///
    /// WHY: Identifiers are the most common AST element. Some keywords like 'tag'
    /// are allowed as identifiers in specific contexts.
    ///
    /// GRAMMAR:
    ///   ident := IDENT | UNDERSCORE | 'tag'
    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        let token = self.advance();
        let span = token.span;
        match token.kind {
            TokenKind::Ident(sym) | TokenKind::Underscore(sym) => Ok(Ident { name: sym, span }),
            TokenKind::Tag => Ok(self.keyword_ident("tag", span)),
            _ => Err(ParseError { kind: ParseErrorKind::Expected, message: "expected identifier".to_owned(), span }),
        }
    }

    /// Parse a member access identifier.
    ///
    /// WHY: Member names can use contextual keywords like 'cape' and 'inter'
    /// that are reserved elsewhere but valid as field/method names.
    fn parse_member_ident(&mut self) -> Result<Ident, ParseError> {
        let token = self.advance();
        let span = token.span;
        match token.kind {
            TokenKind::Ident(sym) | TokenKind::Underscore(sym) => Ok(Ident { name: sym, span }),
            TokenKind::Cape => Ok(self.keyword_ident("cape", span)),
            TokenKind::Inter => Ok(self.keyword_ident("inter", span)),
            TokenKind::Tempta => Ok(self.keyword_ident("tempta", span)),
            TokenKind::Scribe => Ok(self.keyword_ident("scribe", span)),
            TokenKind::Vide => Ok(self.keyword_ident("vide", span)),
            TokenKind::Mone => Ok(self.keyword_ident("mone", span)),
            TokenKind::Nota => Ok(self.keyword_ident("nota", span)),
            TokenKind::Lege => Ok(self.keyword_ident("lege", span)),
            _ => Err(ParseError { kind: ParseErrorKind::Expected, message: "expected identifier".to_owned(), span }),
        }
    }

    /// Intern a keyword as an identifier.
    ///
    /// WHY: Allows keywords to be used as identifiers in specific contexts
    /// by interning them into the string table.
    fn keyword_ident(&mut self, name: &str, span: Span) -> Ident {
        let sym = self.interner.intern(name);
        Ident { name: sym, span }
    }

    /// Parse a string literal.
    ///
    /// GRAMMAR:
    ///   string := STRING
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

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
