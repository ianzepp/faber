//! Executable statement grammar for Faber blocks and entrypoints.
//!
//! This module owns the non-declaration statement forms that can appear inside
//! blocks or at file scope: conditionals, loops, dispatch, transfer, diagnostics,
//! entrypoints, resource scopes, endpoints, and destructuring statements. It
//! shares `StmtKind` construction with `decl.rs`, but keeps executable grammar
//! separate so declaration syntax and runtime control-flow syntax do not blur.
//!
//! DESIGN PHILOSOPHY
//! =================
//! Statement bodies deliberately use a small set of shapes: braced blocks for
//! scoped bodies, `ergo` for single-statement bodies, and `fac { ... } cape ...`
//! for structured error handling. The parser records surface intent and leaves
//! reachability, exhaustiveness, ownership, and endpoint semantics to later
//! phases.
//!
//! INVARIANTS
//! ==========
//! - `sin` is the canonical else-if spelling; `secus si` is diagnosed instead
//!   of being accepted as an alias.
//! - `tempta` is no longer canonical; users are directed to `fac ... cape`.
//! - `elige` is value dispatch, while `discerne` is pattern dispatch. Their AST
//!   shapes stay separate for later exhaustiveness and codegen decisions.
//! - Optional `cape` parsing is centralized so catch clauses attach consistently
//!   to statements that support them.

use super::{ParseError, ParseErrorKind, Parser};
use crate::lexer::TokenKind;
use crate::syntax::*;

// =============================================================================
// CONDITIONAL STATEMENTS
// =============================================================================

impl Parser {
    /// Parse `si` / `sin` / `secus` branching.
    ///
    /// GRAMMAR:
    ///   si-stmt := 'si' expr if-body ['cape' ident block] ['secus' secus-clause | 'sin' si-stmt]
    ///   if-body := block | 'ergo' stmt
    ///
    /// Branch bodies accept either a block or `ergo` single statement. `cape`
    /// attaches to the branch body being parsed, which preserves error-handling
    /// scope for nested `sin` chains.
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

    /// Parse a statement body that can be braced or introduced by `ergo`.
    fn parse_ergo_body(&mut self) -> Result<IfBody, ParseError> {
        if self.check(&TokenKind::LBrace) {
            Ok(IfBody::Block(self.parse_block()?))
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
            let body = self.parse_block()?;
            let catch = self.try_parse_cape_stmt()?;
            Ok(SecusClause::Block { body, catch })
        } else if self.eat_keyword(TokenKind::Ergo) {
            let stmt = Box::new(self.parse_statement()?);
            let catch = self.try_parse_cape_stmt()?;
            Ok(SecusClause::Stmt { stmt, catch })
        } else {
            Err(self.error(ParseErrorKind::MissingBlock, "expected block or 'ergo'"))
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

    /// Parse an `itera` loop and its ownership mode.
    ///
    /// GRAMMAR:
    ///   itera-stmt := 'itera' mode expr ('fixum'|'varia') ident if-body ['cape' ident block]
    ///   mode := 'ex' | 'de' | 'pro'
    ///
    /// The parser records `ex`, `de`, or `pro` as source-level iteration intent.
    /// Typechecking and lowering decide whether the chosen mode is valid for the
    /// iterable expression.
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

    /// Parse value-based `elige` dispatch.
    ///
    /// GRAMMAR:
    ///   elige-stmt := 'elige' expr '{' casu-case* [ceterum-default] '}' ['cape' ident block]
    ///   casu-case := 'casu' expr if-body
    ///   ceterum-default := 'ceterum' if-body
    ///
    /// `elige` cases are expressions, not patterns. Keeping this separate from
    /// `discerne` prevents destructuring-only syntax from leaking into value
    /// dispatch.
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

    /// Parse pattern-based `discerne` dispatch.
    ///
    /// GRAMMAR:
    ///   discerne-stmt := 'discerne' ['omnia'] expr (',' expr)* '{' arm* [default] '}'
    ///   arm := 'casu' pattern (',' pattern)* if-body
    ///
    /// `omnia` is only recorded here; later phases own the exhaustive-coverage
    /// check because they know the subject types and variant sets.
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
    /// Typically used with ergo return bodies for validation sequences.
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

    /// Parse `fac`, the canonical block form for do/try-style execution.
    ///
    /// GRAMMAR:
    ///   fac-stmt := 'fac' block ['cape' ident block] ['dum' expr]
    ///
    /// At statement level, a trailing `dum` turns `fac` into a do-while form.
    /// Closure bodies call the shared construct with `allow_while = false` so
    /// expression syntax cannot smuggle in a statement loop.
    pub(super) fn parse_fac_stmt(&mut self) -> Result<StmtKind, ParseError> {
        Ok(StmtKind::Fac(self.parse_fac_construct(true)?))
    }

    pub(super) fn parse_fac_construct(&mut self, allow_while: bool) -> Result<FacStmt, ParseError> {
        self.expect_keyword(TokenKind::Fac, "expected 'fac'")?;

        let body = self.parse_block()?;
        let catch = self.try_parse_cape_stmt()?;

        let while_ = if self.eat_keyword(TokenKind::Dum) {
            if !allow_while {
                return Err(self.error(ParseErrorKind::InvalidExpression, "closure fac body cannot use 'dum'"));
            }
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(FacStmt { body, catch, while_ })
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

    /// Parse explicit noop statement.
    pub(super) fn parse_tacet_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let span = self.current_span();
        self.expect_keyword(TokenKind::Tacet, "expected 'tacet'")?;
        Ok(StmtKind::Tacet(TacetStmt { span }))
    }

    // =============================================================================
    // ERROR HANDLING
    // =============================================================================

    /// Reject legacy `tempta` with the canonical replacement.
    ///
    /// The token remains recognized so users get a precise migration diagnostic
    /// instead of a generic "expected expression" failure.
    pub(super) fn parse_tempta_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Tempta, "expected 'tempta'")?;
        Err(self.error(
            ParseErrorKind::Expected,
            "tempta is no longer canonical; use 'fac { ... } cape err { ... }'",
        ))
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
    ///   scribe-stmt := ('scribe'|'vide'|'mone'|'nota') expr (',' expr)*
    ///
    /// WHY: Built-in output statements for different severity levels:
    /// - 'scribe' (compatibility alias) - neutral diagnostic output
    /// - 'vide' (debug) - debug output
    /// - 'mone' (warn) - warning output
    /// - 'nota' (note) - neutral diagnostic output
    pub(super) fn parse_scribe_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let kind = if self.eat_keyword(TokenKind::Scribe) {
            ScribeKind::Scribe
        } else if self.eat_keyword(TokenKind::Vide) {
            ScribeKind::Vide
        } else if self.eat_keyword(TokenKind::Mone) {
            ScribeKind::Mone
        } else if self.eat_keyword(TokenKind::Nota) {
            ScribeKind::Nota
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
    ///   incipit-stmt := ('incipit'|'incipiet') ['argumenta' ident] ['exitus' expr] block
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

        let body = IfBody::Block(self.parse_block()?);

        Ok(StmtKind::Incipit(IncipitStmt { is_async, body, args, exitus }))
    }

    /// Parse Zig allocator scope statement.
    ///
    /// GRAMMAR:
    ///   cura-stmt := 'cura' STRING ('fixum'|'varia') type ident block ['cape' ident block]
    ///
    /// WHY: This is intentionally scoped to Zig allocator setup. The allocator kind is
    /// a validated string so allocator strategy names do not become grammar words.
    pub(super) fn parse_cura_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Cura, "expected 'cura'")?;

        let kind_span = self.current_span();
        let kind_symbol = self.parse_string()?;
        let kind_text = self.interner.resolve(kind_symbol);
        let kind = match kind_text {
            "arena" => CuraKind::Arena,
            "page" => CuraKind::Page,
            _ => {
                return Err(ParseError {
                    kind: ParseErrorKind::Expected,
                    message: "expected allocator kind \"arena\" or \"page\"".to_owned(),
                    span: kind_span,
                });
            }
        };

        let mutability = if self.eat_keyword(TokenKind::Fixum) {
            Mutability::Immutable
        } else if self.eat_keyword(TokenKind::Varia) {
            Mutability::Mutable
        } else {
            return Err(self.error(ParseErrorKind::Expected, "expected 'fixum' or 'varia'"));
        };

        let ty = self.parse_type()?;
        let binding = self.parse_ident()?;
        let body = self.parse_block()?;
        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Cura(CuraStmt { kind, mutability, ty, binding, body, catch }))
    }

    /// Parse an `ad` capability-call declaration.
    ///
    /// GRAMMAR:
    ///   ad-stmt := 'ad' string '(' args ')' ['→' type ident ['ut' ident]] ['⇥' type] [block] ['cape' ident block]
    ///
    /// Capability-call parsing records the provider path, call-style arguments, optional
    /// response binding, body, and catch clause. Package/runtime integration owns
    /// verb routing and host framework semantics.
    pub(super) fn parse_ad_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_keyword(TokenKind::Ad, "expected 'ad'")?;

        let path = self.parse_string()?;

        self.expect(&TokenKind::LParen, "expected '('")?;
        let args = self.parse_argument_list()?;
        self.expect(&TokenKind::RParen, "expected ')'")?;

        let binding = self.try_parse_ad_binding()?;
        let err_ty = if self.eat(&TokenKind::ExitArrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.check(&TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let catch = self.try_parse_cape_stmt()?;

        Ok(StmtKind::Ad(AdStmt { path, args, binding, err_ty, body, catch }))
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

    fn try_parse_ad_binding(&mut self) -> Result<Option<AdBinding>, ParseError> {
        if !self.eat(&TokenKind::Arrow) {
            return Ok(None);
        }

        if self.check_keyword(TokenKind::Pro) {
            return Err(self.error(
                ParseErrorKind::Expected,
                "ad success bindings use '→ Type name', not '→ pro name'",
            ));
        }
        let ty = self.parse_type()?;
        let name = self.parse_ident()?;

        let alias = if self.eat_keyword(TokenKind::Ut) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        Ok(Some(AdBinding { verb: EndpointVerb::Fit, ty, name, alias }))
    }

    /// Parse a bare expression as a statement.
    pub(super) fn parse_expr_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let expr = Box::new(self.parse_expression()?);
        Ok(StmtKind::Expr(ExprStmt { expr }))
    }

    /// Try to parse an optional `cape` clause.
    ///
    /// GRAMMAR:
    ///   cape-clause := 'cape' ident block
    ///
    /// Centralizing this grammar keeps catch binding and body spans consistent
    /// across branches, loops, resource scopes, endpoints, and `fac`.
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
