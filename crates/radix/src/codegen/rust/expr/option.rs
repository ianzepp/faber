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

use super::super::type_shape::{resolve_type, type_id_is_option};
use super::*;

pub(super) fn generate_optional_chain_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    chain: &HirOptionalChainKind,
) -> Result<(), CodegenError> {
    let object_is_option = expr_type_is_option(object, emitter.types);
    let value_is_option = optional_chain_value_is_option(emitter.codegen, object, chain, emitter.types);

    if optional_chain_inner_type(object, emitter.types)
        .is_some_and(|ty| matches!(resolve_type(ty, emitter.types), Type::Primitive(Primitive::Nihil)))
    {
        emitter.writer.write("None::<FaberValue>");
        return Ok(());
    }

    match chain {
        HirOptionalChainKind::Member(field) => {
            if generate_optional_map_member_expr(emitter, object, *field, object_is_option, value_is_option)? {
                return Ok(());
            }

            if object_is_option {
                emitter.writer.write("(");
                emitter.expr(object)?;
                if value_is_option {
                    emitter
                        .writer
                        .write(").as_ref().and_then(|__faber_opt| __faber_opt.");
                } else {
                    emitter
                        .writer
                        .write(").as_ref().map(|__faber_opt| __faber_opt.");
                }
                emitter.writer.write(emitter.codegen.resolve_symbol(*field));
                emitter.writer.write(".clone())");
            } else {
                if !value_is_option {
                    emitter.writer.write("Some(");
                }
                emitter.expr(object)?;
                emitter.writer.write(".");
                emitter.writer.write(emitter.codegen.resolve_symbol(*field));
                emitter.writer.write(".clone()");
                if !value_is_option {
                    emitter.writer.write(")");
                }
            }
        }
        HirOptionalChainKind::Index(index) => {
            generate_optional_index_expr(emitter, object, index, object_is_option)?;
        }
        HirOptionalChainKind::Call(args) => {
            emitter.writer.write("(");
            emitter.expr(object)?;
            emitter
                .writer
                .write(").and_then(|__faber_opt| Some(__faber_opt(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    emitter.writer.write(", ");
                }
                emitter.expr(&arg.expr)?;
            }
            emitter.writer.write(")))");
        }
    }
    Ok(())
}

fn generate_optional_map_member_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    field: Symbol,
    object_is_option: bool,
    value_is_option: bool,
) -> Result<bool, CodegenError> {
    let Some(Type::Map(key_ty, _)) =
        optional_chain_inner_type(object, emitter.types).map(|ty| resolve_type(ty, emitter.types))
    else {
        return Ok(false);
    };
    if !matches!(resolve_type(key_ty, emitter.types), Type::Primitive(Primitive::Textus)) {
        return Ok(false);
    }

    if object_is_option {
        emitter.writer.write("(");
        emitter.expr(object)?;
        emitter
            .writer
            .write(").as_ref().and_then(|__faber_opt| __faber_opt.get(\"");
        emitter.writer.write(emitter.codegen.resolve_symbol(field));
        if value_is_option {
            emitter.writer.write("\").cloned().flatten())");
        } else {
            emitter.writer.write("\").cloned())");
        }
    } else {
        emitter.writer.write("(");
        emitter.expr(object)?;
        emitter.writer.write(").get(\"");
        emitter.writer.write(emitter.codegen.resolve_symbol(field));
        if value_is_option {
            emitter.writer.write("\").cloned().flatten()");
        } else {
            emitter.writer.write("\").cloned()");
        }
    }

    Ok(true)
}

fn generate_optional_index_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    index: &HirExpr,
    object_is_option: bool,
) -> Result<(), CodegenError> {
    if object_is_option {
        emitter.writer.write("(");
        emitter.expr(object)?;
        emitter
            .writer
            .write(").as_ref().and_then(|__faber_opt| __faber_opt.get(");
        generate_optional_index_key(emitter, object, index)?;
        emitter.writer.write(").cloned())");
        return Ok(());
    }

    emitter.writer.write("(");
    emitter.expr(object)?;
    emitter.writer.write(").get(");
    generate_optional_index_key(emitter, object, index)?;
    emitter.writer.write(").cloned()");
    Ok(())
}

fn generate_optional_index_key(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    index: &HirExpr,
) -> Result<(), CodegenError> {
    if optional_chain_inner_type(object, emitter.types)
        .is_some_and(|ty| matches!(resolve_type(ty, emitter.types), Type::Array(_)))
    {
        emitter.writer.write("(");
        emitter.expr(index)?;
        emitter.writer.write(") as usize");
        return Ok(());
    }

    emitter.writer.write("&");
    emitter.expr(index)
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

pub(super) fn generate_non_null_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    chain: &HirNonNullKind,
) -> Result<(), CodegenError> {
    emitter.writer.write("(");
    emitter.expr(object)?;
    emitter
        .writer
        .write(").expect(\"nonnull assertion failed\")");
    match chain {
        HirNonNullKind::Member(field) => {
            emitter.writer.write(".");
            emitter.writer.write(emitter.codegen.resolve_symbol(*field));
        }
        HirNonNullKind::Index(index) => {
            emitter.writer.write("[");
            emitter.expr(index)?;
            emitter.writer.write("]");
        }
        HirNonNullKind::Call(args) => {
            emitter.writer.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    emitter.writer.write(", ");
                }
                emitter.expr(&arg.expr)?;
            }
            emitter.writer.write(")");
        }
    }
    Ok(())
}
