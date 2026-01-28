//! Statement lowering
//!
//! Lowers AST statements to HIR statements.

use super::{pattern, HirBlock, HirExpr, HirExprKind, HirStmt, HirStmtKind, Lowerer};
use crate::hir::{HirCasuArm, HirPattern};
use crate::lexer::Span;
use crate::syntax::{Stmt, StmtKind};

fn error_expr(lowerer: &mut Lowerer, span: Span) -> HirExpr {
    HirExpr {
        id: lowerer.next_hir_id(),
        kind: HirExprKind::Error,
        ty: None,
        span,
    }
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
        StmtKind::Redde(ret) => lowerer.lower_redde(ret),
        StmtKind::Rumpe(_) => HirStmtKind::Rumpe,
        StmtKind::Perge(_) => HirStmtKind::Perge,
        StmtKind::Elige(elige_stmt) => lowerer.lower_elige(elige_stmt),
        StmtKind::Discerne(discerne_stmt) => lowerer.lower_discerne(discerne_stmt),
        StmtKind::Block(block) => {
            let block = lowerer.lower_block(block);
            HirStmtKind::Expr(HirExpr {
                id: lowerer.next_hir_id(),
                kind: HirExprKind::Block(block),
                ty: None,
                span,
            })
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
        self.error("variable declaration lowering not implemented");
        HirStmtKind::Expr(error_expr(self, self.current_span))
    }

    /// Lower expression statement
    fn lower_expr_stmt(&mut self, expr: &crate::syntax::ExprStmt) -> HirStmtKind {
        let expr_hir = self.lower_expr(&expr.expr);
        HirStmtKind::Expr(expr_hir)
    }

    /// Lower si (if) statement
    fn lower_si(&mut self, if_stmt: &crate::syntax::SiStmt) -> HirStmtKind {
        if if_stmt.catch.is_some() {
            self.error("si catch clause lowering not implemented");
        }

        let expr = self.lower_si_expr(if_stmt);
        HirStmtKind::Expr(expr)
    }

    /// Lower dum (while) statement
    fn lower_dum(&mut self, while_stmt: &crate::syntax::DumStmt) -> HirStmtKind {
        if while_stmt.catch.is_some() {
            self.error("dum catch clause lowering not implemented");
        }

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
        if iter_stmt.catch.is_some() {
            self.error("itera catch clause lowering not implemented");
        }

        if !matches!(iter_stmt.mode, crate::syntax::IteraMode::Ex) {
            self.error("itera mode lowering not implemented");
        }

        let binding = self.def_id_for(iter_stmt.binding.name);
        let iter = self.lower_expr(&iter_stmt.iterable);
        let body = self.lower_ergo_body(&iter_stmt.body);
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

    fn lower_elige(&mut self, elige_stmt: &crate::syntax::EligeStmt) -> HirStmtKind {
        if elige_stmt.catch.is_some() {
            self.error("elige catch clause lowering not implemented");
        }

        let scrutinee = self.lower_expr(&elige_stmt.expr);
        let mut arms = Vec::new();

        for case in &elige_stmt.cases {
            let pattern = match &case.value.kind {
                crate::syntax::ExprKind::Literal(lit) => {
                    pattern::lower_literal(self, lit, case.span)
                }
                _ => {
                    self.current_span = case.span;
                    self.error("elige case value must be a literal");
                    HirPattern::Wildcard
                }
            };

            let block = self.lower_ergo_body(&case.body);
            let body = self.block_expr(block, case.span);
            arms.push(HirCasuArm {
                pattern,
                guard: None,
                body,
                span: case.span,
            });
        }

        if let Some(default) = &elige_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm {
                pattern: HirPattern::Wildcard,
                guard: None,
                body,
                span: default.span,
            });
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
                HirExpr {
                    id: self.next_hir_id(),
                    kind: HirExprKind::Tuple(items),
                    ty: None,
                    span: self.current_span,
                }
            }
        };

        let mut arms = Vec::new();
        for arm in &discerne_stmt.arms {
            self.current_span = arm.span;

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
            arms.push(HirCasuArm {
                pattern,
                guard: None,
                body,
                span: arm.span,
            });
        }

        if let Some(default) = &discerne_stmt.default {
            let block = self.lower_ergo_body(&default.body);
            let body = self.block_expr(block, default.span);
            arms.push(HirCasuArm {
                pattern: HirPattern::Wildcard,
                guard: None,
                body,
                span: default.span,
            });
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
            crate::syntax::SecusClause::Stmt(stmt) => HirBlock {
                stmts: vec![self.lower_stmt(stmt)],
                expr: None,
                span: stmt.span,
            },
            crate::syntax::SecusClause::InlineReturn(ret) => {
                let stmt = self.lower_inline_return(ret);
                HirBlock {
                    stmts: vec![stmt],
                    expr: None,
                    span: self.current_span,
                }
            }
        }
    }

    fn block_expr(&mut self, block: HirBlock, span: Span) -> HirExpr {
        HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Block(block),
            ty: None,
            span,
        }
    }

    fn block_expr_block(&mut self, expr: HirExpr) -> HirBlock {
        HirBlock {
            stmts: Vec::new(),
            expr: Some(Box::new(expr)),
            span: self.current_span,
        }
    }
}
