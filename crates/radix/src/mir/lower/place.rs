//! Place and projection lowering for addressable MIR operands.
//!
//! This module decides when a HIR expression can be represented as a `MirPlace`
//! and when a value must first be materialized into a temporary. Field and index
//! access preserve addressability whenever the base resolves to a local, temp,
//! or nested projection; constants and value operands are assigned into temps so
//! later projections still have a stable MIR base.
//!
//! NULLABILITY
//! ===========
//! Optional chaining and non-null access are lowered as MIR option operations.
//! This file does not infer nullability: it consumes the expression type
//! assigned by semantic typechecking and reports missing type information
//! through the shared lowering error path.

use super::*;

impl FunctionBuilder<'_> {
    /// Lower field access into a projected place when the receiver can be made
    /// addressable.
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

    /// Lower index access into a projected place, keeping the index as a MIR
    /// operand because it may itself require computation.
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

    /// Lower an optional-chain expression into an option runtime value.
    ///
    /// The base and chain link remain explicit so later codegen can preserve
    /// short-circuit semantics instead of seeing a plain field or call.
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
                    lowered_args.push(self.lower_expr_value(&arg.expr)?);
                }
                Some(MirOptionChainLink::Call { callee: MirCallee::Value(base), args: lowered_args })
            }
        }
    }

    /// Lower non-null member/index access by asserting the base option into a
    /// temp-backed place before applying the requested projection.
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
