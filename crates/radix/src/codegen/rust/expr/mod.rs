//! Rust expression lowering for the Faber backend.
//!
//! This module is the expression boundary between typed HIR and emitted Rust
//! source. It dispatches expression forms to focused submodules, but owns the
//! cross-cutting context that makes Rust expression output valid: whether the
//! current function can propagate failures, whether output is inside the entry
//! point, and whether a local construct has suppressed propagation.
//!
//! PROPAGATION CONTEXT
//! ===================
//! - `can_propagate_failure` means Rust `?` is syntactically legal for calls
//!   known to return a fallible value.
//! - `inside_entrypoint` prevents entrypoint code from growing an implicit fallible
//!   return contract; throws and failable calls must stay entry-compatible.
//! - `propagation_suppressed` is a local override used by catch-like
//!   constructs so a handled body does not leak `?` into the surrounding Rust.
//!
//! FAILURE POLICY
//! ==============
//! Unsupported or poisoned HIR fails closed at this boundary. Codegen should
//! report an explicit [`CodegenError`] rather than guessing Rust for a surface
//! that earlier phases have not made precise.

use super::super::CodeWriter;
use super::type_shape::{resolve_type, type_id_is_faber_value, type_is_option_or_nihil};
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::lexer::Symbol;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

mod access;
mod block;
mod call;
mod collection;
mod control;
mod convert;
mod format;
mod literal;
mod ops;
mod option;
mod pattern;
mod verte;

use access::*;
use block::*;
use call::*;
use collection::*;
use control::*;
use convert::*;
use format::*;
use literal::*;
use ops::*;
use option::*;
use pattern::*;
use verte::*;

#[derive(Clone, Copy)]
pub(super) struct ExprEmitPolicy {
    pub(super) can_propagate_failure: bool,
    pub(super) inside_entrypoint: bool,
    pub(super) propagation_suppressed: bool,
}

impl ExprEmitPolicy {
    pub(super) fn new(can_propagate_failure: bool, inside_entrypoint: bool, propagation_suppressed: bool) -> Self {
        Self { can_propagate_failure, inside_entrypoint, propagation_suppressed }
    }

    pub(super) fn permits_question_mark(self) -> bool {
        self.can_propagate_failure && !self.inside_entrypoint && !self.propagation_suppressed
    }
}

pub(super) struct ExprEmitter<'a, 'cg> {
    pub(super) codegen: &'a RustCodegen<'cg>,
    pub(super) types: &'a TypeTable,
    pub(super) writer: &'a mut CodeWriter,
    pub(super) policy: ExprEmitPolicy,
}

impl<'a, 'cg> ExprEmitter<'a, 'cg> {
    pub(super) fn new(
        codegen: &'a RustCodegen<'cg>,
        types: &'a TypeTable,
        writer: &'a mut CodeWriter,
        policy: ExprEmitPolicy,
    ) -> Self {
        Self { codegen, types, writer, policy }
    }

    pub(super) fn expr(&mut self, expr: &HirExpr) -> Result<(), CodegenError> {
        emit_expr(self, expr)
    }

    pub(super) fn expr_unwrapped(&mut self, expr: &HirExpr) -> Result<(), CodegenError> {
        emit_expr_unwrapped(self, expr)
    }

    pub(super) fn expr_as_type(&mut self, expr: &HirExpr, target_ty: TypeId) -> Result<(), CodegenError> {
        emit_expr_as_type(self, expr, target_ty)
    }

    pub(super) fn expr_as_optional_target(&mut self, expr: &HirExpr, value_ty: TypeId) -> Result<(), CodegenError> {
        emit_expr_as_optional_target(self, expr, value_ty)
    }
}

/// Generate one Rust expression from typed HIR.
///
/// The three context booleans are part of the backend contract, not formatting
/// preferences. They are threaded through every nested expression so callees can
/// decide whether `?`, entrypoint-compatible failure behavior, or catch-local
/// suppression applies at the exact point where Rust syntax is emitted.
///
/// TARGET: Rust-specific `?` operator for error propagation.
/// EDGE: unsupported structured handler HIR and HIR error sentinels return
/// diagnostics instead of best-effort Rust.
pub fn generate_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        writer,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    emit_expr(&mut emitter, expr)
}

fn emit_expr(emitter: &mut ExprEmitter<'_, '_>, expr: &HirExpr) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            if emitter.codegen.current_self_def() == Some(*def_id) {
                emitter.writer.write("self");
            } else if let Some(variant) = emitter.codegen.variant_info(*def_id) {
                emitter
                    .writer
                    .write(emitter.codegen.resolve_def(variant.enum_def));
                emitter.writer.write("::");
                emitter.writer.write(emitter.codegen.resolve_def(*def_id));
            } else {
                emitter.writer.write(emitter.codegen.resolve_def(*def_id));
            }
        }
        HirExprKind::Literal(lit) => {
            generate_literal(emitter.codegen, lit, emitter.writer);
            if matches!(lit, HirLiteral::String(_))
                && expr
                    .ty
                    .is_some_and(|ty| matches!(emitter.types.get(ty), Type::Primitive(Primitive::Textus)))
            {
                emitter.writer.write(".to_string()");
            }
        }
        HirExprKind::Binary(op, lhs, rhs) => {
            generate_binary_expr_with_emitter(emitter, *op, lhs, rhs, expr.ty, true)?;
        }
        HirExprKind::Unary(op, operand) => {
            generate_unary_expr_with_emitter(emitter, *op, operand, true)?;
        }
        HirExprKind::Call(callee, args) => {
            generate_call_expr(emitter, callee, args)?;
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_method_call_expr(emitter, receiver, *method, args)?;
        }
        HirExprKind::Field(obj, field) => {
            generate_field_expr_with_emitter(emitter, obj, *field)?;
        }
        HirExprKind::Index(obj, idx) => {
            generate_index_expr_with_emitter(emitter, obj, idx)?;
        }
        HirExprKind::OptionalChain(object, chain) => generate_optional_chain_expr_with_emitter(emitter, object, chain)?,
        HirExprKind::NonNull(object, chain) => generate_non_null_expr_with_emitter(emitter, object, chain)?,
        HirExprKind::Block(block) => {
            generate_block_with_emitter(emitter, block)?;
        }
        HirExprKind::Tempta { body, catch, finally } => {
            emit_tempta_expr(emitter, body, catch.as_ref(), finally.as_ref())?
        }
        HirExprKind::Si { cond, then_block, then_catch: None, else_block } => {
            generate_if_expr_with_emitter(emitter, cond, then_block, else_block.as_ref(), expr.ty)?
        }
        HirExprKind::Si { then_catch: Some(_), .. } | HirExprKind::Handled { .. } => {
            // Fail closed: these HIR surfaces carry structured handler
            // semantics that this Rust backend slice does not yet lower.
            return Err(CodegenError {
                message: "structured cape handlers are not emitted by Rust codegen in Phase 5C".to_owned(),
            });
        }
        HirExprKind::Discerne(scrutinees, arms) => generate_match_expr_with_emitter(emitter, scrutinees, arms)?,
        HirExprKind::Loop(block) => generate_loop_expr_with_emitter(emitter, block)?,
        HirExprKind::Dum(cond, block) => generate_while_expr_with_emitter(emitter, cond, block)?,
        HirExprKind::Itera(mode, binding, _binding_name, iter, block) => {
            generate_for_expr_with_emitter(emitter, *mode, *binding, iter, block)?
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            generate_range_tuple_expr_with_emitter(emitter, start, end, step.as_deref())?
        }
        HirExprKind::Assign(target, value) => {
            generate_assign_expr_with_emitter(emitter, target, value)?;
        }
        HirExprKind::AssignOp(op, target, value) => generate_assign_op_expr_with_emitter(emitter, *op, target, value)?,
        HirExprKind::Array(elements) => emit_array_expr(emitter, expr.id, expr.ty, elements)?,
        HirExprKind::Vacua => generate_vacua_expr(expr, emitter),
        HirExprKind::Struct(def_id, fields) => emit_struct_expr(emitter, expr.id, *def_id, fields)?,
        HirExprKind::Tuple(elements) => emit_tuple_expr(emitter, elements)?,
        HirExprKind::Scribe(kind, args) => {
            generate_scribe_expr_with_emitter(emitter, *kind, args)?;
        }
        HirExprKind::Scriptum(template, args) => generate_scriptum_expr_with_emitter(emitter, *template, args)?,
        HirExprKind::Adfirma(cond, message) => {
            generate_assert_expr_with_emitter(emitter, cond, message.as_deref())?;
        }
        HirExprKind::Panic(value) => {
            generate_panic_expr_with_emitter(emitter, value)?;
        }
        HirExprKind::Throw(value) => {
            generate_throw_expr_with_emitter(emitter, value)?;
        }
        HirExprKind::Clausura(params, _ret, body) => {
            emit_closure_expr(emitter, params, body)?;
        }
        HirExprKind::Cede(expr) => {
            emit_await_expr(emitter, expr)?;
        }
        HirExprKind::Verte { source, target, entries } => {
            emit_verte_expr(emitter, expr.id, source, expr.ty.unwrap_or(*target), entries.as_deref())?
        }
        HirExprKind::Conversio { source, target, params, fallback } => {
            generate_conversio_expr_with_emitter(emitter, source, *target, params, fallback.as_deref())?
        }
        HirExprKind::Ref(kind, expr) => {
            generate_ref_expr_with_emitter(emitter, *kind, expr)?;
        }
        HirExprKind::Deref(expr) => {
            generate_deref_expr_with_emitter(emitter, expr)?;
        }
        HirExprKind::Error => {
            // Error sentinels mean an earlier phase could not produce sound HIR.
            // Emitting placeholder Rust here would hide the real diagnostic.
            return Err(CodegenError {
                message: format!(
                    "cannot generate Rust for HIR error expression at span {}..{}",
                    expr.span.start, expr.span.end
                ),
            });
        }
    }
    Ok(())
}

fn generate_vacua_expr(expr: &HirExpr, emitter: &mut ExprEmitter<'_, '_>) {
    match expr.ty.map(|ty| emitter.types.get(ty)) {
        Some(Type::Map(_, _)) => emitter.writer.write("std::collections::HashMap::new()"),
        Some(Type::Set(_)) => emitter.writer.write("std::collections::HashSet::new()"),
        Some(Type::Array(_)) => emitter.writer.write("Vec::new()"),
        _ => emitter.writer.write("Vec::new()"),
    }
}

pub(super) fn generate_expr_unwrapped(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    writer: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        writer,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    emit_expr_unwrapped(&mut emitter, expr)
}

fn emit_expr_unwrapped(emitter: &mut ExprEmitter<'_, '_>, expr: &HirExpr) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Binary(op, lhs, rhs) => generate_binary_expr_with_emitter(emitter, *op, lhs, rhs, expr.ty, false),
        HirExprKind::Unary(op, operand) => generate_unary_expr_with_emitter(emitter, *op, operand, false),
        _ => emit_expr(emitter, expr),
    }
}

fn emit_expr_as_type(emitter: &mut ExprEmitter<'_, '_>, expr: &HirExpr, target_ty: TypeId) -> Result<(), CodegenError> {
    if type_id_is_faber_value(target_ty, emitter.types) {
        return generate_expr_as_faber_value_with_emitter(emitter, expr);
    }

    emit_expr_unwrapped(emitter, expr)?;
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::String(_)))
        && !expr
            .ty
            .is_some_and(|ty| matches!(resolve_type(ty, emitter.types), Type::Primitive(Primitive::Textus)))
        && matches!(resolve_type(target_ty, emitter.types), Type::Primitive(Primitive::Textus))
    {
        emitter.writer.write(".to_string()");
    }
    Ok(())
}

fn emit_expr_as_optional_target(
    emitter: &mut ExprEmitter<'_, '_>,
    expr: &HirExpr,
    value_ty: TypeId,
) -> Result<(), CodegenError> {
    if expr_may_already_produce_option(emitter.codegen, expr, emitter.types) {
        emit_expr_unwrapped(emitter, expr)?;
        return Ok(());
    }

    emitter.writer.write("Some(");
    emit_expr_as_type(emitter, expr, value_ty)?;
    emitter.writer.write(")");
    Ok(())
}

pub(super) fn expr_may_already_produce_option(codegen: &RustCodegen<'_>, expr: &HirExpr, types: &TypeTable) -> bool {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Nil) | HirExprKind::OptionalChain(_, _) => true,
        HirExprKind::Path(def_id) => codegen
            .binding_type(*def_id)
            .or(expr.ty)
            .is_some_and(|ty| type_is_option_or_nihil(ty, types)),
        HirExprKind::Verte { target, .. } => type_is_option_or_nihil(*target, types),
        HirExprKind::Call(_, _)
        | HirExprKind::MethodCall(_, _, _)
        | HirExprKind::Field(_, _)
        | HirExprKind::Index(_, _)
        | HirExprKind::NonNull(_, _)
        | HirExprKind::Si { .. } => expr.ty.is_some_and(|ty| type_is_option_or_nihil(ty, types)),
        _ => false,
    }
}
