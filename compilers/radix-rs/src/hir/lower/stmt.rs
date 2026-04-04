//! Statement lowering
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Transforms AST statements into HIR statements, handling control flow
//! constructs (si, dum, itera, discerne) and desugaring ergo/reddit syntax.
//!
//! COMPILER PHASE: HIR Lowering (submodule)
//! INPUT: AST statements (syntax::Stmt)
//! OUTPUT: HIR statements (HirStmt)
//!
//! CONTROL FLOW DESUGARING
//! =======================
//! - ergo: Single statement becomes block with one statement
//! - reddit: Inline return becomes explicit HirStmtKind::Redde
//! - custodi: Multiple clauses desugar to nested si expressions
//! - discerne: Pattern matching with scope management for bindings
//!
//! WHY: Explicit returns and blocks simplify control-flow analysis in later
//! passes (type checking doesn't need to handle multiple return syntaxes).

use super::{pattern, HirBlock, HirExpr, HirExprKind, HirStmt, HirStmtKind, Lowerer};
use crate::hir::{HirCasuArm, HirLiteral, HirPattern};
use crate::lexer::Span;
use crate::syntax::{Stmt, StmtKind};

fn error_expr(lowerer: &mut Lowerer, span: Span) -> HirExpr {
    HirExpr { id: lowerer.next_hir_id(), kind: HirExprKind::Error, ty: None, span }
}

/// Lower a statement
pub fn lower_stmt(lowerer: &mut Lowerer, stmt: &Stmt) -> HirStmt {
    let id = lowerer.next_hir_id();
    let span = stmt.span;
    lowerer.current_span = span;

    let kind = match &stmt.kind {
        StmtKind::Var(decl) => lowerer.lower_var_stmt(decl),
        StmtKind::Expr(expr) => lowerer.lower_expr_stmt(expr),
        StmtKind::Si(if_stmt) => lowerer.lower_si(if_stmt),
        StmtKind::Dum(while_stmt) => lowerer.lower_dum(while_stmt),
        StmtKind::Itera(iter_stmt) => lowerer.lower_itera(iter_stmt),
        StmtKind::Custodi(custodi_stmt) => lowerer.lower_custodi(custodi_stmt),
        StmtKind::Fac(fac_stmt) => lowerer.lower_fac(fac_stmt),
        StmtKind::Redde(ret) => lowerer.lower_redde(ret),
        StmtKind::Rumpe(_) => HirStmtKind::Rumpe,
        StmtKind::Perge(_) => HirStmtKind::Perge,
        StmtKind::Iace(iace_stmt) => lowerer.lower_iace(iace_stmt),
        StmtKind::Mori(mori_stmt) => lowerer.lower_mori(mori_stmt),
        StmtKind::Tempta(tempta_stmt) => lowerer.lower_tempta(tempta_stmt),
        StmtKind::Adfirma(adfirma_stmt) => lowerer.lower_adfirma(adfirma_stmt),
        StmtKind::Scribe(scribe_stmt) => lowerer.lower_scribe(scribe_stmt),
        StmtKind::Incipit(incipit_stmt) => lowerer.lower_incipit_stmt(incipit_stmt),
        StmtKind::Cura(cura_stmt) => lowerer.lower_cura(cura_stmt),
        StmtKind::Ad(ad_stmt) => lowerer.lower_ad(ad_stmt),
        StmtKind::Ex(ex_stmt) => lowerer.lower_ex(ex_stmt),
        StmtKind::Elige(elige_stmt) => lowerer.lower_elige(elige_stmt),
        StmtKind::Discerne(discerne_stmt) => lowerer.lower_discerne(discerne_stmt),
        StmtKind::Block(block) => {
            let block = lowerer.lower_block(block);
            HirStmtKind::Expr(HirExpr { id: lowerer.next_hir_id(), kind: HirExprKind::Block(block), ty: None, span })
        }
        _ => {
            lowerer.error("unhandled statement kind in lowering");
            HirStmtKind::Expr(error_expr(lowerer, span))
        }
    };

    HirStmt { id, kind, span }
}

/// Lower a statement to one or more HIR statements.
pub fn lower_stmt_expanded(lowerer: &mut Lowerer, stmt: &Stmt) -> Vec<HirStmt> {
    match &stmt.kind {
        StmtKind::Var(decl) if matches!(&decl.binding, crate::syntax::BindingPattern::Array { .. }) => {
            lowerer.lower_array_destructure_stmt(decl)
        }
        StmtKind::Ex(stmt) => lowerer.lower_ex_stmt(stmt),
        _ => vec![lower_stmt(lowerer, stmt)],
    }
}

impl<'a> Lowerer<'a> {
    fn lower_array_destructure_stmt(&mut self, decl: &crate::syntax::VarDecl) -> Vec<HirStmt> {
        let Some(init) = &decl.init else {
            self.error("array destructuring requires initializer");
            return Vec::new();
        };

        let mut out = Vec::new();
        let mutable = decl.mutability == crate::syntax::Mutability::Mutable;
        let (elements, rest, span) = match &decl.binding {
            crate::syntax::BindingPattern::Array { elements, rest, span } => (elements, rest, *span),
            _ => return Vec::new(),
        };

        for (idx, pattern) in elements.iter().enumerate() {
            match pattern {
                crate::syntax::BindingPattern::Ident(ident) => {
                    let def_id = self.next_def_id();
                    self.bind_local(ident.name, def_id);
                    let index_expr = HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Literal(HirLiteral::Int(idx as i64)),
                        ty: None,
                        span: ident.span,
                    };
                    let init_expr = HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Index(Box::new(self.lower_expr(init)), Box::new(index_expr)),
                        ty: None,
                        span: ident.span,
                    };
                    out.push(HirStmt {
                        id: self.next_hir_id(),
                        kind: HirStmtKind::Local(crate::hir::HirLocal {
                            def_id,
                            name: ident.name,
                            ty: None,
                            init: Some(init_expr),
                            mutable,
                        }),
                        span: ident.span,
                    });
                }
                crate::syntax::BindingPattern::Wildcard(_) => {}
                crate::syntax::BindingPattern::Array { .. } => {
                    self.error("nested array destructuring is not lowered yet");
                }
            }
        }

        if let Some(rest_ident) = rest {
            let def_id = self.next_def_id();
            self.bind_local(rest_ident.name, def_id);
            out.push(HirStmt {
                id: self.next_hir_id(),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id,
                    name: rest_ident.name,
                    ty: None,
                    init: Some(self.lower_expr(init)),
                    mutable,
                }),
                span: rest_ident.span,
            });
        }

        if out.is_empty() {
            out.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Expr(self.lower_expr(init)), span });
        }

        out
    }

    fn lower_ex_stmt(&mut self, stmt: &crate::syntax::ExStmt) -> Vec<HirStmt> {
        let mut out = Vec::new();

        for field in &stmt.fields {
            let name = field
                .alias
                .as_ref()
                .map(|ident| ident.name)
                .unwrap_or(field.name.name);
            let def_id = self.next_def_id();
            self.bind_local(name, def_id);
            let local = crate::hir::HirLocal {
                def_id,
                name,
                ty: None,
                init: Some(HirExpr {
                    id: self.next_hir_id(),
                    kind: HirExprKind::Field(Box::new(self.lower_expr(&stmt.source)), field.name.name),
                    ty: None,
                    span: field.name.span,
                }),
                mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
            };
            out.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Local(local), span: field.name.span });
        }

        if let Some(rest) = &stmt.rest {
            let def_id = self.next_def_id();
            self.bind_local(rest.name, def_id);
            let local = crate::hir::HirLocal {
                def_id,
                name: rest.name,
                ty: None,
                init: Some(self.lower_expr(&stmt.source)),
                mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
            };
            out.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Local(local), span: rest.span });
        }
        out
    }

    /// Lower variable declaration statement
    fn lower_var_stmt(&mut self, decl: &crate::syntax::VarDecl) -> HirStmtKind {
        match &decl.binding {
            crate::syntax::BindingPattern::Ident(ident) => {
                let def_id = self.next_def_id();
                self.bind_local(ident.name, def_id);
                let ty = decl.ty.as_ref().map(|ty| self.lower_type(ty));
                let init = decl.init.as_ref().map(|expr| self.lower_expr(expr));
                HirStmtKind::Local(crate::hir::HirLocal {
                    def_id,
                    name: ident.name,
                    ty,
                    init,
                    mutable: decl.mutability == crate::syntax::Mutability::Mutable,
                })
            }
            crate::syntax::BindingPattern::Wildcard(span) => {
                let init = decl
                    .init
                    .as_ref()
                    .map(|expr| self.lower_expr(expr))
                    .unwrap_or_else(|| error_expr(self, *span));
                HirStmtKind::Expr(init)
            }
            crate::syntax::BindingPattern::Array { span, .. } => {
                self.error("array destructuring should be expanded before statement lowering");
                HirStmtKind::Expr(error_expr(self, *span))
            }
        }
    }

    /// Lower expression statement
    fn lower_expr_stmt(&mut self, expr: &crate::syntax::ExprStmt) -> HirStmtKind {
        let expr_hir = self.lower_expr(&expr.expr);
        HirStmtKind::Expr(expr_hir)
    }

    /// Lower si (if) statement
    fn lower_si(&mut self, if_stmt: &crate::syntax::SiStmt) -> HirStmtKind {
        let expr = self.lower_si_expr(if_stmt);
        HirStmtKind::Expr(expr)
    }

    /// Lower dum (while) statement
    fn lower_dum(&mut self, while_stmt: &crate::syntax::DumStmt) -> HirStmtKind {
        let cond = self.lower_expr(&while_stmt.cond);
        let body = self.lower_ergo_body(&while_stmt.body);
        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Dum(Box::new(cond), body),
            ty: None,
            span: self.current_span,
        };

        HirStmtKind::Expr(expr)
    }

    /// Lower itera (for) statement
    fn lower_itera(&mut self, iter_stmt: &crate::syntax::IteraStmt) -> HirStmtKind {
        let mode = match iter_stmt.mode {
            crate::syntax::IteraMode::Ex => crate::hir::HirIteraMode::Ex,
            crate::syntax::IteraMode::De => crate::hir::HirIteraMode::De,
            crate::syntax::IteraMode::Pro => crate::hir::HirIteraMode::Pro,
        };

        let binding = self.next_def_id();
        let iter = self.lower_expr(&iter_stmt.iterable);
        self.push_scope();
        self.bind_local(iter_stmt.binding.name, binding);
        let body = self.lower_ergo_body(&iter_stmt.body);
        self.pop_scope();
        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Itera(mode, binding, iter_stmt.binding.name, Box::new(iter), body),
            ty: None,
            span: self.current_span,
        };

        HirStmtKind::Expr(expr)
    }

    /// Lower redde (return) statement
    fn lower_redde(&mut self, ret: &crate::syntax::ReddeStmt) -> HirStmtKind {
        let value = ret.value.as_ref().map(|e| self.lower_expr(e));
        HirStmtKind::Redde(value)
    }

    fn lower_iace(&mut self, stmt: &crate::syntax::IaceStmt) -> HirStmtKind {
        let value = self.lower_expr(&stmt.value);
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Throw(Box::new(value)),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_mori(&mut self, stmt: &crate::syntax::MoriStmt) -> HirStmtKind {
        let value = self.lower_expr(&stmt.value);
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Panic(Box::new(value)),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_tempta(&mut self, stmt: &crate::syntax::TemptaStmt) -> HirStmtKind {
        let body = self.lower_block(&stmt.body);
        let catch = stmt
            .catch
            .as_ref()
            .map(|cape| self.lower_cape_clause_block(cape));
        let finally = stmt.finally.as_ref().map(|block| self.lower_block(block));
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Tempta { body, catch, finally },
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_adfirma(&mut self, stmt: &crate::syntax::AdfirmaStmt) -> HirStmtKind {
        let cond = self.lower_expr(&stmt.cond);
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Adfirma(
                Box::new(cond),
                stmt.message
                    .as_ref()
                    .map(|message| Box::new(self.lower_expr(message))),
            ),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_scribe(&mut self, stmt: &crate::syntax::ScribeStmt) -> HirStmtKind {
        let _ = stmt.kind;
        let args = stmt.args.iter().map(|arg| self.lower_expr(arg)).collect();
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Scribe(args),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_incipit_stmt(&mut self, stmt: &crate::syntax::IncipitStmt) -> HirStmtKind {
        let _ = stmt.is_async;
        self.push_scope();
        let mut args_binding = None;
        if let Some(args) = &stmt.args {
            let def_id = self.next_def_id();
            self.bind_local(args.name, def_id);
            args_binding = Some((args.name, args.span, def_id));
        }
        let mut block = self.lower_ergo_body(&stmt.body);
        if let Some((name, span, def_id)) = args_binding {
            block.stmts.insert(
                0,
                HirStmt {
                    id: self.next_hir_id(),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id,
                        name,
                        ty: Some(
                            self.types
                                .array(self.types.primitive(crate::semantic::Primitive::Textus)),
                        ),
                        init: None,
                        mutable: false,
                    }),
                    span,
                },
            );
        }
        self.pop_scope();
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(block),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_cura(&mut self, stmt: &crate::syntax::CuraStmt) -> HirStmtKind {
        let mut stmts = Vec::new();
        let has_binding = stmt.binding.name.0 != 0;
        let mut def_id = None;
        if has_binding {
            let local_def = self.next_def_id();
            let init = stmt.init.as_ref().map(|expr| self.lower_expr(expr));
            let ty = stmt.ty.as_ref().map(|ty| self.lower_type(ty)).or_else(|| {
                if stmt.kind.is_some() {
                    Some(self.types.primitive(crate::semantic::Primitive::Ignotum))
                } else {
                    None
                }
            });
            stmts.push(HirStmt {
                id: self.next_hir_id(),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id: local_def,
                    name: stmt.binding.name,
                    ty,
                    init,
                    mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
                }),
                span: self.current_span,
            });
            def_id = Some(local_def);
        }

        self.push_scope();
        if let Some(local_def) = def_id {
            self.bind_local(stmt.binding.name, local_def);
        }
        for lowered in self.lower_block(&stmt.body).stmts {
            stmts.push(lowered);
        }
        self.pop_scope();

        if let Some(catch) = &stmt.catch {
            self.lower_cape_clause_stmts(catch, &mut stmts);
        }

        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(HirBlock { stmts, expr: None, span: self.current_span }),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_ad(&mut self, stmt: &crate::syntax::AdStmt) -> HirStmtKind {
        let args = stmt
            .args
            .iter()
            .map(|arg| match &arg.value.kind {
                crate::syntax::ExprKind::Ident(ident) if self.lookup_name(ident.name).is_none() => HirExpr {
                    id: self.next_hir_id(),
                    kind: HirExprKind::Literal(crate::hir::HirLiteral::String(ident.name)),
                    ty: None,
                    span: ident.span,
                },
                _ => self.lower_expr(&arg.value),
            })
            .collect();
        let binding = stmt.binding.as_ref().map(|binding| crate::hir::HirAdBinding {
            verb: match binding.verb {
                crate::syntax::EndpointVerb::Fit => crate::hir::HirEndpointVerb::Fit,
                crate::syntax::EndpointVerb::Fiet => crate::hir::HirEndpointVerb::Fiet,
                crate::syntax::EndpointVerb::Fiunt => crate::hir::HirEndpointVerb::Fiunt,
                crate::syntax::EndpointVerb::Fient => crate::hir::HirEndpointVerb::Fient,
            },
            ty: binding.ty.as_ref().map(|ty| self.lower_type(ty)),
            name: binding.name.name,
            alias: binding.alias.as_ref().map(|alias| alias.name),
        });
        let body = stmt.body.as_ref().map(|body| self.lower_ad_body(body, stmt.binding.as_ref()));
        let catch = stmt.catch.as_ref().map(|catch| self.lower_cape_clause_block(catch));

        HirStmtKind::Ad(crate::hir::HirAd { path: stmt.path, args, binding, body, catch })
    }

    fn lower_ex(&mut self, _stmt: &crate::syntax::ExStmt) -> HirStmtKind {
        self.error("ex destructuring should be expanded before statement lowering");
        HirStmtKind::Expr(error_expr(self, self.current_span))
    }

    fn lower_custodi(&mut self, stmt: &crate::syntax::CustodiStmt) -> HirStmtKind {
        let mut clauses = stmt.clauses.iter();
        let Some(first) = clauses.next() else {
            return HirStmtKind::Expr(error_expr(self, self.current_span));
        };

        let mut expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Si(Box::new(self.lower_expr(&first.cond)), self.lower_ergo_body(&first.body), None),
            ty: None,
            span: first.span,
        };

        for clause in clauses {
            expr = HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Si(
                    Box::new(self.lower_expr(&clause.cond)),
                    self.lower_ergo_body(&clause.body),
                    Some(HirBlock { stmts: Vec::new(), expr: Some(Box::new(expr)), span: clause.span }),
                ),
                ty: None,
                span: clause.span,
            };
        }

        HirStmtKind::Expr(expr)
    }

    fn lower_fac(&mut self, stmt: &crate::syntax::FacStmt) -> HirStmtKind {
        let expr = self.lower_fac_expr(stmt);
        HirStmtKind::Expr(expr)
    }

    fn lower_elige(&mut self, elige_stmt: &crate::syntax::EligeStmt) -> HirStmtKind {
        let scrutinee = self.lower_expr(&elige_stmt.expr);
        let mut arms = Vec::new();

        for case in &elige_stmt.cases {
            let pattern = self.lower_elige_case_pattern(&case.value, case.span);

            let block = self.lower_ergo_body(&case.body);
            let body = self.block_expr(block, case.span);
            arms.push(HirCasuArm { patterns: vec![pattern], guard: None, body, span: case.span });
        }

        if let Some(default) = &elige_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm { patterns: vec![HirPattern::Wildcard], guard: None, body, span: default.span });
        }

        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Discerne(vec![scrutinee], arms),
            ty: None,
            span: self.current_span,
        };

        HirStmtKind::Expr(expr)
    }

    fn lower_elige_case_pattern(&mut self, value: &crate::syntax::Expr, span: crate::lexer::Span) -> HirPattern {
        match &value.kind {
            crate::syntax::ExprKind::Literal(lit) => pattern::lower_literal(self, lit, span),
            crate::syntax::ExprKind::Ident(ident) => {
                let Some(def_id) = self.lookup_name(ident.name) else {
                    self.current_span = span;
                    self.error("elige case value must be a literal or enum variant");
                    return HirPattern::Wildcard;
                };
                if matches!(
                    self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
                    Some(crate::semantic::SymbolKind::Variant)
                ) {
                    HirPattern::Variant(def_id, Vec::new())
                } else {
                    self.current_span = span;
                    self.error("elige case value must be a literal or enum variant");
                    HirPattern::Wildcard
                }
            }
            crate::syntax::ExprKind::Member(member) => {
                let Some(def_id) = self.lookup_name(member.member.name) else {
                    self.current_span = span;
                    self.error("elige case value must be a literal or enum variant");
                    return HirPattern::Wildcard;
                };
                if matches!(
                    self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
                    Some(crate::semantic::SymbolKind::Variant)
                ) {
                    HirPattern::Variant(def_id, Vec::new())
                } else {
                    self.current_span = span;
                    self.error("elige case value must be a literal or enum variant");
                    HirPattern::Wildcard
                }
            }
            _ => {
                self.current_span = span;
                self.error("elige case value must be a literal or enum variant");
                HirPattern::Wildcard
            }
        }
    }

    fn lower_discerne(&mut self, discerne_stmt: &crate::syntax::DiscerneStmt) -> HirStmtKind {
        let scrutinees = match discerne_stmt.subjects.as_slice() {
            [] => {
                self.error("discerne requires at least one subject");
                vec![error_expr(self, self.current_span)]
            }
            many => many.iter().map(|expr| self.lower_expr(expr)).collect(),
        };

        let mut arms = Vec::new();
        for arm in &discerne_stmt.arms {
            self.current_span = arm.span;
            self.push_scope();

            let patterns = match arm.patterns.as_slice() {
                [] => {
                    self.error("discerne casu requires a pattern");
                    vec![HirPattern::Wildcard]
                }
                many => {
                    let lowered: Vec<_> = many.iter().map(|pattern| pattern::lower_pattern(self, pattern)).collect();
                    if lowered.len() != scrutinees.len() {
                        self.error("discerne pattern count must match subject count");
                    }
                    lowered
                }
            };

            let block = self.lower_ergo_body(&arm.body);
            let body = self.block_expr(block, arm.span);
            arms.push(HirCasuArm { patterns, guard: None, body, span: arm.span });
            self.pop_scope();
        }

        if let Some(default) = &discerne_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm {
                patterns: (0..scrutinees.len().max(1)).map(|_| HirPattern::Wildcard).collect(),
                guard: None,
                body,
                span: default.span,
            });
        }

        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Discerne(scrutinees, arms),
            ty: None,
            span: self.current_span,
        };

        HirStmtKind::Expr(expr)
    }

    fn lower_si_expr(&mut self, if_stmt: &crate::syntax::SiStmt) -> HirExpr {
        let cond = self.lower_expr(&if_stmt.cond);
        let then_block = self.lower_ergo_body(&if_stmt.then);
        let else_block = if_stmt
            .else_
            .as_ref()
            .map(|secus| self.lower_secus_clause(secus));

        HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Si(Box::new(cond), then_block, else_block),
            ty: None,
            span: self.current_span,
        }
    }

    fn lower_secus_clause(&mut self, clause: &crate::syntax::SecusClause) -> HirBlock {
        match clause {
            crate::syntax::SecusClause::Sin(stmt) => {
                let expr = self.lower_si_expr(stmt);
                self.block_expr_block(expr)
            }
            crate::syntax::SecusClause::Block(block) => self.lower_block(block),
            crate::syntax::SecusClause::Stmt(stmt) => {
                HirBlock { stmts: lower_stmt_expanded(self, stmt), expr: None, span: stmt.span }
            }
            crate::syntax::SecusClause::InlineReturn(ret) => {
                let stmts = self.lower_inline_return(ret).into_iter().collect();
                HirBlock { stmts, expr: None, span: self.current_span }
            }
        }
    }

    fn block_expr(&mut self, block: HirBlock, span: Span) -> HirExpr {
        HirExpr { id: self.next_hir_id(), kind: HirExprKind::Block(block), ty: None, span }
    }

    fn block_expr_block(&mut self, expr: HirExpr) -> HirBlock {
        HirBlock { stmts: Vec::new(), expr: Some(Box::new(expr)), span: self.current_span }
    }

    fn lower_cape_clause_stmts(&mut self, catch: &crate::syntax::CapeClause, out: &mut Vec<HirStmt>) {
        let block = self.lower_cape_clause_block(catch);
        out.extend(block.stmts);
    }

    fn lower_cape_clause_block(&mut self, catch: &crate::syntax::CapeClause) -> HirBlock {
        self.push_scope();
        let catch_def_id = self.next_def_id();
        self.bind_local(catch.binding.name, catch_def_id);
        let mut stmts = Vec::new();
        stmts.push(HirStmt {
            id: self.next_hir_id(),
            kind: HirStmtKind::Local(crate::hir::HirLocal {
                def_id: catch_def_id,
                name: catch.binding.name,
                ty: Some(self.types.primitive(crate::semantic::Primitive::Ignotum)),
                init: None,
                mutable: false,
            }),
            span: catch.binding.span,
        });
        for lowered in self.lower_block(&catch.body).stmts {
            stmts.push(lowered);
        }
        self.pop_scope();
        HirBlock { stmts, expr: None, span: catch.span }
    }

    fn lower_ad_body(
        &mut self,
        body: &crate::syntax::BlockStmt,
        binding: Option<&crate::syntax::AdBinding>,
    ) -> HirBlock {
        self.push_scope();
        let mut binding_local = None;
        if let Some(binding) = binding {
            let def_id = self.next_def_id();
            self.bind_local(binding.name.name, def_id);
            if let Some(alias) = &binding.alias {
                let alias_def_id = self.next_def_id();
                self.bind_local(alias.name, alias_def_id);
            }
            binding_local = Some((def_id, binding));
        }
        let mut lowered_body = self.lower_block(body);
        if let Some((def_id, binding)) = binding_local {
            lowered_body.stmts.insert(
                0,
                HirStmt {
                    id: self.next_hir_id(),
                    kind: HirStmtKind::Local(crate::hir::HirLocal {
                        def_id,
                        name: binding.name.name,
                        ty: binding.ty.as_ref().map(|ty| self.lower_type(ty)),
                        init: None,
                        mutable: false,
                    }),
                    span: binding.name.span,
                },
            );
            if let Some(alias) = &binding.alias {
                let alias_def_id = self.lookup_name(alias.name).expect("ad alias binding should be in scope");
                lowered_body.stmts.insert(
                    1,
                    HirStmt {
                        id: self.next_hir_id(),
                        kind: HirStmtKind::Local(crate::hir::HirLocal {
                            def_id: alias_def_id,
                            name: alias.name,
                            ty: binding.ty.as_ref().map(|ty| self.lower_type(ty)),
                            init: None,
                            mutable: false,
                        }),
                        span: alias.span,
                    },
                );
            }
        }
        self.pop_scope();
        lowered_body
    }

    fn lower_fac_expr(&mut self, stmt: &crate::syntax::FacStmt) -> HirExpr {
        let block = self.lower_block(&stmt.body);
        let expr = if let Some(cond) = &stmt.while_ {
            let negate_cond = HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Unary(crate::hir::HirUnOp::Not, Box::new(self.lower_expr(cond))),
                ty: None,
                span: cond.span,
            };
            let break_stmt = HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Rumpe, span: cond.span };
            let loop_block = HirBlock {
                stmts: block
                    .stmts
                    .into_iter()
                    .chain(std::iter::once(HirStmt {
                        id: self.next_hir_id(),
                        kind: HirStmtKind::Expr(HirExpr {
                            id: self.next_hir_id(),
                            kind: HirExprKind::Si(
                                Box::new(negate_cond),
                                HirBlock { stmts: vec![break_stmt], expr: None, span: cond.span },
                                None,
                            ),
                            ty: None,
                            span: cond.span,
                        }),
                        span: cond.span,
                    }))
                    .collect(),
                expr: None,
                span: stmt.body.span,
            };
            HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Loop(loop_block),
                ty: None,
                span: self.current_span,
            }
        } else {
            HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Block(block),
                ty: None,
                span: self.current_span,
            }
        };

        if let Some(catch) = &stmt.catch {
            HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Tempta {
                    body: HirBlock { stmts: vec![HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Expr(expr), span: self.current_span }], expr: None, span: self.current_span },
                    catch: Some(self.lower_cape_clause_block(catch)),
                    finally: None,
                },
                ty: None,
                span: self.current_span,
            }
        } else {
            expr
        }
    }
}
