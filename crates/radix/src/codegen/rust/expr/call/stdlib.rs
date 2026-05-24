//! Stdlib collection and text method translations for Rust call emission.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_generate_stdlib_method_call(
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
    let Some(receiver_ty) = receiver.ty.map(|ty| resolve_type(ty, types)) else {
        return Ok(false);
    };
    let method_name = codegen.resolve_symbol(method);

    match receiver_ty {
        Type::Array(elem_ty) => generate_lista_method(
            codegen,
            receiver,
            method_name,
            elem_ty,
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
        Type::Map(key_ty, value_ty) => generate_tabula_method(
            codegen,
            receiver,
            method_name,
            key_ty,
            value_ty,
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
    elem_ty: TypeId,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                elem_ty,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                elem_ty,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                elem_ty,
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
    key_ty: TypeId,
    value_ty: TypeId,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                key_ty,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(", ");
            generate_expr_as_type(
                codegen,
                &args[1].expr,
                value_ty,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                key_ty,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                key_ty,
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
            generate_expr_as_type(
                codegen,
                &args[0].expr,
                key_ty,
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
