//! Expression dispatch for HIR-to-MIR lowering.
//!
//! This module is the expression-facing boundary of the function builder. It
//! maps the HIR expression taxonomy onto narrower lowering routines that own
//! particular MIR contracts: value creation, addressable-place projection,
//! control-flow construction, aggregate construction, and runtime/intrinsic
//! calls. The dispatch is deliberately explicit so unsupported HIR shapes stay
//! visible as diagnostics instead of being silently approximated in MIR.
//!
//! ERROR STRATEGY
//! ==============
//! MIR lowering only consumes HIR that semantic analysis has already typed and
//! resolved. When a shape is not represented in MIR yet, or when required type
//! information is absent, this layer records a `MirError` and returns `None`.
//! That fail-closed behavior prevents later codegen from guessing about
//! upstream semantic facts.

use super::*;
use crate::hir::HirCallArg;

/// Visitor contract for lowering a HIR expression when the caller needs a MIR
/// operand.
///
/// The default dispatcher encodes the supported expression set for this MIR
/// phase. Implementors may return places, temps, constants, or runtime values,
/// but they must report unsupported forms through `MirError` rather than
/// manufacturing placeholder MIR.
pub(super) trait HirExprLoweringVisitor {
    /// Lower a typed HIR expression into the operand form expected by MIR users.
    ///
    /// Control-producing expressions may still terminate or switch the current
    /// block as part of producing their operand. Callers must therefore treat a
    /// `None` result as both "no value" and "current CFG state may have been
    /// intentionally sealed after a diagnostic."
    fn lower_expr_value(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        match &expr.kind {
            HirExprKind::Path(def_id) => self.visit_path_expr(*def_id, expr),
            HirExprKind::Literal(literal) => self.visit_literal_expr(literal, expr),
            HirExprKind::Unary(op, operand) => self.visit_unary_expr(*op, operand, expr),
            HirExprKind::Binary(HirBinOp::Coalesce, lhs, rhs) => self.visit_coalesce_expr(lhs, rhs, expr),
            HirExprKind::Binary(op, lhs, rhs) => self.visit_binary_expr(*op, lhs, rhs, expr),
            HirExprKind::Call(callee, args) => self.visit_call_expr(callee, args, expr),
            HirExprKind::MethodCall(receiver, method, args) => {
                self.visit_method_call_expr(receiver, *method, args, expr)
            }
            HirExprKind::Field(object, name) => self.visit_field_expr(object, *name, expr),
            HirExprKind::Index(object, index) => self.visit_index_expr(object, index, expr),
            HirExprKind::OptionalChain(object, chain) => self.visit_optional_chain_expr(object, chain, expr),
            HirExprKind::NonNull(object, chain) => self.visit_non_null_expr(object, chain, expr),
            HirExprKind::Array(elements) => self.visit_array_expr(elements, expr),
            HirExprKind::Struct(def_id, fields) => self.visit_struct_expr(*def_id, fields, expr),
            HirExprKind::Tuple(items) => self.visit_tuple_expr(items, expr),
            HirExprKind::Verte { source, target, entries } => {
                self.visit_verte_expr(source, *target, entries.as_ref(), expr)
            }
            HirExprKind::Scribe(kind, args) => self.visit_scribe_expr(*kind, args, expr),
            HirExprKind::Scriptum(template, args) => self.visit_scriptum_expr(*template, args, expr),
            HirExprKind::Adfirma(cond, message) => self.visit_adfirma_expr(cond, message.as_deref(), expr),
            HirExprKind::Conversio { source, target, params, fallback } => {
                self.visit_conversio_expr(source, *target, params, fallback.as_deref(), expr)
            }
            HirExprKind::Block(block) => self.visit_block_expr(block, expr),
            HirExprKind::Si { cond, then_block, then_catch, else_block } => {
                self.visit_si_expr(cond, then_block, then_catch.as_deref(), else_block, expr)
            }
            HirExprKind::Discerne(scrutinees, arms) => self.visit_discerne_expr(scrutinees, arms, expr),
            HirExprKind::Dum(cond, block) => self.visit_dum_expr(cond, block, expr),
            HirExprKind::Handled { body, catch } => self.visit_handled_expr(body, catch, expr),
            HirExprKind::Assign(_, _) => self.visit_assignment_expr(expr),
            HirExprKind::Throw(value) => self.visit_throw_expr(value, expr),
            HirExprKind::Panic(value) => self.visit_panic_expr(value, expr),
            HirExprKind::AssignOp(_, _, _) => self.visit_assign_op_expr(expr),
            other => self.visit_unsupported_expr(other, expr),
        }
    }

    fn visit_path_expr(&mut self, def_id: DefId, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_literal_expr(&mut self, literal: &HirLiteral, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_unary_expr(&mut self, op: HirUnOp, operand: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_binary_expr(&mut self, op: HirBinOp, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_coalesce_expr(&mut self, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_call_expr(&mut self, callee: &HirExpr, args: &[HirCallArg], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_method_call_expr(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirCallArg],
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_field_expr(&mut self, object: &HirExpr, name: Symbol, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_index_expr(&mut self, object: &HirExpr, index: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_optional_chain_expr(
        &mut self,
        object: &HirExpr,
        chain: &HirOptionalChainKind,
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_non_null_expr(&mut self, object: &HirExpr, chain: &HirNonNullKind, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_array_expr(&mut self, elements: &[HirArrayElement], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_struct_expr(&mut self, def_id: DefId, fields: &[(Symbol, HirExpr)], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_tuple_expr(&mut self, items: &[HirExpr], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_verte_expr(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        entries: Option<&Vec<HirObjectField>>,
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_scribe_expr(&mut self, kind: HirScribeKind, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_scriptum_expr(&mut self, template: Symbol, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand>;
    fn visit_adfirma_expr(&mut self, cond: &HirExpr, message: Option<&HirExpr>, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_conversio_expr(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        params: &[Symbol],
        fallback: Option<&HirExpr>,
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_block_expr(&mut self, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_si_expr(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: &Option<HirBlock>,
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_discerne_expr(
        &mut self,
        scrutinees: &[HirExpr],
        arms: &[HirCasuArm],
        expr: &HirExpr,
    ) -> Option<MirOperand>;
    fn visit_dum_expr(&mut self, cond: &HirExpr, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_handled_expr(&mut self, body: &HirBlock, catch: &HirCape, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_assignment_expr(&mut self, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_throw_expr(&mut self, value: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_panic_expr(&mut self, value: &HirExpr, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_assign_op_expr(&mut self, expr: &HirExpr) -> Option<MirOperand>;
    fn visit_unsupported_expr(&mut self, kind: &HirExprKind, expr: &HirExpr) -> Option<MirOperand>;
}

impl HirExprLoweringVisitor for FunctionBuilder<'_> {
    fn visit_path_expr(&mut self, def_id: DefId, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_path(def_id, expr.span)
    }

    fn visit_literal_expr(&mut self, literal: &HirLiteral, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_literal(literal, expr.span)
    }

    fn visit_unary_expr(&mut self, op: HirUnOp, operand: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_unary(op, operand, expr)
    }

    fn visit_binary_expr(&mut self, op: HirBinOp, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_binary(op, lhs, rhs, expr)
    }

    fn visit_coalesce_expr(&mut self, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_coalesce(lhs, rhs, expr)
    }

    fn visit_call_expr(&mut self, callee: &HirExpr, args: &[HirCallArg], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_call(callee, args, expr)
    }

    fn visit_method_call_expr(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirCallArg],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_method_call(receiver, method, args, expr)
    }

    fn visit_field_expr(&mut self, object: &HirExpr, name: Symbol, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_field(object, name, expr)
    }

    fn visit_index_expr(&mut self, object: &HirExpr, index: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_index(object, index, expr)
    }

    fn visit_optional_chain_expr(
        &mut self,
        object: &HirExpr,
        chain: &HirOptionalChainKind,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_optional_chain(object, chain, expr)
    }

    fn visit_non_null_expr(&mut self, object: &HirExpr, chain: &HirNonNullKind, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_non_null(object, chain, expr)
    }

    fn visit_array_expr(&mut self, elements: &[HirArrayElement], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_array(elements, expr, MirAggregateKind::Array)
    }

    fn visit_struct_expr(&mut self, def_id: DefId, fields: &[(Symbol, HirExpr)], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_struct_literal(def_id, fields, expr)
    }

    fn visit_tuple_expr(&mut self, items: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_tuple(items, expr)
    }

    fn visit_verte_expr(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        entries: Option<&Vec<HirObjectField>>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_verte(source, target, entries, expr)
    }

    fn visit_scribe_expr(&mut self, kind: HirScribeKind, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_scribe(kind, args, expr)
    }

    fn visit_scriptum_expr(&mut self, template: Symbol, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        self.lower_scriptum(template, args, expr)
    }

    fn visit_adfirma_expr(&mut self, cond: &HirExpr, message: Option<&HirExpr>, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_adfirma(cond, message, expr)
    }

    fn visit_conversio_expr(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        params: &[Symbol],
        fallback: Option<&HirExpr>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_conversio(source, target, params, fallback, expr)
    }

    fn visit_block_expr(&mut self, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_block_expr(block, expr)
    }

    fn visit_si_expr(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: &Option<HirBlock>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_si_expr(cond, then_block, then_catch, else_block, expr)
    }

    fn visit_discerne_expr(
        &mut self,
        scrutinees: &[HirExpr],
        arms: &[HirCasuArm],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        self.lower_discerne_expr(scrutinees, arms, expr)
    }

    fn visit_dum_expr(&mut self, cond: &HirExpr, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_dum_expr(cond, block, expr)
    }

    fn visit_handled_expr(&mut self, body: &HirBlock, catch: &HirCape, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_handled_expr(body, catch, expr)
    }

    fn visit_assignment_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_assignment_expr(expr)
    }

    fn visit_throw_expr(&mut self, value: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_iace(value, expr.span)
    }

    fn visit_panic_expr(&mut self, value: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_mori(value, expr.span)
    }

    fn visit_assign_op_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        self.errors.push(MirError::unsupported(
            expr.span,
            "compound assignment before assignment-op MIR lowering",
        ));
        None
    }

    fn visit_unsupported_expr(&mut self, kind: &HirExprKind, expr: &HirExpr) -> Option<MirOperand> {
        self.errors
            .push(MirError::unsupported(expr.span, unsupported_expr_kind_name(kind)));
        None
    }
}
