//! Argument planning helpers for Rust call emission.

use super::*;

pub(super) fn generate_call_args_with_target_types(
    emitter: &mut ExprEmitter<'_, '_>,
    target_tys: &[TypeId],
    args: &[HirCallArg],
) -> Result<(), CodegenError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            emitter.writer.write(", ");
        }
        generate_call_arg_expr(emitter, &arg.expr, target_tys.get(i).copied())?;
    }
    Ok(())
}

pub(super) fn call_arg_target_types(callee: &HirExpr, types: &TypeTable) -> Option<Vec<TypeId>> {
    match callee.ty.map(|ty| resolve_type(ty, types)) {
        Some(Type::Func(sig)) => Some(sig.params.iter().map(|param| param.ty).collect()),
        _ => None,
    }
}

pub(super) fn generate_direct_call_args_with_optional_params(
    emitter: &mut ExprEmitter<'_, '_>,
    params: &[super::super::super::FunctionParamInfo<'_>],
    args: &[HirCallArg],
) -> Result<(), CodegenError> {
    for (idx, param) in params.iter().enumerate() {
        if idx > 0 {
            emitter.writer.write(", ");
        }

        if let Some(arg) = args.get(idx) {
            if param.optional && param.default.is_none() {
                emitter.expr_as_optional_target(&arg.expr, param.ty)?;
            } else {
                generate_call_arg_expr(emitter, &arg.expr, Some(param.ty))?;
            }
        } else if let Some(default) = param.default {
            emitter.expr(default)?;
        } else if param.optional {
            emitter.writer.write("None");
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

pub(super) fn generate_spread_call_args(
    emitter: &mut ExprEmitter<'_, '_>,
    arg: &HirExpr,
    arity: usize,
) -> Result<(), CodegenError> {
    for index in 0..arity {
        if index > 0 {
            emitter.writer.write(", ");
        }
        emitter.writer.write("(");
        emitter.expr_unwrapped(arg)?;
        emitter.writer.write(&format!("[{index}usize].clone())"));
    }
    Ok(())
}

pub(super) fn generate_call_arg_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    arg: &HirExpr,
    target_ty: Option<TypeId>,
) -> Result<(), CodegenError> {
    if let Some(target_ty) = target_ty {
        emitter.expr_as_type(arg, target_ty)?;
    } else {
        emitter.expr_unwrapped(arg)?;
    }
    if call_arg_clones_owned_path(arg, emitter.types) {
        emitter.writer.write(".clone()");
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
