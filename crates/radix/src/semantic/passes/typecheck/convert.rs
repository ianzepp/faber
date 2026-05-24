//! Conversion, cast, and reference-shape checks.
//!
//! This file owns expression forms that intentionally change the apparent type
//! of a value: dereference, runtime conversion (`conversio`), and typed
//! conversion/constructor syntax (`verte`). The pass validates only the semantic
//! contracts it can prove from HIR and the `TypeTable`; representation details,
//! fallback coercions, and target-specific conversion code remain backend work.
//!
//! ERROR AND ESCAPE POLICY
//! =======================
//! Invalid operands emit diagnostics and return the shared error type where the
//! expression cannot be trusted. Explicit targets are otherwise respected,
//! including `ignotum` and union-like map values when the surrounding code has
//! intentionally chosen an escape or broad shape.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Checks dereference syntax by requiring a reference-shaped operand.
    pub(super) fn check_deref(&mut self, expr: &mut HirExpr, span: crate::lexer::Span) -> TypeId {
        let expr_ty = self.check_expr(expr);
        match self.types.get(self.resolve_type(expr_ty)) {
            Type::Ref(_, inner) => *inner,
            _ => {
                self.error(SemanticErrorKind::InvalidOperandTypes, "deref requires reference", span);
                self.error_type
            }
        }
    }
    /// Checks a runtime value conversion (`conversio`).
    ///
    /// The source is always visited for diagnostics. The fallback expression is
    /// checked too, but mismatches with the requested target are intentionally
    /// not made fatal here yet because backend conversion code owns the runtime
    /// fallback contract. This function therefore returns the declared target
    /// type rather than inventing a common type between source and fallback.
    pub(super) fn check_conversio(
        &mut self,
        source: &mut HirExpr,
        target: TypeId,
        fallback: Option<&mut HirExpr>,
        _span: crate::lexer::Span,
    ) -> TypeId {
        self.check_expr(source);
        if let Some(fallback) = fallback {
            let fb_ty = self.check_expr(fallback);
            let target_resolved = self.resolve_type(target);
            let fb_resolved = self.resolve_type(fb_ty);
            if target_resolved != fb_resolved && !self.is_infer(fb_resolved) && !self.is_infer(target_resolved) {
                // Allow fallback type mismatch for now — codegen handles it
            }
        }
        target
    }

    /// Checks `verte` typed conversion and construction forms.
    ///
    /// The resolved target type decides the validation mode: structs validate
    /// fields, arrays validate element/spread element compatibility, maps
    /// validate keys and values, and all other targets check the source while
    /// trusting the explicit target. Inference targets are unified with the
    /// source type because there is no concrete shape to validate yet.
    pub(super) fn check_verte(
        &mut self,
        source: &mut HirExpr,
        target: TypeId,
        entries: Option<&mut Vec<HirObjectField>>,
        span: crate::lexer::Span,
    ) -> TypeId {
        let target_resolved = self.resolve_type(target);

        // WHY: an infer target carries no conversion shape, so the source use
        // site is the only evidence available to constrain it.
        if self.is_infer(target_resolved) {
            let expr_ty = self.check_expr(source);
            return self.unify(expr_ty, target, source.span, "invalid cast");
        }

        match (self.types.get(target_resolved).clone(), entries) {
            (Type::Struct(def_id), Some(entries)) => {
                self.check_struct_fields(def_id, entries, span);
            }
            (Type::Array(elem_ty), _) => {
                if let HirExprKind::Array(elements) = &mut source.kind {
                    for element in elements {
                        match element {
                            HirArrayElement::Expr(expr) => {
                                let element_ty = self.check_expr(expr);
                                self.unify(element_ty, elem_ty, expr.span, "array element type mismatch");
                            }
                            HirArrayElement::Spread(expr) => {
                                let spread_ty = self.check_expr(expr);
                                match self.types.get(self.resolve_type(spread_ty)) {
                                    Type::Array(inner) => {
                                        self.unify(*inner, elem_ty, expr.span, "array spread element type mismatch");
                                    }
                                    _ => {
                                        self.error(
                                            SemanticErrorKind::InvalidOperandTypes,
                                            "array spread requires lista operand",
                                            expr.span,
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.check_expr(source);
                }
            }
            (Type::Map(key_ty, value_ty), Some(entries)) => {
                let mut inferred_values = Vec::new();
                for field in entries {
                    match &mut field.key {
                        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {
                            let textus = self.textus_type();
                            self.unify(textus, key_ty, span, "map key type mismatch");
                        }
                        HirObjectKey::Computed(key) => {
                            let actual_key_ty = self.check_expr(key);
                            self.unify(actual_key_ty, key_ty, key.span, "map key type mismatch");
                        }
                        HirObjectKey::Spread(expr) => {
                            let spread_ty = self.check_expr(expr);
                            match self.types.get(self.resolve_type(spread_ty)).clone() {
                                Type::Map(spread_key_ty, spread_value_ty) => {
                                    self.unify(spread_key_ty, key_ty, expr.span, "map spread key type mismatch");
                                    inferred_values.push(self.resolve_type(spread_value_ty));
                                    if !self.is_infer(self.resolve_type(value_ty)) {
                                        self.unify(
                                            spread_value_ty,
                                            value_ty,
                                            expr.span,
                                            "map spread value type mismatch",
                                        );
                                    }
                                }
                                _ => {
                                    self.error(
                                        SemanticErrorKind::InvalidOperandTypes,
                                        "object spread requires tabula operand",
                                        expr.span,
                                    );
                                }
                            }
                            continue;
                        }
                    }

                    let Some(value) = &mut field.value else {
                        self.error(SemanticErrorKind::InvalidOperandTypes, "object field requires value", span);
                        continue;
                    };
                    let value_ty_actual = self.check_expr(value);
                    inferred_values.push(self.resolve_type(value_ty_actual));
                    let value_resolved = self.resolve_type(value_ty);
                    // WHY: infer/ignotum/union value targets are already broad
                    // enough to accept the entry while preserving the explicit
                    // map shape. Narrow value targets still get normal unifies.
                    if !(self.is_infer(value_resolved)
                        || matches!(self.types.get(value_resolved), Type::Primitive(Primitive::Ignotum))
                        || matches!(self.types.get(value_resolved), Type::Union(_)))
                    {
                        self.unify(value_ty_actual, value_ty, value.span, "map value type mismatch");
                    }
                }
                if self.is_infer(self.resolve_type(value_ty)) {
                    let inferred_value_ty = match inferred_values.as_slice() {
                        [] => value_ty,
                        [single] => *single,
                        _ => self.types.intern(Type::Union(inferred_values)),
                    };
                    return self.types.map(key_ty, inferred_value_ty);
                }
            }
            _ => {
                self.check_expr(source);
            }
        }
        target_resolved
    }
}
