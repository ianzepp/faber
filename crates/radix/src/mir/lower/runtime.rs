//! Runtime and intrinsic call lowering.
//!
//! This module is the MIR boundary for operations that are not plain local
//! function calls or primitive expressions. Provider imports, collection
//! methods, diagnostic output, string formatting, conversions, and panic all
//! lower to explicit MIR runtime/intrinsic calls. Ordinary direct calls stay as
//! definition calls, while enum variant calls are redirected to aggregate
//! construction.
//!
//! BOUNDARIES
//! ==========
//! - Indirect calls are not guessed into MIR callable values; they fail closed.
//! - Collection methods are recognized only through resolved receiver type,
//!   method name, and arity.
//! - Failable function calls require an active local `cape` handler so the MIR
//!   can encode both success and error CFG edges.
//! - Runtime calls with `vacuum` return type emit no destination; value-returning
//!   calls allocate a temp and return that temp as the expression operand.

use super::*;
use crate::hir::HirCallArg;

impl FunctionBuilder<'_> {
    /// Lower a method call into a provider or collection intrinsic.
    ///
    /// Provider method calls are recognized when the receiver path resolves to a
    /// provider import. Collection methods are deliberately name/type/arity
    /// gated so unsupported methods do not masquerade as generic runtime calls.
    pub(super) fn lower_method_call(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirCallArg],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        if let Some(target) = self.genus_method_target(receiver, method) {
            let mut lowered_args = Vec::with_capacity(args.len() + 1);
            lowered_args.push(self.lower_expr_value(receiver)?);
            for arg in args {
                lowered_args.push(self.lower_expr_value(&arg.expr)?);
            }
            return self.lower_definition_call(target.def_id, lowered_args, expr);
        }

        if let HirExprKind::Path(def_id) = receiver.kind {
            if let Some(import) = self.context.provider_imports.get(&def_id).cloned() {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr_value(&arg.expr)?);
                }
                let ty = self.expr_ty(expr)?;
                let mut module = import.module;
                module.push(import.item);
                return Some(self.runtime_call_value(
                    MirIntrinsic::Provider(MirProvider { module, name: method }),
                    lowered_args,
                    ty,
                    expr.span,
                ));
            }
        }

        let Some(op) = self.collection_method_op(receiver, method, args, expr.span) else {
            self.errors.push(MirError::unsupported(
                expr.span,
                "method call before runtime/provider MIR lowering",
            ));
            return None;
        };

        let mut lowered_args = Vec::with_capacity(args.len() + 1);
        lowered_args.push(self.lower_expr_value(receiver)?);
        for arg in args {
            lowered_args.push(self.lower_expr_value(&arg.expr)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Collection(op), lowered_args, ty, expr.span))
    }

    fn genus_method_target(&self, receiver: &HirExpr, method: Symbol) -> Option<MethodTarget> {
        let receiver_ty = receiver.ty?;
        let Type::Struct(def_id) = self.normalized_type(receiver_ty) else {
            return None;
        };
        self.context.method_targets.get(&(*def_id, method)).copied()
    }

    /// Lower a direct call, enum variant constructor, or locally handled
    /// failable call.
    ///
    /// Only path callees are supported in this MIR phase. Callable values and
    /// other indirect forms remain unsupported until MIR has a representation
    /// for dynamic call targets.
    pub(super) fn lower_call(&mut self, callee: &HirExpr, args: &[HirCallArg], expr: &HirExpr) -> Option<MirOperand> {
        let HirExprKind::Path(def_id) = &callee.kind else {
            self.errors.push(MirError::unsupported(
                callee.span,
                "indirect calls before callable-value MIR lowering",
            ));
            return None;
        };

        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = self.lower_expr_value(&arg.expr)?;
            lowered_args.push(arg);
        }

        let ty = self.expr_ty(expr)?;

        if let Some(import) = self.context.provider_imports.get(def_id).cloned() {
            return Some(self.runtime_call_value(
                MirIntrinsic::Provider(MirProvider { module: import.module, name: import.item }),
                lowered_args,
                ty,
                expr.span,
            ));
        }

        if self.context.variant_parents.contains_key(def_id) {
            let fields = self.variant_payload(*def_id, lowered_args);
            return Some(self.construct_temp(MirAggregateKind::EnumVariant(*def_id), fields, ty, expr.span));
        }

        self.lower_definition_call(*def_id, lowered_args, expr)
    }

    fn lower_definition_call(
        &mut self,
        def_id: DefId,
        lowered_args: Vec<MirOperand>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;

        if let Some(_err_ty) = self.context.function_errors.get(&def_id).copied() {
            let Some(handler) = self.handlers.last().cloned() else {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "failable call without an active local cape handler",
                ));
                return None;
            };

            let ok_block = self.fresh_block(expr.span);
            if self.is_vacuum(ty) {
                self.terminate_current(
                    MirTerminatorKind::TryCall {
                        destination: None,
                        callee: MirCallee::Definition(def_id),
                        args: lowered_args,
                        ok_block,
                        error_place: handler.error_place,
                        error_block: handler.error_block,
                    },
                    expr.span,
                );
                self.switch_to(ok_block);
                return Some(MirOperand::Constant(MirConstant::Unit));
            }

            let destination = self.push_temp(ty, expr.span);
            self.terminate_current(
                MirTerminatorKind::TryCall {
                    destination: Some(MirPlace::temp(destination)),
                    callee: MirCallee::Definition(def_id),
                    args: lowered_args,
                    ok_block,
                    error_place: handler.error_place,
                    error_block: handler.error_block,
                },
                expr.span,
            );
            self.switch_to(ok_block);
            return Some(MirOperand::Temp(destination));
        }

        if self.is_vacuum(ty) {
            self.append_stmt(MirStmt {
                kind: MirStmtKind::Call {
                    destination: None,
                    callee: MirCallee::Definition(def_id),
                    args: lowered_args,
                },
                span: expr.span,
            });
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        let destination = self.push_temp(ty, expr.span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::Call {
                destination: Some(MirPlace::temp(destination)),
                callee: MirCallee::Definition(def_id),
                args: lowered_args,
            },
            span: expr.span,
        });
        Some(MirOperand::Temp(destination))
    }

    /// Resolve a HIR method call into the small set of collection intrinsics
    /// currently represented in MIR.
    fn collection_method_op(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirCallArg],
        span: Span,
    ) -> Option<MirCollectionOp> {
        let Some(receiver_ty) = receiver.ty else {
            self.errors
                .push(MirError::missing_type(span, "collection method receiver"));
            return None;
        };
        let method_name = self.resolve_method_name(method)?;
        let receiver_ty = self.normalized_type(receiver_ty);
        let is_array = matches!(receiver_ty, Type::Array(_));
        let is_map = matches!(receiver_ty, Type::Map(_, _));
        let is_set = matches!(receiver_ty, Type::Set(_));
        let is_text = matches!(receiver_ty, Type::Primitive(Primitive::Textus));

        match method_name {
            "appende" | "adde" if args.len() == 1 && is_array => Some(MirCollectionOp::Append),
            "addita" if args.len() == 1 && is_array => Some(MirCollectionOp::AppendImmutable),
            "accipe" if args.len() == 1 && (is_array || is_map || is_text) => Some(MirCollectionOp::Index),
            "longitudo" if args.is_empty() && (is_array || is_map || is_set || is_text) => {
                Some(MirCollectionOp::Length)
            }
            "continet" if args.len() == 1 && (is_array || is_set || is_text) => Some(MirCollectionOp::Contains),
            "habet" if args.len() == 1 && is_map => Some(MirCollectionOp::Contains),
            "primus" if args.is_empty() && is_array => Some(MirCollectionOp::First),
            "ultimus" if args.is_empty() && is_array => Some(MirCollectionOp::Last),
            "inversa" if args.is_empty() && is_array => Some(MirCollectionOp::Reverse),
            "inverte" if args.is_empty() && is_array => Some(MirCollectionOp::ReverseInPlace),
            "ordinata" if args.is_empty() && is_array => Some(MirCollectionOp::Sort),
            "filtrata" | "mappata" | "map" => {
                self.errors.push(MirError::unsupported(
                    span,
                    "collection higher-order methods before callable-value MIR lowering",
                ));
                None
            }
            _ => None,
        }
    }

    fn resolve_method_name(&self, method: Symbol) -> Option<&str> {
        self.context
            .interner
            .map(|interner| interner.resolve(method))
    }

    /// Lower `mori`/panic into a runtime call followed by an unreachable
    /// terminator.
    pub(super) fn lower_mori(&mut self, value: &HirExpr, span: Span) -> Option<MirOperand> {
        let value = self.lower_expr_value(value)?;
        let numquam = MirType::semantic(self.types.primitive(Primitive::Numquam));
        self.append_stmt(MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: None,
                call: crate::mir::MirRuntimeCall {
                    intrinsic: crate::mir::MirIntrinsic::Panic,
                    args: vec![value],
                    return_ty: numquam,
                },
            },
            span,
        });
        self.terminate_current(MirTerminatorKind::Unreachable, span);
        None
    }

    /// Lower diagnostic output expressions to diagnostic intrinsics.
    pub(super) fn lower_scribe(&mut self, kind: HirScribeKind, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let intrinsic = MirIntrinsic::Diagnostic(mir_diagnostic_kind(kind));
        let return_ty = self.expr_ty(expr)?;
        for arg in args {
            let arg = self.lower_expr_value(arg)?;
            self.runtime_call_value(intrinsic.clone(), vec![arg], return_ty, expr.span);
        }
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    /// Lower `adfirma` into a target-neutral assertion runtime intrinsic.
    ///
    /// The condition has already been typechecked as `bivalens`; MIR validation
    /// rechecks that invariant so backend probes can trust the first operand.
    pub(super) fn lower_adfirma(
        &mut self,
        cond: &HirExpr,
        message: Option<&HirExpr>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let mut args = vec![self.lower_expr_value(cond)?];
        if let Some(message) = message {
            args.push(self.lower_expr_value(message)?);
        }
        let return_ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Assert, args, return_ty, expr.span))
    }

    /// Lower a string-template application to a format-string intrinsic.
    pub(super) fn lower_scriptum(&mut self, template: Symbol, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expr_value(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::FormatString { template }, lowered_args, ty, expr.span))
    }

    /// Lower runtime conversions, including optional fallback expressions.
    ///
    /// Aggregate `verte` construction is handled in `aggregate.rs`; this path
    /// represents conversion work that remains a runtime intrinsic.
    pub(super) fn lower_conversio(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        params: &[Symbol],
        fallback: Option<&HirExpr>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let source = self.lower_expr_value(source)?;
        let fallback = match fallback {
            Some(fallback) => Some(self.lower_expr_value(fallback)?),
            None => None,
        };
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(
            MirIntrinsic::Convert(MirConversion {
                flavor: MirConversionFlavor::Runtime,
                target_ty: MirType::semantic(target),
                params: params.to_vec(),
                fallback,
            }),
            vec![source],
            ty,
            expr.span,
        ))
    }

    /// Emit a runtime call and return the operand shape promised by its return
    /// type.
    ///
    /// This keeps runtime-call lowering consistent across providers,
    /// diagnostics, formatting, and conversions: `vacuum` calls are statements,
    /// while value calls allocate one destination temp.
    pub(super) fn runtime_call_value(
        &mut self,
        intrinsic: MirIntrinsic,
        args: Vec<MirOperand>,
        return_ty: MirType,
        span: Span,
    ) -> MirOperand {
        if self.is_vacuum(return_ty) {
            self.append_stmt(MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: None,
                    call: MirRuntimeCall { intrinsic, args, return_ty },
                },
                span,
            });
            return MirOperand::Constant(MirConstant::Unit);
        }

        let temp = self.push_temp(return_ty, span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: Some(MirPlace::temp(temp)),
                call: MirRuntimeCall { intrinsic, args, return_ty },
            },
            span,
        });
        MirOperand::Temp(temp)
    }
}

fn mir_diagnostic_kind(kind: HirScribeKind) -> MirDiagnosticKind {
    match kind {
        HirScribeKind::Nota => MirDiagnosticKind::Nota,
        HirScribeKind::Vide => MirDiagnosticKind::Vide,
        HirScribeKind::Mone => MirDiagnosticKind::Mone,
        HirScribeKind::Scribe => MirDiagnosticKind::Scribe,
    }
}
