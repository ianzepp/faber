//! Path, assignment target, member, index, and nullability access typing.
//!
//! Access forms sit on the boundary between resolved symbols and expression
//! shape. Name resolution has already chosen `DefId`s; this file decides
//! whether those definitions are usable as values, whether an access is a valid
//! lvalue, and which type is produced by fields, indexes, optional chains, and
//! non-null chains.
//!
//! NULLABILITY AND UNKNOWN POLICY
//! ==============================
//! Optional chaining strips one `optio` layer for the member/index/call check
//! and then wraps the result back in `optio`. Non-null chaining strips one
//! `optio` layer without producing an optional result. `ignotum` and unions are
//! treated as permissive access shapes only where the code explicitly says so;
//! they are recovery/escape behavior, not target-specific codegen policy.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Checks that an expression can appear on the left side of assignment and
    /// returns the type the assigned value must match.
    ///
    /// Paths must resolve to mutable local bindings; constants report
    /// immutability but still return their declared type so the right-hand side
    /// can be checked. Field and index lvalues reuse the normal access rules.
    pub(super) fn check_lvalue(&mut self, target: &mut HirExpr) -> TypeId {
        let ty = match &mut target.kind {
            HirExprKind::Path(def_id) => {
                if let Some(binding) = self.lookup_binding(*def_id) {
                    if !binding.mutable {
                        self.error(
                            SemanticErrorKind::ImmutableAssignment,
                            "assignment to immutable binding",
                            target.span,
                        );
                    }
                    binding.ty
                } else if let Some(ty) = self.consts.get(def_id).copied() {
                    self.error(SemanticErrorKind::ImmutableAssignment, "assignment to constant", target.span);
                    ty
                } else {
                    self.error(
                        SemanticErrorKind::InvalidAssignmentTarget,
                        "invalid assignment target",
                        target.span,
                    );
                    self.error_type
                }
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
        };
        target.ty = Some(self.resolve_type(ty));
        ty
    }
    /// Computes the result of indexing an already-typed object with an
    /// already-typed index expression.
    ///
    /// Arrays require numeric indexes, maps unify the index with the map key,
    /// and `textus` accepts numerus/range-like numeric unions while returning
    /// `textus`. Unions and `ignotum` preserve forward progress by producing an
    /// unknown element shape rather than guessing a concrete target layout.
    pub(super) fn check_index_from_type(
        &mut self,
        object_ty: TypeId,
        idx_ty: TypeId,
        object_span: crate::lexer::Span,
        index_span: crate::lexer::Span,
    ) -> TypeId {
        let resolved = self.resolve_type(object_ty);
        let resolved_kind = self.types.get(resolved).clone();
        if matches!(resolved_kind, Type::Primitive(Primitive::Textus)) {
            if !self.is_textus_index_type(idx_ty) {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "textus index must be numerus or range",
                    index_span,
                );
            }
            return self.types.primitive(Primitive::Textus);
        }
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
    fn is_textus_index_type(&mut self, idx_ty: TypeId) -> bool {
        if self.is_integer(idx_ty) {
            return true;
        }

        match self.types.get(self.resolve_type(idx_ty)).clone() {
            Type::Union(types) => types.into_iter().all(|ty| self.is_integer(ty)),
            _ => false,
        }
    }
    /// Resolves field access from a precomputed object type.
    ///
    /// Struct fields come from the collected struct table, maps expose their
    /// value type for property-style access, and records must contain the named
    /// field. Missing record fields are diagnosed here; other unsupported
    /// object shapes return the shared error type so the caller decides whether
    /// a higher-level access form should emit a user-facing diagnostic.
    pub(super) fn check_field_from_type(
        &mut self,
        object_ty: TypeId,
        name: Symbol,
        _span: crate::lexer::Span,
    ) -> TypeId {
        if let Some(struct_def) = self.struct_def_from_type(object_ty) {
            if let Some(info) = self.structs.get(&struct_def) {
                if let Some(field) = info.fields.get(&name).copied() {
                    return field.ty;
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
    /// Checks a non-null chain segment after removing one optional layer from
    /// the receiver when present.
    ///
    /// This is a type-level assertion for subsequent access only. It does not
    /// insert runtime checks or decide how a backend should represent failed
    /// non-null assertions.
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
    /// Checks an optional chain segment and wraps the access result in `optio`.
    ///
    /// The receiver is checked once, then an existing option is unwrapped for
    /// the member/index/call rule. A non-optional receiver still produces an
    /// optional result because the syntax's observable contract is nullable
    /// access, not a proof that the receiver was nullable before this point.
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
    /// Checks ordinary index syntax by synthesizing both operands first.
    pub(super) fn check_index(&mut self, object: &mut HirExpr, index: &mut HirExpr) -> TypeId {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);
        self.check_index_from_type(obj_ty, idx_ty, object.span, index.span)
    }
    /// Checks ordinary field syntax by synthesizing the receiver first.
    pub(super) fn check_field(&mut self, object: &mut HirExpr, name: Symbol) -> TypeId {
        let obj_ty = self.check_expr(object);
        self.check_field_from_type(obj_ty, name, object.span)
    }
    /// Converts a resolved definition path into the value type visible at an
    /// expression site.
    ///
    /// Local bindings, constants, functions, enum variants, structs, interfaces,
    /// and modules have different expression roles. Modules deliberately produce
    /// `ignotum` so qualified lookup can keep progressing without pretending the
    /// module itself is a first-class runtime value.
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
