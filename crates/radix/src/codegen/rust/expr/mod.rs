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
//! - `in_failable_fn` means Rust `?` is syntactically legal for calls known to
//!   return a fallible value.
//! - `in_entry` prevents entrypoint code from growing an implicit fallible
//!   return contract; throws and failable calls must stay entry-compatible.
//! - `suppress_error_propagation` is a local override used by catch-like
//!   constructs so a handled body does not leak `?` into the surrounding Rust.
//!
//! FAILURE POLICY
//! ==============
//! Unsupported or poisoned HIR fails closed at this boundary. Codegen should
//! report an explicit [`CodegenError`] rather than guessing Rust for a surface
//! that earlier phases have not made precise.

use super::super::CodeWriter;
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
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            if codegen.current_self_def() == Some(*def_id) {
                w.write("self");
            } else if let Some(variant) = codegen.variant_info(*def_id) {
                w.write(codegen.resolve_def(variant.enum_def));
                w.write("::");
                w.write(codegen.resolve_def(*def_id));
            } else {
                w.write(codegen.resolve_def(*def_id));
            }
        }
        HirExprKind::Literal(lit) => {
            generate_literal(codegen, lit, w);
            if matches!(lit, HirLiteral::String(_))
                && expr
                    .ty
                    .is_some_and(|ty| matches!(types.get(ty), Type::Primitive(Primitive::Textus)))
            {
                w.write(".to_string()");
            }
        }
        HirExprKind::Binary(op, lhs, rhs) => {
            generate_binary_expr(
                codegen,
                *op,
                lhs,
                rhs,
                expr.ty,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
                true,
            )?;
        }
        HirExprKind::Unary(op, operand) => {
            generate_unary_expr(
                codegen,
                *op,
                operand,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
                true,
            )?;
        }
        HirExprKind::Call(callee, args) => {
            generate_call_expr(
                codegen,
                callee,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_method_call_expr(
                codegen,
                receiver,
                *method,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Field(obj, field) => {
            generate_field_expr(
                codegen,
                obj,
                *field,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Index(obj, idx) => {
            generate_index_expr(
                codegen,
                obj,
                idx,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::OptionalChain(object, chain) => generate_optional_chain_expr(
            codegen,
            object,
            chain,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::NonNull(object, chain) => generate_non_null_expr(
            codegen,
            object,
            chain,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Block(block) => {
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Tempta { body, catch, finally } => generate_tempta_expr(
            codegen,
            body,
            catch.as_ref(),
            finally.as_ref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Si { cond, then_block, then_catch: None, else_block } => generate_if_expr(
            codegen,
            cond,
            then_block,
            else_block.as_ref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Si { then_catch: Some(_), .. } | HirExprKind::Handled { .. } => {
            // Fail closed: these HIR surfaces carry structured handler
            // semantics that this Rust backend slice does not yet lower.
            return Err(CodegenError {
                message: "structured cape handlers are not emitted by Rust codegen in Phase 5C".to_owned(),
            });
        }
        HirExprKind::Discerne(scrutinees, arms) => generate_match_expr(
            codegen,
            scrutinees,
            arms,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Loop(block) => {
            generate_loop_expr(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?
        }
        HirExprKind::Dum(cond, block) => generate_while_expr(
            codegen,
            cond,
            block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Itera(mode, binding, _binding_name, iter, block) => generate_for_expr(
            codegen,
            *mode,
            *binding,
            iter,
            block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Intervallum { start, end, step, .. } => generate_range_tuple_expr(
            codegen,
            start,
            end,
            step.as_deref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Assign(target, value) => {
            generate_assign_expr(
                codegen,
                target,
                value,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::AssignOp(op, target, value) => generate_assign_op_expr(
            codegen,
            *op,
            target,
            value,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Array(elements) => generate_array_expr(
            codegen,
            expr.id,
            elements,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Vacua => generate_vacua_expr(expr, types, w),
        HirExprKind::Struct(def_id, fields) => generate_struct_expr(
            codegen,
            *def_id,
            fields,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Tuple(elements) => generate_tuple_expr(
            codegen,
            elements,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Scribe(kind, args) => {
            generate_scribe_expr(
                codegen,
                *kind,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Scriptum(template, args) => generate_scriptum_expr(
            codegen,
            *template,
            args,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Adfirma(cond, message) => {
            generate_assert_expr(
                codegen,
                cond,
                message.as_deref(),
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Panic(value) => {
            generate_panic_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Throw(value) => {
            generate_throw_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Clausura(params, _ret, body) => {
            generate_closure_expr(
                codegen,
                params,
                body,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Cede(expr) => {
            generate_await_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Verte { source, target, entries } => generate_verte_expr(
            codegen,
            expr.id,
            source,
            *target,
            entries.as_deref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Conversio { source, target, params, fallback } => generate_conversio_expr(
            codegen,
            source,
            *target,
            params,
            fallback.as_deref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Ref(kind, expr) => {
            generate_ref_expr(
                codegen,
                *kind,
                expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Deref(expr) => {
            generate_deref_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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

fn generate_vacua_expr(expr: &HirExpr, types: &TypeTable, w: &mut CodeWriter) {
    match expr.ty.map(|ty| types.get(ty)) {
        Some(Type::Map(_, _)) => w.write("std::collections::HashMap::new()"),
        Some(Type::Set(_)) => w.write("std::collections::HashSet::new()"),
        Some(Type::Array(_)) => w.write("Vec::new()"),
        _ => w.write("Vec::new()"),
    }
}

pub(super) fn generate_expr_unwrapped(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Binary(op, lhs, rhs) => generate_binary_expr(
            codegen,
            *op,
            lhs,
            rhs,
            expr.ty,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
            false,
        ),
        HirExprKind::Unary(op, operand) => generate_unary_expr(
            codegen,
            *op,
            operand,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
            false,
        ),
        _ => generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation),
    }
}

fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        ty => ty.clone(),
    }
}
