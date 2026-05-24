//! Argument planning helpers for Rust call emission.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_call_args_with_target_types(
    codegen: &RustCodegen<'_>,
    target_tys: &[TypeId],
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        generate_call_arg_expr(
            codegen,
            &arg.expr,
            target_tys.get(i).copied(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    }
    Ok(())
}

pub(super) fn call_arg_target_types(callee: &HirExpr, types: &TypeTable) -> Option<Vec<TypeId>> {
    match callee.ty.map(|ty| resolve_type(ty, types)) {
        Some(Type::Func(sig)) => Some(sig.params.iter().map(|param| param.ty).collect()),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_direct_call_args_with_optional_params(
    codegen: &RustCodegen<'_>,
    params: &[super::super::super::FunctionParamInfo<'_>],
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
            if param.optional && param.default.is_none() {
                generate_expr_as_optional_target(
                    codegen,
                    &arg.expr,
                    param.ty,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            } else {
                generate_call_arg_expr(
                    codegen,
                    &arg.expr,
                    Some(param.ty),
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

pub(super) fn direct_spread_call_arity(
    codegen: &RustCodegen<'_>,
    callee: &HirExpr,
    args: &[HirCallArg],
) -> Option<usize> {
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
pub(super) fn generate_spread_call_args(
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
pub(super) fn generate_call_arg_expr(
    codegen: &RustCodegen<'_>,
    arg: &HirExpr,
    target_ty: Option<TypeId>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(target_ty) = target_ty {
        generate_expr_as_type(
            codegen,
            arg,
            target_ty,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    } else {
        generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
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
