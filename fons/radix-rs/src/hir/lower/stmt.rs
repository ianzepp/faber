//! Statement lowering
//!
//! Lowers AST statements to HIR statements.

use super::{HirBlock, HirExpr, HirExprKind, HirStmt, HirStmtKind, Lowerer};
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
        self.error("if statement lowering not implemented");
        HirStmtKind::Expr(error_expr(self, self.current_span))
    }

    /// Lower dum (while) statement
    fn lower_dum(&mut self, while_stmt: &crate::syntax::DumStmt) -> HirStmtKind {
        self.error("while statement lowering not implemented");
        HirStmtKind::Expr(error_expr(self, self.current_span))
    }

    /// Lower itera (for) statement
    fn lower_itera(&mut self, iter_stmt: &crate::syntax::IteraStmt) -> HirStmtKind {
        self.error("for statement lowering not implemented");
        HirStmtKind::Expr(error_expr(self, self.current_span))
    }

    /// Lower redde (return) statement
    fn lower_redde(&mut self, ret: &crate::syntax::ReddeStmt) -> HirStmtKind {
        let value = ret.value.as_ref().map(|e| self.lower_expr(e));
        HirStmtKind::Redde(value)
    }
}
