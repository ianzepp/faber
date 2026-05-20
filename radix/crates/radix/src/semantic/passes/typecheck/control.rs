use super::*;

impl<'a> TypeChecker<'a> {
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
        else_block: Option<&mut HirBlock>,
        expected: Option<TypeId>,
    ) -> TypeId {
        self.check_condition(cond);
        let then_ty = self.check_block(then_block, expected);
        let else_ty = else_block
            .map(|block| self.check_block(block, expected))
            .unwrap_or_else(|| self.vacuum_type());

        self.common_type(then_ty, else_ty, cond.span)
    }
}
