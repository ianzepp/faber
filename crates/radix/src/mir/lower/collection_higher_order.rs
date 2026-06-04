//! Higher-order collection method lowering (`filtrata`, `mappata`).
//!
//! Closure callbacks lower to synthetic MIR functions and collection loops that
//! reuse existing collection intrinsics (`length`, `index`, `append`).

use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn lower_filtrata(
        &mut self,
        receiver: &HirExpr,
        closure: &HirExpr,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let bivalens = MirType::semantic(self.types.primitive(Primitive::Bivalens));
        let predicate = self.lower_closure_function(closure, bivalens)?;
        self.lower_collection_transform(receiver, predicate, TransformKind::Filter, expr)
    }

    pub(super) fn lower_mappata(
        &mut self,
        receiver: &HirExpr,
        closure: &HirExpr,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let Some(receiver_ty) = receiver.ty else {
            self.errors
                .push(MirError::missing_type(receiver.span, "mappata receiver"));
            return None;
        };
        let Some(element_ty) = self.collection_element_ty(receiver_ty) else {
            self.errors.push(MirError::unsupported(
                receiver.span,
                "mappata receiver is not a supported collection type",
            ));
            return None;
        };
        let mapper = self.lower_closure_function(closure, element_ty)?;
        self.lower_collection_transform(receiver, mapper, TransformKind::Map, expr)
    }

    fn lower_closure_function(&mut self, closure: &HirExpr, expected_return: MirType) -> Option<MirFunctionId> {
        let HirExprKind::Clausura(params, ret_ty, err_ty, body) = &closure.kind else {
            self.errors.push(MirError::unsupported(
                closure.span,
                "collection callback must be a closure expression",
            ));
            return None;
        };
        if err_ty.is_some() {
            self.errors.push(MirError::unsupported(
                closure.span,
                "failable collection callbacks before callable-value MIR lowering",
            ));
            return None;
        }
        if params.len() != 1 {
            self.errors.push(MirError::unsupported(
                closure.span,
                "collection callbacks must take exactly one parameter",
            ));
            return None;
        }
        let param = &params[0];
        let return_ty = ret_ty.map(MirType::semantic).unwrap_or(expected_return);

        let nested = FunctionBuilder::for_function(
            self.types,
            None,
            Self::clone_context(&self.context),
            MirFunctionId(self.parent_function_id.0 + 1 + self.synthetic_functions.len() as u32),
        );
        let mut nested = nested;
        nested.add_param(param.def_id, param.name, param.ty, param.span);
        let blocks = match &body.kind {
            HirExprKind::Block(block) => nested.lower_body(block),
            _ => {
                let entry = nested.fresh_block(body.span);
                nested.switch_to(entry);
                if let Some(value) = nested.lower_expr_value(body) {
                    nested.terminate_current(MirTerminatorKind::Return(Some(value)), body.span);
                }
                nested.finish_blocks()
            }
        };
        self.errors.extend(nested.errors);
        if blocks.is_empty() {
            self.errors.push(MirError::unsupported(
                closure.span,
                "collection callback produced no MIR blocks",
            ));
            return None;
        }

        let id = MirFunctionId(self.parent_function_id.0 + 1 + self.synthetic_functions.len() as u32);
        self.synthetic_functions.push(MirFunction {
            id,
            source: None,
            name: None,
            params: nested.params,
            locals: nested.locals,
            temps: nested.temps,
            blocks,
            return_ty,
            error_ty: None,
            span: closure.span,
        });
        Some(id)
    }

    fn lower_collection_transform(
        &mut self,
        receiver: &HirExpr,
        callback: MirFunctionId,
        kind: TransformKind,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let Some(receiver_ty) = receiver.ty else {
            self.errors
                .push(MirError::missing_type(receiver.span, "collection transform receiver"));
            return None;
        };
        let normalized = self.normalized_type(receiver_ty);
        let item_ty = match normalized {
            Type::Array(item) => MirType::semantic(*item),
            _ => {
                self.errors.push(MirError::unsupported(
                    receiver.span,
                    "collection transform before array MIR lowering",
                ));
                return None;
            }
        };
        let collection_ty = MirType::semantic(receiver_ty);
        let result_ty = self.expr_ty(expr)?;
        let numerus = MirType::semantic(self.types.primitive(Primitive::Numerus));
        let bivalens = MirType::semantic(self.types.primitive(Primitive::Bivalens));
        let vacuum = MirType::semantic(self.types.primitive(Primitive::Vacuum));

        let collection = self.lower_expr_value(receiver)?;
        let collection = self.materialize_operand_place(collection, collection_ty, receiver.span);
        let result = self.construct_temp(
            MirAggregateKind::Array,
            MirAggregateFields::Ordered(Vec::new()),
            result_ty,
            expr.span,
        );
        let result = self.materialize_operand_place(result, result_ty, expr.span);

        let index = self.next_local_id();
        self.locals
            .push(MirLocalDecl { id: index, name: None, ty: numerus, mutable: true, span: expr.span });
        self.assign(
            MirPlace::local(index),
            MirOperand::Constant(MirConstant::Int(0)),
            numerus,
            expr.span,
        );

        let length = self.runtime_call_value(
            MirIntrinsic::Collection(MirCollectionOp::Length),
            vec![MirOperand::Place(collection.clone())],
            numerus,
            expr.span,
        );
        if !self.current_is_open() {
            return None;
        }

        let cond_id = self.fresh_block(expr.span);
        let body_id = self.fresh_block(expr.span);
        let append_id = self.fresh_block(expr.span);
        let increment_id = self.fresh_block(expr.span);
        let after_id = self.fresh_block(expr.span);
        self.terminate_current(MirTerminatorKind::Goto(cond_id), expr.span);

        self.switch_to(cond_id);
        let continues = self.binary_temp(
            MirBinOp::Lt,
            MirOperand::Place(MirPlace::local(index)),
            length,
            bivalens,
            expr.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch { condition: continues, then_block: body_id, else_block: after_id },
            expr.span,
        );

        self.switch_to(body_id);
        let mut item_place = collection.clone();
        item_place
            .projections
            .push(MirProjection::Index(MirOperand::Place(MirPlace::local(index))));
        let item = MirOperand::Place(item_place);
        let callback_return = match kind {
            TransformKind::Filter => bivalens,
            TransformKind::Map => item_ty,
        };
        let mapped = self.call_function_value(callback, vec![item.clone()], callback_return, expr.span);
        if !self.current_is_open() {
            return None;
        }

        match kind {
            TransformKind::Filter => {
                self.terminate_current(
                    MirTerminatorKind::Branch { condition: mapped, then_block: append_id, else_block: increment_id },
                    expr.span,
                );
                self.switch_to(append_id);
                self.runtime_call_value(
                    MirIntrinsic::Collection(MirCollectionOp::Append),
                    vec![MirOperand::Place(result.clone()), item],
                    vacuum,
                    expr.span,
                );
            }
            TransformKind::Map => {
                self.terminate_current(MirTerminatorKind::Goto(append_id), expr.span);
                self.switch_to(append_id);
                self.runtime_call_value(
                    MirIntrinsic::Collection(MirCollectionOp::Append),
                    vec![MirOperand::Place(result.clone()), mapped],
                    vacuum,
                    expr.span,
                );
            }
        }
        if !self.current_is_open() {
            return None;
        }
        self.terminate_open_current(MirTerminatorKind::Goto(increment_id), expr.span);

        self.switch_to(increment_id);
        let next = self.binary_temp(
            MirBinOp::Add,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Constant(MirConstant::Int(1)),
            numerus,
            expr.span,
        );
        self.assign(MirPlace::local(index), next, numerus, expr.span);
        self.terminate_current(MirTerminatorKind::Goto(cond_id), expr.span);

        self.switch_to(after_id);
        Some(MirOperand::Place(result))
    }

    fn call_function_value(
        &mut self,
        callee: MirFunctionId,
        args: Vec<MirOperand>,
        return_ty: MirType,
        span: Span,
    ) -> MirOperand {
        if self.is_vacuum(return_ty) {
            self.append_stmt(MirStmt {
                kind: MirStmtKind::Call { destination: None, callee: MirCallee::Function(callee), args },
                span,
            });
            return MirOperand::Constant(MirConstant::Unit);
        }

        let destination = self.push_temp(return_ty, span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::Call {
                destination: Some(MirPlace::temp(destination)),
                callee: MirCallee::Function(callee),
                args,
            },
            span,
        });
        MirOperand::Temp(destination)
    }

    fn collection_element_ty(&self, ty: TypeId) -> Option<MirType> {
        match self.normalized_type(ty) {
            Type::Array(item) => Some(MirType::semantic(*item)),
            _ => None,
        }
    }

    fn clone_context<'a>(context: &'a FunctionBuilderContext<'a>) -> FunctionBuilderContext<'a> {
        FunctionBuilderContext {
            interner: context.interner,
            function_errors: context.function_errors.clone(),
            structs: context.structs.clone(),
            variant_parents: context.variant_parents.clone(),
            variant_fields: context.variant_fields.clone(),
            variant_field_tys: context.variant_field_tys.clone(),
            provider_imports: context.provider_imports.clone(),
            method_targets: context.method_targets.clone(),
        }
    }

    pub(super) fn take_synthetic_functions(&mut self) -> Vec<MirFunction> {
        std::mem::take(&mut self.synthetic_functions)
    }
}

#[derive(Debug, Clone, Copy)]
enum TransformKind {
    Filter,
    Map,
}
