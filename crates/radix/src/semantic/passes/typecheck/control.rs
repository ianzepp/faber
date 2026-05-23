//! Typechecking for control expressions whose value type depends on nested
//! scopes, catches, or multiple arms.
//!
//! Control forms are where Faber's ordinary expression typing meets alternate
//! exits and pattern-sensitive scopes. This module keeps those concerns local:
//! handled blocks create a temporary error sink for `iace`, catches bind the
//! resulting error value, `si` merges branch result types, and `discerne`
//! checks pattern bindings before typing each arm body.
//!
//! INVARIANTS
//! ==========
//! - Catch bindings are immutable locals scoped only to the catch body.
//! - A handled body's alternate-exit type is inferred independently from its
//!   normal result type.
//! - Conditions must be `bivalens`; expected expression type propagation does
//!   not make non-boolean conditions acceptable.
//! - Match arms are checked in isolated scopes so pattern bindings never leak
//!   between arms.
//! - Empty matches type to `vacuum`; non-empty matches use the common type of
//!   arm bodies.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Check a catch clause for a handled alternate exit.
    ///
    /// If no `iace` expression constrained the local error inference variable,
    /// the catch still receives a usable binding typed as `ignotum`. That keeps
    /// error-channel handling explicit without forcing unreachable catches to
    /// manufacture a concrete error type.
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

    /// Check a block with an attached catch and return the block's normal type.
    ///
    /// The temporary `ErrorSink::Local` captures the type of values thrown by
    /// `iace` inside the body. The catch consumes that alternate-exit type, while
    /// the expression as a whole keeps the body's normal result type.
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

    /// Check a `discerne` expression and merge arm result types.
    ///
    /// Scrutinees are typed once, then each arm receives a fresh scope for its
    /// pattern bindings and guard. Pattern checking binds names to the matched
    /// scrutinee/field types, while the arm body is checked against the caller's
    /// expected expression type so match expressions participate in
    /// bidirectional inference.
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

    /// Check a condition expression under the language's boolean requirement.
    ///
    /// The expected type helps shape unresolved expressions, but the final
    /// predicate still checks the resolved result so permissive inference cannot
    /// silently accept a non-`bivalens` condition.
    pub(super) fn check_condition(&mut self, cond: &mut HirExpr) {
        let bivalens = self.bool_type();
        let cond_ty = self.check_expr_with_expected(cond, Some(bivalens));
        if !self.is_bool(cond_ty) {
            self.error(SemanticErrorKind::InvalidOperandTypes, "condition must be bivalens", cond.span);
        }
    }

    /// Check an `si` expression and compute its normal result type.
    ///
    /// A catch attached to the then-branch captures only alternate exits from
    /// that branch. The final `si` type is still the common type of the normal
    /// then and else results, with a missing else branch contributing `vacuum`.
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
