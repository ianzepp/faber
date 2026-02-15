//! Statement lowering
//!
//! Lowers AST statements to HIR statements.

use super::{pattern, HirBlock, HirExpr, HirExprKind, HirStmt, HirStmtKind, Lowerer};
use crate::hir::{HirCasuArm, HirPattern};
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

impl<'a> Lowerer<'a> {
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
                let mut names = Vec::new();
                Self::collect_binding_names(&decl.binding, &mut names);
                let mut stmts = Vec::new();

                for (name, name_span) in names {
                    let def_id = self.next_def_id();
                    self.bind_local(name, def_id);
                    stmts.push(HirStmt {
                        id: self.next_hir_id(),
                        kind: HirStmtKind::Local(crate::hir::HirLocal {
                            def_id,
                            name,
                            ty: None,
                            init: Some(HirExpr {
                                id: self.next_hir_id(),
                                kind: HirExprKind::Error,
                                ty: None,
                                span: name_span,
                            }),
                            mutable: decl.mutability == crate::syntax::Mutability::Mutable,
                        }),
                        span: name_span,
                    });
                }

                if let Some(init) = &decl.init {
                    stmts.push(HirStmt {
                        id: self.next_hir_id(),
                        kind: HirStmtKind::Expr(self.lower_expr(init)),
                        span: init.span,
                    });
                }

                HirStmtKind::Expr(HirExpr {
                    id: self.next_hir_id(),
                    kind: HirExprKind::Block(HirBlock { stmts, expr: None, span: *span }),
                    ty: None,
                    span: *span,
                })
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
        let _ = iter_stmt.mode;

        let binding = self.next_def_id();
        let iter = self.lower_expr(&iter_stmt.iterable);
        self.push_scope();
        self.bind_local(iter_stmt.binding.name, binding);
        let body = self.lower_ergo_body(&iter_stmt.body);
        self.pop_scope();
        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Itera(binding, Box::new(iter), body),
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
        HirStmtKind::Expr(value)
    }

    fn lower_mori(&mut self, stmt: &crate::syntax::MoriStmt) -> HirStmtKind {
        let value = self.lower_expr(&stmt.value);
        HirStmtKind::Expr(value)
    }

    fn lower_tempta(&mut self, stmt: &crate::syntax::TemptaStmt) -> HirStmtKind {
        let mut stmts = Vec::new();
        for lowered in self.lower_block(&stmt.body).stmts {
            stmts.push(lowered);
        }
        if let Some(catch) = &stmt.catch {
            self.push_scope();
            let catch_def_id = self.next_def_id();
            self.bind_local(catch.binding.name, catch_def_id);
            stmts.push(HirStmt {
                id: self.next_hir_id(),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id: catch_def_id,
                    name: catch.binding.name,
                    ty: None,
                    init: Some(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Error,
                        ty: None,
                        span: catch.binding.span,
                    }),
                    mutable: false,
                }),
                span: catch.binding.span,
            });
            for lowered in self.lower_block(&catch.body).stmts {
                stmts.push(lowered);
            }
            self.pop_scope();
        }
        if let Some(finally) = &stmt.finally {
            for lowered in self.lower_block(finally).stmts {
                stmts.push(lowered);
            }
        }
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(HirBlock { stmts, expr: None, span: self.current_span }),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_adfirma(&mut self, stmt: &crate::syntax::AdfirmaStmt) -> HirStmtKind {
        let cond = self.lower_expr(&stmt.cond);
        // STUB: lowered as tuple placeholder [condition, message?]; needs dedicated assert HIR node.
        let mut items = vec![cond];
        if let Some(message) = &stmt.message {
            items.push(self.lower_expr(message));
        }
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Tuple(items),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_scribe(&mut self, stmt: &crate::syntax::ScribeStmt) -> HirStmtKind {
        let _ = stmt.kind;
        let args = stmt.args.iter().map(|arg| self.lower_expr(arg)).collect();
        // STUB: lowered as tuple placeholder; needs dedicated print/log HIR node.
        HirStmtKind::Expr(HirExpr { id: self.next_hir_id(), kind: HirExprKind::Tuple(args), ty: None, span: self.current_span })
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
                        ty: None,
                        init: Some(HirExpr { id: self.next_hir_id(), kind: HirExprKind::Error, ty: None, span }),
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
        let def_id = self.next_def_id();
        let init = stmt.init.as_ref().map(|expr| self.lower_expr(expr));
        stmts.push(HirStmt {
            id: self.next_hir_id(),
            kind: HirStmtKind::Local(crate::hir::HirLocal {
                def_id,
                name: stmt.binding.name,
                ty: stmt.ty.as_ref().map(|ty| self.lower_type(ty)),
                init,
                mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
            }),
            span: self.current_span,
        });

        self.push_scope();
        self.bind_local(stmt.binding.name, def_id);
        for lowered in self.lower_block(&stmt.body).stmts {
            stmts.push(lowered);
        }
        self.pop_scope();

        if let Some(catch) = &stmt.catch {
            self.push_scope();
            let catch_def_id = self.next_def_id();
            self.bind_local(catch.binding.name, catch_def_id);
            stmts.push(HirStmt {
                id: self.next_hir_id(),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id: catch_def_id,
                    name: catch.binding.name,
                    ty: None,
                    init: Some(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Error,
                        ty: None,
                        span: catch.binding.span,
                    }),
                    mutable: false,
                }),
                span: catch.binding.span,
            });
            for lowered in self.lower_block(&catch.body).stmts {
                stmts.push(lowered);
            }
            self.pop_scope();
        }

        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(HirBlock { stmts, expr: None, span: self.current_span }),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_ad(&mut self, stmt: &crate::syntax::AdStmt) -> HirStmtKind {
        // STUB: lowered as tuple placeholder [args..., body?]; needs dedicated annotation/directive HIR node.
        let mut items: Vec<HirExpr> = stmt.args.iter().map(|arg| self.lower_expr(&arg.value)).collect();
        if let Some(body) = &stmt.body {
            items.push(HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Block(self.lower_block(body)),
                ty: None,
                span: self.current_span,
            });
        }
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Tuple(items),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_ex(&mut self, stmt: &crate::syntax::ExStmt) -> HirStmtKind {
        let mut stmts = Vec::new();
        let source = self.lower_expr(&stmt.source);

        for field in &stmt.fields {
            let name = field.alias.as_ref().map(|ident| ident.name).unwrap_or(field.name.name);
            let def_id = self.next_def_id();
            self.bind_local(name, def_id);
            let local = crate::hir::HirLocal {
                def_id,
                name,
                ty: None,
                init: Some(HirExpr { id: self.next_hir_id(), kind: HirExprKind::Error, ty: None, span: field.name.span }),
                mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
            };
            stmts.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Local(local), span: field.name.span });
        }

        if let Some(rest) = &stmt.rest {
            let def_id = self.next_def_id();
            self.bind_local(rest.name, def_id);
            let local = crate::hir::HirLocal {
                def_id,
                name: rest.name,
                ty: None,
                init: Some(HirExpr { id: self.next_hir_id(), kind: HirExprKind::Error, ty: None, span: rest.span }),
                mutable: stmt.mutability == crate::syntax::Mutability::Mutable,
            };
            stmts.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Local(local), span: rest.span });
        }

        stmts.push(HirStmt { id: self.next_hir_id(), kind: HirStmtKind::Expr(source), span: self.current_span });
        HirStmtKind::Expr(HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(HirBlock { stmts, expr: None, span: self.current_span }),
            ty: None,
            span: self.current_span,
        })
    }

    fn lower_custodi(&mut self, stmt: &crate::syntax::CustodiStmt) -> HirStmtKind {
        let mut clauses = stmt.clauses.iter();
        let Some(first) = clauses.next() else {
            return HirStmtKind::Expr(error_expr(self, self.current_span));
        };

        let mut expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Si(
                Box::new(self.lower_expr(&first.cond)),
                self.lower_ergo_body(&first.body),
                None,
            ),
            ty: None,
            span: first.span,
        };

        for clause in clauses {
            expr = HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Si(
                    Box::new(self.lower_expr(&clause.cond)),
                    self.lower_ergo_body(&clause.body),
                    Some(HirBlock {
                        stmts: Vec::new(),
                        expr: Some(Box::new(expr)),
                        span: clause.span,
                    }),
                ),
                ty: None,
                span: clause.span,
            };
        }

        HirStmtKind::Expr(expr)
    }

    fn lower_fac(&mut self, stmt: &crate::syntax::FacStmt) -> HirStmtKind {
        let mut block = self.lower_block(&stmt.body);
        if let Some(cond) = &stmt.while_ {
            block.expr = Some(Box::new(HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Dum(Box::new(self.lower_expr(cond)), HirBlock { stmts: Vec::new(), expr: None, span: cond.span }),
                ty: None,
                span: cond.span,
            }));
        }
        if let Some(catch) = &stmt.catch {
            self.push_scope();
            let catch_def_id = self.next_def_id();
            self.bind_local(catch.binding.name, catch_def_id);
            block.stmts.push(HirStmt {
                id: self.next_hir_id(),
                kind: HirStmtKind::Local(crate::hir::HirLocal {
                    def_id: catch_def_id,
                    name: catch.binding.name,
                    ty: None,
                    init: Some(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Error,
                        ty: None,
                        span: catch.binding.span,
                    }),
                    mutable: false,
                }),
                span: catch.binding.span,
            });
            for lowered in self.lower_block(&catch.body).stmts {
                block.stmts.push(lowered);
            }
            self.pop_scope();
        }
        HirStmtKind::Expr(HirExpr { id: self.next_hir_id(), kind: HirExprKind::Block(block), ty: None, span: self.current_span })
    }

    fn lower_elige(&mut self, elige_stmt: &crate::syntax::EligeStmt) -> HirStmtKind {
        let scrutinee = self.lower_expr(&elige_stmt.expr);
        let mut arms = Vec::new();

        for case in &elige_stmt.cases {
            let pattern = match &case.value.kind {
                crate::syntax::ExprKind::Literal(lit) => pattern::lower_literal(self, lit, case.span),
                _ => {
                    self.current_span = case.span;
                    self.error("elige case value must be a literal");
                    HirPattern::Wildcard
                }
            };

            let block = self.lower_ergo_body(&case.body);
            let body = self.block_expr(block, case.span);
            arms.push(HirCasuArm { pattern, guard: None, body, span: case.span });
        }

        if let Some(default) = &elige_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm { pattern: HirPattern::Wildcard, guard: None, body, span: default.span });
        }

        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Discerne(Box::new(scrutinee), arms),
            ty: None,
            span: self.current_span,
        };

        HirStmtKind::Expr(expr)
    }

    fn lower_discerne(&mut self, discerne_stmt: &crate::syntax::DiscerneStmt) -> HirStmtKind {
        let scrutinee = match discerne_stmt.subjects.as_slice() {
            [one] => self.lower_expr(one),
            [] => {
                self.error("discerne requires at least one subject");
                error_expr(self, self.current_span)
            }
            many => {
                self.error("discerne with multiple subjects lowered as tuple");
                let items = many.iter().map(|expr| self.lower_expr(expr)).collect();
                HirExpr { id: self.next_hir_id(), kind: HirExprKind::Tuple(items), ty: None, span: self.current_span }
            }
        };

        let mut arms = Vec::new();
        for arm in &discerne_stmt.arms {
            self.current_span = arm.span;
            self.push_scope();

            let pattern = match arm.patterns.as_slice() {
                [one] => pattern::lower_pattern(self, one),
                [] => {
                    self.error("discerne casu requires a pattern");
                    HirPattern::Wildcard
                }
                _ => {
                    self.error("multiple patterns in casu are not lowered yet");
                    HirPattern::Wildcard
                }
            };

            let block = self.lower_ergo_body(&arm.body);
            let body = self.block_expr(block, arm.span);
            arms.push(HirCasuArm { pattern, guard: None, body, span: arm.span });
            self.pop_scope();
        }

        if let Some(default) = &discerne_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm { pattern: HirPattern::Wildcard, guard: None, body, span: default.span });
        }

        let expr = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Discerne(Box::new(scrutinee), arms),
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
                HirBlock { stmts: vec![self.lower_stmt(stmt)], expr: None, span: stmt.span }
            }
            crate::syntax::SecusClause::InlineReturn(ret) => {
                let stmt = self.lower_inline_return(ret);
                HirBlock { stmts: vec![stmt], expr: None, span: self.current_span }
            }
        }
    }

    fn collect_binding_names(
        pattern: &crate::syntax::BindingPattern,
        out: &mut Vec<(crate::lexer::Symbol, crate::lexer::Span)>,
    ) {
        match pattern {
            crate::syntax::BindingPattern::Ident(ident) => out.push((ident.name, ident.span)),
            crate::syntax::BindingPattern::Wildcard(_) => {}
            crate::syntax::BindingPattern::Array { elements, rest, .. } => {
                for element in elements {
                    Self::collect_binding_names(element, out);
                }
                if let Some(rest) = rest {
                    out.push((rest.name, rest.span));
                }
            }
        }
    }

    fn block_expr(&mut self, block: HirBlock, span: Span) -> HirExpr {
        HirExpr { id: self.next_hir_id(), kind: HirExprKind::Block(block), ty: None, span }
    }

    fn block_expr_block(&mut self, expr: HirExpr) -> HirBlock {
        HirBlock { stmts: Vec::new(), expr: Some(Box::new(expr)), span: self.current_span }
    }
}
