//! Expression parsing with precedence climbing

use super::{Parser, ParseError, ParseErrorKind};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Parse an expression
    pub(super) fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    /// Assignment (lowest precedence)
    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let expr = self.parse_ternary()?;

        let op = match self.peek().kind {
            TokenKind::Eq => Some(AssignOp::Assign),
            TokenKind::PlusEq => Some(AssignOp::AddAssign),
            TokenKind::MinusEq => Some(AssignOp::SubAssign),
            TokenKind::StarEq => Some(AssignOp::MulAssign),
            TokenKind::SlashEq => Some(AssignOp::DivAssign),
            TokenKind::AmpEq => Some(AssignOp::BitAndAssign),
            TokenKind::PipeEq => Some(AssignOp::BitOrAssign),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let value = Box::new(self.parse_assignment()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Assign(AssignExpr {
                    op,
                    target: Box::new(expr),
                    value,
                }),
                span,
            });
        }

        Ok(expr)
    }

    /// Ternary conditional
    fn parse_ternary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let cond = self.parse_or()?;

        // ? : style
        if self.eat(&TokenKind::Question) {
            let then = Box::new(self.parse_expression()?);
            self.expect(&TokenKind::Colon, "expected ':' in ternary")?;
            let else_ = Box::new(self.parse_ternary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Ternary(TernaryExpr {
                    cond: Box::new(cond),
                    then,
                    else_,
                    style: TernaryStyle::QuestionColon,
                }),
                span,
            });
        }

        // sic secus style
        if self.eat_keyword(TokenKind::Sic) {
            let then = Box::new(self.parse_expression()?);
            self.expect_keyword(TokenKind::Secus, "expected 'secus' after 'sic'")?;
            let else_ = Box::new(self.parse_ternary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Ternary(TernaryExpr {
                    cond: Box::new(cond),
                    then,
                    else_,
                    style: TernaryStyle::SicSecus,
                }),
                span,
            });
        }

        Ok(cond)
    }

    /// Logical OR
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::PipePipe) || self.check_keyword(TokenKind::Aut) {
            self.advance();
            let right = self.parse_and()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::Or,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        // Nullish coalesce: vel
        while self.check_keyword(TokenKind::Vel) {
            self.advance();
            let right = self.parse_and()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::Coalesce,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Logical AND
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::AmpAmp) || self.check_keyword(TokenKind::Et) {
            self.advance();
            let right = self.parse_equality()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::And,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Equality
    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::BangEq => BinOp::NotEq,
                TokenKind::EqEqEq => BinOp::StrictEq,
                TokenKind::BangEqEq => BinOp::StrictNotEq,
                TokenKind::Est => BinOp::Is,
                _ => break,
            };
            self.advance();

            // Check for 'non est'
            let op = if op == BinOp::Is && self.eat_keyword(TokenKind::Non) {
                BinOp::IsNot
            } else {
                op
            };

            let right = self.parse_comparison()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Comparison
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_or()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::GtEq => BinOp::GtEq,
                TokenKind::Intra => BinOp::InRange,
                TokenKind::Inter => BinOp::Between,
                _ => break,
            };
            self.advance();
            let right = self.parse_bitwise_or()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Bitwise OR
    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_xor()?;

        while self.check(&TokenKind::Pipe) && !self.check(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::BitOr,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Bitwise XOR
    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_and()?;

        while self.check(&TokenKind::Caret) {
            self.advance();
            let right = self.parse_bitwise_and()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::BitXor,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Bitwise AND
    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_shift()?;

        while self.check(&TokenKind::Amp) && !self.check(&TokenKind::AmpAmp) {
            self.advance();
            let right = self.parse_shift()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op: BinOp::BitAnd,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Shift operations
    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_range()?;

        loop {
            let op = if self.check_keyword(TokenKind::Sinistratum) {
                self.advance();
                BinOp::Shl
            } else if self.check_keyword(TokenKind::Dextratum) {
                self.advance();
                BinOp::Shr
            } else {
                break;
            };

            let right = self.parse_range()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Range expressions
    fn parse_range(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let left = self.parse_additive()?;

        let kind = if self.check(&TokenKind::DotDot) {
            self.advance();
            Some(RangeKind::Exclusive)
        } else if self.check_keyword(TokenKind::Ante) {
            self.advance();
            Some(RangeKind::Ante)
        } else if self.check_keyword(TokenKind::Usque) {
            self.advance();
            Some(RangeKind::Usque)
        } else {
            None
        };

        if let Some(kind) = kind {
            let end = Box::new(self.parse_additive()?);
            let step = if self.eat_keyword(TokenKind::Per) {
                Some(Box::new(self.parse_additive()?))
            } else {
                None
            };
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Range(RangeExpr {
                    start: Box::new(left),
                    end,
                    step,
                    kind,
                }),
                span,
            });
        }

        Ok(left)
    }

    /// Additive
    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Multiplicative
    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                }),
                span,
            };
        }

        Ok(left)
    }

    /// Unary operators
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();

        let op = match self.peek().kind {
            TokenKind::Minus => Some(UnOp::Neg),
            TokenKind::Tilde => Some(UnOp::BitNot),
            TokenKind::Non => Some(UnOp::Not),
            TokenKind::Nulla => Some(UnOp::IsNull),
            TokenKind::Nonnulla => Some(UnOp::IsNotNull),
            TokenKind::Negativum => Some(UnOp::IsNeg),
            TokenKind::Positivum => Some(UnOp::IsPos),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Unary(UnaryExpr { op, operand }),
                span,
            });
        }

        // await
        if self.check_keyword(TokenKind::Cede) {
            self.advance();
            let expr = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr {
                id,
                kind: ExprKind::Await(AwaitExpr { expr }),
                span,
            });
        }

        // novum
        if self.check_keyword(TokenKind::Novum) {
            return self.parse_new_expr();
        }

        // finge
        if self.check_keyword(TokenKind::Finge) {
            return self.parse_variant_expr();
        }

        self.parse_postfix()
    }

    /// Postfix operations (cast, call, member, index)
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LParen) {
                // Function call
                self.advance();
                let args = self.parse_argument_list()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Call(CallExpr {
                        callee: Box::new(expr),
                        args,
                    }),
                    span,
                };
            } else if self.check(&TokenKind::Dot) {
                // Member access
                self.advance();
                let member = self.parse_ident()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Member(MemberExpr {
                        object: Box::new(expr),
                        member,
                    }),
                    span,
                };
            } else if self.check(&TokenKind::LBracket) {
                // Index access
                self.advance();
                let index = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Index(IndexExpr {
                        object: Box::new(expr),
                        index,
                    }),
                    span,
                };
            } else if self.check(&TokenKind::QuestionDot) {
                // Optional member
                self.advance();
                let member = self.parse_ident()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::OptionalChain(OptionalChainExpr {
                        object: Box::new(expr),
                        chain: OptionalChainKind::Member(member),
                    }),
                    span,
                };
            } else if self.check(&TokenKind::QuestionBracket) {
                // Optional index
                self.advance();
                let index = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::OptionalChain(OptionalChainExpr {
                        object: Box::new(expr),
                        chain: OptionalChainKind::Index(index),
                    }),
                    span,
                };
            } else if self.check(&TokenKind::QuestionParen) {
                // Optional call
                self.advance();
                let args = self.parse_argument_list()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::OptionalChain(OptionalChainExpr {
                        object: Box::new(expr),
                        chain: OptionalChainKind::Call(args),
                    }),
                    span,
                };
            } else if self.check_keyword(TokenKind::Qua) {
                // Type cast
                self.advance();
                let ty = self.parse_type()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Cast(CastExpr {
                        expr: Box::new(expr),
                        ty,
                    }),
                    span,
                };
            } else if self.check_keyword(TokenKind::Innatum) {
                // Native construction
                self.advance();
                let ty = self.parse_type()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Construct(ConstructExpr {
                        expr: Box::new(expr),
                        ty,
                    }),
                    span,
                };
            } else if self.check_keyword(TokenKind::Numeratum)
                || self.check_keyword(TokenKind::Fractatum)
                || self.check_keyword(TokenKind::Textatum)
                || self.check_keyword(TokenKind::Bivalentum)
            {
                // Type conversion
                let kind = match self.peek().kind {
                    TokenKind::Numeratum => ConversionKind::ToInt,
                    TokenKind::Fractatum => ConversionKind::ToFloat,
                    TokenKind::Textatum => ConversionKind::ToString,
                    TokenKind::Bivalentum => ConversionKind::ToBool,
                    _ => unreachable!(),
                };
                self.advance();

                let type_params = self.try_parse_type_args()?;

                let fallback = if self.eat_keyword(TokenKind::Vel) {
                    Some(Box::new(self.parse_unary()?))
                } else {
                    None
                };

                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::Conversion(ConversionExpr {
                        expr: Box::new(expr),
                        kind,
                        type_params,
                        fallback,
                    }),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Primary expressions
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let id = self.next_id();

        let kind = match self.peek().kind.clone() {
            // Literals
            TokenKind::Integer(n) => {
                self.advance();
                ExprKind::Literal(Literal::Integer(n))
            }
            TokenKind::Float(n) => {
                self.advance();
                ExprKind::Literal(Literal::Float(n))
            }
            TokenKind::String(s) => {
                self.advance();
                ExprKind::Literal(Literal::String(s))
            }
            TokenKind::TemplateString(s) => {
                self.advance();
                ExprKind::Literal(Literal::TemplateString(s))
            }
            TokenKind::Verum => {
                self.advance();
                ExprKind::Literal(Literal::Bool(true))
            }
            TokenKind::Falsum => {
                self.advance();
                ExprKind::Literal(Literal::Bool(false))
            }
            TokenKind::Nihil => {
                self.advance();
                ExprKind::Literal(Literal::Nil)
            }

            // Self
            TokenKind::Ego => {
                let span = self.peek().span;
                self.advance();
                ExprKind::Ego(span)
            }

            // Builtin: scriptum("template", args...)
            TokenKind::Scriptum => {
                return self.parse_script_expr();
            }

            // Builtin: lege / lineam
            TokenKind::Lege | TokenKind::Lineam => {
                return self.parse_read_expr();
            }

            // Builtin: praefixum(expr)
            TokenKind::Praefixum => {
                return self.parse_prefix_expr();
            }

            // Identifier
            TokenKind::Ident(_) => {
                let ident = self.parse_ident()?;
                ExprKind::Ident(ident)
            }

            // Array literal
            TokenKind::LBracket => {
                self.advance();
                let elements = self.parse_array_elements()?;
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                ExprKind::Array(ArrayExpr { elements })
            }

            // Object literal
            TokenKind::LBrace => {
                self.advance();
                let fields = self.parse_object_fields()?;
                self.expect(&TokenKind::RBrace, "expected '}'")?;
                ExprKind::Object(ObjectExpr { fields })
            }

            // Parenthesized expression or closure
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                ExprKind::Paren(Box::new(inner))
            }

            // Closure
            TokenKind::Clausura => {
                return self.parse_closure_expr();
            }

            // Collection DSL
            TokenKind::Ab => {
                return self.parse_collection_expr();
            }

            _ => {
                return Err(self.error(ParseErrorKind::InvalidExpression, "expected expression"));
            }
        };

        let span = start.merge(self.previous_span());
        Ok(Expr { id, kind, span })
    }

    fn parse_new_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Novum, "expected 'novum'")?;

        let ty = self.parse_ident()?;

        let args = if self.check(&TokenKind::LParen) {
            self.advance();
            let args = self.parse_argument_list()?;
            self.expect(&TokenKind::RParen, "expected ')'")?;
            Some(args)
        } else {
            None
        };

        let init = if self.check(&TokenKind::LBrace) {
            self.advance();
            let fields = self.parse_object_fields()?;
            self.expect(&TokenKind::RBrace, "expected '}'")?;
            Some(NewInit::Object(fields))
        } else if self.eat_keyword(TokenKind::De) {
            Some(NewInit::From(Box::new(self.parse_expression()?)))
        } else {
            None
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::New(NewExpr { ty, args, init }),
            span,
        })
    }

    fn parse_variant_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Finge, "expected 'finge'")?;

        let variant = self.parse_ident()?;

        let fields = if self.check(&TokenKind::LBrace) {
            self.advance();
            let mut fields = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                let field_start = self.current_span();
                let name = self.parse_ident()?;
                self.expect(&TokenKind::Colon, "expected ':'")?;
                let value = Box::new(self.parse_expression()?);
                let span = field_start.merge(self.previous_span());
                fields.push(VariantFieldInit { name, value, span });
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace, "expected '}'")?;
            fields
        } else {
            Vec::new()
        };

        let cast = if self.eat_keyword(TokenKind::Qua) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Variant(VariantExpr {
                variant,
                fields,
                cast,
            }),
            span,
        })
    }

    fn parse_closure_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Clausura, "expected 'clausura'")?;

        // Parse parameters: type name, type name, ...
        let mut params = Vec::new();
        loop {
            // Check for end of params (-> or :)
            if self.check(&TokenKind::Arrow) || self.check(&TokenKind::Colon) {
                break;
            }

            let param_start = self.current_span();
            let ty = self.parse_type()?;
            let name = self.parse_ident()?;
            let param_span = param_start.merge(self.previous_span());
            params.push(ClosureParam {
                ty,
                name,
                span: param_span,
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        // Optional return type
        let ret = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body: either `: expr` or `{ block }`
        let body = if self.eat(&TokenKind::Colon) {
            ClosureBody::Expr(Box::new(self.parse_expression()?))
        } else if self.check(&TokenKind::LBrace) {
            ClosureBody::Block(self.parse_block()?)
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected ':' or '{' after closure parameters"));
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Closure(ClosureExpr { params, ret, body }),
            span,
        })
    }

    fn parse_collection_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Ab, "expected 'ab'")?;

        let source = Box::new(self.parse_expression()?);

        let filter = if self.eat_keyword(TokenKind::Ubi) {
            let negated = self.eat_keyword(TokenKind::Non);
            let cond = Box::new(self.parse_expression()?);
            Some(CollectionFilter {
                negated,
                kind: CollectionFilterKind::Condition(cond),
            })
        } else if let Some(ident) = self.try_parse_ident() {
            Some(CollectionFilter {
                negated: false,
                kind: CollectionFilterKind::Property(ident),
            })
        } else {
            None
        };

        let mut transforms = Vec::new();
        while self.eat(&TokenKind::Comma) {
            let transform_start = self.current_span();
            let kind = if self.eat_keyword(TokenKind::Prima) {
                TransformKind::First
            } else if self.eat_keyword(TokenKind::Ultima) {
                TransformKind::Last
            } else if self.eat_keyword(TokenKind::Summa) {
                TransformKind::Sum
            } else {
                break;
            };

            let arg = if !self.check(&TokenKind::Comma)
                && !self.check(&TokenKind::RBrace)
                && !self.is_at_end()
            {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            let span = transform_start.merge(self.previous_span());
            transforms.push(CollectionTransform { kind, arg, span });
        }

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Collection(CollectionExpr {
                source,
                filter,
                transforms,
            }),
            span,
        })
    }

    /// Parse argument list (for calls)
    pub(super) fn parse_argument_list(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();

        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let start = self.current_span();
            let spread = self.eat_keyword(TokenKind::Sparge);
            let value = Box::new(self.parse_expression()?);
            let span = start.merge(self.previous_span());
            args.push(Argument { spread, value, span });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_array_elements(&mut self) -> Result<Vec<ArrayElement>, ParseError> {
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Sparge) {
                elements.push(ArrayElement::Spread(Box::new(self.parse_expression()?)));
            } else {
                elements.push(ArrayElement::Expr(Box::new(self.parse_expression()?)));
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(elements)
    }

    fn parse_object_fields(&mut self) -> Result<Vec<ObjectField>, ParseError> {
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let start = self.current_span();

            // Spread
            if self.eat_keyword(TokenKind::Sparge) {
                let expr = Box::new(self.parse_expression()?);
                let span = start.merge(self.previous_span());
                fields.push(ObjectField {
                    key: ObjectKey::Spread(expr),
                    value: None,
                    span,
                });
            } else if let TokenKind::String(s) = self.peek().kind {
                // String key
                self.advance();
                self.expect(&TokenKind::Colon, "expected ':'")?;
                let value = Some(Box::new(self.parse_expression()?));
                let span = start.merge(self.previous_span());
                fields.push(ObjectField {
                    key: ObjectKey::String(s),
                    value,
                    span,
                });
            } else if self.check(&TokenKind::LBracket) {
                // Computed key
                self.advance();
                let key_expr = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                self.expect(&TokenKind::Colon, "expected ':'")?;
                let value = Some(Box::new(self.parse_expression()?));
                let span = start.merge(self.previous_span());
                fields.push(ObjectField {
                    key: ObjectKey::Computed(key_expr),
                    value,
                    span,
                });
            } else {
                // Identifier key (possibly shorthand)
                let ident = self.parse_ident()?;
                let value = if self.eat(&TokenKind::Colon) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None // Shorthand
                };
                let span = start.merge(self.previous_span());
                fields.push(ObjectField {
                    key: ObjectKey::Ident(ident),
                    value,
                    span,
                });
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    fn try_parse_type_args(&mut self) -> Result<Vec<TypeExpr>, ParseError> {
        if !self.eat(&TokenKind::Lt) {
            return Ok(Vec::new());
        }

        let mut args = Vec::new();
        loop {
            args.push(self.parse_type()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect(&TokenKind::Gt, "expected '>'")?;
        Ok(args)
    }

    /// Parse scriptum("template", args...)
    fn parse_script_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Scriptum, "expected 'scriptum'")?;
        self.expect(&TokenKind::LParen, "expected '(' after 'scriptum'")?;

        // First arg is the template string
        let template = self.parse_string()?;

        // Remaining args
        let mut args = Vec::new();
        while self.eat(&TokenKind::Comma) {
            args.push(self.parse_expression()?);
        }

        self.expect(&TokenKind::RParen, "expected ')'")?;

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Script(ScriptExpr { template, args }),
            span,
        })
    }

    /// Parse lege / lineam
    fn parse_read_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let line = if self.eat_keyword(TokenKind::Lineam) {
            true
        } else {
            self.expect_keyword(TokenKind::Lege, "expected 'lege' or 'lineam'")?;
            false
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Read(ReadExpr { line, span }),
            span,
        })
    }

    /// Parse praefixum(expr)
    fn parse_prefix_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Praefixum, "expected 'praefixum'")?;
        self.expect(&TokenKind::LParen, "expected '(' after 'praefixum'")?;

        let body = PrefixBody::Expr(Box::new(self.parse_expression()?));

        self.expect(&TokenKind::RParen, "expected ')'")?;

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr {
            id,
            kind: ExprKind::Prefix(PrefixExpr { body }),
            span,
        })
    }
}
