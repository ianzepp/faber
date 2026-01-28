//! Pattern parsing for match expressions

use super::{Parser, ParseError, ParseErrorKind};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Parse comma-separated patterns
    pub(super) fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();

        loop {
            patterns.push(self.parse_pattern()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // Check if we hit the body (don't parse it as pattern)
            if self.check(&TokenKind::LBrace)
                || self.check_keyword(TokenKind::Reddit)
                || self.check_keyword(TokenKind::Iacit)
                || self.check_keyword(TokenKind::Moritor)
                || self.check_keyword(TokenKind::Tacet)
            {
                break;
            }
        }

        Ok(patterns)
    }

    /// Parse a single pattern
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Wildcard
        if let TokenKind::Ident(sym) = self.peek().kind {
            // Check if it's underscore (wildcard)
            // We'd need to check the actual symbol value
            let span = self.peek().span;
            let ident = self.parse_ident()?;

            // Check for binding: ut NAME or pro NAME, NAME
            let bind = if self.eat_keyword(TokenKind::Ut) {
                let alias = self.parse_ident()?;
                Some(PatternBind::Alias(alias))
            } else if self.eat_keyword(TokenKind::Pro) {
                let mut names = Vec::new();
                loop {
                    names.push(self.parse_ident()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                    // Check if we're at end of pattern list
                    if self.check(&TokenKind::LBrace)
                        || self.check_keyword(TokenKind::Reddit)
                    {
                        break;
                    }
                }
                Some(PatternBind::Destructure(names))
            } else {
                None
            };

            return Ok(Pattern::Ident(ident, bind));
        }

        Err(self.error(ParseErrorKind::InvalidPattern, "expected pattern"))
    }
}
