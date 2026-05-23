use super::*;

impl HirVisitor for FunctionBuilder<'_> {
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

    fn visit_expr(&mut self, expr: &HirExpr) {
        if matches!(expr.kind, HirExprKind::Assign(_, _)) {
            self.lower_assignment_expr(expr);
        } else {
            let _ = self.lower_expr_value(expr);
        }
    }
}
