use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn build_call_signature(&mut self, args: &mut [HirExpr]) -> FuncSig {
        let params = args
            .iter_mut()
            .map(|arg| ParamType { ty: self.check_expr(arg), mode: ParamMode::Owned, optional: false })
            .collect();
        FuncSig { params, ret: self.fresh_infer(), err: None, is_async: false, is_generator: false }
    }
    pub(super) fn function_signature_from_type(&self, ty: TypeId) -> Option<FuncSig> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Func(sig) => Some(sig.clone()),
            _ => None,
        }
    }
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
            let arg_ty = self.check_expr(arg);
            self.unify(arg_ty, param.ty, arg.span, "argument type mismatch");
        }
    }
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
