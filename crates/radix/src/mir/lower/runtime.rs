use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn lower_method_call(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirExpr],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        if let HirExprKind::Path(def_id) = receiver.kind {
            if let Some(import) = self.context.provider_imports.get(&def_id).cloned() {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr_value(arg)?);
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
            lowered_args.push(self.lower_expr_value(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Collection(op), lowered_args, ty, expr.span))
    }

    pub(super) fn lower_call(&mut self, callee: &HirExpr, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let HirExprKind::Path(def_id) = &callee.kind else {
            self.errors.push(MirError::unsupported(
                callee.span,
                "indirect calls before callable-value MIR lowering",
            ));
            return None;
        };

        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = self.lower_expr_value(arg)?;
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

        if let Some(_err_ty) = self.context.function_errors.get(def_id).copied() {
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
                        callee: MirCallee::Definition(*def_id),
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
                    callee: MirCallee::Definition(*def_id),
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
                    callee: MirCallee::Definition(*def_id),
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
                callee: MirCallee::Definition(*def_id),
                args: lowered_args,
            },
            span: expr.span,
        });
        Some(MirOperand::Temp(destination))
    }

    fn collection_method_op(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirExpr],
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
            _ => None,
        }
    }

    fn resolve_method_name(&self, method: Symbol) -> Option<&str> {
        self.context
            .interner
            .map(|interner| interner.resolve(method))
    }
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

    pub(super) fn lower_scribe(&mut self, kind: HirScribeKind, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expr_value(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Diagnostic(mir_diagnostic_kind(kind)), lowered_args, ty, expr.span))
    }

    pub(super) fn lower_scriptum(&mut self, template: Symbol, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expr_value(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::FormatString { template }, lowered_args, ty, expr.span))
    }

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
    fn runtime_call_value(
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
