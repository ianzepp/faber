//! Go expression dispatcher for lowered HIR.
//!
//! This module is the routing boundary between semantic HIR and the Go backend's
//! expression emitters. It keeps the broad expression taxonomy visible in one
//! place while delegating target-specific compromises to focused submodules:
//! access, calls, control forms, collection literals, conversions, options, and
//! variants.
//!
//! INVARIANTS
//! ==========
//! - Every emitted expression must be syntactically valid Go at the call site.
//!   When Faber needs statement-only Go forms, helper modules wrap them in
//!   immediately invoked functions.
//! - Unsupported or poisoned HIR should fail closed with a codegen diagnostic
//!   instead of fabricating target code with unclear semantics; existing
//!   placeholders such as expression-position `discerne` remain visible TODOs.
//! - Missing type information should be handled by earlier phases; this layer
//!   only uses conservative fallbacks where Go requires a concrete expression
//!   shape.

use super::stmt;
use super::types;
use super::{CodeWriter, CodegenError, GoCodegen};
use crate::hir::{HirArrayElement, HirExpr, HirExprKind, HirLiteral};
use crate::semantic::{Primitive, Type, TypeTable};

mod access;
mod call;
mod collection;
mod control;
mod convert;
mod literal;
mod ops;
mod option;
mod variants;

use access::*;
use call::*;
use collection::*;
use control::*;
use convert::*;
use literal::*;
use ops::*;
use option::*;
use variants::*;

pub fn generate_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // This match is the Go backend's expression support matrix. A branch that
    // cannot preserve Faber semantics should return an error rather than emit
    // a placeholder that later phases or users might mistake for supported Go.
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            if codegen.is_variant_def(*def_id) {
                w.write(codegen.resolve_def(*def_id));
                w.write("{}");
            } else if codegen.is_struct_def(*def_id) {
                w.write("*self");
            } else {
                w.write(codegen.resolve_def(*def_id));
            }
        }
        HirExprKind::Literal(lit) => generate_literal(codegen, lit, w),
        HirExprKind::Binary(op, lhs, rhs) => generate_binary_expr(codegen, expr, *op, lhs, rhs, types, w)?,
        HirExprKind::Unary(op, operand) => generate_unary_expr(codegen, *op, operand, types, w)?,
        HirExprKind::Call(callee, args) => generate_call_expr(codegen, callee, args, types, w)?,
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_method_call_expr(codegen, receiver, *method, args, types, w)?;
        }
        HirExprKind::Field(object, field) => generate_field_expr(codegen, expr, object, *field, types, w)?,
        HirExprKind::Index(object, index) => generate_index_expr(codegen, object, index, types, w)?,
        HirExprKind::OptionalChain(object, chain) => {
            generate_optional_chain_expr(codegen, object, chain, expr, types, w)?;
        }
        HirExprKind::NonNull(object, chain) => generate_non_null_expr(codegen, object, chain, types, w)?,
        HirExprKind::Assign(lhs, rhs) => generate_assign_expr(codegen, lhs, rhs, types, w)?,
        HirExprKind::AssignOp(op, lhs, rhs) => generate_assign_op_expr(codegen, *op, lhs, rhs, types, w)?,
        HirExprKind::Array(elements) => {
            generate_array_expr(codegen, expr, elements, types, w)?;
        }
        HirExprKind::Vacua => generate_vacua_expr(codegen, expr, types, w),
        HirExprKind::Struct(def_id, fields) => generate_struct_expr(codegen, *def_id, fields, types, w)?,
        HirExprKind::Tuple(elements) => generate_tuple_expr(codegen, elements, types, w)?,
        HirExprKind::Scribe(kind, args) => generate_scribe_expr(codegen, *kind, args, types, w)?,
        HirExprKind::Scriptum(template, args) => generate_scriptum_expr(codegen, *template, args, types, w)?,
        HirExprKind::Panic(value) | HirExprKind::Throw(value) => generate_panic_expr(codegen, value, types, w)?,
        HirExprKind::Tempta { body, catch, .. } => generate_tempta_expr(codegen, body, catch.as_ref(), types, w)?,
        HirExprKind::Clausura(params, ret_ty, body) => {
            w.write("func(");
            for (idx, param) in params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                w.write(" ");
                w.write(&types::type_to_go(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = ret_ty.or(body.ty) {
                let ret = types::type_to_go(codegen, ret_ty, types);
                if !ret.is_empty() {
                    w.write(" ");
                    w.write(&ret);
                }
            }
            w.write(" { return ");
            generate_expr(codegen, body, types, w)?;
            w.write(" }");
        }
        HirExprKind::Cede(inner) => {
            // WHY: Go has no await — goroutines handle concurrency differently.
            // Emit the inner expression as-is; async support is a future concern.
            generate_expr(codegen, inner, types, w)?;
        }
        HirExprKind::Verte { source, target, entries } => {
            generate_verte_expr(codegen, source, *target, entries.as_deref(), types, w)?;
        }
        HirExprKind::Conversio { source, target, fallback, .. } => {
            generate_conversio_expr(codegen, source, *target, fallback.as_deref(), types, w)?;
        }
        // WHY: Go is GC'd — ref/deref are no-ops, just emit the inner expression.
        HirExprKind::Ref(_, inner) | HirExprKind::Deref(inner) => generate_expr(codegen, inner, types, w)?,
        HirExprKind::Block(block) => generate_block_expr(codegen, expr, block, types, w)?,
        HirExprKind::Si { cond, then_block, then_catch: None, else_block } => {
            generate_if_expr(codegen, expr, cond, then_block, else_block.as_ref(), types, w)?;
        }
        HirExprKind::Si { then_catch: Some(_), .. } | HirExprKind::Handled { .. } => {
            return Err(crate::codegen::CodegenError {
                message: "structured cape handlers are not emitted by Go codegen in Phase 5C".to_owned(),
            });
        }
        HirExprKind::Discerne(_, _) => {
            // TODO: switch statement codegen
            w.write("nil");
        }
        HirExprKind::Loop(block) => generate_loop_expr(codegen, block, types, w)?,
        HirExprKind::Dum(cond, block) => generate_while_expr(codegen, cond, block, types, w)?,
        HirExprKind::Itera(mode, def_id, _binding_name, iter, block) => {
            generate_for_expr(codegen, *mode, *def_id, iter, block, types, w)?;
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            generate_range_expr(codegen, start, end, step.as_deref(), types, w)?;
        }
        HirExprKind::Adfirma(cond, message) => generate_assert_expr(codegen, cond, message.as_deref(), types, w)?,
        HirExprKind::Error => {
            return Err(CodegenError { message: "cannot emit Go for error expression".to_owned() });
        }
    }
    Ok(())
}

fn generate_vacua_expr(codegen: &GoCodegen<'_>, expr: &HirExpr, types: &TypeTable, w: &mut CodeWriter) {
    match expr.ty.map(|ty| types.get(ty)) {
        Some(Type::Map(key, value)) => {
            w.write("make(map[");
            w.write(&types::type_to_go(codegen, *key, types));
            w.write("]");
            w.write(&types::type_to_go(codegen, *value, types));
            w.write(")");
        }
        Some(Type::Array(elem)) => {
            w.write("make([]");
            w.write(&types::type_to_go(codegen, *elem, types));
            w.write(", 0)");
        }
        Some(Type::Set(elem)) => {
            w.write("make(map[");
            w.write(&types::type_to_go(codegen, *elem, types));
            w.write("]struct{})");
        }
        _ => w.write("nil"),
    }
}

pub(super) fn generate_expr_for_go_type(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    expected_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match (&expr.kind, types.get(expected_ty)) {
        (_, Type::Option(inner)) => generate_option_wrapped_expr(codegen, expr, *inner, types, w),
        (HirExprKind::Array(elements), Type::Array(elem_ty)) => {
            generate_typed_array_expr(codegen, *elem_ty, elements, types, w)
        }
        (HirExprKind::Verte { entries: Some(entries), .. }, Type::Map(key_ty, value_ty)) => {
            generate_map_literal(codegen, *key_ty, *value_ty, Some(entries), types, w)
        }
        _ => generate_expr(codegen, expr, types, w),
    }
}

pub(super) fn normalize_receiver_type<'a>(mut ty: &'a Type, types: &'a TypeTable) -> &'a Type {
    loop {
        match ty {
            Type::Ref(_, inner) | Type::Alias(_, inner) => ty = types.get(*inner),
            _ => return ty,
        }
    }
}

pub(super) fn expr_return_type(expr: &HirExpr, types: &TypeTable, codegen: &GoCodegen<'_>) -> String {
    expr.ty
        .map(|ty| types::type_to_go(codegen, ty, types))
        .filter(|ty| !ty.is_empty())
        .unwrap_or_else(|| "any".to_owned())
}
pub(super) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}
