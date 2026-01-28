//! Pattern parsing for match expressions

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Parse comma-separated patterns
    pub(super) fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();

        loop {
            patterns.push(self.parse_pattern()?);
            if self.eat(&TokenKind::Comma) {
                // continue
            } else if self.eat_keyword(TokenKind::Et) {
                // allow 'et' as pattern separator
            } else {
                break;
            }
            // Check if we hit the body (don't parse it as pattern)
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

    /// Parse a single pattern
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if let TokenKind::Underscore(_) = self.peek().kind {
            let span = self.peek().span;
            self.advance();
            return Ok(Pattern::Wildcard(span));
        }
        if let Some(literal) = self.try_parse_pattern_literal() {
            return Ok(literal);
        }
        // Identifier pattern
        if let TokenKind::Ident(sym) = self.peek().kind {
            let span = self.peek().span;
            let ident = self.parse_ident()?;

            // Check for binding: ut NAME
            let bind = if self.eat_keyword(TokenKind::Ut) {
                let alias = self.parse_ident()?;
                Some(PatternBind::Alias(alias))
            } else if self.eat_keyword(TokenKind::Fixum) {
                let mut names = Vec::new();
                loop {
                    names.push(self.parse_ident()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
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
                Some(PatternBind::Bindings {
                    mutability: Mutability::Immutable,
                    names,
                })
            } else if self.eat_keyword(TokenKind::Varia) {
                let mut names = Vec::new();
                loop {
                    names.push(self.parse_ident()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
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
                Some(PatternBind::Bindings {
                    mutability: Mutability::Mutable,
                    names,
                })
            } else {
                None
            };

            return Ok(Pattern::Ident(ident, bind));
        }

        Err(self.error(ParseErrorKind::InvalidPattern, "expected pattern"))
    }

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
