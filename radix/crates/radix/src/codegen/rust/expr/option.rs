use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_optional_chain_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    chain: &HirOptionalChainKind,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("(");
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    match chain {
        HirOptionalChainKind::Member(field) => {
            w.write(").as_ref().map(|__faber_opt| __faber_opt.");
            w.write(codegen.resolve_symbol(*field));
            w.write(")");
        }
        HirOptionalChainKind::Index(index) => {
            w.write(").as_ref().map(|__faber_opt| __faber_opt[");
            generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("])");
        }
        HirOptionalChainKind::Call(args) => {
            w.write(").and_then(|__faber_opt| Some(__faber_opt(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")))");
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_non_null_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    chain: &HirNonNullKind,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("(");
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(").expect(\"nonnull assertion failed\")");
    match chain {
        HirNonNullKind::Member(field) => {
            w.write(".");
            w.write(codegen.resolve_symbol(*field));
        }
        HirNonNullKind::Index(index) => {
            w.write("[");
            generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("]");
        }
        HirNonNullKind::Call(args) => {
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
        }
    }
    Ok(())
}
