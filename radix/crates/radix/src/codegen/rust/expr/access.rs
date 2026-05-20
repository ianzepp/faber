use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_field_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    field: Symbol,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(".");
    w.write(codegen.resolve_symbol(field));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_index_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("[");
    generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("]");
    Ok(())
}
