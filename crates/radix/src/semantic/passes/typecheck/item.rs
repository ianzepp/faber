//! Item-level typechecking for declarations that establish function bodies and
//! global values.
//!
//! This file is the point where HIR items become checked bodies rather than
//! collected signatures. Earlier typecheck setup has already registered
//! callable, constant, struct, interface, and enum metadata; the routines here
//! use those tables to validate the bodies and to write inferred declaration
//! types back into both the HIR and the checker-side signature maps.
//!
//! INVARIANTS
//! ==========
//! - Function parameters and CLI argument bindings are scoped only to the
//!   function body.
//! - `current_return` tracks the normal return channel, while `current_error`
//!   tracks the alternate-exit channel used by `iace`/catchable control flow.
//! - Unannotated function returns are finalized from observed `redde` values,
//!   falling back to `vacuum` only when no return expression is seen.
//! - Struct field defaults are checked against their declared field types; they
//!   do not create fresh field types.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Check a function body under its declared signature context.
    ///
    /// This temporarily installs the function's normal and alternate-exit
    /// result types so nested `redde` and `iace` expressions validate against
    /// the signature rather than against an enclosing body. When the normal
    /// return type is omitted, the checker infers it from all `redde` sites and
    /// updates the collected function signature so later calls see the same
    /// resolved contract as the HIR.
    pub(super) fn check_function(&mut self, def_id: DefId, func: &mut HirFunction) {
        self.push_scope();
        for param in &mut func.params {
            if let Some(default) = &mut param.default {
                let default_ty = self.check_expr_with_expected(default, Some(param.ty));
                self.unify(default_ty, param.ty, default.span, "parameter default type mismatch");
            }
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            self.insert_binding(param.def_id, param.ty, mutable);
        }
        if let Some(param) = &func.cli_args {
            self.insert_binding(param.def_id, param.ty, false);
        }

        let prev_return = self.current_return;
        let prev_error = self.current_error;
        let prev_inferred = self.inferred_return;
        self.current_return = func.ret_ty;
        self.current_error = func.err_ty.map(ErrorSink::Function);
        self.inferred_return = None;

        if let Some(body) = &mut func.body {
            self.check_block(body, None);
        }

        let inferred = self.inferred_return.take();
        if func.ret_ty.is_none() {
            let ret_ty = inferred.unwrap_or(self.vacuum_type());
            func.ret_ty = Some(ret_ty);
            if let Some(sig) = self.functions.get_mut(&def_id) {
                sig.ret = ret_ty;
            }
        }

        self.current_return = prev_return;
        self.current_error = prev_error;
        self.inferred_return = prev_inferred;
        self.pop_scope();
    }

    /// Check a constant initializer and record the resulting global type.
    ///
    /// Constants may either state their type or synthesize it from the
    /// initializer. The HIR and `consts` table are kept in lockstep because path
    /// lookup during the rest of typechecking reads from the table, while later
    /// phases read the annotated HIR.
    pub(super) fn check_const(&mut self, def_id: DefId, const_item: &mut crate::hir::HirConst) {
        let value_ty = self.check_expr(&mut const_item.value);

        let ty = if let Some(annotated) = const_item.ty {
            self.unify(
                value_ty,
                annotated,
                const_item.value.span,
                "constant value does not match annotation",
            );
            annotated
        } else {
            value_ty
        };

        const_item.ty = Some(ty);
        self.consts.insert(def_id, ty);
    }

    /// Dispatch item checking after the type metadata collection pass.
    ///
    /// Only items with executable or value-bearing bodies need work here.
    /// Metadata-only declarations were already harvested into lookup tables, and
    /// their structural contracts are enforced where expressions or patterns use
    /// them.
    pub(super) fn check_item(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(func) => self.check_function(item.def_id, func),
            HirItemKind::Const(const_item) => self.check_const(item.def_id, const_item),
            HirItemKind::Struct(struct_item) => {
                for field in &mut struct_item.fields {
                    if let Some(init) = &mut field.init {
                        let init_ty = self.check_expr(init);
                        self.unify(init_ty, field.ty, init.span, "field default type mismatch");
                    }
                }
                for method in &mut struct_item.methods {
                    self.check_function(method.def_id, &mut method.func);
                }
            }
            _ => {}
        }
    }
}
