//! Pattern grammar for `discerne` arms.
//!
//! Pattern parsing is deliberately narrower than expression parsing. It accepts
//! the shapes that can appear after `casu` inside `discerne`: wildcards,
//! literals, identifiers, qualified paths, aliases, and shallow binding lists.
//! It does not parse arbitrary expressions or nested destructuring trees.
//!
//! DESIGN PHILOSOPHY
//! =================
//! The parser records syntactic pattern intent and leaves type-aware matching,
//! variant resolution, and exhaustiveness to later phases. This keeps the
//! recursive-descent boundary simple while still preserving enough source shape
//! for diagnostics and lowering.
//!
//! INVARIANTS
//! ==========
//! - `,` and `et` separate patterns, but body-starting tokens stop the pattern
//!   list so `ergo` and `{` remain statement-body syntax.
//! - Pattern literals are parsed independently from expression literals because
//!   this context accepts constants, not operators or calls.
//! - `fixum` and `varia` in pattern binds describe captured binding mutability;
//!   they are not declaration statements inside the arm head.

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// PATTERN PARSING
// =============================================================================

impl Parser {
    /// Parse one or more arm-head patterns.
    ///
    /// GRAMMAR:
    ///   patterns := pattern (',' pattern | 'et' pattern)*
    ///
    /// Both comma and `et` are accepted as pattern separators. The parser stops
    /// before `{` or `ergo` so the arm body is not consumed as another pattern.
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
            if self.check(&TokenKind::LBrace) || self.check_keyword(TokenKind::Ergo) {
                break;
            }
        }

        Ok(patterns)
    }

    /// Parse one pattern atom plus its optional binding suffix.
    ///
    /// GRAMMAR:
    ///   pattern := '_' | literal | ident [bind] | path bind
    ///   bind := 'ut' ident | ('fixum'|'varia') ident-list
    ///
    /// Qualified paths are kept syntactic here; later phases decide whether a
    /// path names an enum variant, union variant, or invalid pattern target.
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

    /// Parse an optional binding suffix for a pattern.
    ///
    /// GRAMMAR:
    ///   bind := 'ut' ident | ('fixum'|'varia') ident (',' ident)*
    ///
    /// `ut name` aliases the whole matched value. `fixum` and `varia` capture a
    /// shallow list of names with the requested mutability; deeper shape checks
    /// belong to semantic analysis.
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
                || self.check_keyword(TokenKind::Ergo)
                || self.check_keyword(TokenKind::Et)
            {
                break;
            }
        }
        Ok(names)
    }

    /// Try to parse a literal pattern in arm-head context.
    ///
    /// This intentionally stops at constants. Operators, calls, and object/array
    /// expressions are not pattern syntax.
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
