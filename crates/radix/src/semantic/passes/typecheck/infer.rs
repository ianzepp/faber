//! Inference-variable unification and substitution resolution.
//!
//! This file is the checker-local inference engine. `Type::Infer` nodes live in
//! the shared [`TypeTable`], but their meaning is completed by the
//! `TypeChecker` substitution map here. Any code that needs the current meaning
//! of a type must resolve through `TypeChecker::resolve_type`, not by inspecting
//! the table entry alone.
//!
//! ERROR RECOVERY
//! ==============
//! Unification reports hard type errors and returns the table-local error type
//! when compatibility fails. The checker then keeps walking the HIR so one bad
//! expression does not hide unrelated diagnostics. Unresolved inference
//! variables are diagnosed later by finalization as missing annotations.
//!
//! INVARIANTS
//! ==========
//! - `InferVar` IDs are scoped to one `TypeChecker` run.
//! - Substitutions may point at aliases or other inferred types; resolving must
//!   chase both substitution and alias chains.
//! - The occurs check prevents recursive inference substitutions from creating
//!   an infinite semantic type.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Return whether `var` occurs inside `ty` after current substitutions.
    ///
    /// This is the guard that keeps `T = lista<T>`-style inference cycles out of
    /// the type table. It walks structural type shapes because compound entries
    /// are allocated in the shared arena rather than expanded inline.
    pub(super) fn occurs_in(&self, var: InferVar, ty: TypeId) -> bool {
        let resolved = self.resolve_type(ty);
        if let Some(found) = self.infer_var_of(resolved) {
            return found == var;
        }
        match self.types.get(resolved) {
            Type::Array(inner) | Type::Option(inner) | Type::Ref(_, inner) => self.occurs_in(var, *inner),
            Type::Set(inner) => self.occurs_in(var, *inner),
            Type::Map(key, value) => self.occurs_in(var, *key) || self.occurs_in(var, *value),
            Type::Func(sig) => {
                sig.params.iter().any(|param| self.occurs_in(var, param.ty))
                    || self.occurs_in(var, sig.ret)
                    || sig.err.is_some_and(|err| self.occurs_in(var, err))
            }
            Type::Applied(base, args) => self.occurs_in(var, *base) || args.iter().any(|arg| self.occurs_in(var, *arg)),
            Type::Union(types) => types.iter().any(|inner| self.occurs_in(var, *inner)),
            _ => false,
        }
    }

    /// Bind an inference variable to a resolved type or merge with its old binding.
    ///
    /// Existing substitutions are unified rather than overwritten so multiple
    /// constraints on the same inferred type converge through the normal
    /// compatibility rules.
    pub(super) fn bind_infer(&mut self, var: InferVar, ty: TypeId, span: crate::lexer::Span, message: &str) -> TypeId {
        let resolved = self.resolve_type(ty);
        if let Some(existing) = self.substitutions.get(&var) {
            return self.unify(*existing, resolved, span, message);
        }

        if self.occurs_in(var, resolved) {
            self.error(SemanticErrorKind::TypeMismatch, message, span);
            return self.error_type;
        }

        self.substitutions.insert(var, resolved);
        resolved
    }

    /// Make two table-local types compatible and return the resulting type.
    ///
    /// This operation is intentionally broader than structural equality. It
    /// resolves aliases and inference substitutions, binds fresh variables,
    /// unifies compatible compound shapes, accepts the language's implicit
    /// numeric/assignment flows, and records a type error for everything else.
    pub(super) fn unify(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span, message: &str) -> TypeId {
        let left = self.resolve_type(a);
        let right = self.resolve_type(b);
        if left == right {
            return left;
        }

        if let Some(var) = self.infer_var_of(left) {
            return self.bind_infer(var, right, span, message);
        }
        if let Some(var) = self.infer_var_of(right) {
            return self.bind_infer(var, left, span, message);
        }

        let left_ty = self.types.get(left).clone();
        let right_ty = self.types.get(right).clone();

        match (left_ty, right_ty) {
            (Type::Primitive(Primitive::Numerus), Type::Primitive(Primitive::Fractus))
            | (Type::Primitive(Primitive::Fractus), Type::Primitive(Primitive::Numerus)) => {
                return self.fractus_type();
            }
            (Type::Primitive(a), Type::Primitive(b)) if a == b => return left,
            (Type::Array(a), Type::Array(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.array(inner);
            }
            (Type::Map(ka, va), Type::Map(kb, vb)) => {
                let key = self.unify(ka, kb, span, message);
                let value = self.unify(va, vb, span, message);
                return self.types.map(key, value);
            }
            (Type::Set(a), Type::Set(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.set(inner);
            }
            (Type::Option(a), Type::Option(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.option(inner);
            }
            (Type::Ref(ma, a), Type::Ref(mb, b)) if ma == mb => {
                let inner = self.unify(a, b, span, message);
                return self.types.reference(ma, inner);
            }
            (Type::Func(sig_a), Type::Func(sig_b)) => {
                if sig_a.params.len() != sig_b.params.len() {
                    self.error(SemanticErrorKind::WrongArity, message, span);
                    return self.error_type;
                }
                for (param_a, param_b) in sig_a.params.iter().zip(sig_b.params.iter()) {
                    self.unify(param_a.ty, param_b.ty, span, message);
                }
                let ret = self.unify(sig_a.ret, sig_b.ret, span, message);
                let err = match (sig_a.err, sig_b.err) {
                    (Some(a_err), Some(b_err)) => Some(self.unify(a_err, b_err, span, message)),
                    (None, None) => None,
                    _ => {
                        self.error(SemanticErrorKind::TypeMismatch, message, span);
                        return self.error_type;
                    }
                };
                return self.types.function(FuncSig {
                    params: sig_a.params.clone(),
                    ret,
                    err,
                    // Async/generator markers describe effects at the call
                    // boundary; unifying compatible function values preserves
                    // either effect instead of silently erasing it.
                    is_async: sig_a.is_async || sig_b.is_async,
                    is_generator: sig_a.is_generator || sig_b.is_generator,
                });
            }
            _ => {
                if self.types.assignable(left, right) || self.types.assignable(right, left) {
                    return right;
                }
            }
        }

        self.error(SemanticErrorKind::TypeMismatch, message, span);
        self.error_type
    }

    /// Return whether a type still names an inference variable after resolution.
    pub(super) fn is_infer(&self, ty: TypeId) -> bool {
        self.infer_var_of(ty).is_some()
    }

    /// Extract the inference variable token from a table entry if present.
    pub(super) fn infer_var_of(&self, ty: TypeId) -> Option<InferVar> {
        match self.types.get(ty) {
            Type::Infer(var) => Some(*var),
            _ => None,
        }
    }

    /// Resolve checker-local substitutions and semantic aliases to a usable type.
    ///
    /// `TypeId` itself is only an arena index. For inferred or aliased types, the
    /// current semantic meaning is the fixed point reached by following the
    /// checker substitution map and `Type::Alias` entries.
    pub(super) fn resolve_type(&self, ty: TypeId) -> TypeId {
        let mut current = ty;
        loop {
            if let Some(infer) = self.infer_var_of(current) {
                if let Some(subst) = self.substitutions.get(&infer) {
                    current = *subst;
                    continue;
                }
            }
            match self.types.get(current) {
                Type::Alias(_, resolved) => current = *resolved,
                _ => return current,
            }
        }
    }

    /// Allocate a fresh inference variable in the shared type table.
    ///
    /// The returned `TypeId` can be stored on HIR nodes immediately, but it is
    /// only final when `substitutions` later resolves its `InferVar`.
    pub(super) fn fresh_infer(&mut self) -> TypeId {
        let var = InferVar(self.next_infer);
        self.next_infer += 1;
        let id = self.types.intern(Type::Infer(var));
        self.infer_ids.insert(var, id);
        id
    }
}
