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
                    self.errors.push(MirError::unsupported(
                        arm.span,
                        "non-literal discerne pattern before switch MIR lowering",
                    ));
                    return None;
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
