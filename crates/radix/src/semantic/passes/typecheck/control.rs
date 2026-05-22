use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_cape(&mut self, catch: &mut HirCape, err_ty: TypeId) {
        let err_ty = match self.resolve_type(err_ty) {
            resolved if self.is_infer(resolved) => self.types.primitive(Primitive::Ignotum),
            resolved => resolved,
        };
        catch.binding_ty = Some(err_ty);

        self.push_scope();
        self.insert_binding(catch.binding_def_id, err_ty, false);
        self.check_block(&mut catch.body, None);
        self.pop_scope();
    }

    pub(super) fn check_handled_block(
        &mut self,
        body: &mut HirBlock,
        catch: &mut HirCape,
        expected: Option<TypeId>,
    ) -> TypeId {
        let err_ty = self.fresh_infer();
        let prev_error = self.current_error;
        self.current_error = Some(ErrorSink::Local(err_ty));
        let body_ty = self.check_block(body, expected);
        self.current_error = prev_error;

        self.check_cape(catch, err_ty);
        body_ty
    }

    pub(super) fn check_match(
        &mut self,
        scrutinees: &mut [HirExpr],
        arms: &mut [HirCasuArm],
        expected: Option<TypeId>,
    ) -> TypeId {
        let scrutinee_tys: Vec<_> = scrutinees
            .iter_mut()
            .map(|scrutinee| self.check_expr(scrutinee))
            .collect();
        let mut result_ty = None;

        for arm in arms {
            self.push_scope();
            for (pattern, scrutinee_ty) in arm.patterns.iter().zip(scrutinee_tys.iter().copied()) {
                self.check_pattern(pattern, scrutinee_ty, arm.span);
            }
            if let Some(guard) = &mut arm.guard {
                self.check_condition(guard);
            }
            let body_ty = self.check_expr_with_expected(&mut arm.body, expected);
            result_ty = Some(match result_ty {
                None => body_ty,
                Some(existing) => self.common_type(existing, body_ty, arm.span),
            });
            self.pop_scope();
        }

        result_ty.unwrap_or_else(|| self.vacuum_type())
    }
    pub(super) fn check_condition(&mut self, cond: &mut HirExpr) {
        let bivalens = self.bool_type();
        let cond_ty = self.check_expr_with_expected(cond, Some(bivalens));
        if !self.is_bool(cond_ty) {
            self.error(SemanticErrorKind::InvalidOperandTypes, "condition must be bivalens", cond.span);
        }
    }
    pub(super) fn check_if(
        &mut self,
        cond: &mut HirExpr,
        then_block: &mut HirBlock,
        then_catch: Option<&mut HirCape>,
        else_block: Option<&mut HirBlock>,
        expected: Option<TypeId>,
    ) -> TypeId {
        let then_ty = if let Some(catch) = then_catch {
            let err_ty = self.fresh_infer();
            let prev_error = self.current_error;
            self.current_error = Some(ErrorSink::Local(err_ty));
            self.check_condition(cond);
            let then_ty = self.check_block(then_block, expected);
            self.current_error = prev_error;
            self.check_cape(catch, err_ty);
            then_ty
        } else {
            self.check_condition(cond);
            self.check_block(then_block, expected)
        };
        let else_ty = else_block
            .map(|block| self.check_block(block, expected))
            .unwrap_or_else(|| self.vacuum_type());

        self.common_type(then_ty, else_ty, cond.span)
    }
}
