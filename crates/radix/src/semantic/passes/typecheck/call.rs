//! Function, constructor, and method call typing.
//!
//! Calls are where expression inference meets the callable shapes collected
//! earlier in semantic analysis. This file accepts already-resolved HIR callees,
//! validates argument arity and parameter compatibility, builds provisional
//! function types for inference variables, and enforces the local rule that
//! failable calls need an active local handler.
//!
//! INVARIANTS
//! ==========
//! - Known function signatures drive expected-type checking of each argument.
//! - Infer-typed callees are unified with a synthesized function signature from
//!   the actual arguments, allowing call sites to constrain unknown values.
//! - `ignotum` callees and receivers keep checking their arguments but return
//!   `ignotum` instead of manufacturing hard errors from an intentional escape.
//! - Backend method translation and stdlib target names are not decided here;
//!   this pass only establishes the semantic result type and diagnostics.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Builds a provisional function signature from actual arguments.
    ///
    /// This is an inference device, not a declaration model. All arguments are
    /// checked first, then the callee's inference variable can be unified with a
    /// callable type that reflects the use site.
    pub(super) fn build_call_signature(&mut self, args: &mut [HirExpr]) -> FuncSig {
        let params = args
            .iter_mut()
            .map(|arg| ParamType { ty: self.check_expr(arg), mode: ParamMode::Owned, optional: false })
            .collect();
        FuncSig { params, ret: self.fresh_infer(), err: None, is_async: false, is_generator: false }
    }
    /// Returns a callable signature only for resolved function-shaped types.
    pub(super) fn function_signature_from_type(&self, ty: TypeId) -> Option<FuncSig> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Func(sig) => Some(sig.clone()),
            _ => None,
        }
    }
    /// Checks a call when the callee type is already known by another access
    /// form, such as optional chaining or non-null call chaining.
    ///
    /// Error recovery mirrors direct calls: known functions validate arguments,
    /// inference variables become provisional functions, `ignotum` preserves the
    /// escape hatch while still visiting arguments, and non-callables emit one
    /// diagnostic before returning the shared error type.
    pub(super) fn check_call_from_type(
        &mut self,
        callee_ty: TypeId,
        args: &mut [HirExpr],
        span: crate::lexer::Span,
    ) -> TypeId {
        let resolved = self.resolve_type(callee_ty);
        if let Some(sig) = self.function_signature_from_type(resolved) {
            self.check_call_args(&sig, args, span);
            if self.reject_failable_call(&sig, span) {
                return self.error_type;
            }
            return self.resolve_type(sig.ret);
        }

        if self.is_infer(resolved) {
            let sig = self.build_call_signature(args);
            let func_ty = self.types.function(sig.clone());
            self.unify(resolved, func_ty, span, "callee is not callable");
            self.check_call_args(&sig, args, span);
            return sig.ret;
        }

        if matches!(self.types.get(resolved), Type::Primitive(Primitive::Ignotum)) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        for arg in args {
            self.check_expr(arg);
        }
        self.error(SemanticErrorKind::NotCallable, "callee is not callable", span);
        self.error_type
    }
    /// Accepts a single spread-array argument when it can satisfy every required
    /// non-optional parameter in a fixed signature.
    ///
    /// The helper is intentionally conservative: optional parameters disable
    /// this compatibility mode because a single array cannot describe which
    /// trailing arguments were omitted.
    pub(super) fn check_spread_array_compat(
        &mut self,
        sig: &FuncSig,
        arg: &mut HirExpr,
        span: crate::lexer::Span,
    ) -> bool {
        let arg_ty = self.check_expr(arg);
        let resolved = self.resolve_type(arg_ty);
        let inner = match self.types.get(resolved) {
            Type::Array(inner) => *inner,
            _ => return false,
        };
        if sig.params.iter().any(|param| param.optional) {
            return false;
        }
        for param in &sig.params {
            self.unify(inner, param.ty, span, "argument type mismatch");
        }
        true
    }
    /// Validates argument count and parameter types for a known signature.
    ///
    /// Each ordinary argument is checked with its parameter type as expected
    /// context, which lets aggregates, closures, and empty literals adopt the
    /// shape required by the callee before the final mismatch diagnostic is
    /// considered.
    pub(super) fn check_call_args(&mut self, sig: &FuncSig, args: &mut [HirExpr], span: crate::lexer::Span) {
        let required = sig.params.iter().filter(|param| !param.optional).count();
        let spread_compat =
            args.len() == 1 && sig.params.len() > 1 && self.check_spread_array_compat(sig, &mut args[0], span);
        if !spread_compat && (args.len() < required || args.len() > sig.params.len()) {
            self.error(SemanticErrorKind::WrongArity, "wrong number of arguments", span);
        }
        if spread_compat {
            return;
        }

        for (arg, param) in args.iter_mut().zip(sig.params.iter()) {
            let arg_ty = self.check_expr_with_expected(arg, Some(param.ty));
            self.unify(arg_ty, param.ty, arg.span, "argument type mismatch");
        }
    }
    /// Checks method-call syntax after the receiver has been synthesized.
    ///
    /// Declared/interface methods are preferred. The array branch preserves a
    /// small built-in collection-method surface until stdlib-backed method
    /// metadata covers all call shapes. Unknown interface members are hard
    /// diagnostics; unknown concrete members recover with a fresh inference type
    /// to avoid cascading failures from incomplete receiver information.
    pub(super) fn check_method_call(&mut self, receiver: &mut HirExpr, name: Symbol, args: &mut [HirExpr]) -> TypeId {
        let receiver_ty = self.check_expr(receiver);
        if let Some(sig) = self.lookup_method_signature(receiver_ty, name) {
            self.check_call_args(&sig, args, receiver.span);
            if self.reject_failable_call(&sig, receiver.span) {
                return self.error_type;
            }
            return sig.ret;
        }

        let array_inner = match self.types.get(self.resolve_type(receiver_ty)) {
            Type::Array(inner) => Some(*inner),
            _ => None,
        };
        if let Some(inner) = array_inner {
            if args.is_empty() {
                return self.numerus_type();
            }
            if let [arg] = args {
                let arg_ty = self.check_expr(arg);
                if self
                    .function_signature_from_type(self.resolve_type(arg_ty))
                    .is_none()
                {
                    self.unify(arg_ty, inner, arg.span, "argument type mismatch");
                    return self.vacuum_type();
                }
            }
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.array(inner);
        }

        if matches!(
            self.types.get(self.resolve_type(receiver_ty)),
            Type::Primitive(Primitive::Ignotum)
        ) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        if self.interface_def_from_type(receiver_ty).is_some() {
            for arg in args {
                self.check_expr(arg);
            }
            self.error(SemanticErrorKind::UndefinedMember, "unknown method", receiver.span);
            return self.error_type;
        }

        for arg in args {
            self.check_expr(arg);
        }
        self.fresh_infer()
    }
    /// Checks direct call syntax, including enum variant construction.
    ///
    /// Variant calls are constructors whose result is the parent enum type.
    /// Ordinary calls then follow the same signature/inference/`ignotum`
    /// recovery contract as calls from precomputed callee types.
    pub(super) fn check_call(&mut self, callee: &mut HirExpr, args: &mut [HirExpr]) -> TypeId {
        if let HirExprKind::Path(def_id) = &callee.kind {
            if let Some(parent) = self.variant_parent.get(def_id).copied() {
                let fields = self.variant_fields.get(def_id).cloned().unwrap_or_default();
                if args.len() != fields.len() {
                    self.error(SemanticErrorKind::WrongArity, "wrong number of arguments", callee.span);
                }
                for (arg, field_ty) in args.iter_mut().zip(fields.iter()) {
                    let arg_ty = self.check_expr(arg);
                    self.unify(arg_ty, *field_ty, arg.span, "argument type mismatch");
                }
                return self.types.intern(Type::Enum(parent));
            }
        }

        let callee_ty = self.check_expr(callee);

        let resolved = self.resolve_type(callee_ty);
        if let Some(sig) = self.function_signature_from_type(resolved) {
            self.check_call_args(&sig, args, callee.span);
            if self.reject_failable_call(&sig, callee.span) {
                return self.error_type;
            }
            return self.resolve_type(sig.ret);
        }

        if self.is_infer(resolved) {
            let sig = self.build_call_signature(args);
            let func_ty = self.types.function(sig.clone());
            self.unify(resolved, func_ty, callee.span, "callee is not callable");
            self.check_call_args(&sig, args, callee.span);
            return sig.ret;
        }

        if matches!(self.types.get(resolved), Type::Primitive(Primitive::Ignotum)) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        self.error(SemanticErrorKind::NotCallable, "callee is not callable", callee.span);
        self.error_type
    }

    /// Enforces local handling for functions that can produce alternate exits.
    ///
    /// A local error sink consumes the failable call after unifying the thrown
    /// type with the handler's expectation. A function-level alternate-exit
    /// declaration is not enough for an ordinary call expression; propagation
    /// must be expressed by surrounding syntax instead of being inferred here.
    pub(super) fn reject_failable_call(&mut self, sig: &FuncSig, span: crate::lexer::Span) -> bool {
        let Some(err_ty) = sig.err else {
            return false;
        };

        if let Some(ErrorSink::Local(handler_ty)) = self.current_error {
            self.unify(err_ty, handler_ty, span, "handled failable call error type mismatch");
            return false;
        }

        self.error(
            SemanticErrorKind::InvalidOperandTypes,
            "failable call requires handling or propagation syntax",
            span,
        );
        true
    }
}
