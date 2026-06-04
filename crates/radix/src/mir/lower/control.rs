//! Control-flow and alternate-exit lowering for MIR functions.
//!
//! This module turns expression-level control constructs into explicit MIR
//! blocks and terminators. It owns the contracts for `si`, `dum`, `rumpe`,
//! `perge`, `iace`, and `cape`: each construct either wires concrete CFG edges
//! or reports an unsupported lowering diagnostic. Value-producing control flow
//! writes into a destination place so branch arms join through storage rather
//! than through an implicit expression stack.
//!
//! ERROR HANDLING
//! ==============
//! Local `cape` handlers are modeled as a stack of handler contexts. `iace` and
//! failable calls assign the error value into the active handler place and jump
//! to its handler block. Without a local handler, `iace` can only lower for
//! functions with a declared alternate-exit type; otherwise it fails closed.
//!
//! CFG INVARIANTS
//! ==============
//! - Branch and loop helpers create all successor blocks before terminating the
//!   current block.
//! - Reachable arms jump to an explicit join block.
//! - Unreachable joins are sealed with `Unreachable` so MIR validation sees a
//!   terminator on every block.

use super::*;
use crate::hir::visit::HirVisitor;

struct RangeIteraLowering<'a> {
    binding: DefId,
    name: Symbol,
    start: &'a HirExpr,
    end: &'a HirExpr,
    step: Option<&'a HirExpr>,
    kind: HirRangeKind,
    body: &'a HirBlock,
    span: Span,
}

struct ArrayIteraLowering<'a> {
    mode: HirIteraMode,
    binding: DefId,
    name: Symbol,
    iter: &'a HirExpr,
    body: &'a HirBlock,
    span: Span,
}

impl FunctionBuilder<'_> {
    /// Lower `iace` either to the innermost active `cape` handler or to a
    /// function-level alternate return.
    pub(super) fn lower_iace(&mut self, value: &HirExpr, span: Span) -> Option<MirOperand> {
        if let Some(handler) = self.handlers.last().cloned() {
            let error_ty = self.expr_ty(value)?;
            let value = self.lower_transfer_expr(value)?;
            let place = handler.error_place;
            self.assign(place, value, error_ty, span);
            self.terminate_current(MirTerminatorKind::Goto(handler.error_block), span);
            return None;
        }

        if self.error_ty.is_none() {
            self.errors
                .push(MirError::unsupported(span, "iace without a declared alternate-exit type"));
            return None;
        }

        let value = self.lower_transfer_expr(value)?;
        self.terminate_current(MirTerminatorKind::ReturnError(value), span);
        None
    }

    /// Lower a handled expression when the handler is statement-like.
    ///
    /// Expression-valued handlers are intentionally rejected until MIR has a
    /// value-join representation for handler results.
    pub(super) fn lower_handled_expr(
        &mut self,
        body: &HirBlock,
        catch: &HirCape,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors.push(MirError::unsupported(
                expr.span,
                "expression-valued cape handler before value-join MIR lowering",
            ));
            return None;
        }

        self.lower_handled_block(body, catch, expr.span);
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    fn lower_handled_block(&mut self, body: &HirBlock, catch: &HirCape, span: Span) {
        let Some(error_ty) = catch.binding_ty.map(MirType::semantic) else {
            self.errors
                .push(MirError::missing_type(catch.span, "cape handler binding"));
            return;
        };

        let handler_id = self.fresh_block(catch.body.span);
        let after_id = self.fresh_block(span);
        let handler_local = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: handler_local,
            name: Some(catch.binding_name),
            ty: error_ty,
            mutable: false,
            span: catch.span,
        });
        let handler_binding = LocalBinding { local: handler_local, ty: error_ty };
        self.bindings.insert(catch.binding_def_id, handler_binding);

        self.handlers
            .push(HandlerContext { error_place: MirPlace::local(handler_local), error_block: handler_id });
        self.visit_block(body);
        self.handlers.pop();
        let body_reaches = self.terminate_open_current(MirTerminatorKind::Goto(after_id), span);

        self.switch_to(handler_id);
        self.visit_block(&catch.body);
        let handler_reaches = self.terminate_open_current(MirTerminatorKind::Goto(after_id), catch.span);

        if body_reaches || handler_reaches {
            self.switch_to(after_id);
        } else {
            self.seal_unreachable(after_id, span);
        }
    }

    /// Lower any expression into an already-chosen destination place.
    ///
    /// This is the join-point contract for expression-valued blocks and `si`:
    /// complex control forms assign their eventual value into `destination`,
    /// while ordinary expressions lower to an operand and are assigned once.
    pub(super) fn lower_expr_to_destination(
        &mut self,
        expr: &HirExpr,
        destination: MirPlace,
        ty: MirType,
    ) -> Option<()> {
        match &expr.kind {
            HirExprKind::Block(block) => self.lower_block_to_destination(block, destination, ty, expr.span),
            HirExprKind::Si { cond, then_block, then_catch, else_block } => {
                if then_catch.is_some() {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "expression-valued si with cape before handler value MIR lowering",
                    ));
                    return None;
                }
                self.lower_si_to_destination(cond, then_block, else_block.as_ref(), destination, ty, expr.span)
            }
            _ => {
                let value = self.lower_expr_value(expr)?;
                self.assign(destination, value, ty, expr.span);
                Some(())
            }
        }
    }

    /// Lower a HIR block used as an expression.
    ///
    /// Non-`vacuum` blocks allocate a temp and route the tail expression into
    /// it. Blocks with no tail expression are only valid when the expected MIR
    /// type is `vacuum`.
    pub(super) fn lower_block_expr(&mut self, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if self.is_vacuum(ty) {
            self.visit_block(block);
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        let temp = self.push_temp(ty, expr.span);
        self.lower_block_to_destination(block, MirPlace::temp(temp), ty, expr.span)?;
        Some(MirOperand::Temp(temp))
    }

    /// Lower `si` in expression position.
    ///
    /// `vacuum` branches are statement-like. Value-producing branches require
    /// a `secus` block and share a temp destination so the two arms join through
    /// ordinary MIR storage.
    pub(super) fn lower_si_expr(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: &Option<HirBlock>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if self.is_vacuum(ty) {
            self.lower_si_statement(cond, then_block, then_catch, else_block.as_ref(), expr.span);
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        if then_catch.is_some() {
            self.errors.push(MirError::unsupported(
                expr.span,
                "expression-valued si with cape before handler value MIR lowering",
            ));
            return None;
        }

        let temp = self.push_temp(ty, expr.span);
        self.lower_si_to_destination(cond, then_block, else_block.as_ref(), MirPlace::temp(temp), ty, expr.span)?;
        Some(MirOperand::Temp(temp))
    }

    /// Lower `dum` in expression position.
    ///
    /// Loops currently have a statement-like MIR contract: they may produce
    /// `vacuum`, but a non-`vacuum` loop result is rejected until the language
    /// has a defined value-yielding loop model.
    pub(super) fn lower_dum_expr(&mut self, cond: &HirExpr, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors
                .push(MirError::unsupported(expr.span, "dum expression with non-vacuum result"));
            return None;
        }

        self.lower_dum(cond, block, expr.span);
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    /// Lower supported `itera` subsets into target-neutral MIR.
    ///
    /// Numeric ranges and array-backed collection loops can use ordinary locals,
    /// branches, gotos, runtime length calls, and index projections. Map, set,
    /// text, and cursor iteration still need iterator/runtime ABI policy.
    pub(super) fn lower_itera_expr(
        &mut self,
        mode: HirIteraMode,
        binding: DefId,
        name: Symbol,
        iter: &HirExpr,
        body: &HirBlock,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors
                .push(MirError::unsupported(expr.span, "itera expression with non-vacuum result"));
            return None;
        }
        match mode {
            HirIteraMode::Ab => {
                let HirExprKind::Intervallum { start, end, step, kind } = &iter.kind else {
                    self.errors
                        .push(MirError::unsupported(iter.span, "itera ab source before range MIR lowering"));
                    return None;
                };

                self.lower_range_itera(RangeIteraLowering {
                    binding,
                    name,
                    start,
                    end,
                    step: step.as_deref(),
                    kind: *kind,
                    body,
                    span: expr.span,
                });
            }
            HirIteraMode::Ex | HirIteraMode::De => {
                self.lower_array_itera(ArrayIteraLowering { mode, binding, name, iter, body, span: expr.span })?;
            }
        }
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    /// Lower the literal `elige` subset into a MIR switch.
    ///
    /// HIR represents `elige` as `Discerne` with one scrutinee and literal
    /// patterns. Destructuring, enum variants, guards, multi-subject matching,
    /// and value-producing matches remain explicit later-phase work.
    pub(super) fn lower_discerne_expr(
        &mut self,
        scrutinees: &[HirExpr],
        arms: &[HirCasuArm],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors.push(MirError::unsupported(
                expr.span,
                "value-producing discerne before switch MIR lowering",
            ));
            return None;
        }
        let [scrutinee] = scrutinees else {
            self.errors.push(MirError::unsupported(
                expr.span,
                "multi-subject discerne before switch MIR lowering",
            ));
            return None;
        };

        let value = self.lower_expr_value(scrutinee)?;
        let join_id = self.fresh_block(expr.span);
        let mut cases = Vec::new();
        let mut bodies = Vec::new();
        let mut default_id = join_id;
        let mut default_body = None;

        for arm in arms {
            if arm.guard.is_some() {
                self.errors
                    .push(MirError::unsupported(arm.span, "guarded discerne before switch MIR lowering"));
                return None;
            }
            let [pattern] = arm.patterns.as_slice() else {
                self.errors.push(MirError::unsupported(
                    arm.span,
                    "multi-pattern discerne arm before switch MIR lowering",
                ));
                return None;
            };
            match pattern {
                HirPattern::Literal(literal) => {
                    let constant = self.literal_switch_constant(literal, arm.span)?;
                    let target = self.fresh_block(arm.span);
                    cases.push(MirSwitchCase { value: constant, target });
                    bodies.push((target, &arm.body, arm.span));
                }
                HirPattern::Wildcard if default_body.is_none() => {
                    default_id = self.fresh_block(arm.span);
                    default_body = Some((&arm.body, arm.span));
                }
                HirPattern::Wildcard => {
                    self.errors.push(MirError::unsupported(
                        arm.span,
                        "multiple discerne defaults before switch MIR lowering",
                    ));
                    return None;
                }
                _ => {
                    return self.lower_unit_variant_discerne(value, scrutinee, arms, expr);
                }
            }
        }

        self.terminate_current(MirTerminatorKind::Switch { value, cases, default: default_id }, expr.span);

        let mut reaches_join = default_body.is_none();

        for (target, body, span) in bodies {
            self.switch_to(target);
            let _ = self.lower_expr_value(body);
            reaches_join |= self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);
        }

        if let Some((body, span)) = default_body {
            self.switch_to(default_id);
            let _ = self.lower_expr_value(body);
            reaches_join |= self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);
        }

        if reaches_join {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, expr.span);
        }
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    fn lower_unit_variant_discerne(
        &mut self,
        value: MirOperand,
        scrutinee: &HirExpr,
        arms: &[HirCasuArm],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let scrutinee_ty = self.expr_ty(scrutinee)?;
        let bool_ty = MirType::semantic(self.types.primitive(Primitive::Bivalens));
        let scrutinee_place = self.materialize_operand_place(value.clone(), scrutinee_ty, scrutinee.span);
        let join_id = self.fresh_block(expr.span);
        let mut default_body = None;

        for arm in arms {
            if arm.guard.is_some() {
                self.errors
                    .push(MirError::unsupported(arm.span, "guarded discerne before switch MIR lowering"));
                return None;
            }
            let [pattern] = arm.patterns.as_slice() else {
                self.errors.push(MirError::unsupported(
                    arm.span,
                    "multi-pattern discerne arm before switch MIR lowering",
                ));
                return None;
            };
            match pattern {
                HirPattern::Variant(variant, fields) => {
                    let then_id = self.fresh_block(arm.span);
                    let else_id = self.fresh_block(arm.span);
                    let fields = if fields.is_empty() {
                        MirAggregateFields::Ordered(Vec::new())
                    } else {
                        self.variant_pattern_probe_fields(*variant, &scrutinee_place, arm.span)?
                    };
                    let variant =
                        self.construct_temp(MirAggregateKind::EnumVariant(*variant), fields, scrutinee_ty, arm.span);
                    let condition = self.assign_temp(
                        MirValueKind::Binary { op: MirBinOp::Eq, lhs: value.clone(), rhs: variant },
                        bool_ty,
                        arm.span,
                    );
                    self.terminate_current(
                        MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
                        arm.span,
                    );

                    self.switch_to(then_id);
                    self.bind_variant_payload(pattern, &scrutinee_place, scrutinee_ty, arm.span)?;
                    let _ = self.lower_expr_value(&arm.body);
                    self.terminate_open_current(MirTerminatorKind::Goto(join_id), arm.span);

                    self.switch_to(else_id);
                }
                HirPattern::Alias(_, _, inner) if matches!(inner.as_ref(), HirPattern::Variant(_, _)) => {
                    let then_id = self.fresh_block(arm.span);
                    let else_id = self.fresh_block(arm.span);
                    let HirPattern::Alias(_, _, inner) = pattern else {
                        unreachable!("alias pattern shape was matched");
                    };
                    let HirPattern::Variant(variant, fields) = inner.as_ref() else {
                        unreachable!("inner variant pattern shape was matched");
                    };
                    let fields = if fields.is_empty() {
                        MirAggregateFields::Ordered(Vec::new())
                    } else {
                        self.variant_pattern_probe_fields(*variant, &scrutinee_place, arm.span)?
                    };
                    let variant =
                        self.construct_temp(MirAggregateKind::EnumVariant(*variant), fields, scrutinee_ty, arm.span);
                    let condition = self.assign_temp(
                        MirValueKind::Binary { op: MirBinOp::Eq, lhs: value.clone(), rhs: variant },
                        bool_ty,
                        arm.span,
                    );
                    self.terminate_current(
                        MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
                        arm.span,
                    );

                    self.switch_to(then_id);
                    self.bind_variant_payload(pattern, &scrutinee_place, scrutinee_ty, arm.span)?;
                    let _ = self.lower_expr_value(&arm.body);
                    self.terminate_open_current(MirTerminatorKind::Goto(join_id), arm.span);

                    self.switch_to(else_id);
                }
                HirPattern::Wildcard if default_body.is_none() => {
                    default_body = Some((&arm.body, arm.span));
                    break;
                }
                HirPattern::Wildcard => {
                    self.errors.push(MirError::unsupported(
                        arm.span,
                        "multiple discerne defaults before switch MIR lowering",
                    ));
                    return None;
                }
                _ => {
                    self.errors.push(MirError::unsupported(
                        arm.span,
                        "non-literal discerne pattern before switch MIR lowering",
                    ));
                    return None;
                }
            }
        }

        let mut reaches_join = false;
        if let Some((body, span)) = default_body {
            let _ = self.lower_expr_value(body);
            reaches_join |= self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);
        } else {
            if let Some(current) = self.current {
                self.seal_unreachable(current, expr.span);
            }
        }

        if reaches_join {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, expr.span);
        }
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    fn variant_pattern_probe_fields(
        &mut self,
        variant: DefId,
        scrutinee: &MirPlace,
        span: Span,
    ) -> Option<MirAggregateFields> {
        let Some(field_names) = self.context.variant_fields.get(&variant).cloned() else {
            self.errors
                .push(MirError::unsupported(span, "variant payload pattern is missing field metadata"));
            return None;
        };

        Some(MirAggregateFields::Named(
            field_names
                .into_iter()
                .map(|field| {
                    let mut place = scrutinee.clone();
                    place
                        .projections
                        .push(MirProjection::VariantField { variant, field });
                    MirNamedOperand { name: field, value: MirOperand::Place(place) }
                })
                .collect(),
        ))
    }

    fn bind_variant_payload(
        &mut self,
        pattern: &HirPattern,
        scrutinee: &MirPlace,
        scrutinee_ty: MirType,
        span: Span,
    ) -> Option<()> {
        match pattern {
            HirPattern::Variant(variant, fields) => self.bind_variant_fields(*variant, fields, scrutinee, span),
            HirPattern::Alias(alias_def, alias_name, inner) => {
                let alias = self.next_local_id();
                self.locals.push(MirLocalDecl {
                    id: alias,
                    name: Some(*alias_name),
                    ty: scrutinee_ty,
                    mutable: false,
                    span,
                });
                self.bindings
                    .insert(*alias_def, LocalBinding { local: alias, ty: scrutinee_ty });
                self.assign(MirPlace::local(alias), MirOperand::Place(scrutinee.clone()), scrutinee_ty, span);
                self.bind_variant_payload(inner, scrutinee, scrutinee_ty, span)
            }
            _ => {
                self.errors
                    .push(MirError::unsupported(span, "non-variant discerne payload binding"));
                None
            }
        }
    }

    fn bind_variant_fields(
        &mut self,
        variant: DefId,
        fields: &[HirPattern],
        scrutinee: &MirPlace,
        span: Span,
    ) -> Option<()> {
        let Some(field_names) = self.context.variant_fields.get(&variant).cloned() else {
            self.errors
                .push(MirError::unsupported(span, "variant payload pattern is missing field metadata"));
            return None;
        };
        let Some(field_tys) = self.context.variant_field_tys.get(&variant).cloned() else {
            self.errors
                .push(MirError::missing_type(span, "variant payload pattern fields"));
            return None;
        };
        if fields.len() != field_names.len() {
            self.errors
                .push(MirError::unsupported(span, "variant payload pattern arity mismatch"));
            return None;
        }

        for (field, subpattern) in field_names.into_iter().zip(fields) {
            match subpattern {
                HirPattern::Binding(def_id, name) => {
                    let Some(ty) = field_tys.get(&field).copied() else {
                        self.errors
                            .push(MirError::missing_type(span, "variant payload pattern field"));
                        return None;
                    };
                    let local = self.next_local_id();
                    self.locals
                        .push(MirLocalDecl { id: local, name: Some(*name), ty, mutable: false, span });
                    self.bindings.insert(*def_id, LocalBinding { local, ty });
                    let mut place = scrutinee.clone();
                    place
                        .projections
                        .push(MirProjection::VariantField { variant, field });
                    self.assign(MirPlace::local(local), MirOperand::Place(place), ty, span);
                }
                HirPattern::Wildcard => {}
                _ => {
                    self.errors.push(MirError::unsupported(
                        span,
                        "nested variant payload patterns before MIR lowering",
                    ));
                    return None;
                }
            }
        }
        Some(())
    }

    fn literal_switch_constant(&mut self, literal: &HirLiteral, span: Span) -> Option<MirConstant> {
        match literal {
            HirLiteral::Int(value) => Some(MirConstant::Int(*value)),
            HirLiteral::Float(value) => Some(MirConstant::Float(*value)),
            HirLiteral::String(symbol) => Some(MirConstant::String(*symbol)),
            HirLiteral::Bool(value) => Some(MirConstant::Bool(*value)),
            HirLiteral::Nil | HirLiteral::Regex(_, _) => {
                self.errors
                    .push(MirError::unsupported(span, "literal pattern before switch MIR lowering"));
                None
            }
        }
    }

    /// Lower a block into a destination place, preserving early terminators.
    ///
    /// Statement prefixes are emitted first. If they leave the current block
    /// sealed, no tail assignment is generated; otherwise the tail expression
    /// supplies the destination value.
    pub(super) fn lower_block_to_destination(
        &mut self,
        block: &HirBlock,
        destination: MirPlace,
        ty: MirType,
        span: Span,
    ) -> Option<()> {
        for stmt in &block.stmts {
            if !self.current_is_open() {
                return Some(());
            }
            self.visit_stmt(stmt);
        }

        if !self.current_is_open() {
            return Some(());
        }

        let Some(expr) = &block.expr else {
            if self.is_vacuum(ty) {
                self.assign(destination, MirOperand::Constant(MirConstant::Unit), ty, span);
                return Some(());
            }
            self.errors.push(MirError::unsupported(
                block.span,
                "expression-valued block without a tail expression",
            ));
            return None;
        };

        self.lower_expr_to_destination(expr, destination, ty)
    }

    /// Lower `si` when its result is used only for control effects.
    ///
    /// The helper creates an explicit branch and an explicit join even when no
    /// `secus` block exists; the missing else arm simply reaches the join.
    pub(super) fn lower_si_statement(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: Option<&HirBlock>,
        span: Span,
    ) {
        if let Some(catch) = then_catch {
            self.lower_handled_si_statement(cond, then_block, catch, else_block, span);
            return;
        }

        let Some(condition) = self.lower_expr_value(cond) else {
            return;
        };

        let then_id = self.fresh_block(then_block.span);
        let (else_id, join_id) = match else_block {
            Some(block) => {
                let else_id = self.fresh_block(block.span);
                let join_id = self.fresh_block(span);
                (else_id, join_id)
            }
            None => {
                let join_id = self.fresh_block(span);
                (join_id, join_id)
            }
        };

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.visit_block(then_block);
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        let else_reaches = if let Some(block) = else_block {
            self.switch_to(else_id);
            self.visit_block(block);
            self.terminate_open_current(MirTerminatorKind::Goto(join_id), span)
        } else {
            true
        };

        if then_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }
    }

    fn lower_handled_si_statement(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        catch: &HirCape,
        else_block: Option<&HirBlock>,
        span: Span,
    ) {
        let Some(error_ty) = catch.binding_ty.map(MirType::semantic) else {
            self.errors
                .push(MirError::missing_type(catch.span, "cape handler binding"));
            return;
        };

        let then_id = self.fresh_block(then_block.span);
        let (else_id, join_id) = match else_block {
            Some(block) => {
                let else_id = self.fresh_block(block.span);
                let join_id = self.fresh_block(span);
                (else_id, join_id)
            }
            None => {
                let join_id = self.fresh_block(span);
                (join_id, join_id)
            }
        };
        let handler_id = self.fresh_block(catch.body.span);

        let handler_local = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: handler_local,
            name: Some(catch.binding_name),
            ty: error_ty,
            mutable: false,
            span: catch.span,
        });
        self.bindings
            .insert(catch.binding_def_id, LocalBinding { local: handler_local, ty: error_ty });

        self.handlers
            .push(HandlerContext { error_place: MirPlace::local(handler_local), error_block: handler_id });
        let Some(condition) = self.lower_expr_value(cond) else {
            self.handlers.pop();
            self.seal_unreachable(handler_id, catch.span);
            self.switch_to(join_id);
            return;
        };

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.visit_block(then_block);
        self.handlers.pop();
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        self.switch_to(handler_id);
        self.visit_block(&catch.body);
        let handler_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), catch.span);

        let else_reaches = if let Some(block) = else_block {
            self.switch_to(else_id);
            self.visit_block(block);
            self.terminate_open_current(MirTerminatorKind::Goto(join_id), span)
        } else {
            true
        };

        if then_reaches || handler_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }
    }

    /// Lower expression-valued `si` into an existing destination.
    ///
    /// A missing `secus` branch is rejected because MIR has no implicit default
    /// value for an expression result.
    pub(super) fn lower_si_to_destination(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        else_block: Option<&HirBlock>,
        destination: MirPlace,
        ty: MirType,
        span: Span,
    ) -> Option<()> {
        let Some(else_block) = else_block else {
            self.errors
                .push(MirError::unsupported(span, "expression-valued si without secus destination"));
            return None;
        };

        let condition = self.lower_expr_value(cond)?;
        let then_id = self.fresh_block(then_block.span);
        let else_id = self.fresh_block(else_block.span);
        let join_id = self.fresh_block(span);

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.lower_block_to_destination(then_block, destination.clone(), ty, then_block.span)?;
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        self.switch_to(else_id);
        self.lower_block_to_destination(else_block, destination, ty, else_block.span)?;
        let else_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        if then_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }

        Some(())
    }

    /// Lower `dum` into condition, body, and after blocks.
    ///
    /// `perge` jumps back to the condition block and `rumpe` jumps to the after
    /// block through the loop context pushed while lowering the body.
    pub(super) fn lower_dum(&mut self, cond: &HirExpr, body: &HirBlock, span: Span) {
        let cond_id = self.fresh_block(cond.span);
        let body_id = self.fresh_block(body.span);
        let after_id = self.fresh_block(span);

        self.terminate_current(MirTerminatorKind::Goto(cond_id), span);

        self.switch_to(cond_id);
        let Some(condition) = self.lower_expr_value(cond) else {
            self.seal_unreachable(cond_id, cond.span);
            self.switch_to(after_id);
            return;
        };
        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: body_id, else_block: after_id },
            cond.span,
        );

        self.loops
            .push(LoopContext { perge_target: cond_id, rumpe_target: after_id });
        self.switch_to(body_id);
        self.visit_block(body);
        self.loops.pop();
        self.terminate_open_current(MirTerminatorKind::Goto(cond_id), span);

        self.switch_to(after_id);
    }

    fn lower_range_itera(&mut self, range: RangeIteraLowering<'_>) {
        let numerus = MirType::semantic(self.types.primitive(Primitive::Numerus));
        let bivalens = MirType::semantic(self.types.primitive(Primitive::Bivalens));

        let index = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: index,
            name: Some(range.name),
            ty: numerus,
            mutable: true,
            span: range.span,
        });
        self.bindings
            .insert(range.binding, LocalBinding { local: index, ty: numerus });

        let end_local = self.next_local_id();
        self.locals
            .push(MirLocalDecl { id: end_local, name: None, ty: numerus, mutable: false, span: range.span });
        let step_local = self.next_local_id();
        self.locals
            .push(MirLocalDecl { id: step_local, name: None, ty: numerus, mutable: false, span: range.span });

        self.lower_expr_to_destination(range.start, MirPlace::local(index), numerus);
        self.lower_expr_to_destination(range.end, MirPlace::local(end_local), numerus);
        if !self.current_is_open() {
            return;
        }

        let cond_id = self.fresh_block(range.span);
        match range.step {
            Some(step) => {
                self.lower_expr_to_destination(step, MirPlace::local(step_local), numerus);
                self.terminate_open_current(MirTerminatorKind::Goto(cond_id), range.span);
            }
            None => self.lower_default_range_step(index, end_local, step_local, cond_id, bivalens, range.span),
        }

        let positive_check_id = self.fresh_block(range.span);
        let negative_or_zero_id = self.fresh_block(range.span);
        let negative_check_id = self.fresh_block(range.span);
        let body_id = self.fresh_block(range.body.span);
        let increment_id = self.fresh_block(range.span);
        let after_id = self.fresh_block(range.span);

        self.switch_to(cond_id);
        let step_is_positive = self.binary_temp(
            MirBinOp::Gt,
            MirOperand::Place(MirPlace::local(step_local)),
            MirOperand::Constant(MirConstant::Int(0)),
            bivalens,
            range.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch {
                condition: step_is_positive,
                then_block: positive_check_id,
                else_block: negative_or_zero_id,
            },
            range.span,
        );

        self.switch_to(positive_check_id);
        let positive_op = match range.kind {
            HirRangeKind::Exclusive => MirBinOp::Lt,
            HirRangeKind::Inclusive => MirBinOp::LtEq,
        };
        let positive_continues = self.binary_temp(
            positive_op,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Place(MirPlace::local(end_local)),
            bivalens,
            range.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch { condition: positive_continues, then_block: body_id, else_block: after_id },
            range.span,
        );

        self.switch_to(negative_or_zero_id);
        let step_is_negative = self.binary_temp(
            MirBinOp::Lt,
            MirOperand::Place(MirPlace::local(step_local)),
            MirOperand::Constant(MirConstant::Int(0)),
            bivalens,
            range.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch {
                condition: step_is_negative,
                then_block: negative_check_id,
                else_block: after_id,
            },
            range.span,
        );

        self.switch_to(negative_check_id);
        let negative_op = match range.kind {
            HirRangeKind::Exclusive => MirBinOp::Gt,
            HirRangeKind::Inclusive => MirBinOp::GtEq,
        };
        let negative_continues = self.binary_temp(
            negative_op,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Place(MirPlace::local(end_local)),
            bivalens,
            range.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch { condition: negative_continues, then_block: body_id, else_block: after_id },
            range.span,
        );

        self.loops
            .push(LoopContext { perge_target: increment_id, rumpe_target: after_id });
        self.switch_to(body_id);
        self.visit_block(range.body);
        self.loops.pop();
        self.terminate_open_current(MirTerminatorKind::Goto(increment_id), range.span);

        self.switch_to(increment_id);
        let next = self.binary_temp(
            MirBinOp::Add,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Place(MirPlace::local(step_local)),
            numerus,
            range.span,
        );
        self.assign(MirPlace::local(index), next, numerus, range.span);
        self.terminate_current(MirTerminatorKind::Goto(cond_id), range.span);

        self.switch_to(after_id);
    }

    fn lower_array_itera(&mut self, iter: ArrayIteraLowering<'_>) -> Option<()> {
        let Some(iter_ty) = iter.iter.ty else {
            self.errors
                .push(MirError::missing_type(iter.iter.span, "itera collection source"));
            return None;
        };
        let item_ty = match self.normalized_type(iter_ty) {
            Type::Array(item) => MirType::semantic(*item),
            _ => {
                self.errors.push(MirError::unsupported(
                    iter.iter.span,
                    "itera collection before iterator MIR lowering",
                ));
                return None;
            }
        };

        let numerus = MirType::semantic(self.types.primitive(Primitive::Numerus));
        let bivalens = MirType::semantic(self.types.primitive(Primitive::Bivalens));
        let collection_ty = MirType::semantic(iter_ty);
        let collection = self.lower_expr_value(iter.iter)?;
        let collection = self.materialize_operand_place(collection, collection_ty, iter.iter.span);

        let index = self.next_local_id();
        self.locals
            .push(MirLocalDecl { id: index, name: None, ty: numerus, mutable: true, span: iter.span });
        self.assign(
            MirPlace::local(index),
            MirOperand::Constant(MirConstant::Int(0)),
            numerus,
            iter.span,
        );

        let length = self.runtime_call_value(
            MirIntrinsic::Collection(MirCollectionOp::Length),
            vec![MirOperand::Place(collection.clone())],
            numerus,
            iter.span,
        );
        if !self.current_is_open() {
            return None;
        }

        let binding_ty = if matches!(iter.mode, HirIteraMode::De) {
            numerus
        } else {
            item_ty
        };
        let binding = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: binding,
            name: Some(iter.name),
            ty: binding_ty,
            mutable: true,
            span: iter.span,
        });
        self.bindings
            .insert(iter.binding, LocalBinding { local: binding, ty: binding_ty });

        let cond_id = self.fresh_block(iter.span);
        let body_id = self.fresh_block(iter.body.span);
        let increment_id = self.fresh_block(iter.span);
        let after_id = self.fresh_block(iter.span);
        self.terminate_current(MirTerminatorKind::Goto(cond_id), iter.span);

        self.switch_to(cond_id);
        let condition = self.binary_temp(
            MirBinOp::Lt,
            MirOperand::Place(MirPlace::local(index)),
            length,
            bivalens,
            iter.span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: body_id, else_block: after_id },
            iter.span,
        );

        self.switch_to(body_id);
        let value = if matches!(iter.mode, HirIteraMode::De) {
            MirOperand::Place(MirPlace::local(index))
        } else {
            let mut item_place = collection;
            item_place
                .projections
                .push(MirProjection::Index(MirOperand::Place(MirPlace::local(index))));
            MirOperand::Place(item_place)
        };
        self.assign(MirPlace::local(binding), value, binding_ty, iter.span);
        self.loops
            .push(LoopContext { perge_target: increment_id, rumpe_target: after_id });
        self.visit_block(iter.body);
        self.loops.pop();
        self.terminate_open_current(MirTerminatorKind::Goto(increment_id), iter.span);

        self.switch_to(increment_id);
        let next = self.binary_temp(
            MirBinOp::Add,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Constant(MirConstant::Int(1)),
            numerus,
            iter.span,
        );
        self.assign(MirPlace::local(index), next, numerus, iter.span);
        self.terminate_current(MirTerminatorKind::Goto(cond_id), iter.span);

        self.switch_to(after_id);
        Some(())
    }

    pub(super) fn materialize_operand_place(&mut self, operand: MirOperand, ty: MirType, span: Span) -> MirPlace {
        match operand {
            MirOperand::Place(place) => place,
            MirOperand::Temp(temp) => MirPlace::temp(temp),
            other => {
                let temp = self.push_temp(ty, span);
                self.assign(MirPlace::temp(temp), other, ty, span);
                MirPlace::temp(temp)
            }
        }
    }

    fn lower_default_range_step(
        &mut self,
        index: MirLocalId,
        end: MirLocalId,
        step: MirLocalId,
        cond_id: MirBlockId,
        bivalens: MirType,
        span: Span,
    ) {
        let positive_step_id = self.fresh_block(span);
        let negative_step_id = self.fresh_block(span);
        let ascends = self.binary_temp(
            MirBinOp::LtEq,
            MirOperand::Place(MirPlace::local(index)),
            MirOperand::Place(MirPlace::local(end)),
            bivalens,
            span,
        );
        self.terminate_current(
            MirTerminatorKind::Branch {
                condition: ascends,
                then_block: positive_step_id,
                else_block: negative_step_id,
            },
            span,
        );

        self.switch_to(positive_step_id);
        self.assign(
            MirPlace::local(step),
            MirOperand::Constant(MirConstant::Int(1)),
            MirType::semantic(self.types.primitive(Primitive::Numerus)),
            span,
        );
        self.terminate_current(MirTerminatorKind::Goto(cond_id), span);

        self.switch_to(negative_step_id);
        self.assign(
            MirPlace::local(step),
            MirOperand::Constant(MirConstant::Int(-1)),
            MirType::semantic(self.types.primitive(Primitive::Numerus)),
            span,
        );
        self.terminate_current(MirTerminatorKind::Goto(cond_id), span);
    }

    pub(super) fn binary_temp(
        &mut self,
        op: MirBinOp,
        lhs: MirOperand,
        rhs: MirOperand,
        ty: MirType,
        span: Span,
    ) -> MirOperand {
        self.assign_temp(MirValueKind::Binary { op, lhs, rhs }, ty, span)
    }

    /// Lower `rumpe` to the active loop's exit edge.
    pub(super) fn lower_rumpe(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "rumpe without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.rumpe_target), span);
    }

    /// Lower `perge` to the active loop's condition edge.
    pub(super) fn lower_perge(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "perge without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.perge_target), span);
    }
}
