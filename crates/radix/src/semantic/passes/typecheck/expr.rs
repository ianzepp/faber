//! Expression synthesis and expected-type propagation for HIR type checking.
//!
//! This file is the central dispatcher for expression typing after name
//! resolution and HIR lowering have already chosen expression forms and
//! `DefId`s. Most expressions synthesize a type from their children, while a
//! smaller set accepts an expected type from declarations, arguments, closures,
//! or control-flow joins. That expected context is advisory until the final
//! `unify` at the bottom of `check_expr_with_expected`; helpers may use it
//! earlier for constructs such as `vacua`, array literals, blocks, branches,
//! and closures where syntax alone cannot recover the intended shape.
//!
//! ERROR STRATEGY
//! ==============
//! The checker keeps walking after local failures. Invalid lowered expressions
//! and missing contextual type information return the shared error type so later
//! phases can still attach diagnostics to neighboring code instead of losing the
//! rest of the file. `ignotum` remains an explicit escape type handled by the
//! specialized helpers; it is not used here as a silent replacement for missing
//! collection or nullability information.
//!
//! BOUNDARY
//! ========
//! This pass records resolved expression types on HIR nodes. It does not choose
//! target-specific lowering policy, library translation, or codegen fallbacks.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Type-checks an `ab` collection pipeline while preserving its source
    /// collection shape unless a transform explicitly changes the result.
    ///
    /// `ab` is mostly a dataflow expression: filters must be valid conditions
    /// and transform arguments must be checked, but target-specific pipeline
    /// lowering remains a backend concern. The semantic commitment made here is
    /// only the result type visible to surrounding Faber expressions.
    pub(super) fn check_ab(
        &mut self,
        source: &mut HirExpr,
        filter: Option<&mut crate::hir::HirCollectionFilter>,
        transforms: &mut [crate::hir::HirCollectionTransform],
    ) -> TypeId {
        let source_ty = self.check_expr(source);

        if let Some(filter) = filter {
            match &mut filter.kind {
                crate::hir::HirCollectionFilterKind::Condition(cond) => {
                    self.check_condition(cond);
                }
                crate::hir::HirCollectionFilterKind::Property(_name) => {}
            }
        }

        let mut has_sum = false;
        for transform in transforms {
            if let Some(arg) = &mut transform.arg {
                self.check_expr(arg);
            }
            if matches!(transform.kind, crate::hir::HirTransformKind::Sum) {
                has_sum = true;
            }
        }

        if has_sum {
            self.numerus_type()
        } else {
            source_ty
        }
    }
    /// Synthesizes an expression type, optionally using an expected type as
    /// context for shape-sensitive expressions.
    ///
    /// The expected type is not a target annotation and does not bypass local
    /// checking. It gives ambiguous forms enough information to type themselves,
    /// then the synthesized result is unified with that expectation before the
    /// node's final `expr.ty` is recorded. Recovery paths return `error_type`
    /// after emitting diagnostics so callers can continue checking sibling
    /// expressions with a stable sentinel.
    pub(super) fn check_expr_with_expected(&mut self, expr: &mut HirExpr, expected: Option<TypeId>) -> TypeId {
        let ty = match &mut expr.kind {
            HirExprKind::Path(def_id) => self.check_path(*def_id, expr.span),
            HirExprKind::Literal(lit) => self.literal_type(lit),
            HirExprKind::Vacua => self.check_vacua(expected, expr.span),
            HirExprKind::Binary(op, lhs, rhs) => self.check_binary(*op, lhs, rhs),
            HirExprKind::Unary(op, operand) => self.check_unary(*op, operand),
            HirExprKind::Call(callee, args) => self.check_call(callee, args),
            HirExprKind::MethodCall(receiver, name, args) => self.check_method_call(receiver, *name, args),
            HirExprKind::Field(object, name) => self.check_field(object, *name),
            HirExprKind::Index(object, index) => self.check_index(object, index),
            HirExprKind::OptionalChain(object, chain) => self.check_optional_chain(object, chain, expr.span),
            HirExprKind::NonNull(object, chain) => self.check_non_null(object, chain, expr.span),
            HirExprKind::Ab { source, filter, transforms } => self.check_ab(source, filter.as_mut(), transforms),
            HirExprKind::Block(block) => self.check_block(block, expected),
            HirExprKind::Si { cond, then_block, then_catch, else_block } => {
                self.check_if(cond, then_block, then_catch.as_deref_mut(), else_block.as_mut(), expected)
            }
            HirExprKind::Discerne(scrutinees, arms) => self.check_match(scrutinees, arms, expected),
            HirExprKind::Loop(block) => {
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Dum(cond, block) => {
                self.check_condition(cond);
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Itera(mode, binding, _, iter, block) => {
                let iter_ty = self.check_expr(iter);
                let elem_ty = match self.types.get(self.resolve_type(iter_ty)) {
                    Type::Array(inner) => match mode {
                        crate::hir::HirIteraMode::De => self.numerus_type(),
                        crate::hir::HirIteraMode::Ex | crate::hir::HirIteraMode::Pro => *inner,
                    },
                    Type::Map(key, value) => match mode {
                        crate::hir::HirIteraMode::Ex => *value,
                        crate::hir::HirIteraMode::De | crate::hir::HirIteraMode::Pro => *key,
                    },
                    Type::Union(items) if matches!(mode, crate::hir::HirIteraMode::Pro) && items.len() >= 2 => {
                        self.numerus_type()
                    }
                    _ => self.numerus_type(),
                };
                self.push_scope();
                self.insert_binding(*binding, elem_ty, true);
                self.check_block(block, None);
                self.pop_scope();
                self.vacuum_type()
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                let start_ty = self.check_expr(start);
                let end_ty = self.check_expr(end);
                if let Some(step) = step {
                    self.check_expr(step);
                }
                self.types.intern(Type::Union(vec![start_ty, end_ty]))
            }
            HirExprKind::Assign(target, value) => self.check_assign(target, value),
            HirExprKind::AssignOp(op, target, value) => self.check_assign_op(*op, target, value),
            HirExprKind::Array(elements) => self.check_array(elements, expr.span, expected),
            HirExprKind::Struct(def_id, fields) => self.check_struct_literal(*def_id, fields),
            HirExprKind::Verte { source, target, entries } => {
                self.check_verte(source, *target, entries.as_mut(), expr.span)
            }
            HirExprKind::Conversio { source, target, params: _, fallback } => {
                self.check_conversio(source, *target, fallback.as_deref_mut(), expr.span)
            }
            HirExprKind::Tuple(items) => self.check_tuple(items),
            HirExprKind::Scribe(_, items) => {
                for item in items {
                    self.check_expr(item);
                }
                self.vacuum_type()
            }
            HirExprKind::Scriptum(_template, args) => {
                for arg in args {
                    self.check_expr(arg);
                }
                self.textus_type()
            }
            HirExprKind::Adfirma(cond, message) => {
                let cond_ty = self.check_expr(cond);
                let bool_ty = self.bool_type();
                self.unify(cond_ty, bool_ty, cond.span, "assert condition must be boolean");
                if let Some(message) = message {
                    self.check_expr(message);
                }
                self.vacuum_type()
            }
            HirExprKind::Panic(value) => {
                self.check_expr(value);
                self.vacuum_type()
            }
            HirExprKind::Throw(value) => {
                let value_ty = self.check_expr(value);
                if let Some(err_ty) = self.current_error.map(|sink| match sink {
                    ErrorSink::Function(ty) | ErrorSink::Local(ty) => ty,
                }) {
                    self.unify(value_ty, err_ty, value.span, "alternate exit value type mismatch");
                } else {
                    self.error(
                        SemanticErrorKind::TypeMismatch,
                        "iace requires an enclosing function with a '⇥' alternate-exit type",
                        expr.span,
                    );
                }
                self.vacuum_type()
            }
            HirExprKind::Handled { body, catch } => self.check_handled_block(body, catch, expected),
            HirExprKind::Tempta { body, catch, finally } => {
                if catch.is_some() {
                    let prev_error = self.current_error;
                    self.current_error = Some(ErrorSink::Local(self.types.primitive(Primitive::Ignotum)));
                    self.check_block(body, None);
                    self.current_error = prev_error;
                } else {
                    self.check_block(body, None);
                }
                if let Some(catch) = catch {
                    self.check_block(catch, None);
                }
                if let Some(finally) = finally {
                    self.check_block(finally, None);
                }
                self.vacuum_type()
            }
            HirExprKind::Clausura(params, ret, body) => self.check_closure(params, ret.as_mut(), body, expected),
            HirExprKind::Cede(inner) => self.check_expr(inner),
            HirExprKind::Ref(kind, inner) => {
                let inner_ty = self.check_expr(inner);
                let mutability = match kind {
                    crate::hir::HirRefKind::Shared => crate::semantic::Mutability::Immutable,
                    crate::hir::HirRefKind::Mutable => crate::semantic::Mutability::Mutable,
                };
                self.types.reference(mutability, inner_ty)
            }
            HirExprKind::Deref(inner) => self.check_deref(inner, expr.span),
            HirExprKind::Error => {
                if self.errored_exprs.insert(expr.id) {
                    self.error(
                        SemanticErrorKind::LoweringError,
                        "invalid expression produced during lowering",
                        expr.span,
                    );
                }
                self.error_type
            }
        };

        let ty = if let Some(expected) = expected {
            self.unify(ty, expected, expr.span, "expression type mismatch")
        } else {
            ty
        };
        expr.ty = Some(self.resolve_type(ty));
        ty
    }

    /// Checks the empty collection literal.
    ///
    /// `vacua` is deliberately expected-context driven: without an explicit
    /// declared collection type the checker cannot know whether it is a list,
    /// map, or set, and codegen must not guess that shape later.
    fn check_vacua(&mut self, expected: Option<TypeId>, span: crate::lexer::Span) -> TypeId {
        let Some(expected) = expected else {
            self.error(
                SemanticErrorKind::MissingTypeAnnotation,
                "vacua requires an explicit declared type",
                span,
            );
            return self.error_type;
        };

        let resolved = self.resolve_type(expected);
        if self.is_infer(resolved) {
            self.error(
                SemanticErrorKind::MissingTypeAnnotation,
                "vacua requires an explicit declared type",
                span,
            );
            return self.error_type;
        }

        match self.types.get(resolved) {
            Type::Array(_) | Type::Map(_, _) | Type::Set(_) => resolved,
            _ => {
                self.error(SemanticErrorKind::InvalidOperandTypes, "vacua requires a collection type", span);
                self.error_type
            }
        }
    }

    /// Synthesizes an expression type without contextual expectations.
    ///
    /// Callers should prefer `check_expr_with_expected` when a declaration,
    /// function parameter, branch join, or enclosing aggregate already supplies
    /// a meaningful target shape.
    pub(super) fn check_expr(&mut self, expr: &mut HirExpr) -> TypeId {
        self.check_expr_with_expected(expr, None)
    }
}
