use super::*;

impl<'a> TypeChecker<'a> {
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
    /// Check a runtime value conversion (conversio).
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

    /// Validate struct field entries against the struct definition.
    pub(super) fn check_verte(
        &mut self,
        source: &mut HirExpr,
        target: TypeId,
        entries: Option<&mut Vec<HirObjectField>>,
        span: crate::lexer::Span,
    ) -> TypeId {
        let target_resolved = self.resolve_type(target);

        // For infer-typed targets, unify with the source type
        if self.is_infer(target_resolved) {
            let expr_ty = self.check_expr(source);
            return self.unify(expr_ty, target, source.span, "invalid cast");
        }

        match (self.types.get(target_resolved).clone(), entries) {
            // Struct instantiation — validate field names and types
            (Type::Struct(def_id), Some(entries)) => {
                self.check_struct_fields(def_id, entries, span);
            }
            // Array construction — validate element types
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
            // Map construction — validate key-value entry types
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
                                    self.unify(spread_value_ty, value_ty, expr.span, "map spread value type mismatch");
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
            // Cast / other — check the source, trust the target type
            _ => {
                self.check_expr(source);
            }
        }
        target_resolved
    }
}
