//! Expression grammar and precedence construction.
//!
//! Expressions are the densest recursive-descent surface in the parser. This
//! module encodes precedence as a call chain from assignment down to primaries,
//! then folds postfix operations in a loop so calls, indexing, member access,
//! optional chaining, non-null chaining, and conversions all bind tighter than
//! infix operators.
//!
//! DESIGN PHILOSOPHY
//! =================
//! The parser prefers explicit grammar functions over a data-driven operator
//! table. That makes the accepted syntax visible at the cost of boilerplate, and
//! it keeps contextual Faber forms close to their diagnostics. Binary operators
//! associate left through loops; assignment and ternary forms recurse on the
//! right where the language expects right associativity.
//!
//! INVARIANTS
//! ==========
//! - No expression form may invent declaration syntax; constructors and casts
//!   are parsed only through tokenized grammar surfaces.
//! - Compact closures are the only expression form that uses speculative
//!   rollback, and rollback restores both token position and node-id allocation.
//! - `vacua` is recognized as a value-level empty collection marker; required
//!   collection types are supplied by declaration/type contexts upstream.
//! - Field separators in object-like expression syntax are canonical `=`, not
//!   colon aliases.

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// EXPRESSION ENTRY POINT
// =============================================================================

impl Parser {
    /// Parse an expression from the lowest-precedence level.
    ///
    /// GRAMMAR:
    ///   expression := assignment
    pub(super) fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    // =============================================================================
    // PRECEDENCE CLIMBING
    // =============================================================================
    // Each function handles one precedence level, calling the next-higher
    // precedence level for its operands

    /// Parse assignment expression (lowest precedence).
    ///
    /// GRAMMAR:
    ///   assignment := ternary [assign-op assignment]
    ///   assign-op := '←' | '⊕' | '⊖' | '⊛' | '⊘' | '⊜' | '⊚'
    ///
    /// WHY: Right-associative via recursion on RHS. Allows `a ← b ← c`.
    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let expr = self.parse_ternary()?;

        let op = match self.peek().kind {
            TokenKind::Assign => Some(AssignOp::Assign),
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
            return Ok(Expr { id, kind: ExprKind::Assign(AssignExpr { op, target: Box::new(expr), value }), span });
        }

        Ok(expr)
    }

    /// Parse ternary conditional expression.
    ///
    /// GRAMMAR:
    ///   ternary := or ['?' expr ':' ternary | 'sic' expr 'secus' ternary]
    ///
    /// WHY: Faber supports both C-style (? :) and keyword-style (sic secus) ternary.
    /// Right-associative via recursion on else branch.
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

        while self.check_keyword(TokenKind::Aut) {
            self.advance();
            let right = self.parse_and()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::Or, lhs: Box::new(left), rhs: Box::new(right) }),
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
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::Coalesce, lhs: Box::new(left), rhs: Box::new(right) }),
                span,
            };
        }

        Ok(left)
    }

    /// Logical AND
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_equality()?;

        while self.check_keyword(TokenKind::Et) {
            self.advance();
            let right = self.parse_equality()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::And, lhs: Box::new(left), rhs: Box::new(right) }),
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
            left =
                Expr { id, kind: ExprKind::Binary(BinaryExpr { op, lhs: Box::new(left), rhs: Box::new(right) }), span };
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
            left =
                Expr { id, kind: ExprKind::Binary(BinaryExpr { op, lhs: Box::new(left), rhs: Box::new(right) }), span };
        }

        Ok(left)
    }

    /// Bitwise OR
    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_xor()?;

        while self.check(&TokenKind::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::BitOr, lhs: Box::new(left), rhs: Box::new(right) }),
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
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::BitXor, lhs: Box::new(left), rhs: Box::new(right) }),
                span,
            };
        }

        Ok(left)
    }

    /// Bitwise AND
    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let mut left = self.parse_shift()?;

        while self.check(&TokenKind::Amp) {
            self.advance();
            let right = self.parse_shift()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left = Expr {
                id,
                kind: ExprKind::Binary(BinaryExpr { op: BinOp::BitAnd, lhs: Box::new(left), rhs: Box::new(right) }),
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
            let op = if self.check(&TokenKind::Sinistratum) {
                self.advance();
                BinOp::Shl
            } else if self.check(&TokenKind::Dextratum) {
                self.advance();
                BinOp::Shr
            } else {
                break;
            };

            let right = self.parse_range()?;
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            left =
                Expr { id, kind: ExprKind::Binary(BinaryExpr { op, lhs: Box::new(left), rhs: Box::new(right) }), span };
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
        } else if self.check(&TokenKind::Ellipsis) {
            self.advance();
            Some(RangeKind::Inclusive)
        } else if self.check_keyword(TokenKind::Ante) {
            self.advance();
            Some(RangeKind::Exclusive)
        } else if self.check_keyword(TokenKind::Usque) {
            self.advance();
            Some(RangeKind::Inclusive)
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
                kind: ExprKind::Intervallum(IntervallumExpr { start: Box::new(left), end, step, kind }),
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
            left =
                Expr { id, kind: ExprKind::Binary(BinaryExpr { op, lhs: Box::new(left), rhs: Box::new(right) }), span };
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
            left =
                Expr { id, kind: ExprKind::Binary(BinaryExpr { op, lhs: Box::new(left), rhs: Box::new(right) }), span };
        }

        Ok(left)
    }

    /// Parse unary prefix operators.
    ///
    /// GRAMMAR:
    ///   unary := unary-op unary | postfix
    ///   unary-op := '-' | '¬' | 'non' | 'nulla' | 'nonnulla' | 'verum' | 'falsum'
    ///             | 'nihil' | 'nonnihil' | 'negativum' | 'positivum' | 'cede'
    ///
    /// WHY: Some keywords (verum, falsum, nihil) can be either unary operators or
    /// literals. Lookahead disambiguates: if followed by an operand, it's an operator.
    ///
    /// EDGE: Special handling for 'finge' (variant construction). 'novum' is no longer a keyword.
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();

        if self.check_keyword(TokenKind::Verum) && self.can_start_prefix_operand(&self.peek_at(1).kind) {
            self.advance();
            let operand = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr { id, kind: ExprKind::Unary(UnaryExpr { op: UnOp::IsTrue, operand }), span });
        }

        if self.check_keyword(TokenKind::Falsum) && self.can_start_prefix_operand(&self.peek_at(1).kind) {
            self.advance();
            let operand = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr { id, kind: ExprKind::Unary(UnaryExpr { op: UnOp::IsFalse, operand }), span });
        }

        if self.check_keyword(TokenKind::Nihil) && self.can_start_prefix_operand(&self.peek_at(1).kind) {
            self.advance();
            let operand = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr { id, kind: ExprKind::Unary(UnaryExpr { op: UnOp::IsNil, operand }), span });
        }

        let op = match self.peek().kind {
            TokenKind::Minus => Some(UnOp::Neg),
            TokenKind::Tilde => Some(UnOp::BitNot),
            TokenKind::Non => Some(UnOp::Not),
            TokenKind::Nulla => Some(UnOp::IsNull),
            TokenKind::Nonnulla => Some(UnOp::IsNotNull),
            TokenKind::Nonnihil => Some(UnOp::IsNotNil),
            TokenKind::Negativum => Some(UnOp::IsNeg),
            TokenKind::Positivum => Some(UnOp::IsPos),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr { id, kind: ExprKind::Unary(UnaryExpr { op, operand }), span });
        }

        // await
        if self.check_keyword(TokenKind::Cede) {
            self.advance();
            let expr = Box::new(self.parse_unary()?);
            let span = start.merge(self.previous_span());
            let id = self.next_id();
            return Ok(Expr { id, kind: ExprKind::Cede(CedeExpr { expr }), span });
        }

        // finge
        if self.check_keyword(TokenKind::Finge) {
            return self.parse_variant_expr();
        }

        self.parse_postfix()
    }

    fn can_start_expression(&self, kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Ident(_)
                | TokenKind::Underscore(_)
                | TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::Verum
                | TokenKind::Falsum
                | TokenKind::Nihil
                | TokenKind::Ego
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Scriptum
                | TokenKind::Clausura
                | TokenKind::Ab
                | TokenKind::Praefixum
                | TokenKind::Sed
                | TokenKind::Lege
                | TokenKind::Finge
                | TokenKind::Non
                | TokenKind::Minus
                | TokenKind::Tilde
                | TokenKind::Nulla
                | TokenKind::Nonnulla
                | TokenKind::Negativum
                | TokenKind::Positivum
                | TokenKind::Cede
        )
    }

    fn can_start_prefix_operand(&self, kind: &TokenKind) -> bool {
        self.can_start_expression(kind) && !matches!(kind, TokenKind::LBrace)
    }

    /// Parse postfix operations that bind tighter than every infix operator.
    ///
    /// GRAMMAR:
    ///   postfix := primary (call | member | index | cast | optional-chain | conversion)*
    ///   call := '(' args ')'
    ///   member := '.' ident
    ///   index := '[' expr ']'
    ///   cast := '∷' type   (static type ascription; qua/innatum/novum aliases removed)
    ///   optional-chain := '?.' ident | '?[' expr ']' | '?(' args ')'
    ///   conversion := '⇒' type [type-params] ['vel' fallback]
    ///
    /// Keeping every postfix form in one loop preserves left-to-right chaining
    /// (`a.b(c)[d]`) and prevents casts/conversions from accidentally binding at
    /// a different level than calls or optional chaining.
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
                expr = Expr { id, kind: ExprKind::Call(CallExpr { callee: Box::new(expr), args }), span };
            } else if self.check(&TokenKind::Dot) {
                // Member access
                self.advance();
                let member = self.parse_member_ident()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr { id, kind: ExprKind::Member(MemberExpr { object: Box::new(expr), member }), span };
            } else if self.check(&TokenKind::LBracket) {
                // Index access
                self.advance();
                let index = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr { id, kind: ExprKind::Index(IndexExpr { object: Box::new(expr), index }), span };
            } else if self.check(&TokenKind::QuestionDot) {
                // Optional member
                self.advance();
                let member = self.parse_member_ident()?;
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
            } else if self.check(&TokenKind::BangDot) {
                // Non-null assertion member
                self.advance();
                let member = self.parse_member_ident()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::NonNull(NonNullExpr { object: Box::new(expr), chain: NonNullKind::Member(member) }),
                    span,
                };
            } else if self.check(&TokenKind::BangBracket) {
                // Non-null assertion index
                self.advance();
                let index = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::NonNull(NonNullExpr { object: Box::new(expr), chain: NonNullKind::Index(index) }),
                    span,
                };
            } else if self.check(&TokenKind::BangParen) {
                // Non-null assertion call
                self.advance();
                let args = self.parse_argument_list()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr {
                    id,
                    kind: ExprKind::NonNull(NonNullExpr { object: Box::new(expr), chain: NonNullKind::Call(args) }),
                    span,
                };
            } else if self.check_keyword(TokenKind::Verte) {
                // Static type ascription via ∷ (only accepted spelling post clean-break)
                self.advance();
                let ty = self.parse_type()?;
                let span = start.merge(self.previous_span());
                let id = self.next_id();
                expr = Expr { id, kind: ExprKind::Verte(VerteExpr { expr: Box::new(expr), ty }), span };
            } else if self.check_keyword(TokenKind::Conversio) {
                // Runtime value conversion: ⇒ type [<params>] [vel fallback]
                self.advance();
                let ty = self.parse_type()?;
                let target = ConversioTarget::Explicit(ty);
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
                    kind: ExprKind::Conversio(ConversioExpr { expr: Box::new(expr), target, type_params, fallback }),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // =============================================================================
    // PRIMARY EXPRESSIONS
    // =============================================================================

    /// Parse primary expressions, the atoms that postfix and infix forms compose.
    ///
    /// GRAMMAR:
    ///   primary := literal | ident | array | object | paren | closure | collection-dsl
    ///   literal := integer | float | string | bool | nil
    ///   array := '[' (expr | 'sparge' expr) (',' (expr | 'sparge' expr))* ']'
    ///   object := '{' field (',' field)* '}'
    ///   paren := '(' expr ')'
    ///   closure := 'clausura' params ['→' type] ('{' block '}' | ':' expr)
    ///
    /// This is the only expression entry point that accepts literals, object and
    /// array literals, builtin expression forms, `vacua`, and constructor-looking
    /// object initialization. Anything that starts with a non-primary token must
    /// be introduced by an earlier precedence layer.
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(expr) = self.try_parse_compact_clausura_expr()? {
            return Ok(expr);
        }

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
                return self.parse_scriptum_expr();
            }

            // Builtin: lege / lineam
            TokenKind::Lege | TokenKind::Lineam => {
                return self.parse_lege_expr();
            }

            // Comptime expression: praefixum(expr)
            TokenKind::Praefixum => {
                return self.parse_praefixum_expr();
            }

            // Regex literal: sed "pattern"
            TokenKind::Sed => {
                return self.parse_sed_expr();
            }

            // Identifier
            TokenKind::Ident(_) | TokenKind::Tag => {
                let ident = self.parse_ident()?;
                if self.interner.resolve(ident.name) == "vacua" {
                    ExprKind::Vacua(ident.span)
                } else if self.check(&TokenKind::LBrace) && self.looks_like_typed_constructor_fields() {
                    let type_span = ident.span;
                    self.advance();
                    let fields = self.parse_object_fields()?;
                    self.expect(&TokenKind::RBrace, "expected '}'")?;
                    let object = Expr {
                        id: self.next_id(),
                        kind: ExprKind::Object(ObjectExpr { fields }),
                        span: ident.span.merge(self.previous_span()),
                    };
                    let ty = TypeExpr {
                        nullable: false,
                        mode: None,
                        kind: TypeExprKind::Named(ident, Vec::new()),
                        span: type_span,
                    };
                    ExprKind::Verte(VerteExpr { expr: Box::new(object), ty })
                } else {
                    ExprKind::Ident(ident)
                }
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
                return self.parse_clausura_expr();
            }

            // Collection DSL
            TokenKind::Ab => {
                self.advance();
                return self.parse_ab_expr();
            }

            _ => {
                return Err(self.error(ParseErrorKind::InvalidExpression, "expected expression"));
            }
        };

        let span = start.merge(self.previous_span());
        Ok(Expr { id, kind, span })
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
                self.expect_field_value_separator()?;
                let value = Box::new(self.parse_expression()?);
                let span = field_start.merge(self.previous_span());
                fields.push(FingeFieldInit { name, value, span });
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace, "expected '}'")?;
            fields
        } else {
            Vec::new()
        };

        let cast = if self.eat_keyword(TokenKind::Verte) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Finge(FingeExpr { variant, fields, cast }), span })
    }

    fn parse_clausura_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Clausura, "expected 'clausura'")?;

        // Parse parameters: type name, type name, ...
        let mut params = Vec::new();
        loop {
            // Check for end of params (→ or :)
            if self.check(&TokenKind::Arrow) || self.check(&TokenKind::Colon) {
                break;
            }

            params.push(self.parse_clausura_param()?);

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        let (ret, err) = self.parse_optional_clausura_signature()?;

        // Body: either `: expr` or `{ block }`
        let body = if self.eat(&TokenKind::Colon) {
            ClausuraBody::Expr(Box::new(self.parse_expression()?))
        } else if self.check(&TokenKind::LBrace) {
            ClausuraBody::Block(self.parse_block()?)
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected ':' or '{' after closure parameters"));
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Clausura(ClausuraExpr { params, ret, err, body }), span })
    }

    /// Speculatively parse the compact closure form without stealing ordinary expressions.
    ///
    /// Compact closure syntax overlaps with parenthesized expressions and
    /// type-first expression starts. Failed speculation rolls back parser
    /// position and node allocation unless `ergo` was consumed, in which case
    /// the user clearly wrote a closure and should receive its real error.
    fn try_parse_compact_clausura_expr(&mut self) -> Result<Option<Expr>, ParseError> {
        if !self.can_start_compact_clausura() {
            return Ok(None);
        }

        let saved_pos = self.pos;
        let saved_next_id = self.next_node_id;
        match self.parse_compact_clausura_expr() {
            Ok(expr) => Ok(Some(expr)),
            Err(err)
                if self.tokens[saved_pos..self.pos]
                    .iter()
                    .any(|token| token.kind == TokenKind::Ergo) =>
            {
                Err(err)
            }
            Err(_) => {
                self.pos = saved_pos;
                self.next_node_id = saved_next_id;
                Ok(None)
            }
        }
    }

    fn can_start_compact_clausura(&self) -> bool {
        matches!(
            (&self.peek().kind, &self.peek_at(1).kind),
            (
                TokenKind::Underscore(_),
                TokenKind::Ident(_) | TokenKind::Underscore(_) | TokenKind::Tag
            ) | (
                TokenKind::Ident(_),
                TokenKind::Ident(_) | TokenKind::Underscore(_) | TokenKind::Tag
            ) | (TokenKind::LParen, _)
                | (TokenKind::De | TokenKind::In, TokenKind::Underscore(_) | TokenKind::Ident(_))
        )
    }

    fn parse_compact_clausura_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let params = if self.eat(&TokenKind::LParen) {
            let params = self.parse_clausura_param_list_until(&TokenKind::RParen)?;
            self.expect(&TokenKind::RParen, "expected ')' after closure parameters")?;
            params
        } else {
            vec![self.parse_clausura_param()?]
        };

        let (ret, err) = self.parse_optional_clausura_signature()?;
        self.expect_keyword(TokenKind::Ergo, "expected '∴' after closure signature")?;
        let body = self.parse_compact_clausura_body()?;

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Clausura(ClausuraExpr { params, ret, err, body }), span })
    }

    fn parse_clausura_param_list_until(&mut self, end: &TokenKind) -> Result<Vec<ClausuraParam>, ParseError> {
        let mut params = Vec::new();
        while !self.check(end) && !self.is_at_end() {
            params.push(self.parse_clausura_param()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_clausura_param(&mut self) -> Result<ClausuraParam, ParseError> {
        let param_start = self.current_span();
        let ty = self.parse_type()?;
        let name = self.parse_ident()?;
        let span = param_start.merge(self.previous_span());
        Ok(ClausuraParam { ty, name, span })
    }

    fn parse_optional_clausura_signature(&mut self) -> Result<(Option<TypeExpr>, Option<TypeExpr>), ParseError> {
        let ret = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let err = if self.eat(&TokenKind::ExitArrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok((ret, err))
    }

    fn parse_compact_clausura_body(&mut self) -> Result<ClausuraBody, ParseError> {
        if self.check_keyword(TokenKind::Fac) {
            return Ok(ClausuraBody::Fac(self.parse_fac_construct(false)?));
        }
        if self.check(&TokenKind::LBrace) {
            return Err(self.error(ParseErrorKind::InvalidExpression, "closure block body must use 'fac' after '∴'"));
        }
        Ok(ClausuraBody::Expr(Box::new(self.parse_expression()?)))
    }

    // =============================================================================
    // SPECIAL EXPRESSION FORMS
    // =============================================================================

    /// Parse collection DSL expression.
    ///
    /// GRAMMAR:
    ///   ab-expr := 'ab' source [filter] (',' transform)*
    ///   filter := ident | 'non' ident
    ///   transform := 'prima' [expr] | 'ultima' [expr] | 'summa' [expr]
    ///
    /// WHY: Collection DSL provides a concise syntax for filtering and transforming
    /// collections. Example: `ab users activus, prima` gets first active user.
    fn parse_ab_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        // Note: 'ab' token already consumed by caller in parse_primary

        // Parse source - use postfix to avoid recursion through ab
        let source = Box::new(self.parse_postfix()?);

        // Check for optional filter after source expression
        let filter = if matches!(self.peek().kind, TokenKind::Comma | TokenKind::RBrace) || self.is_at_end() {
            // No filter - go straight to transforms
            None
        } else if self.eat_keyword(TokenKind::Non) {
            // Negated property: non activus
            self.try_parse_ident()
                .map(|ident| CollectionFilter { negated: true, kind: CollectionFilterKind::Property(ident) })
        } else {
            self.try_parse_ident()
                .map(|ident| CollectionFilter { negated: false, kind: CollectionFilterKind::Property(ident) })
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

            let arg = if self.can_start_expression(&self.peek().kind) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            let span = transform_start.merge(self.previous_span());
            transforms.push(CollectionTransform { kind, arg, span });
        }

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Ab(AbExpr { source, filter, transforms }), span })
    }

    /// Parse a function-call argument list after `(` has been consumed.
    ///
    /// GRAMMAR:
    ///   args := [['sparge'] expr (',' ['sparge'] expr)*]
    ///
    /// Arguments can use `sparge` to unpack arrays or objects. The closing `)` is
    /// left to the caller so this helper can be shared by normal and optional
    /// call postfix parsing.
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
                fields.push(ObjectField { key: ObjectKey::Spread(expr), value: None, span });
            } else if let TokenKind::String(s) = self.peek().kind {
                // String key
                self.advance();
                self.expect_field_value_separator()?;
                let value = Some(Box::new(self.parse_expression()?));
                let span = start.merge(self.previous_span());
                fields.push(ObjectField { key: ObjectKey::String(s), value, span });
            } else if self.check(&TokenKind::LBracket) {
                // Computed key
                self.advance();
                let key_expr = Box::new(self.parse_expression()?);
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                self.expect_field_value_separator()?;
                let value = Some(Box::new(self.parse_expression()?));
                let span = start.merge(self.previous_span());
                fields.push(ObjectField { key: ObjectKey::Computed(key_expr), value, span });
            } else {
                // Identifier key (possibly shorthand)
                let ident = self.parse_ident()?;
                let value = if self.eat(&TokenKind::Eq) {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None // Shorthand
                };
                let span = start.merge(self.previous_span());
                fields.push(ObjectField { key: ObjectKey::Ident(ident), value, span });
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    fn looks_like_typed_constructor_fields(&self) -> bool {
        matches!(self.peek_at(1).kind, TokenKind::Ident(_) | TokenKind::String(_))
            && matches!(self.peek_at(2).kind, TokenKind::Eq)
    }

    fn expect_field_value_separator(&mut self) -> Result<(), ParseError> {
        if self.eat(&TokenKind::Eq) {
            return Ok(());
        }

        Err(self.error(ParseErrorKind::Expected, "expected '='"))
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

    /// Parse the `scriptum("template", args...)` formatting builtin.
    fn parse_scriptum_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Scriptum, "expected 'scriptum'")?;
        self.expect(&TokenKind::LParen, "expected '(' after 'scriptum'")?;

        // First arg is the format string
        let template = self.parse_string()?;

        // Remaining args
        let mut args = Vec::new();
        while self.eat(&TokenKind::Comma) {
            args.push(self.parse_expression()?);
        }

        self.expect(&TokenKind::RParen, "expected ')'")?;

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Scriptum(ScriptumExpr { template, args }), span })
    }

    /// Parse `lege` and `lineam` input builtins.
    fn parse_lege_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        let line = if self.eat_keyword(TokenKind::Lineam) {
            true
        } else {
            self.expect_keyword(TokenKind::Lege, "expected 'lege' or 'lineam'")?;
            false
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Lege(LegeExpr { line, span }), span })
    }

    /// Parse `praefixum(expr)`, the source-level compile-time evaluation marker.
    fn parse_praefixum_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Praefixum, "expected 'praefixum'")?;
        self.expect(&TokenKind::LParen, "expected '(' after 'praefixum'")?;

        let body = PraefixumBody::Expr(Box::new(self.parse_expression()?));

        self.expect(&TokenKind::RParen, "expected ')'")?;

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Praefixum(PraefixumExpr { body }), span })
    }

    /// Parse regex literal syntax: `sed "pattern" [flags]`.
    fn parse_sed_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Sed, "expected 'sed'")?;

        let pattern = self.parse_string()?;
        let flags = if let TokenKind::Ident(sym) = self.peek().kind {
            self.advance();
            Some(sym)
        } else {
            None
        };

        let span = start.merge(self.previous_span());
        let id = self.next_id();
        Ok(Expr { id, kind: ExprKind::Sed(SedExpr { pattern, flags, span }), span })
    }
}
