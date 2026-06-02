//! Statement lowering into MIR block bodies and terminators.
//!
//! Statements are where expression lowering meets CFG construction. This file
//! owns the HIR visitor hooks that keep the current MIR block well-formed:
//! ordinary statements append work to the open block, `redde` seals it with a
//! return terminator, and loop-control statements delegate to the active loop
//! context. Once a terminator is emitted, later HIR in the same block is treated
//! as a lowering error instead of being appended after a sealed block.
//!
//! INVARIANTS
//! ==========
//! - Every emitted MIR statement belongs to an open block.
//! - Statement-level unsupported forms become diagnostics, not partial MIR.
//! - Assignment expressions are routed through place-aware lowering so the LHS
//!   remains addressable instead of becoming a value-only operand.

use super::*;
use crate::hir::visit::HirVisitor;

impl HirVisitor for FunctionBuilder<'_> {
    /// Lower a HIR block in source order until a MIR terminator seals the
    /// current block.
    fn visit_block(&mut self, block: &HirBlock) {
        for stmt in &block.stmts {
            if !self.current_is_open() {
                break;
            }
            self.visit_stmt(stmt);
        }

        if let Some(expr) = &block.expr {
            if self.current_is_open() {
                self.visit_expr(expr);
            }
        }
    }

    /// Lower one HIR statement into the current MIR block.
    ///
    /// The visitor only accepts statement forms whose MIR contracts are already
    /// represented. Provider blocks and `tacet` still fail closed here because
    /// they require later effect or statement-level lowering policy.
    fn visit_stmt(&mut self, stmt: &HirStmt) {
        if !self.current_is_open() {
            self.errors
                .push(MirError::unsupported(stmt.span, "statement after a MIR terminator"));
            return;
        }

        match &stmt.kind {
            HirStmtKind::Local(local) => self.lower_local(local, stmt.span),
            HirStmtKind::Expr(expr) => self.visit_expr(expr),
            HirStmtKind::Redde(Some(expr)) => {
                if let Some(value) = self.lower_return_expr(expr) {
                    self.terminate_current(MirTerminatorKind::Return(Some(value)), stmt.span);
                }
            }
            HirStmtKind::Redde(None) => {
                self.terminate_current(MirTerminatorKind::Return(None), stmt.span);
            }
            HirStmtKind::Ad(_) => self.errors.push(MirError::unsupported(
                stmt.span,
                "ad provider blocks before effectful MIR lowering",
            )),
            HirStmtKind::Rumpe => self.lower_rumpe(stmt.span),
            HirStmtKind::Perge => self.lower_perge(stmt.span),
            HirStmtKind::Tacet => self
                .errors
                .push(MirError::unsupported(stmt.span, "tacet before statement-level MIR lowering")),
        }
    }

    /// Lower a value-position expression used as a statement.
    ///
    /// Assignments are special because their left-hand side must remain a
    /// `MirPlace`; every other expression can be lowered for its value and then
    /// discarded if the caller does not need the operand.
    fn visit_expr(&mut self, expr: &HirExpr) {
        match expr.kind {
            HirExprKind::Assign(_, _) => {
                self.lower_assignment_expr(expr);
            }
            HirExprKind::AssignOp(_, _, _) => {
                self.lower_assign_op_expr(expr);
            }
            _ => {
                let _ = self.lower_expr_value(expr);
            }
        }
    }
}
