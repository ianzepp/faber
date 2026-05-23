//! Local lookup and primitive type helpers for HIR typechecking.
//!
//! This module keeps the checker's low-level table access in one place:
//! lexical bindings, declaration lookup helpers, semantic category predicates,
//! primitive constructors, and member-definition extraction. Most helpers first
//! resolve inference substitutions and aliases so callers check the current
//! semantic meaning rather than the raw arena entry.
//!
//! NULLABILITY AND ACCESS
//! ======================
//! Struct, enum, and interface extraction intentionally look through references,
//! optional wrappers, and applied generic shells. That lets field/access/call
//! checking operate on the underlying declaration contract while the expression
//! checker still owns whether optional chaining, non-null access, or direct
//! access is legal for the surface syntax.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Record a hard typecheck diagnostic while preserving traversal.
    pub(super) fn error(&mut self, kind: SemanticErrorKind, message: &str, span: crate::lexer::Span) {
        self.errors
            .push(SemanticError::new(kind, message.to_owned(), span));
    }

    /// Find the innermost visible binding for a resolved definition.
    pub(super) fn lookup_binding(&self, def_id: DefId) -> Option<BindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(&def_id) {
                return Some(*info);
            }
        }
        None
    }

    /// Insert a binding into the current lexical scope.
    pub(super) fn insert_binding(&mut self, def_id: DefId, ty: TypeId, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(def_id, BindingInfo { ty, mutable });
        }
    }

    /// Start a lexical scope for block, branch, loop, or pattern-local bindings.
    pub(super) fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    /// Leave the innermost lexical scope.
    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Return whether the resolved type participates in numeric operators.
    pub(super) fn is_numeric(&self, ty: TypeId) -> bool {
        self.is_integer(ty) || self.is_fractus(ty)
    }

    pub(super) fn is_integer(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Numerus))
    }

    pub(super) fn is_fractus(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Fractus))
    }

    pub(super) fn is_textus(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Textus))
    }

    pub(super) fn is_bool(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Bivalens))
    }

    pub(super) fn numerus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Numerus)
    }

    pub(super) fn fractus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Fractus)
    }

    pub(super) fn textus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Textus)
    }

    pub(super) fn bool_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Bivalens)
    }

    pub(super) fn regex_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Regex)
    }

    pub(super) fn nil_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Nihil)
    }

    pub(super) fn vacuum_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Vacuum)
    }

    /// Resolve only aliases, without consuming checker inference substitutions.
    ///
    /// Most typecheck policy should use `resolve_type`; this helper remains for
    /// callers that specifically need the declared alias target shape.
    #[allow(dead_code)]
    pub(super) fn resolve_alias(&self, ty: TypeId) -> TypeId {
        let mut current = ty;
        loop {
            match self.types.get(current) {
                Type::Alias(_, resolved) => current = *resolved,
                _ => return current,
            }
        }
    }

    /// Look up a method contract for a struct or interface receiver.
    ///
    /// Receiver type wrappers are normalized by `*_def_from_type`; this lookup
    /// returns only declared contracts. Fallback collection methods and `ignotum`
    /// escape behavior are handled by call checking.
    pub(super) fn lookup_method_signature(&self, receiver_ty: TypeId, name: Symbol) -> Option<FuncSig> {
        if let Some(struct_def) = self.struct_def_from_type(receiver_ty) {
            if let Some(info) = self.structs.get(&struct_def) {
                if let Some(sig) = info.methods.get(&name) {
                    return Some(sig.clone());
                }
            }
        }
        if let Some(interface_def) = self.interface_def_from_type(receiver_ty) {
            if let Some(methods) = self.interfaces.get(&interface_def) {
                if let Some(sig) = methods.get(&name) {
                    return Some(sig.clone());
                }
            }
        }
        None
    }

    /// Extract the underlying struct definition from a receiver-like type.
    pub(super) fn struct_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Struct(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.struct_def_from_type(*inner),
            Type::Option(inner) => self.struct_def_from_type(*inner),
            Type::Applied(base, _) => self.struct_def_from_type(*base),
            _ => None,
        }
    }

    /// Extract the underlying enum definition from a scrutinee-like type.
    pub(super) fn enum_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Enum(def_id) => Some(*def_id),
            Type::Option(inner) => self.enum_def_from_type(*inner),
            Type::Applied(base, _) => self.enum_def_from_type(*base),
            _ => None,
        }
    }

    /// Extract the underlying interface definition from a receiver-like type.
    pub(super) fn interface_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Interface(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.interface_def_from_type(*inner),
            Type::Option(inner) => self.interface_def_from_type(*inner),
            Type::Applied(base, _) => self.interface_def_from_type(*base),
            _ => None,
        }
    }

    /// Map a literal node to its primitive semantic type.
    ///
    /// `nihil` is kept as the primitive nil type here; optional/nullability
    /// acceptance is decided later by unification, coalescing, and
    /// `TypeTable::assignable`.
    pub(super) fn literal_type(&mut self, lit: &HirLiteral) -> TypeId {
        match lit {
            HirLiteral::Int(_) => self.numerus_type(),
            HirLiteral::Float(_) => self.fractus_type(),
            HirLiteral::String(_) => self.textus_type(),
            HirLiteral::Regex(_, _) => self.regex_type(),
            HirLiteral::Bool(_) => self.bool_type(),
            HirLiteral::Nil => self.nil_type(),
        }
    }
}
