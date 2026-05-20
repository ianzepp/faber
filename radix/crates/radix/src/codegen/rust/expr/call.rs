use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_call_expr(
    codegen: &RustCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let is_failable_call = matches!(&callee.kind, HirExprKind::Path(def_id) if codegen.is_failable_def(*def_id));
    generate_expr(codegen, callee, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
        w.write("?");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_method_call_expr(
    codegen: &RustCodegen<'_>,
    receiver: &HirExpr,
    method: Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
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
        generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
        w.write("?");
    }
    Ok(())
}
