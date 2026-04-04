//! Recursive Descent Parser - Token Stream to AST Transformation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module implements a hand-written recursive descent parser that transforms
//! token streams into an Abstract Syntax Tree (AST). The parser uses predictive
//! parsing with manual lookahead, maintaining a single-pass design that collects
//! errors rather than panicking on malformed input.
//!
//! COMPILER PHASE: Parsing
//! INPUT: Token stream from lexer (Vec<Token>) + interned string table
//! OUTPUT: Abstract Syntax Tree (Program node) + collected parse errors
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Error recovery: Collect all errors, synchronize at statement boundaries
//! - Never crash: Malformed input produces errors, not panics
//! - Single pass: No backtracking, predictive lookahead only
//! - Comment transparency: Comments are skipped automatically during token navigation
//! - Type-first syntax: Enforces Faber's `type name` ordering (not `name: type`)
//!
//! TRADE-OFFS
//! ==========
//! - Manual recursion over parser combinators: Better error messages and recovery,
//!   at the cost of more boilerplate code
//! - Synchronization points: We skip to statement boundaries on errors, which may
//!   miss nested errors within malformed statements
//! - NodeId allocation: Sequential allocation is simple but makes parallel parsing
//!   impossible (acceptable for single-file compilation units)
//!
//! ERROR HANDLING
//! ==============
//! The parser collects errors in a Vec and continues parsing after synchronization.
//! Synchronization occurs at statement boundaries (keywords like functio, genus, si).
//! This allows reporting multiple errors per file rather than stopping at the first.

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

/// Result of parsing a complete source file.
///
/// WHY: Bundles the AST, errors, and string interner together so callers receive
/// all information needed for semantic analysis in a single structure.
pub struct ParseResult {
    pub program: Option<Program>,
    pub errors: Vec<ParseError>,
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

/// Internal parser state machine.
///
/// WHY: Encapsulates all mutable state required during parsing, including token
/// position, error accumulation, and node ID generation. Keeping state centralized
/// makes it easier to reason about parser behavior and prevents accidental state
/// sharing between parse operations.
///
/// INVARIANTS
/// ----------
/// INV-1: `pos` never exceeds `tokens.len()`
/// INV-2: `next_node_id` is monotonically increasing
/// INV-3: Token stream always ends with EOF token
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

/// Parse tokens into an Abstract Syntax Tree.
///
/// WHY: This is the primary entry point for the parsing phase. It handles lexer
/// error propagation and constructs the parser state machine.
///
/// ERROR RECOVERY: If lexing failed, we propagate lexer errors as parse errors
/// and return None for the program. This maintains the "collect all errors" principle.
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

    /// Parse the entire program.
    ///
    /// GRAMMAR:
    ///   program := directive* statement*
    ///
    /// WHY: Entry point for top-level parsing. Directives (§ declarations) must
    /// appear before statements, enforcing file structure conventions.
    ///
    /// ERROR RECOVERY: Uses synchronization after each failed statement/directive
    /// to continue parsing and collect multiple errors.
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
    /// WHY: Every AST node needs a unique identifier for semantic analysis passes.
    /// Sequential allocation is simple and sufficient for single-threaded parsing.
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
    /// WHY: Needed for lookahead decisions like distinguishing `fixum name =` from
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
        self.tokens.last().unwrap() // INVARIANT: token stream always ends with EOF
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

    /// Check if this is a simple variable declaration without type annotation.
    ///
    /// WHY: Faber allows both `fixum name = value` and `fixum type name = value`.
    /// Lookahead distinguishes these forms before parsing the type.
    ///
    /// GRAMMAR:
    ///   simple-var := identifier '='
    ///   typed-var  := type identifier '='
    fn is_simple_var_decl(&self) -> bool {
        matches!(self.peek().kind, TokenKind::LBracket)
            || (matches!(self.peek().kind, TokenKind::Ident(_)) && matches!(self.peek_at(1).kind, TokenKind::Eq))
    }

    /// Create an error at current position.
    ///
    /// WHY: Centralizes error construction with automatic span capture.
    fn error(&self, kind: ParseErrorKind, message: &str) -> ParseError {
        ParseError { kind, message: message.to_owned(), span: self.current_span() }
    }

    /// Check whether a token can safely restart statement parsing after an error.
    ///
    /// WHY: Recovery should stop at the same keyword-led boundaries the parser
    /// already treats as statement starts, plus block closure. If this list
    /// drifts from `parse_statement`, malformed input can erase valid following
    /// structure.
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
                | TokenKind::Tempta
                | TokenKind::Adfirma
                | TokenKind::Scribe
                | TokenKind::Vide
                | TokenKind::Mone
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

    /// Synchronize after error - skip to next statement boundary.
    ///
    /// WHY: Error recovery strategy. After encountering a malformed construct,
    /// skip tokens until we reach a known statement start keyword. This prevents
    /// cascading errors from a single syntax mistake.
    ///
    /// TRADE-OFF: May skip over nested errors within the malformed statement,
    /// but allows parsing to continue and report subsequent errors.
    fn synchronize(&mut self, stop_at_rbrace: bool) {
        if Self::is_recovery_boundary(&self.peek().kind) && (stop_at_rbrace || !matches!(self.peek().kind, TokenKind::RBrace))
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

    /// Try to parse an identifier if present.
    ///
    /// WHY: For optional identifier syntax like in collection DSL filters.
    fn try_parse_ident(&mut self) -> Option<Ident> {
        match self.peek().kind {
            TokenKind::Ident(sym) | TokenKind::Underscore(sym) => {
                let span = self.peek().span;
                self.advance();
                Some(Ident { name: sym, span })
            }
            _ => None,
        }
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
