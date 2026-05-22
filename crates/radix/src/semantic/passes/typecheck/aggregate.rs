use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_struct_fields(
        &mut self,
        def_id: DefId,
        fields: &mut [HirObjectField],
        span: crate::lexer::Span,
    ) {
        let field_types = match self.structs.get(&def_id) {
            Some(info) => info.fields.clone(),
            None => {
                self.error(
                    SemanticErrorKind::UndefinedType,
                    "unknown struct",
                    fields
                        .first()
                        .and_then(|field| field.value.as_ref().map(|expr| expr.span))
                        .unwrap_or(span),
                );
                return;
            }
        };
        let mut supplied = FxHashSet::default();

        for field in fields.iter_mut() {
            match &mut field.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    let Some(field_info) = field_types.get(name).copied() else {
                        let error_span = field.value.as_ref().map(|expr| expr.span).unwrap_or(span);
                        self.error(SemanticErrorKind::UndefinedMember, "unknown field", error_span);
                        continue;
                    };
                    supplied.insert(*name);
                    let Some(value) = &mut field.value else {
                        self.error(SemanticErrorKind::UndefinedMember, "struct field requires value", span);
                        continue;
                    };
                    let value_ty = self.check_expr(value);
                    self.unify(value_ty, field_info.ty, value.span, "field initializer type mismatch");
                }
                HirObjectKey::Computed(expr) => {
                    self.check_expr(expr);
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "computed keys are not valid in struct construction",
                        expr.span,
                    );
                }
                HirObjectKey::Spread(expr) => {
                    self.check_expr(expr);
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "spread fields are not valid in struct construction",
                        expr.span,
                    );
                }
            }
        }

        for (name, info) in field_types {
            if info.required && !supplied.contains(&name) {
                self.error(SemanticErrorKind::UndefinedMember, "missing required struct field", info.span);
            }
        }
    }
    #[allow(clippy::ptr_arg)]
    pub(super) fn check_closure(
        &mut self,
        params: &mut Vec<HirParam>,
        ret: Option<&mut TypeId>,
        body: &mut HirExpr,
        expected: Option<TypeId>,
    ) -> TypeId {
        let expected_sig = expected.and_then(|ty| self.function_signature_from_type(ty));

        self.push_scope();
        for (idx, param) in params.iter().enumerate() {
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            if let Some(sig) = &expected_sig {
                if let Some(expected_param) = sig.params.get(idx) {
                    self.unify(param.ty, expected_param.ty, param.span, "closure parameter type mismatch");
                }
            }
            self.insert_binding(param.def_id, param.ty, mutable);
        }

        let expected_ret = ret
            .as_ref()
            .map(|ty| **ty)
            .or_else(|| expected_sig.as_ref().map(|sig| sig.ret));
        let body_ty = self.check_expr_with_expected(body, expected_ret);
        let ret_ty = match ret {
            Some(ty) => {
                self.unify(body_ty, *ty, body.span, "closure return type mismatch");
                *ty
            }
            None => body_ty,
        };

        self.pop_scope();

        let sig = FuncSig {
            params: params
                .iter()
                .map(|param| ParamType {
                    ty: param.ty,
                    mode: param_mode_from_hir(param.mode),
                    optional: param.optional,
                })
                .collect(),
            ret: ret_ty,
            err: None,
            is_async: false,
            is_generator: false,
        };
        self.types.function(sig)
    }

    /// Unified type conversion check — dispatches on the resolved target type to determine
    pub(super) fn check_tuple(&mut self, items: &mut [HirExpr]) -> TypeId {
        let mut types = Vec::new();
        for item in items {
            types.push(self.check_expr(item));
        }
        self.types.intern(Type::Union(types))
    }
    #[allow(clippy::ptr_arg)]
    pub(super) fn check_struct_literal(&mut self, def_id: DefId, fields: &mut Vec<(Symbol, HirExpr)>) -> TypeId {
        let field_types = match self.structs.get(&def_id) {
            Some(info) => info.fields.clone(),
            None => {
                self.error(
                    SemanticErrorKind::UndefinedType,
                    "unknown struct",
                    fields
                        .first()
                        .map(|(_, expr)| expr.span)
                        .unwrap_or_default(),
                );
                return self.error_type;
            }
        };

        let mut supplied = FxHashSet::default();
        for (name, value) in fields.iter_mut() {
            let Some(field_info) = field_types.get(name).copied() else {
                self.error(SemanticErrorKind::UndefinedMember, "unknown field", value.span);
                continue;
            };
            supplied.insert(*name);
            let value_ty = self.check_expr(value);
            self.unify(value_ty, field_info.ty, value.span, "field initializer type mismatch");
        }

        for (name, info) in field_types {
            if info.required && !supplied.contains(&name) {
                self.error(SemanticErrorKind::UndefinedMember, "missing required struct field", info.span);
            }
        }

        self.types.intern(Type::Struct(def_id))
    }
    pub(super) fn array_common_type(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span) -> TypeId {
        let a_resolved = self.resolve_type(a);
        let b_resolved = self.resolve_type(b);

        if let Type::Array(a_inner) = self.types.get(a_resolved).clone() {
            if !matches!(self.types.get(b_resolved), Type::Array(_)) {
                return self.common_type(a_inner, b, span);
            }
        }

        if let Type::Array(b_inner) = self.types.get(b_resolved).clone() {
            if !matches!(self.types.get(a_resolved), Type::Array(_)) {
                return self.common_type(a, b_inner, span);
            }
        }

        self.common_type(a, b, span)
    }
    pub(super) fn check_array(
        &mut self,
        elements: &mut [HirArrayElement],
        _span: crate::lexer::Span,
        expected: Option<TypeId>,
    ) -> TypeId {
        let expected_elem = expected.and_then(|ty| match self.types.get(self.resolve_type(ty)) {
            Type::Array(inner) => Some(*inner),
            _ => None,
        });

        if elements.is_empty() {
            if let Some(inner) = expected_elem {
                return self.types.array(inner);
            }
            let infer = self.fresh_infer();
            return self.types.array(infer);
        }

        let mut element_ty = None;
        for element in elements {
            let (expr, spread) = match element {
                HirArrayElement::Expr(expr) => (expr, false),
                HirArrayElement::Spread(expr) => (expr, true),
            };
            let ty = if spread {
                if let Some(expected) = expected_elem {
                    let expected_array = self.types.array(expected);
                    self.check_expr_with_expected(expr, Some(expected_array))
                } else {
                    self.check_expr(expr)
                }
            } else if let Some(expected) = expected_elem {
                self.check_expr_with_expected(expr, Some(expected))
            } else {
                self.check_expr(expr)
            };
            let ty = if spread {
                match self.types.get(self.resolve_type(ty)) {
                    Type::Array(inner) => *inner,
                    _ => {
                        self.error(
                            SemanticErrorKind::InvalidOperandTypes,
                            "array spread requires lista operand",
                            expr.span,
                        );
                        self.error_type
                    }
                }
            } else {
                ty
            };
            element_ty = Some(match element_ty {
                None => ty,
                Some(existing) => self.array_common_type(existing, ty, expr.span),
            });
        }

        let elem_ty = element_ty.unwrap_or_else(|| self.fresh_infer());
        self.types.array(elem_ty)
    }
}
