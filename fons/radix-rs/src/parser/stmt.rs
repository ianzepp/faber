//! Control Flow and Statement Parsing
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module handles parsing of Faber's control flow constructs (conditionals,
//! loops, pattern matching), transfer statements (return, break, continue, throw),
//! and special statement forms (try-catch, assertions, I/O, resource management).
//!
//! COMPILER PHASE: Parsing
//! INPUT: Token stream (via Parser methods)
//! OUTPUT: Statement AST nodes (StmtKind variants)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Inline return syntax: Statements like `si` support `reddit expr` as shorthand
//! - Error attachment: Many statements support `cape` (catch) clauses for error handling
//! - Expression-oriented: Statement bodies can be blocks, ergo (single stmt), or inline returns
//! - Pattern exhaustiveness: `discerne omnia` requires matching all variants
//!
//! GRAMMAR COVERAGE
//! ================
//! This module implements parsers for:
//! - Conditionals: si/sin/secus (if/else-if/else)
//! - Loops: dum (while), itera (for), fac...dum (do-while)
//! - Pattern matching: discerne (match), elige (switch)
//! - Guards: custodi (multi-branch early return)
//! - Transfers: redde (return), rumpe (break), perge (continue), iace (throw), mori (panic)
//! - Error handling: tempta/cape/demum (try/catch/finally)
//! - Assertions: adfirma (assert)
//! - I/O: scribe/vide/mone (print/debug/warn)
//! - Entry points: incipit/incipiet (main/async main)
//! - Resources: cura (resource management)
//! - Endpoints: ad (HTTP endpoint definition)
//! - Destructuring: ex (extract/destructure)

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::{Span, Symbol, TokenKind};
use crate::syntax::*;

// =============================================================================
// CONDITIONAL STATEMENTS
// =============================================================================

impl Parser {
    /// Parse if statement.
    ///
    /// GRAMMAR:
    ///   si-stmt := 'si' expr if-body ['cape' ident block] ['secus' secus-clause | 'sin' si-stmt]
    ///   if-body := block | 'ergo' stmt | inline-return
    ///   inline-return := 'reddit' expr | 'iacit' expr | 'moritor' expr | 'tacet'
    ///
    /// WHY: Faber supports multiple if-body styles for conciseness. Block style for
    /// multi-statement bodies, 'ergo' for single statements, inline returns for
    /// early returns without braces.
    ///
    /// ERROR HANDLING: Optional 'cape' clause catches errors thrown in condition or body.
    pub(super) fn parse_si_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Si, "expected 'si'")?;

        let cond = Box::new(self.parse_expression()?);
        let then = self.parse_ergo_body()?;

        // Optional catch clause
        let catch = self.try_parse_cape_stmt()?;

        // Optional else clause
        let else_ = if self.eat_keyword(TokenKind::Secus) {
            Some(self.parse_secus_stmt()?)
        } else if self.eat_keyword(TokenKind::Sin) {
            // else-if chain: sin <cond> { }
            Some(SecusClause::Sin(Box::new(self.parse_sin_stmt()?)))
        } else {
            None
        };

        Ok(StmtKind::Si(SiStmt { cond, then, catch, else_ }))
    }

    /// Parse sin statement - sin already consumed
    fn parse_sin_stmt(&mut self) -> Result<SiStmt, ParseError> {
        let cond = Box::new(self.parse_expression()?);
        let then = self.parse_ergo_body()?;
        let catch = self.try_parse_cape_stmt()?;

        let else_ = if self.eat_keyword(TokenKind::Secus) {
            Some(self.parse_secus_stmt()?)
        } else if self.eat_keyword(TokenKind::Sin) {
            Some(SecusClause::Sin(Box::new(self.parse_sin_stmt()?)))
        } else {
            None
        };

        Ok(SiStmt { cond, then, catch, else_ })
    }

    /// Parse if body (block, ergo, or inline return)
    fn parse_ergo_body(&mut self) -> Result<IfBody, ParseError> {
        if self.check(&TokenKind::LBrace) {
            Ok(IfBody::Block(self.parse_block()?))
        } else if self.eat_keyword(TokenKind::Reddit) {
            Ok(IfBody::InlineReturn(InlineReturn::Reddit(Box::new(self.parse_expression()?))))
        } else if self.eat_keyword(TokenKind::Iacit) {
            Ok(IfBody::InlineReturn(InlineReturn::Iacit(Box::new(self.parse_expression()?))))
        } else if self.eat_keyword(TokenKind::Moritor) {
            Ok(IfBody::InlineReturn(InlineReturn::Moritor(Box::new(self.parse_expression()?))))
        } else if self.eat_keyword(TokenKind::Tacet) {
            Ok(IfBody::InlineReturn(InlineReturn::Tacet))
        } else if self.eat_keyword(TokenKind::Ergo) {
            // "ergo" style - single statement treated as block
            Ok(IfBody::Ergo(Box::new(self.parse_statement()?)))
        } else {
            Err(self.error(ParseErrorKind::MissingBlock, "expected block or 'ergo'"))
        }
    }

    fn parse_secus_stmt(&mut self) -> Result<SecusClause, ParseError> {
        if self.check_keyword(TokenKind::Si) {
            return Err(self.error(ParseErrorKind::Expected, "use 'sin' for else-if"));
        }

        if self.check(&TokenKind::LBrace) {
            Ok(SecusClause::Block(self.parse_block()?))
        } else if self.eat_keyword(TokenKind::Reddit) {
            Ok(SecusClause::InlineReturn(InlineReturn::Reddit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Iacit) {
            Ok(SecusClause::InlineReturn(InlineReturn::Iacit(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Moritor) {
            Ok(SecusClause::InlineReturn(InlineReturn::Moritor(Box::new(
                self.parse_expression()?,
            ))))
        } else if self.eat_keyword(TokenKind::Tacet) {
            Ok(SecusClause::InlineReturn(InlineReturn::Tacet))
        } else if self.eat_keyword(TokenKind::Ergo) {
            Ok(SecusClause::Stmt(Box::new(self.parse_statement()?)))
        } else {
            let id = self.next_id();
            let start = self.current_span();
            let kind = self.parse_expr_stmt()?;
            let span = start.merge(self.previous_span());
            Ok(SecusClause::Stmt(Box::new(Stmt { id, kind, span, annotations: Vec::new() })))
        }
    }

    // =============================================================================
    // LOOP STATEMENTS
    // =============================================================================

    /// Parse while loop.
    ///
    /// GRAMMAR:
    ///   dum-stmt := 'dum' expr if-body ['cape' ident block]
    ///
    /// WHY: Standard while loop with optional error handling via 'cape'.
    pub(super) fn parse_dum_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Dum, "expected 'dum'")?;

        let cond = Box::new(self.parse_expression()?);
        let body = self.parse_ergo_body()?;
        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Dum(DumStmt { cond, body, catch }))
    }

    /// Parse iteration loop (for-each).
    ///
    /// GRAMMAR:
    ///   itera-stmt := 'itera' mode expr ('fixum'|'varia') ident if-body ['cape' ident block]
    ///   mode := 'ex' | 'de' | 'pro'
    ///
    /// WHY: Ownership modes (ex/de/pro) control iteration semantics:
    /// - 'ex' moves items out of collection
    /// - 'de' borrows items immutably
    /// - 'pro' enumerates (index/value pairs)
    pub(super) fn parse_itera_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Itera, "expected 'itera'")?;

        let mode = if self.eat_keyword(TokenKind::Ex) {
            IteraMode::Ex
        } else if self.eat_keyword(TokenKind::De) {
            IteraMode::De
        } else if self.eat_keyword(TokenKind::Pro) {
            IteraMode::Pro
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'ex', 'de', or 'pro'"));
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
        let body = self.parse_ergo_body()?;
        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Itera(IteraStmt { mode, iterable, mutability, binding, body, catch }))
    }

    // =============================================================================
    // PATTERN MATCHING AND DISPATCH
    // =============================================================================

    /// Parse switch statement (value-based dispatch).
    ///
    /// GRAMMAR:
    ///   elige-stmt := 'elige' expr '{' casu-case* [ceterum-default] '}' ['cape' ident block]
    ///   casu-case := 'casu' expr if-body
    ///   ceterum-default := 'ceterum' if-body
    ///
    /// WHY: Simple value-based dispatch without destructuring. Cases compared via
    /// equality (not pattern matching). Use 'discerne' for pattern-based matching.
    pub(super) fn parse_elige_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Elige, "expected 'elige'")?;

        let expr = Box::new(self.parse_expression()?);

        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            if self.eat_keyword(TokenKind::Casu) {
                let start = self.current_span();
                let value = Box::new(self.parse_expression()?);
                let body = self.parse_ergo_body()?;
                let span = start.merge(self.previous_span());
                cases.push(CasuCase { value, body, span });
            } else if self.eat_keyword(TokenKind::Ceterum) {
                let start = self.current_span();
                let body = self.parse_ergo_body()?;
                let span = start.merge(self.previous_span());
                default = Some(CeterumDefault { body, span });
                break;
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected 'casu' or 'ceterum'"));
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Elige(EligeStmt { expr, cases, default, catch }))
    }

    /// Parse match statement (pattern-based dispatch).
    ///
    /// GRAMMAR:
    ///   discerne-stmt := 'discerne' ['omnia'] expr (',' expr)* '{' arm* [default] '}'
    ///   arm := 'casu' pattern (',' pattern)* if-body
    ///
    /// WHY: Pattern matching with destructuring and binding. 'omnia' requires
    /// exhaustive matching (all variants covered). Supports multiple subjects
    /// for tuple-like matching.
    pub(super) fn parse_discerne_stmt(&mut self) -> Result<StmtKind, ParseError> {
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
                let body = self.parse_ergo_body()?;
                let span = start.merge(self.previous_span());
                arms.push(CasuArm { patterns, body, span });
            } else if self.eat_keyword(TokenKind::Ceterum) {
                let start = self.current_span();
                let body = self.parse_ergo_body()?;
                let span = start.merge(self.previous_span());
                default = Some(CeterumDefault { body, span });
                break;
            } else {
                return Err(self.error(ParseErrorKind::Expected, "expected 'casu' or 'ceterum'"));
            }
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Discerne(DiscerneStmt { exhaustive, subjects, arms, default }))
    }

    /// Parse guard statement (multi-branch early return).
    ///
    /// GRAMMAR:
    ///   custodi-stmt := 'custodi' '{' ('si' expr if-body)+ '}'
    ///
    /// WHY: Guard clauses for early returns. Similar to Swift's guard statement.
    /// Typically used with inline return bodies for validation sequences.
    pub(super) fn parse_custodi_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Custodi, "expected 'custodi'")?;
        self.expect(&TokenKind::LBrace, "expected '{'")?;

        let mut clauses = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.expect_keyword(TokenKind::Si, "expected 'si'")?;
            let start = self.current_span();
            let cond = Box::new(self.parse_expression()?);
            let body = self.parse_ergo_body()?;
            let span = start.merge(self.previous_span());
            clauses.push(CustodiClause { cond, body, span });
        }

        self.expect(&TokenKind::RBrace, "expected '}'")?;

        Ok(StmtKind::Custodi(CustodiStmt { clauses }))
    }

    /// Parse do-while loop.
    ///
    /// GRAMMAR:
    ///   fac-stmt := 'fac' block ['cape' ident block] ['dum' expr]
    ///
    /// WHY: Body executes at least once, optional while condition controls repetition.
    /// Without 'dum' clause, executes exactly once (equivalent to bare block with error handling).
    pub(super) fn parse_fac_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Fac, "expected 'fac'")?;

        let body = self.parse_block()?;
        let catch = self.try_parse_cape_stmt()?;

        let while_ = if self.eat_keyword(TokenKind::Dum) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Fac(FacStmt { body, catch, while_ }))
    }

    // =============================================================================
    // TRANSFER STATEMENTS
    // =============================================================================

    /// Parse return statement.
    ///
    /// GRAMMAR:
    ///   redde-stmt := 'redde' [expr]
    ///
    /// WHY: Returns a value from the current function. Optional value for void functions.
    pub(super) fn parse_redde_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Redde, "expected 'redde'")?;

        // Check for value - if next token could start expression
        let value = if !self.is_at_end() && !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Redde(ReddeStmt { value }))
    }

    /// Parse break statement
    pub(super) fn parse_rumpe_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.expect_keyword(TokenKind::Rumpe, "expected 'rumpe'")?;
        Ok(StmtKind::Rumpe(RumpeStmt { span }))
    }

    /// Parse continue statement
    pub(super) fn parse_perge_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.expect_keyword(TokenKind::Perge, "expected 'perge'")?;
        Ok(StmtKind::Perge(PergeStmt { span }))
    }

    /// Parse throw statement
    pub(super) fn parse_iace_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Iace, "expected 'iace'")?;
        let value = Box::new(self.parse_expression()?);
        Ok(StmtKind::Iace(IaceStmt { value }))
    }

    /// Parse panic statement
    pub(super) fn parse_mori_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Mori, "expected 'mori'")?;
        let value = Box::new(self.parse_expression()?);
        Ok(StmtKind::Mori(MoriStmt { value }))
    }

    // =============================================================================
    // ERROR HANDLING
    // =============================================================================

    /// Parse try-catch-finally statement.
    ///
    /// GRAMMAR:
    ///   tempta-stmt := 'tempta' block ['cape' ident block] ['demum' block]
    ///
    /// WHY: Structured error handling. 'cape' binds error value, 'demum' runs
    /// unconditionally (like finally). Both are optional.
    pub(super) fn parse_tempta_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Tempta, "expected 'tempta'")?;

        let body = self.parse_block()?;
        let catch = self.try_parse_cape_stmt()?;

        let finally = if self.eat_keyword(TokenKind::Demum) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(StmtKind::Tempta(TemptaStmt { body, catch, finally }))
    }

    /// Parse assert statement.
    ///
    /// GRAMMAR:
    ///   adfirma-stmt := 'adfirma' expr [',' expr]
    ///
    /// WHY: Runtime assertions for invariant checking. Optional message expression
    /// provides diagnostic information on failure.
    pub(super) fn parse_adfirma_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Adfirma, "expected 'adfirma'")?;

        let cond = Box::new(self.parse_expression()?);

        let message = if self.eat(&TokenKind::Comma) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(StmtKind::Adfirma(AdfirmaStmt { cond, message }))
    }

    // =============================================================================
    // I/O AND DEBUGGING
    // =============================================================================

    /// Parse output statement.
    ///
    /// GRAMMAR:
    ///   scribe-stmt := ('scribe'|'vide'|'mone') expr (',' expr)*
    ///
    /// WHY: Built-in output statements for different severity levels:
    /// - 'scribe' (println) - standard output
    /// - 'vide' (debug) - debug output
    /// - 'mone' (warn) - warning output
    pub(super) fn parse_scribe_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let kind = if self.eat_keyword(TokenKind::Scribe) {
            ScribeKind::Scribe
        } else if self.eat_keyword(TokenKind::Vide) {
            ScribeKind::Vide
        } else if self.eat_keyword(TokenKind::Mone) {
            ScribeKind::Mone
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

        Ok(StmtKind::Scribe(ScribeStmt { kind, args }))
    }

    // =============================================================================
    // SPECIAL STATEMENTS
    // =============================================================================

    /// Parse entry point declaration.
    ///
    /// GRAMMAR:
    ///   incipit-stmt := ('incipit'|'incipiet') ['argumenta' ident] ['exitus' expr] if-body
    ///
    /// WHY: Program entry points. 'incipit' for sync main, 'incipiet' for async main.
    /// Optional 'argumenta' binds command-line args, 'exitus' sets exit code.
    pub(super) fn parse_incipit_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let is_async = if self.eat_keyword(TokenKind::Incipit) {
            false
        } else if self.eat_keyword(TokenKind::Incipiet) {
            true
        } else {
            unreachable!()
        };

        let args = if self.eat_keyword(TokenKind::Argumenta) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        let exitus = if self.eat_keyword(TokenKind::Exitus) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let body = self.parse_ergo_body()?;

        Ok(StmtKind::Incipit(IncipitStmt { is_async, body, args, exitus }))
    }

    /// Parse resource management statement.
    ///
    /// GRAMMAR:
    ///   cura-stmt := 'cura' ['arena'] [expr] ('fixum'|'varia') [type] ident block ['cape' ident block]
    ///
    /// WHY: Resource management with automatic cleanup (like Rust's Drop or C++'s RAII).
    /// 'arena' creates arena allocator for block scope. Resource bound to identifier,
    /// cleanup happens when block exits.
    ///
    /// EDGE: Anonymous scopes allowed: `cura arena { ... }` without binding.
    pub(super) fn parse_cura_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Cura, "expected 'cura'")?;

        let kind = if self.eat_keyword(TokenKind::Arena) {
            Some(CuraKind::Arena)
        } else if matches!(self.peek().kind, TokenKind::Ident(sym) if self.interner.resolve(sym) == "page") {
            self.advance();
            Some(CuraKind::Page)
        } else {
            None
        };

        // Check for anonymous scope: cura arena { } without binding
        if self.check(&TokenKind::LBrace) {
            let body = self.parse_block()?;
            let catch = self.try_parse_cape_stmt()?;
            return Ok(StmtKind::Cura(CuraStmt {
                kind,
                init: None,
                mutability: Mutability::Immutable,
                ty: None,
                binding: Ident {
                    name: Symbol(0), // anonymous
                    span: Span::default(),
                },
                body,
                catch,
            }));
        }

        let init = if !self.check_keyword(TokenKind::Fixum) && !self.check_keyword(TokenKind::Varia) {
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

        // Check: is this "name {" (no type) or "type name {" (with type)?
        let ty = if matches!(self.peek().kind, TokenKind::Ident(_)) && matches!(self.peek_at(1).kind, TokenKind::LBrace)
        {
            None
        } else {
            Some(self.parse_type()?)
        };

        let binding = self.parse_ident()?;
        let body = self.parse_block()?;
        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Cura(CuraStmt { kind, init, mutability, ty, binding, body, catch }))
    }

    /// Parse HTTP endpoint declaration.
    ///
    /// GRAMMAR:
    ///   ad-stmt := 'ad' string '(' args ')' ['->' [type] 'pro' ident ['ut' ident]] [block] ['cape' ident block]
    ///
    /// WHY: Declares HTTP endpoints for web services. Path string supports templates,
    /// args bind to request parameters, optional binding captures response.
    pub(super) fn parse_ad_stmt(&mut self) -> Result<StmtKind, ParseError> {
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

        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Ad(AdStmt { path, args, binding, body, catch }))
    }

    /// Parse destructuring statement.
    ///
    /// GRAMMAR:
    ///   ex-stmt := 'ex' expr ('fixum'|'varia') field (',' field)* ['ceteri' ident]
    ///   field := ident ['ut' ident]
    ///
    /// WHY: Extracts fields from objects/structs into local variables. Similar to
    /// JavaScript destructuring. 'ceteri' (rest) captures remaining fields.
    pub(super) fn parse_extract_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let start = self.current_span();
        self.expect_keyword(TokenKind::Ex, "expected 'ex'")?;

        let source = Box::new(self.parse_expression()?);

        let mutability = if self.eat_keyword(TokenKind::Fixum) {
            Mutability::Immutable
        } else if self.eat_keyword(TokenKind::Varia) {
            Mutability::Mutable
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'fixum' or 'varia'"));
        };

        let mut fields = Vec::new();
        let mut rest = None;

        loop {
            if self.eat_keyword(TokenKind::Ceteri) {
                if rest.is_some() {
                    return Err(self.error(ParseErrorKind::Expected, "rest pattern already specified"));
                }
                rest = Some(self.parse_ident()?);
                break;
            }

            let name = self.parse_ident()?;
            let alias = if self.eat_keyword(TokenKind::Ut) {
                Some(self.parse_ident()?)
            } else {
                None
            };
            fields.push(ExField { name, alias });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        let span = start.merge(self.previous_span());
        Ok(StmtKind::Ex(ExStmt { source, mutability, fields, rest, span }))
    }

    fn try_parse_endpoint_binding(&mut self) -> Result<Option<AdBinding>, ParseError> {
        if !self.eat(&TokenKind::Arrow) {
            return Ok(None);
        }

        let ty = if self.check_keyword(TokenKind::Pro) {
            None
        } else {
            Some(self.parse_type()?)
        };

        self.expect_keyword(TokenKind::Pro, "expected 'pro'")?;
        let name = self.parse_ident()?;

        let alias = if self.eat_keyword(TokenKind::Ut) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        Ok(Some(AdBinding { verb: EndpointVerb::Fit, ty, name, alias }))
    }

    /// Parse expression statement
    pub(super) fn parse_expr_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let expr = Box::new(self.parse_expression()?);
        Ok(StmtKind::Expr(ExprStmt { expr }))
    }

    /// Try to parse optional catch clause.
    ///
    /// GRAMMAR:
    ///   cape-clause := 'cape' ident block
    ///
    /// WHY: Many statements support optional error handling via 'cape'. This helper
    /// is shared across statement parsers to maintain consistent syntax.
    fn try_parse_cape_stmt(&mut self) -> Result<Option<CapeClause>, ParseError> {
        if !self.eat_keyword(TokenKind::Cape) {
            return Ok(None);
        }

        let start = self.current_span();
        let binding = self.parse_ident()?;
        let body = self.parse_block()?;
        let span = start.merge(self.previous_span());

        Ok(Some(CapeClause { binding, body, span }))
    }
}
