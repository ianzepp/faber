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
    let object_is_option = expr_type_is_option(object, types);
    let value_is_option = optional_chain_value_is_option(codegen, object, chain, types);

    match chain {
        HirOptionalChainKind::Member(field) => {
            if object_is_option {
                w.write("(");
                generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                if value_is_option {
                    w.write(").as_ref().and_then(|__faber_opt| __faber_opt.");
                } else {
                    w.write(").as_ref().map(|__faber_opt| __faber_opt.");
                }
                w.write(codegen.resolve_symbol(*field));
                w.write(".clone())");
            } else {
                if !value_is_option {
                    w.write("Some(");
                }
                generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(".");
                w.write(codegen.resolve_symbol(*field));
                w.write(".clone()");
                if !value_is_option {
                    w.write(")");
                }
            }
        }
        HirOptionalChainKind::Index(index) => {
            generate_optional_index_expr(
                codegen,
                object,
                index,
                object_is_option,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirOptionalChainKind::Call(args) => {
            w.write("(");
            generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
fn generate_optional_index_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    object_is_option: bool,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if object_is_option {
        w.write("(");
        generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(").as_ref().and_then(|__faber_opt| __faber_opt.get(");
        generate_optional_index_key(
            codegen,
            object,
            index,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(").cloned())");
        return Ok(());
    }

    w.write("(");
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(").get(");
    generate_optional_index_key(
        codegen,
        object,
        index,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )?;
    w.write(").cloned()");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_optional_index_key(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if optional_chain_inner_type(object, types).is_some_and(|ty| matches!(resolve_type(ty, types), Type::Array(_))) {
        w.write("(");
        generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(") as usize");
        return Ok(());
    }

    w.write("&");
    generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

fn optional_chain_value_is_option(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    chain: &HirOptionalChainKind,
    types: &TypeTable,
) -> bool {
    match chain {
        HirOptionalChainKind::Member(field) => {
            optional_chain_inner_type(object, types).is_some_and(|ty| match resolve_type(ty, types) {
                Type::Struct(def_id) => codegen.struct_field_stores_option(def_id, *field, types),
                Type::Map(_, value_ty) => type_id_is_option(value_ty, types),
                _ => false,
            })
        }
        HirOptionalChainKind::Index(_) => {
            optional_chain_inner_type(object, types).is_some_and(|ty| match resolve_type(ty, types) {
                Type::Array(elem) | Type::Map(_, elem) => type_id_is_option(elem, types),
                _ => false,
            })
        }
        HirOptionalChainKind::Call(_) => false,
    }
}

fn optional_chain_inner_type(expr: &HirExpr, types: &TypeTable) -> Option<TypeId> {
    let ty = expr.ty?;
    match resolve_type(ty, types) {
        Type::Option(inner) => Some(inner),
        _ => Some(ty),
    }
}

fn expr_type_is_option(expr: &HirExpr, types: &TypeTable) -> bool {
    expr.ty.is_some_and(|ty| type_id_is_option(ty, types))
}

fn type_id_is_option(type_id: TypeId, types: &TypeTable) -> bool {
    match types.get(type_id) {
        Type::Option(_) => true,
        Type::Alias(_, resolved) => type_id_is_option(*resolved, types),
        _ => false,
    }
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
