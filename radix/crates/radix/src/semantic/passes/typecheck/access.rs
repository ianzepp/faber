use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_lvalue(&mut self, target: &mut HirExpr) -> TypeId {
        match &mut target.kind {
            HirExprKind::Path(def_id) => {
                if let Some(binding) = self.lookup_binding(*def_id) {
                    if !binding.mutable {
                        self.error(
                            SemanticErrorKind::ImmutableAssignment,
                            "assignment to immutable binding",
                            target.span,
                        );
                    }
                    return binding.ty;
                }

                if let Some(ty) = self.consts.get(def_id).copied() {
                    self.error(SemanticErrorKind::ImmutableAssignment, "assignment to constant", target.span);
                    return ty;
                }

                self.error(
                    SemanticErrorKind::InvalidAssignmentTarget,
                    "invalid assignment target",
                    target.span,
                );
                self.error_type
            }
            HirExprKind::Field(object, name) => self.check_field(object, *name),
            HirExprKind::Index(object, index) => self.check_index(object, index),
            _ => {
                self.error(
                    SemanticErrorKind::InvalidAssignmentTarget,
                    "invalid assignment target",
                    target.span,
                );
                self.error_type
            }
        }
    }
    pub(super) fn check_index_from_type(
        &mut self,
        object_ty: TypeId,
        idx_ty: TypeId,
        object_span: crate::lexer::Span,
        index_span: crate::lexer::Span,
    ) -> TypeId {
        let resolved = self.resolve_type(object_ty);
        let resolved_kind = self.types.get(resolved).clone();
        let kind = match resolved_kind {
            Type::Array(elem) => Some((Some(elem), None, None)),
            Type::Map(key, value) => Some((None, Some(key), Some(value))),
            Type::Union(_) => Some((Some(self.types.primitive(Primitive::Ignotum)), None, None)),
            Type::Primitive(Primitive::Ignotum) => Some((Some(self.types.primitive(Primitive::Ignotum)), None, None)),
            Type::Infer(_) => Some((Some(self.fresh_infer()), None, None)),
            _ => None,
        };

        match kind {
            Some((Some(elem), None, None)) => {
                if !self.is_integer(idx_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "array index must be numerus",
                        index_span,
                    );
                }
                elem
            }
            Some((None, Some(key), Some(value))) => {
                self.unify(idx_ty, key, index_span, "map index type mismatch");
                value
            }
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "indexing requires array or map",
                    object_span,
                );
                self.error_type
            }
        }
    }
    pub(super) fn check_field_from_type(
        &mut self,
        object_ty: TypeId,
        name: Symbol,
        _span: crate::lexer::Span,
    ) -> TypeId {
        if let Some(struct_def) = self.struct_def_from_type(object_ty) {
            if let Some(info) = self.structs.get(&struct_def) {
                if let Some(field_ty) = info.fields.get(&name).copied() {
                    return field_ty;
                }
            }
        }
        if let Type::Map(_, value_ty) = self.types.get(self.resolve_type(object_ty)) {
            return *value_ty;
        }
        if let Type::Record(fields) = self.types.get(self.resolve_type(object_ty)) {
            if let Some(field_ty) = fields.get(&name).copied() {
                return field_ty;
            }
            self.error(SemanticErrorKind::UndefinedVariable, "unknown record field", _span);
            return self.error_type;
        }
        self.error_type
    }
    pub(super) fn check_non_null(
        &mut self,
        object: &mut HirExpr,
        chain: &mut crate::hir::HirNonNullKind,
        span: crate::lexer::Span,
    ) -> TypeId {
        let object_ty = self.check_expr(object);
        let inner_ty = match self.types.get(self.resolve_type(object_ty)) {
            Type::Option(inner) => *inner,
            _ => object_ty,
        };

        match chain {
            crate::hir::HirNonNullKind::Member(name) => self.check_field_from_type(inner_ty, *name, object.span),
            crate::hir::HirNonNullKind::Index(index) => {
                let idx_ty = self.check_expr(index);
                self.check_index_from_type(inner_ty, idx_ty, object.span, index.span)
            }
            crate::hir::HirNonNullKind::Call(args) => self.check_call_from_type(inner_ty, args, span),
        }
    }
    pub(super) fn check_optional_chain(
        &mut self,
        object: &mut HirExpr,
        chain: &mut crate::hir::HirOptionalChainKind,
        span: crate::lexer::Span,
    ) -> TypeId {
        let object_ty = self.check_expr(object);
        let object_resolved = self.resolve_type(object_ty);

        let inner_ty = match self.types.get(object_resolved) {
            Type::Option(inner) => *inner,
            _ => object_ty,
        };

        let result = match chain {
            crate::hir::HirOptionalChainKind::Member(name) => self.check_field_from_type(inner_ty, *name, object.span),
            crate::hir::HirOptionalChainKind::Index(index) => {
                let idx_ty = self.check_expr(index);
                self.check_index_from_type(inner_ty, idx_ty, object.span, index.span)
            }
            crate::hir::HirOptionalChainKind::Call(args) => self.check_call_from_type(inner_ty, args, span),
        };

        self.types.option(result)
    }
    pub(super) fn check_index(&mut self, object: &mut HirExpr, index: &mut HirExpr) -> TypeId {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);
        self.check_index_from_type(obj_ty, idx_ty, object.span, index.span)
    }
    pub(super) fn check_field(&mut self, object: &mut HirExpr, name: Symbol) -> TypeId {
        let obj_ty = self.check_expr(object);
        self.check_field_from_type(obj_ty, name, object.span)
    }
    pub(super) fn check_path(&mut self, def_id: DefId, span: crate::lexer::Span) -> TypeId {
        if let Some(binding) = self.lookup_binding(def_id) {
            return binding.ty;
        }

        if let Some(ty) = self.consts.get(&def_id) {
            return *ty;
        }

        if let Some(sig) = self.functions.get(&def_id) {
            return self.types.function(sig.clone());
        }

        if let Some(parent) = self.variant_parent.get(&def_id).copied() {
            return self.types.intern(Type::Enum(parent));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Struct)
        ) {
            return self.types.intern(Type::Struct(def_id));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Interface)
        ) {
            return self.types.intern(Type::Interface(def_id));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Module)
        ) {
            return self.types.primitive(Primitive::Ignotum);
        }

        self.error(SemanticErrorKind::UndefinedVariable, "unknown identifier", span);
        self.error_type
    }
}
