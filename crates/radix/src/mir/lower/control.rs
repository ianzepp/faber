use super::*;
use crate::hir::visit::HirVisitor;

impl FunctionBuilder<'_> {
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

    pub(super) fn lower_rumpe(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "rumpe without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.rumpe_target), span);
    }

    pub(super) fn lower_perge(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "perge without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.perge_target), span);
    }
}
