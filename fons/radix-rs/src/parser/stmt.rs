//! Statement parsing

use super::{Parser, ParseError, ParseErrorKind};
use crate::lexer::TokenKind;
use crate::syntax::*;

impl Parser {
    /// Parse if statement
    pub(super) fn parse_if_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Si, "expected 'si'")?;

        let cond = Box::new(self.parse_expression()?);
        let then = self.parse_if_body()?;

        // Optional catch clause
        let catch = self.try_parse_catch()?;

        // Optional else clause
        let else_ = if self.eat_keyword(TokenKind::Secus) {
            Some(self.parse_else_clause()?)
        } else if self.eat_keyword(TokenKind::Sin) {
            // else-if chain
            if let StmtKind::If(nested) = self.parse_if_stmt()? {
                Some(ElseClause::If(Box::new(nested)))
            } else {
                unreachable!()
            }
        } else {
            None
        };

        Ok(StmtKind::If(IfStmt {
            cond,
            then,
            catch,
            else_,
        }))
    }

    /// Parse if body (block, ergo, or inline return)
    fn parse_if_body(&mut self) -> Result<IfBody, ParseError> {
        if self.check(&TokenKind::LBrace) {
            Ok(IfBody::Block(self.parse_block()?))
        } else if self.eat_keyword(TokenKind::Reddit) {
            Ok(IfBody::InlineReturn(InlineReturn::Reddit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Iacit) {
            Ok(IfBody::InlineReturn(InlineReturn::Iacit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Moritor) {
            Ok(IfBody::InlineReturn(InlineReturn::Moritor(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Tacet) {
            Ok(IfBody::InlineReturn(InlineReturn::Tacet))
        } else if self.eat_keyword(TokenKind::Ergo) {
            // "ergo" style - single statement treated as block
            Ok(IfBody::Ergo(Box::new(self.parse_statement()?)))
        } else {
            Err(self.error(ParseErrorKind::MissingBlock, "expected block or 'ergo'"))
        }
    }

    fn parse_else_clause(&mut self) -> Result<ElseClause, ParseError> {
        if self.check_keyword(TokenKind::Si) {
            if let StmtKind::If(nested) = self.parse_if_stmt()? {
                Ok(ElseClause::If(Box::new(nested)))
            } else {
                unreachable!()
            }
        } else if self.check(&TokenKind::LBrace) {
            Ok(ElseClause::Block(self.parse_block()?))
        } else if self.eat_keyword(TokenKind::Reddit) {
            Ok(ElseClause::InlineReturn(InlineReturn::Reddit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Iacit) {
            Ok(ElseClause::InlineReturn(InlineReturn::Iacit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Moritor) {
            Ok(ElseClause::InlineReturn(InlineReturn::Moritor(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Tacet) {
            Ok(ElseClause::InlineReturn(InlineReturn::Tacet))
        } else if self.eat_keyword(TokenKind::Ergo) {
            Ok(ElseClause::Stmt(Box::new(self.parse_statement()?)))
        } else {
            let id = self.next_id();
            let start = self.current_span();
            let kind = self.parse_expr_stmt()?;
            let span = start.merge(self.previous_span());
            Ok(ElseClause::Stmt(Box::new(Stmt { id, kind, span })))
        }
    }

    /// Parse while loop
    pub(super) fn parse_while_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Dum, "expected 'dum'")?;

        let cond = Box::new(self.parse_expression()?);
        let body = self.parse_if_body()?;
        let catch = self.try_parse_catch()?;

        Ok(StmtKind::While(WhileStmt { cond, body, catch }))
    }

    /// Parse iteration loop
    pub(super) fn parse_iter_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Itera, "expected 'itera'")?;

        let mode = if self.eat_keyword(TokenKind::Ex) {
            IterMode::Values
        } else if self.eat_keyword(TokenKind::De) {
            IterMode::Keys
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'ex' or 'de'"));
        };

        let iterable = Box::new(self.parse_expression()?);

        let mutability = if self.eat_keyword(TokenKind::Fixum) {
            Mutability::Immutable
        } else if self.eat_keyword(TokenKind::Varia) {
            Mutability::Mutable
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'fixum' or 'varia'"));
        };

        let binding = self.parse_ident()?;
        let body = self.parse_if_body()?;
        let catch = self.try_parse_catch()?;

        Ok(StmtKind::Iter(IterStmt {
            mode,
            iterable,
            mutability,
            binding,
            body,
            catch,
        }))
    }

    /// Parse switch statement
    pub(super) fn parse_switch_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Elige, "expected 'elige'")?;

        let expr = Box::new(self.parse_expression()?);

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Casu) {
                let start = self.current_span();
                let value = Box::new(self.parse_expression()?);
                let body = self.parse_if_body()?;
                let span = start.merge(self.previous_span());
                cases.push(SwitchCase { value, body, span });
            } else if self.eat_keyword(TokenKind::Ceterum) {
                let start = self.current_span();
                let body = self.parse_if_body()?;
                let span = start.merge(self.previous_span());
                default = Some(SwitchDefault { body, span });
                break;
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected 'casu' or 'ceterum'"));
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let catch = self.try_parse_catch()?;

        Ok(StmtKind::Switch(SwitchStmt {
            expr,
            cases,
            default,
            catch,
        }))
    }

    /// Parse match statement
    pub(super) fn parse_match_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Discerne, "expected 'discerne'")?;

        let exhaustive = self.eat_keyword(TokenKind::Omnia);

        // Parse discriminants (comma-separated expressions)
        let mut subjects = Vec::new();
        loop {
            subjects.push(self.parse_expression()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // Check if we hit the brace (don't parse it as expression)
            if self.check(&TokenKind::LBrace) {
                break;
            }
        }

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut arms = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Casu) {
                let start = self.current_span();
                let patterns = self.parse_patterns()?;
                let body = self.parse_if_body()?;
                let span = start.merge(self.previous_span());
                arms.push(MatchArm {
                    patterns,
                    body,
                    span,
                });
            } else if self.eat_keyword(TokenKind::Ceterum) {
                let start = self.current_span();
                let body = self.parse_if_body()?;
                let span = start.merge(self.previous_span());
                default = Some(SwitchDefault { body, span });
                break;
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected 'casu' or 'ceterum'"));
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Match(MatchStmt {
            exhaustive,
            subjects,
            arms,
            default,
        }))
    }

    /// Parse guard statement
    pub(super) fn parse_guard_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Custodi, "expected 'custodi'")?;
        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut clauses = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.expect_keyword(TokenKind::Si, "expected 'si'")?;
            let start = self.current_span();
            let cond = Box::new(self.parse_expression()?);
            let body = self.parse_if_body()?;
            let span = start.merge(self.previous_span());
            clauses.push(GuardClause { cond, body, span });
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Guard(GuardStmt { clauses }))
    }

    /// Parse fac statement
    pub(super) fn parse_fac_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Fac, "expected 'fac'")?;

        let body = self.parse_block()?;
        let catch = self.try_parse_catch()?;

        let while_ = if self.eat_keyword(TokenKind::Dum) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Fac(FacStmt {
            body,
            catch,
            while_,
        }))
    }

    /// Parse return statement
    pub(super) fn parse_return_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Redde, "expected 'redde'")?;

        // Check for value - if next token could start expression
        let value = if !self.is_at_end()
            && !self.check(&TokenKind::RBrace)
            && !self.check(&TokenKind::Semicolon)
        {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Return(ReturnStmt { value }))
    }

    /// Parse break statement
    pub(super) fn parse_break_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.expect_keyword(TokenKind::Rumpe, "expected 'rumpe'")?;
        Ok(StmtKind::Break(BreakStmt { span }))
    }

    /// Parse continue statement
    pub(super) fn parse_continue_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.expect_keyword(TokenKind::Perge, "expected 'perge'")?;
        Ok(StmtKind::Continue(ContinueStmt { span }))
    }

    /// Parse throw statement
    pub(super) fn parse_throw_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Iace, "expected 'iace'")?;
        let value = Box::new(self.parse_expression()?);
        Ok(StmtKind::Throw(ThrowStmt { value }))
    }

    /// Parse panic statement
    pub(super) fn parse_panic_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Mori, "expected 'mori'")?;
        let value = Box::new(self.parse_expression()?);
        Ok(StmtKind::Panic(PanicStmt { value }))
    }

    /// Parse try statement
    pub(super) fn parse_try_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Tempta, "expected 'tempta'")?;

        let body = self.parse_block()?;
        let catch = self.try_parse_catch()?;

        let finally = if self.eat_keyword(TokenKind::Demum) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(StmtKind::Try(TryStmt {
            body,
            catch,
            finally,
        }))
    }

    /// Parse assert statement
    pub(super) fn parse_assert_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Adfirma, "expected 'adfirma'")?;

        let cond = Box::new(self.parse_expression()?);

        let message = if self.eat(&TokenKind::Comma) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Assert(AssertStmt { cond, message }))
    }

    /// Parse output statement
    pub(super) fn parse_output_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let kind = if self.eat_keyword(TokenKind::Scribe) {
            OutputKind::Log
        } else if self.eat_keyword(TokenKind::Vide) {
            OutputKind::Debug
        } else if self.eat_keyword(TokenKind::Mone) {
            OutputKind::Warn
        } else {
            unreachable!()
        };

        let mut args = Vec::new();
        loop {
            args.push(self.parse_expression()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(StmtKind::Output(OutputStmt { kind, args }))
    }

    /// Parse entry statement
    pub(super) fn parse_entry_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let is_async = if self.eat_keyword(TokenKind::Incipit) {
            false
        } else if self.eat_keyword(TokenKind::Incipiet) {
            true
        } else {
            unreachable!()
        };

        let body = self.parse_if_body()?;

        Ok(StmtKind::Entry(EntryStmt { is_async, body }))
    }

    /// Parse resource statement
    pub(super) fn parse_resource_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Cura, "expected 'cura'")?;

        let kind = if self.eat_keyword(TokenKind::Arena) {
            Some(ResourceKind::Arena)
        } else {
            None // 'page' would go here too
        };

        let init = if !self.check_keyword(TokenKind::Fixum) && !self.check_keyword(TokenKind::Varia)
        {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let mutability = if self.eat_keyword(TokenKind::Fixum) {
            Mutability::Immutable
        } else if self.eat_keyword(TokenKind::Varia) {
            Mutability::Mutable
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'fixum' or 'varia'"));
        };

        let ty = self.try_parse_type()?;
        let binding = self.parse_ident()?;
        let body = self.parse_block()?;
        let catch = self.try_parse_catch()?;

        Ok(StmtKind::Resource(ResourceStmt {
            kind,
            init,
            mutability,
            ty,
            binding,
            body,
            catch,
        }))
    }

    /// Parse endpoint statement
    pub(super) fn parse_endpoint_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Ad, "expected 'ad'")?;

        let path = self.parse_string()?;

        self.expect(&TokenKind::LParen, "expected '('")?;
        let args = self.parse_argument_list()?;
        self.expect(&TokenKind::RParen, "expected ')'")?;

        let binding = self.try_parse_endpoint_binding()?;

        let body = if self.check(&TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let catch = self.try_parse_catch()?;

        Ok(StmtKind::Endpoint(EndpointStmt {
            path,
            args,
            binding,
            body,
            catch,
        }))
    }

    fn try_parse_endpoint_binding(&mut self) -> Result<Option<EndpointBinding>, ParseError> {
        let verb = if self.eat_keyword(TokenKind::Casu) {
            // Using 'casu' as placeholder - actual verbs would be fit/fiet/fiunt/fient
            EndpointVerb::Fit
        } else {
            return Ok(None);
        };

        let ty = self.try_parse_type()?;

        self.expect_keyword(TokenKind::Pro, "expected 'pro'")?;
        let name = self.parse_ident()?;

        let alias = if self.eat_keyword(TokenKind::Ut) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        Ok(Some(EndpointBinding {
            verb,
            ty,
            name,
            alias,
        }))
    }

    /// Parse expression statement
    pub(super) fn parse_expr_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let expr = Box::new(self.parse_expression()?);
        Ok(StmtKind::Expr(ExprStmt { expr }))
    }

    /// Try to parse catch clause
    fn try_parse_catch(&mut self) -> Result<Option<CatchClause>, ParseError> {
        if !self.eat_keyword(TokenKind::Cape) {
            return Ok(None);
        }

        let start = self.current_span();
        let binding = self.parse_ident()?;
        let body = self.parse_block()?;
        let span = start.merge(self.previous_span());

        Ok(Some(CatchClause {
            binding,
            body,
            span,
        }))
    }
}
