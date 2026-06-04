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

pub(super) fn generate_call_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    callee: &HirExpr,
    args: &[HirCallArg],
) -> Result<(), CodegenError> {
    // Only direct path calls carry stable failable-def metadata here. Other
    // callable shapes are emitted without speculative propagation.
    if let HirExprKind::Path(def_id) = callee.kind {
        if let Some(variant) = emitter.codegen.variant_info(def_id) {
            return generate_variant_constructor_expr(emitter, def_id, variant, args);
        }
    }

    let is_failable_call =
        matches!(&callee.kind, HirExprKind::Path(def_id) if emitter.codegen.is_failable_def(*def_id));
    emitter.expr(callee)?;
    emitter.writer.write("(");
    if let Some(spread) = direct_spread_call_arity(emitter.codegen, callee, args) {
        generate_spread_call_args(emitter, &args[0].expr, spread)?;
    } else if let HirExprKind::Path(def_id) = callee.kind {
        if let Some(params) = emitter.codegen.function_params(def_id) {
            generate_direct_call_args_with_optional_params(emitter, params, args)?;
        } else if let Some(target_tys) = call_arg_target_types(callee, emitter.types) {
            generate_call_args_with_target_types(emitter, &target_tys, args)?;
        } else {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    emitter.writer.write(", ");
                }
                generate_call_arg_expr(emitter, &arg.expr, None)?;
            }
        }
    } else {
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                emitter.writer.write(", ");
            }
            generate_call_arg_expr(emitter, &arg.expr, None)?;
        }
    }
    emitter.writer.write(")");
    if is_failable_call && emitter.policy.permits_question_mark() {
        emitter.writer.write("?");
    }
    Ok(())
}

fn generate_variant_constructor_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    variant: &super::super::VariantInfo,
    args: &[HirCallArg],
) -> Result<(), CodegenError> {
    emitter
        .writer
        .write(emitter.codegen.resolve_def(variant.enum_def));
    emitter.writer.write("::");
    emitter.writer.write(emitter.codegen.resolve_def(def_id));

    if variant.fields.is_empty() {
        return Ok(());
    }

    emitter.writer.write(" { ");
    for (idx, field) in variant.fields.iter().enumerate() {
        if idx > 0 {
            emitter.writer.write(", ");
        }
        emitter.writer.write(emitter.codegen.resolve_symbol(*field));
        emitter.writer.write(": ");
        let arg = args
            .iter()
            .find(|arg| arg.name == Some(*field))
            .or_else(|| args.get(idx));
        if let Some(arg) = arg {
            generate_call_arg_expr(emitter, &arg.expr, None)?;
        } else {
            emitter.writer.write("Default::default()");
        }
    }
    emitter.writer.write(" }");
    Ok(())
}

pub(super) fn generate_method_call_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    receiver: &HirExpr,
    method: Symbol,
    args: &[HirCallArg],
) -> Result<(), CodegenError> {
    if try_generate_stdlib_method_call(emitter, receiver, method, args)? {
        return Ok(());
    }

    // Library pactum imports typecheck as module-like values in Faber, but
    // target metadata may link them as functions in a runtime crate.
    if let HirExprKind::Path(def_id) = &receiver.kind {
        if let Some(runtime_module) = library_runtime_module_path(*def_id, &emitter.codegen.libraries) {
            emitter.writer.write(runtime_module);
            emitter.writer.write("::");
            emitter
                .writer
                .write(&library_runtime_method_name(emitter.codegen.resolve_symbol(method)));
            emitter.writer.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    emitter.writer.write(", ");
                }
                generate_call_arg_expr(emitter, &arg.expr, None)?;
            }
            emitter.writer.write(")");
            if emitter.codegen.is_failable_method_name(method) && emitter.policy.permits_question_mark() {
                emitter.writer.write("?");
            }
            return Ok(());
        }
    }

    let is_failable_call = emitter.codegen.is_failable_method_name(method);
    emitter.expr(receiver)?;
    emitter.writer.write(".");
    emitter.writer.write(emitter.codegen.resolve_symbol(method));
    emitter.writer.write("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            emitter.writer.write(", ");
        }
        generate_call_arg_expr(emitter, &arg.expr, None)?;
    }
    emitter.writer.write(")");
    if is_failable_call && emitter.policy.permits_question_mark() {
        emitter.writer.write("?");
    }
    Ok(())
}
