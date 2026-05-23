//! Optional access lowering for Faber nullable values.
//!
//! Nullable Faber types are represented through Rust `Option` at this boundary.
//! Optional chaining maps to `as_ref().map(...)` or `and_then(...)`, while
//! non-null assertions map to `expect(...)` before applying the requested
//! member, index, or call. The emitted code is explicit about cloning because
//! optional access returns owned values in the current HIR contracts. Binary
//! coalescing is lowered in the sibling operator module because it is parsed as
//! `HirBinOp::Coalesce`, but it shares the same `Option<T>` storage model.
//!
//! EDGE CASES
//! ==========
//! - Optional member and index access borrow the option, then clone the selected
//!   value.
//! - Optional call uses `and_then` with `Some(...)`; it does not infer or flatten
//!   arbitrary user-returned nullable contracts beyond that shape.
//! - Non-null assertions intentionally panic with a stable message if a value is
//!   `None`.

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
            w.write(".clone())");
        }
        HirOptionalChainKind::Index(index) => {
            w.write(").as_ref().map(|__faber_opt| __faber_opt[");
            generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("].clone())");
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
