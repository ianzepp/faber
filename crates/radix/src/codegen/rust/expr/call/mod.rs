//! Function, method, and stdlib-call lowering for Rust expressions.
//!
//! This module owns the backend's call-shape decisions. Ordinary calls and
//! methods preserve their HIR names, selected stdlib collection methods are
//! translated to native Rust collection APIs, and a small built-in `norma`
//! module bridge rewrites module-like Faber receivers to runtime crate paths.
//!
//! PROPAGATION POLICY
//! ==================
//! A call receives `?` only when all of these are true:
//! - the Rust backend failable prepass marked the callee or method name as
//!   failable;
//! - the current Rust function can legally use `?`;
//! - codegen is not emitting the entrypoint; and
//! - the surrounding construct has not suppressed propagation.
//!
//! The stdlib translation boundary is intentionally narrow. If a method is not
//! one of the collection/runtime forms recognized here, this module falls back
//! to direct Rust-style method emission instead of inventing target behavior.

mod args;
mod runtime;
mod stdlib;

use super::*;
use args::*;
use runtime::*;
use stdlib::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_call_expr(
    codegen: &RustCodegen<'_>,
    callee: &HirExpr,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // Only direct path calls carry stable failable-def metadata here. Other
    // callable shapes are emitted without speculative propagation.
    if let HirExprKind::Path(def_id) = callee.kind {
        if let Some(variant) = codegen.variant_info(def_id) {
            return generate_variant_constructor_expr(
                codegen,
                def_id,
                variant,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
        }
    }

    let is_failable_call = matches!(&callee.kind, HirExprKind::Path(def_id) if codegen.is_failable_def(*def_id));
    generate_expr(codegen, callee, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("(");
    if let Some(spread) = direct_spread_call_arity(codegen, callee, args) {
        generate_spread_call_args(
            codegen,
            &args[0].expr,
            spread,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    } else if let HirExprKind::Path(def_id) = callee.kind {
        if let Some(params) = codegen.function_params(def_id) {
            generate_direct_call_args_with_optional_params(
                codegen,
                params,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else if let Some(target_tys) = call_arg_target_types(callee, types) {
            generate_call_args_with_target_types(
                codegen,
                &target_tys,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
                    None,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            }
        }
    } else {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            generate_call_arg_expr(
                codegen,
                &arg.expr,
                None,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
    }
    w.write(")");
    if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
        w.write("?");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_variant_constructor_expr(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    variant: &super::super::VariantInfo,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(variant.enum_def));
    w.write("::");
    w.write(codegen.resolve_def(def_id));

    if variant.fields.is_empty() {
        return Ok(());
    }

    w.write(" { ");
    for (idx, field) in variant.fields.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(*field));
        w.write(": ");
        let arg = args
            .iter()
            .find(|arg| arg.name == Some(*field))
            .or_else(|| args.get(idx));
        if let Some(arg) = arg {
            generate_call_arg_expr(
                codegen,
                &arg.expr,
                None,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
            w.write("Default::default()");
        }
    }
    w.write(" }");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_method_call_expr(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method: Symbol,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if try_generate_stdlib_method_call(
        codegen,
        receiver,
        method,
        args,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )? {
        return Ok(());
    }

    // Targeted bridge for built-in norma library modules.
    // WHY: Built-in pactum imports typecheck as module-like values in Faber,
    // but Rust links them as functions in the norma runtime crate.
    if let HirExprKind::Path(def_id) = &receiver.kind {
        let recv_name = codegen.resolve_def(*def_id);
        if let Some(runtime_module) = norma_runtime_module_path(recv_name) {
            w.write(runtime_module);
            w.write("::");
            w.write(&norma_runtime_method_name(codegen.resolve_symbol(method)));
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
                    None,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            }
            w.write(")");
            if codegen.is_failable_method_name(method) && in_failable_fn && !in_entry && !suppress_error_propagation {
                w.write("?");
            }
            return Ok(());
        }
    }

    let is_failable_call = codegen.is_failable_method_name(method);
    generate_expr(
        codegen,
        receiver,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )?;
    w.write(".");
    w.write(codegen.resolve_symbol(method));
    w.write("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        generate_call_arg_expr(
            codegen,
            &arg.expr,
            None,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    }
    w.write(")");
    if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
        w.write("?");
    }
    Ok(())
}
