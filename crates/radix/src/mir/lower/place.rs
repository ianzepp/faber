use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn lower_field(
        &mut self,
        object: &HirExpr,
        name: crate::lexer::Symbol,
        _expr: &HirExpr,
    ) -> Option<MirOperand> {
        let mut place = self.lower_projectable_place(object)?;
        place.projections.push(MirProjection::Field(name));
        Some(MirOperand::Place(place))
    }

    pub(super) fn lower_index(&mut self, object: &HirExpr, index: &HirExpr, _expr: &HirExpr) -> Option<MirOperand> {
        let mut place = self.lower_projectable_place(object)?;
        let index = self.lower_expr_value(index)?;
        place.projections.push(MirProjection::Index(index));
        Some(MirOperand::Place(place))
    }

    fn lower_projectable_place(&mut self, expr: &HirExpr) -> Option<MirPlace> {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                let Some(binding) = self.bindings.get(def_id).copied() else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "projection base that does not resolve to a local value",
                    ));
                    return None;
                };
                Some(MirPlace::local(binding.local))
            }
            HirExprKind::Field(object, name) => {
                let mut place = self.lower_projectable_place(object)?;
                place.projections.push(MirProjection::Field(*name));
                Some(place)
            }
            HirExprKind::Index(object, index) => {
                let mut place = self.lower_projectable_place(object)?;
                let index = self.lower_expr_value(index)?;
                place.projections.push(MirProjection::Index(index));
                Some(place)
            }
            _ => {
                let ty = self.expr_ty(expr)?;
                let operand = self.lower_expr_value(expr)?;
                match operand {
                    MirOperand::Place(place) => Some(place),
                    MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
                    MirOperand::Constant(_) | MirOperand::Value(_) => {
                        let temp = self.assign_temp(MirValueKind::Operand(operand), ty, expr.span);
                        match temp {
                            MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
                            _ => unreachable!("assign_temp always returns a temp operand"),
                        }
                    }
                }
            }
        }
    }

    pub(super) fn lower_optional_chain(
        &mut self,
        object: &HirExpr,
        chain: &HirOptionalChainKind,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let base = self.lower_expr_value(object)?;
        let link = self.lower_optional_chain_link(base.clone(), chain)?;
        let ty = self.expr_ty(expr)?;
        Some(self.assign_temp(MirValueKind::Option(MirOptionOp::Chain { base, link }), ty, expr.span))
    }

    fn lower_optional_chain_link(
        &mut self,
        base: MirOperand,
        chain: &HirOptionalChainKind,
    ) -> Option<MirOptionChainLink> {
        match chain {
            HirOptionalChainKind::Member(name) => Some(MirOptionChainLink::Field(*name)),
            HirOptionalChainKind::Index(index) => Some(MirOptionChainLink::Index(self.lower_expr_value(index)?)),
            HirOptionalChainKind::Call(args) => {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr_value(arg)?);
                }
                Some(MirOptionChainLink::Call { callee: MirCallee::Value(base), args: lowered_args })
            }
        }
    }

    pub(super) fn lower_non_null(
        &mut self,
        object: &HirExpr,
        chain: &HirNonNullKind,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let mut place = self.lower_non_null_base(object)?;
        match chain {
            HirNonNullKind::Member(name) => {
                place.projections.push(MirProjection::Field(*name));
                Some(MirOperand::Place(place))
            }
            HirNonNullKind::Index(index) => {
                let index = self.lower_expr_value(index)?;
                place.projections.push(MirProjection::Index(index));
                Some(MirOperand::Place(place))
            }
            HirNonNullKind::Call(_) => {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "non-null calls before callable-value MIR lowering",
                ));
                None
            }
        }
    }

    fn lower_non_null_base(&mut self, object: &HirExpr) -> Option<MirPlace> {
        let value = self.lower_expr_value(object)?;
        let inner_ty = self
            .option_inner_ty(object)
            .unwrap_or(self.expr_ty(object)?);
        let temp = self.assign_temp(
            MirValueKind::Option(MirOptionOp::Unwrap { value, mode: MirOptionUnwrapMode::Assert }),
            inner_ty,
            object.span,
        );
        match temp {
            MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
            _ => unreachable!("assign_temp always returns a temp operand"),
        }
    }

    fn option_inner_ty(&mut self, expr: &HirExpr) -> Option<MirType> {
        let ty = expr.ty?;
        match self.types.get(ty) {
            Type::Option(inner) => Some(MirType::semantic(*inner)),
            _ => None,
        }
    }
}
