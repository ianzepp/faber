//! Pattern Parsing for Match Expressions
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module handles parsing of patterns used in match expressions (discerne).
//! Patterns support literals, wildcards, identifiers, paths, and binding/aliasing.
//!
//! COMPILER PHASE: Parsing
//! INPUT: Token stream (via Parser methods)
//! OUTPUT: Pattern AST nodes
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Multiple separators: Patterns can be separated by commas or 'et' (and)
//! - Binding variants: Patterns support aliasing (ut) and destructuring (fixum/varia)
//! - Path patterns: Qualified names like `Result.Ok` for enum variant matching
//! - Literal patterns: Direct matching against constants
//!
//! GRAMMAR COVERAGE
//! ================
//! - Wildcard: _ (matches anything, no binding)
//! - Literal: integers, floats, strings, booleans, nil
//! - Identifier: single name, optionally with binding
//! - Path: qualified name (A.B.C), optionally with binding
//! - Binding: 'ut' for alias, 'fixum'/'varia' for destructuring
//!
//! TRADE-OFFS
//! ==========
//! - No nested destructuring: Patterns are shallow for parser simplicity, deep
//!   destructuring deferred to semantic analysis
//! - Lookahead for body detection: Checks for body-starting keywords to avoid
//!   mis-parsing them as pattern separators

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// PATTERN PARSING
// =============================================================================

impl Parser {
    /// Parse comma/et-separated patterns.
    ///
    /// GRAMMAR:
    ///   patterns := pattern (',' pattern | 'et' pattern)*
    ///
    /// WHY: Match arms can match multiple patterns. Both comma and 'et' (and)
    /// serve as separators for readability.
    ///
    /// EDGE: Must detect if-body start tokens to avoid consuming them as separators.
    pub(super) fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();

        loop {
            patterns.push(self.parse_pattern()?);
            if self.eat(&TokenKind::Comma) {
                // Continue parsing patterns
            } else if self.eat_keyword(TokenKind::Et) {
                // 'et' also separates patterns
            } else {
                break;
            }
            // EDGE: Stop if we hit body-starting tokens
            if self.check(&TokenKind::LBrace)
                || self.check_keyword(TokenKind::Reddit)
                || self.check_keyword(TokenKind::Iacit)
                || self.check_keyword(TokenKind::Moritor)
                || self.check_keyword(TokenKind::Tacet)
                || self.check_keyword(TokenKind::Ergo)
            {
                break;
            }
        }

        Ok(patterns)
    }

    /// Parse a single pattern.
    ///
    /// GRAMMAR:
    ///   pattern := '_' | literal | ident [bind] | path bind
    ///   bind := 'ut' ident | ('fixum'|'varia') ident-list
    ///
    /// WHY: Patterns support wildcard, literal matching, simple identifiers, and
    /// qualified paths. Bindings allow capturing matched values or destructuring.
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Wildcard pattern
        if let TokenKind::Underscore(_) = self.peek().kind {
            let span = self.peek().span;
            self.advance();
            return Ok(Pattern::Wildcard(span));
        }

        // Literal pattern
        if let Some(literal) = self.try_parse_pattern_literal() {
            return Ok(literal);
        }

        // Identifier or path pattern
        if let TokenKind::Ident(_) = self.peek().kind {
            let start = self.current_span();
            let mut segments = Vec::new();
            segments.push(self.parse_ident()?);

            // Parse path segments (e.g., Result.Ok)
            while self.eat(&TokenKind::Dot) {
                segments.push(self.parse_ident()?);
            }

            let bind = self.parse_pattern_bind()?;

            // Simple identifier pattern
            if segments.len() == 1 {
                return Ok(Pattern::Ident(segments.remove(0), bind));
            }

            // Path pattern
            let span = start.merge(self.previous_span());
            return Ok(Pattern::Path(PathPattern { segments, bind, span }));
        }

        Err(self.error(ParseErrorKind::InvalidPattern, "expected pattern"))
    }

    /// Parse optional pattern binding.
    ///
    /// GRAMMAR:
    ///   bind := 'ut' ident | ('fixum'|'varia') ident (',' ident)*
    ///
    /// WHY: Patterns can bind matched values in two ways:
    /// - Alias: 'ut name' gives the whole match a new name
    /// - Destructuring: 'fixum name1, name2' extracts fields/elements
    fn parse_pattern_bind(&mut self) -> Result<Option<PatternBind>, ParseError> {
        let bind = if self.eat_keyword(TokenKind::Ut) {
            let alias = self.parse_ident()?;
            Some(PatternBind::Alias(alias))
        } else if self.eat_keyword(TokenKind::Fixum) {
            let names = self.parse_pattern_bind_list()?;
            Some(PatternBind::Bindings { mutability: Mutability::Immutable, names })
        } else if self.eat_keyword(TokenKind::Varia) {
            let names = self.parse_pattern_bind_list()?;
            Some(PatternBind::Bindings { mutability: Mutability::Mutable, names })
        } else {
            None
        };

        Ok(bind)
    }

    /// Parse comma-separated identifier list for destructuring.
    fn parse_pattern_bind_list(&mut self) -> Result<Vec<Ident>, ParseError> {
        let mut names = Vec::new();
        loop {
            names.push(self.parse_ident()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // EDGE: Stop at body-starting tokens or pattern separators
            if self.check(&TokenKind::LBrace)
                || self.check_keyword(TokenKind::Reddit)
                || self.check_keyword(TokenKind::Iacit)
                || self.check_keyword(TokenKind::Moritor)
                || self.check_keyword(TokenKind::Tacet)
                || self.check_keyword(TokenKind::Ergo)
                || self.check_keyword(TokenKind::Et)
            {
                break;
            }
        }
        Ok(names)
    }

    /// Try to parse a literal pattern.
    ///
    /// WHY: Literal patterns enable matching against constant values. Separated
    /// from expression literal parsing because patterns have different context.
    fn try_parse_pattern_literal(&mut self) -> Option<Pattern> {
        match self.peek().kind {
            TokenKind::Integer(n) => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::Integer(n), span))
            }
            TokenKind::Float(n) => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::Float(n), span))
            }
            TokenKind::String(sym) => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::String(sym), span))
            }
            TokenKind::TemplateString(sym) => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::TemplateString(sym), span))
            }
            TokenKind::Verum => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::Bool(true), span))
            }
            TokenKind::Falsum => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::Bool(false), span))
            }
            TokenKind::Nihil => {
                let span = self.peek().span;
                self.advance();
                Some(Pattern::Literal(Literal::Nil, span))
            }
            _ => None,
        }
    }
}
