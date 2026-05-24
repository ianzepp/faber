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

use super::*;

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
        if let Some(params) = codegen
            .function_params(def_id)
            .filter(|params| params.iter().any(|param| param.optional))
        {
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
        } else {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
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
fn generate_direct_call_args_with_optional_params(
    codegen: &RustCodegen<'_>,
    params: &[super::super::FunctionParamInfo<'_>],
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }

        if let Some(arg) = args.get(idx) {
            if param.optional
                && param.default.is_none()
                && !matches!(arg.expr.kind, HirExprKind::Literal(HirLiteral::Nil))
            {
                w.write("Some(");
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
                w.write(")");
            } else {
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            }
        } else if let Some(default) = param.default {
            generate_expr(codegen, default, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        } else if param.optional {
            w.write("None");
        }
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
        if let Some(arg) = args.get(idx) {
            generate_call_arg_expr(
                codegen,
                &arg.expr,
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

fn direct_spread_call_arity(codegen: &RustCodegen<'_>, callee: &HirExpr, args: &[HirCallArg]) -> Option<usize> {
    if args.len() != 1 || !args[0].spread {
        return None;
    }
    let HirExprKind::Path(def_id) = callee.kind else {
        return None;
    };
    codegen
        .function_param_count(def_id)
        .filter(|count| *count > 1)
}

#[allow(clippy::too_many_arguments)]
fn generate_spread_call_args(
    codegen: &RustCodegen<'_>,
    arg: &HirExpr,
    arity: usize,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    for index in 0..arity {
        if index > 0 {
            w.write(", ");
        }
        w.write("(");
        generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(&format!("[{index}usize].clone())"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_call_arg_expr(
    codegen: &RustCodegen<'_>,
    arg: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if call_arg_clones_owned_path(arg, types) {
        w.write(".clone()");
    }
    Ok(())
}

fn call_arg_clones_owned_path(arg: &HirExpr, types: &TypeTable) -> bool {
    if !matches!(arg.kind, HirExprKind::Path(_)) {
        return false;
    }

    arg.ty.is_some_and(|ty| {
        matches!(
            resolve_type(ty, types),
            Type::Array(_) | Type::Map(_, _) | Type::Option(_) | Type::Primitive(Primitive::Textus)
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn try_generate_stdlib_method_call(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method: Symbol,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<bool, CodegenError> {
    // TARGET: stdlib methods are recognized by receiver type, not by every
    // method name globally. This keeps ordinary user methods free to share
    // Latin names without being captured by the Rust backend.
    let Some(receiver_ty) = receiver.ty.map(|ty| types.get(ty)) else {
        return Ok(false);
    };
    let method_name = codegen.resolve_symbol(method);

    match receiver_ty {
        Type::Array(_) => generate_lista_method(
            codegen,
            receiver,
            method_name,
            args,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        Type::Primitive(Primitive::Textus) => generate_textus_method(
            codegen,
            receiver,
            method_name,
            args,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        Type::Map(_, _) => generate_tabula_method(
            codegen,
            receiver,
            method_name,
            args,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_lista_method(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method_name: &str,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<bool, CodegenError> {
    match method_name {
        "appende" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".push(");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(")");
        }
        "longitudo" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".len() as i64");
        }
        "vacua" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".is_empty()");
        }
        "continet" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".contains(&");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(")");
        }
        "accipe" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".get(");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(" as usize).cloned()");
        }
        "primus" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".first().cloned()");
        }
        "ultimus" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".last().cloned()");
        }
        "remove" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".pop()");
        }
        "decapita" if args.is_empty() => {
            w.write("if ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".is_empty() { None } else { Some(");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".remove(0)) }");
        }
        "addita" if args.len() == 1 => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            w.write("{ let mut ");
            w.write(&temp);
            w.write(" = ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".clone(); ");
            w.write(&temp);
            w.write(".push(");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write("); ");
            w.write(&temp);
            w.write(" }");
        }
        "filtrata" if args.len() == 1 => {
            let pred = format!("__faber_pred_{}", receiver.id.0);
            w.write("{ let mut ");
            w.write(&pred);
            w.write(" = ");
            generate_expr(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write("; ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".iter().cloned().filter(|__faber_item| ");
            w.write(&pred);
            w.write("((*__faber_item).clone())).collect::<Vec<_>>() }");
        }
        "map" | "mappata" if args.len() == 1 => {
            let mapper = format!("__faber_map_{}", receiver.id.0);
            w.write("{ let mut ");
            w.write(&mapper);
            w.write(" = ");
            generate_expr(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write("; ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".iter().cloned().map(|__faber_item| ");
            w.write(&mapper);
            w.write("(__faber_item)).collect::<Vec<_>>() }");
        }
        "inversa" if args.is_empty() => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            w.write("{ let mut ");
            w.write(&temp);
            w.write(" = ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".clone(); ");
            w.write(&temp);
            w.write(".reverse(); ");
            w.write(&temp);
            w.write(" }");
        }
        "inverte" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".reverse()");
        }
        "ordinata" if args.is_empty() => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            w.write("{ let mut ");
            w.write(&temp);
            w.write(" = ");
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".clone(); ");
            w.write(&temp);
            w.write(".sort(); ");
            w.write(&temp);
            w.write(" }");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn generate_textus_method(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method_name: &str,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<bool, CodegenError> {
    match method_name {
        "longitudo" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".len() as i64");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn generate_tabula_method(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method_name: &str,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<bool, CodegenError> {
    match method_name {
        "pone" if args.len() == 2 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".insert(");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(", ");
            generate_expr_unwrapped(
                codegen,
                &args[1].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(")");
        }
        "accipe" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".get(&");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(").cloned()");
        }
        "habet" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".contains_key(&");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(")");
        }
        "dele" if args.len() == 1 => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".remove(&");
            generate_expr_unwrapped(
                codegen,
                &args[0].expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(").is_some()");
        }
        "longitudo" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".len() as i64");
        }
        "vacua" if args.is_empty() => {
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".is_empty()");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn norma_runtime_module_path(receiver_name: &str) -> Option<&'static str> {
    match receiver_name {
        "json" => Some("norma::json"),
        "toml" => Some("norma::toml"),
        "consolum" => Some("norma::hal::consolum"),
        _ => None,
    }
}

fn norma_runtime_method_name(method_name: &str) -> String {
    let mut lowered = String::with_capacity(method_name.len());
    for (i, ch) in method_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                lowered.push('_');
            }
            lowered.push(ch.to_ascii_lowercase());
        } else {
            lowered.push(ch);
        }
    }
    lowered
}
