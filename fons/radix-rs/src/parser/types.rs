//! Type annotation parsing

use super::{Parser, ParseError, ParseErrorKind};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Try to parse a type annotation (returns None if not a type)
    pub(super) fn try_parse_type(&mut self) -> Result<Option<TypeExpr>, ParseError> {
        // Check for type modifiers or type name
        if self.check_keyword(TokenKind::Si)
            || self.check_keyword(TokenKind::De)
            || self.check_keyword(TokenKind::In)
            || matches!(self.peek().kind, TokenKind::Ident(_))
            || self.check(&TokenKind::LParen)
        {
            Ok(Some(self.parse_type()?))
        } else {
            Ok(None)
        }
    }

    /// Parse a type annotation
    pub(super) fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let start = self.current_span();

        // Nullable: si
        let nullable = self.eat_keyword(TokenKind::Si);

        // Ownership mode: de/in
        let mode = if self.eat_keyword(TokenKind::De) {
            Some(TypeMode::Ref)
        } else if self.eat_keyword(TokenKind::In) {
            Some(TypeMode::MutRef)
        } else {
            None
        };

        // Function type: (params) -> ret
        if self.check(&TokenKind::LParen) {
            let func = self.parse_func_type()?;
            let span = start.merge(self.previous_span());
            return Ok(TypeExpr {
                nullable,
                mode,
                kind: TypeExprKind::Func(func),
                span,
            });
        }

        // Named type
        let name = self.parse_ident()?;

        // Type parameters
        let params = if self.eat(&TokenKind::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::Gt, "expected '>'")?;
            params
        } else {
            Vec::new()
        };

        let mut kind = TypeExprKind::Named(name, params);

        // Array brackets
        while self.check(&TokenKind::LBracket) {
            self.advance();
            self.expect(&TokenKind::RBracket, "expected ']'")?;
            let span = start.merge(self.previous_span());
            kind = TypeExprKind::Array(Box::new(TypeExpr {
                nullable: false,
                mode: None,
                kind,
                span,
            }));
        }

        let span = start.merge(self.previous_span());
        Ok(TypeExpr {
            nullable,
            mode,
            kind,
            span,
        })
    }

    /// Parse function type: (A, B) -> C
    fn parse_func_type(&mut self) -> Result<FuncTypeExpr, ParseError> {
        self.expect(&TokenKind::LParen, "expected '('")?;

        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            params.push(self.parse_type()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::RParen, "expected ')'")?;
        self.expect(&TokenKind::Arrow, "expected '->'")?;

        let ret = Box::new(self.parse_type()?);

        Ok(FuncTypeExpr { params, ret })
    }
}
