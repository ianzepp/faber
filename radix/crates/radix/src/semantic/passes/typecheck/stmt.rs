use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_return(&mut self, value: Option<&mut HirExpr>, span: crate::lexer::Span) {
        let value_ty = match value {
            Some(expr) => {
                if let Some(expected) = self.current_return {
                    self.check_expr_with_expected(expr, Some(expected))
                } else {
                    self.check_expr(expr)
                }
            }
            None => self.vacuum_type(),
        };

        if let Some(expected) = self.current_return {
            self.unify(value_ty, expected, span, "return type does not match function signature");
            return;
        }

        match self.inferred_return {
            None => self.inferred_return = Some(value_ty),
            Some(existing) => {
                self.unify(value_ty, existing, span, "incompatible return types");
            }
        }
    }
    pub(super) fn check_local(&mut self, local: &mut HirLocal) {
        let inferred = match (&local.ty, &mut local.init) {
            (Some(ty), Some(init)) => {
                let init_ty = self.check_expr_with_expected(init, Some(*ty));
                self.unify(init_ty, *ty, init.span, "initializer does not match annotation");
                *ty
            }
            (Some(ty), None) => *ty,
            (None, Some(init)) => self.check_expr(init),
            (None, None) => self.fresh_infer(),
        };

        if local.ty.is_none() {
            local.ty = Some(inferred);
        }

        self.insert_binding(local.def_id, inferred, local.mutable);
    }
    pub(super) fn check_stmt(&mut self, stmt: &mut HirStmt) {
        match &mut stmt.kind {
            HirStmtKind::Local(local) => self.check_local(local),
            HirStmtKind::Expr(expr) => {
                let expr_ty = self.check_expr(expr);
                if self.is_infer(self.resolve_type(expr_ty)) {
                    let vacuum = self.vacuum_type();
                    self.unify(expr_ty, vacuum, expr.span, "ignored expression result must resolve");
                }
            }
            HirStmtKind::Ad(ad) => {
                for arg in &mut ad.args {
                    self.check_expr(arg);
                }
                if let Some(body) = &mut ad.body {
                    self.check_block(body, None);
                }
                if let Some(catch) = &mut ad.catch {
                    self.check_block(catch, None);
                }
            }
            HirStmtKind::Redde(value) => self.check_return(value.as_mut(), stmt.span),
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }
    pub(super) fn check_block(&mut self, block: &mut HirBlock, expected: Option<TypeId>) -> TypeId {
        self.push_scope();
        for stmt in &mut block.stmts {
            self.check_stmt(stmt);
        }
        let ty = if let Some(expr) = &mut block.expr {
            self.check_expr_with_expected(expr, expected)
        } else {
            self.vacuum_type()
        };
        self.pop_scope();
        ty
    }
}
