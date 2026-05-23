use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn lower_tuple(&mut self, items: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut fields = Vec::with_capacity(items.len());
        for item in items {
            fields.push(MirAggregateItem::Operand(self.lower_expr_value(item)?));
        }
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(MirAggregateKind::Tuple, MirAggregateFields::Ordered(fields), ty, expr.span))
    }

    pub(super) fn lower_array(
        &mut self,
        elements: &[HirArrayElement],
        expr: &HirExpr,
        kind: MirAggregateKind,
    ) -> Option<MirOperand> {
        let fields = self.lower_array_items(elements)?;
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(kind, MirAggregateFields::Ordered(fields), ty, expr.span))
    }

    fn lower_array_items(&mut self, elements: &[HirArrayElement]) -> Option<Vec<MirAggregateItem>> {
        let mut fields = Vec::with_capacity(elements.len());
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    fields.push(MirAggregateItem::Operand(self.lower_expr_value(expr)?));
                }
                HirArrayElement::Spread(expr) => {
                    fields.push(MirAggregateItem::Spread(self.lower_expr_value(expr)?));
                }
            }
        }
        Some(fields)
    }

    pub(super) fn lower_struct_literal(
        &mut self,
        def_id: DefId,
        fields: &[(crate::lexer::Symbol, HirExpr)],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let mut named = Vec::with_capacity(fields.len());
        for (name, value) in fields {
            named.push(MirNamedOperand { name: *name, value: self.lower_expr_value(value)? });
        }
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(
            MirAggregateKind::Struct(def_id),
            MirAggregateFields::Named(named),
            ty,
            expr.span,
        ))
    }

    pub(super) fn lower_verte(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        entries: Option<&Vec<HirObjectField>>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        match self.types.get(target) {
            Type::Struct(def_id) => {
                let Some(entries) = entries else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "struct construction without object entries before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_struct_object_fields(*def_id, entries, expr.span)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(
                    MirAggregateKind::Struct(*def_id),
                    MirAggregateFields::Named(fields),
                    ty,
                    expr.span,
                ))
            }
            Type::Map(_, _) => {
                let Some(entries) = entries else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "map construction without object entries before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_map_object_fields(entries, expr.span)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Map, MirAggregateFields::Keyed(fields), ty, expr.span))
            }
            Type::Array(_) => {
                let HirExprKind::Array(elements) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "array construction from non-array source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_array_items(elements)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Array, MirAggregateFields::Ordered(fields), ty, expr.span))
            }
            Type::Set(_) => {
                let HirExprKind::Array(elements) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "set construction from non-array source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_array_items(elements)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Set, MirAggregateFields::Ordered(fields), ty, expr.span))
            }
            Type::Enum(_) => {
                let HirExprKind::Call(callee, args) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from non-variant source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let HirExprKind::Path(def_id) = &callee.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from indirect variant before aggregate MIR lowering",
                    ));
                    return None;
                };
                if !self.context.variant_parents.contains_key(def_id) {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from non-variant call before aggregate MIR lowering",
                    ));
                    return None;
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr_value(arg)?);
                }
                let fields = self.variant_payload(*def_id, lowered_args);
                Some(self.construct_temp(
                    MirAggregateKind::EnumVariant(*def_id),
                    fields,
                    MirType::semantic(target),
                    expr.span,
                ))
            }
            _ => {
                self.errors
                    .push(MirError::unsupported(expr.span, "verte cast before aggregate MIR lowering"));
                None
            }
        }
    }

    fn lower_struct_object_fields(
        &mut self,
        def_id: DefId,
        entries: &[HirObjectField],
        span: Span,
    ) -> Option<Vec<MirNamedOperand>> {
        let mut supplied = FxHashSet::default();
        let mut fields = Vec::new();
        for entry in entries {
            let name = match &entry.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => *name,
                HirObjectKey::Computed(expr) => {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "computed struct keys before aggregate MIR lowering",
                    ));
                    return None;
                }
                HirObjectKey::Spread(expr) => {
                    self.errors
                        .push(MirError::unsupported(expr.span, "struct spread before aggregate MIR lowering"));
                    return None;
                }
            };
            let Some(value) = &entry.value else {
                self.errors.push(MirError::unsupported(
                    span,
                    "struct field without value before aggregate MIR lowering",
                ));
                return None;
            };
            supplied.insert(name);
            fields.push(MirNamedOperand { name, value: self.lower_expr_value(value)? });
        }

        if let Some(defaults) = self.context.structs.get(&def_id).cloned() {
            for field in defaults {
                if supplied.contains(&field.name) {
                    continue;
                }
                if let Some(init) = &field.init {
                    fields.push(MirNamedOperand { name: field.name, value: self.lower_expr_value(init)? });
                }
            }
        }

        Some(fields)
    }

    fn lower_map_object_fields(&mut self, entries: &[HirObjectField], span: Span) -> Option<Vec<MirKeyValueOperand>> {
        let mut fields = Vec::with_capacity(entries.len());
        for entry in entries {
            let key = match &entry.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    MirOperand::Constant(MirConstant::String(*name))
                }
                HirObjectKey::Computed(expr) => self.lower_expr_value(expr)?,
                HirObjectKey::Spread(expr) => {
                    self.errors
                        .push(MirError::unsupported(expr.span, "map spread before aggregate MIR lowering"));
                    return None;
                }
            };
            let Some(value) = &entry.value else {
                self.errors.push(MirError::unsupported(
                    span,
                    "map field without value before aggregate MIR lowering",
                ));
                return None;
            };
            fields.push(MirKeyValueOperand { key, value: self.lower_expr_value(value)? });
        }
        Some(fields)
    }
}
