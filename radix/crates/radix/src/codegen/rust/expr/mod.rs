//! Rust Expression Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Generates Rust expressions from HIR, handling error propagation (`?` operator),
//! async await (`.await`), reference creation (`&` and `&mut`), and control flow.
//!
//! COMPILER PHASE: Codegen (submodule)
//! INPUT: HirExpr nodes
//! OUTPUT: Rust expression source text
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Error propagation context: `?` operator inserted only in failable functions.
//!   WHY: Rust requires `?` to appear only in functions returning Result/Option.
//! - Suppress propagation in entry blocks: Entry code uses panic! for throws.
//!   WHY: Faber's `incipit` has no error return type; crashes are appropriate.
//! - Catch blocks suppress `?`: Errors are handled locally, not propagated.
//!   WHY: `tempta { iace "err" } cape { ... }` should not add `?` to the throw.

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

/// Generate a Rust expression.
///
/// TRANSFORMS:
///   salve(n)           -> salve(n) or salve(n)? (if failable)
///   iace "err"         -> return Err(String::from("err")) or panic!("err")
///   obj.method()       -> obj.method() or obj.method()? (if failable)
///   cede future_expr   -> future_expr.await
///   de expr            -> &expr
///   in expr            -> &mut expr
///
/// TARGET: Rust-specific `?` operator for error propagation.
/// EDGE: suppress_error_propagation=true in tempta catch blocks.
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
            w.write(codegen.resolve_def(*def_id));
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
        HirExprKind::Ab { source, filter, transforms } => generate_ab_expr(
            codegen,
            expr.id,
            source,
            filter.as_ref(),
            transforms,
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
        HirExprKind::Si(cond, then, else_) => generate_if_expr(
            codegen,
            cond,
            then,
            else_.as_ref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
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
        HirExprKind::Itera(_, binding, _binding_name, iter, block) => generate_for_expr(
            codegen,
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
        HirExprKind::Scribe(args) => {
            generate_scribe_expr(codegen, args, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
        HirExprKind::Conversio { source, target, params: _, fallback } => generate_conversio_expr(
            codegen,
            source,
            *target,
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
