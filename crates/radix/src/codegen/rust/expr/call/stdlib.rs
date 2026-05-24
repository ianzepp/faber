//! Stdlib collection and text method translations for Rust call emission.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_generate_stdlib_method_call(
    emitter: &mut ExprEmitter<'_, '_>,
    receiver: &HirExpr,
    method: Symbol,
    args: &[HirCallArg],
) -> Result<bool, CodegenError> {
    // TARGET: stdlib methods are recognized by receiver type, not by every
    // method name globally. This keeps ordinary user methods free to share
    // Latin names without being captured by the Rust backend.
    let Some(receiver_ty) = receiver.ty.map(|ty| resolve_type(ty, emitter.types)) else {
        return Ok(false);
    };
    let method_name = emitter.codegen.resolve_symbol(method);

    match receiver_ty {
        Type::Array(elem_ty) => generate_lista_method(emitter, receiver, method_name, elem_ty, args),
        Type::Primitive(Primitive::Textus) => generate_textus_method(emitter, receiver, method_name, args),
        Type::Map(key_ty, value_ty) => generate_tabula_method(emitter, receiver, method_name, key_ty, value_ty, args),
        _ => Ok(false),
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_lista_method(
    emitter: &mut ExprEmitter<'_, '_>,
    receiver: &HirExpr,
    method_name: &str,
    elem_ty: TypeId,
    args: &[HirCallArg],
) -> Result<bool, CodegenError> {
    match method_name {
        "appende" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".push(");
            emitter.expr_as_type(&args[0].expr, elem_ty)?;
            emitter.writer.write(")");
        }
        "longitudo" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".len() as i64");
        }
        "vacua" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".is_empty()");
        }
        "continet" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".contains(&");
            emitter.expr_as_type(&args[0].expr, elem_ty)?;
            emitter.writer.write(")");
        }
        "accipe" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".get(");
            emitter.expr_unwrapped(&args[0].expr)?;
            emitter.writer.write(" as usize).cloned()");
        }
        "primus" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".first().cloned()");
        }
        "ultimus" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".last().cloned()");
        }
        "remove" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".pop()");
        }
        "decapita" if args.is_empty() => {
            emitter.writer.write("if ");
            emitter.expr(receiver)?;
            emitter.writer.write(".is_empty() { None } else { Some(");
            emitter.expr(receiver)?;
            emitter.writer.write(".remove(0)) }");
        }
        "addita" if args.len() == 1 => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            emitter.writer.write("{ let mut ");
            emitter.writer.write(&temp);
            emitter.writer.write(" = ");
            emitter.expr(receiver)?;
            emitter.writer.write(".clone(); ");
            emitter.writer.write(&temp);
            emitter.writer.write(".push(");
            emitter.expr_as_type(&args[0].expr, elem_ty)?;
            emitter.writer.write("); ");
            emitter.writer.write(&temp);
            emitter.writer.write(" }");
        }
        "filtrata" if args.len() == 1 => {
            let pred = format!("__faber_pred_{}", receiver.id.0);
            emitter.writer.write("{ let mut ");
            emitter.writer.write(&pred);
            emitter.writer.write(" = ");
            emitter.expr(&args[0].expr)?;
            emitter.writer.write("; ");
            emitter.expr(receiver)?;
            emitter
                .writer
                .write(".iter().cloned().filter(|__faber_item| ");
            emitter.writer.write(&pred);
            emitter
                .writer
                .write("((*__faber_item).clone())).collect::<Vec<_>>() }");
        }
        "map" | "mappata" if args.len() == 1 => {
            let mapper = format!("__faber_map_{}", receiver.id.0);
            emitter.writer.write("{ let mut ");
            emitter.writer.write(&mapper);
            emitter.writer.write(" = ");
            emitter.expr(&args[0].expr)?;
            emitter.writer.write("; ");
            emitter.expr(receiver)?;
            emitter.writer.write(".iter().cloned().map(|__faber_item| ");
            emitter.writer.write(&mapper);
            emitter
                .writer
                .write("(__faber_item)).collect::<Vec<_>>() }");
        }
        "inversa" if args.is_empty() => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            emitter.writer.write("{ let mut ");
            emitter.writer.write(&temp);
            emitter.writer.write(" = ");
            emitter.expr(receiver)?;
            emitter.writer.write(".clone(); ");
            emitter.writer.write(&temp);
            emitter.writer.write(".reverse(); ");
            emitter.writer.write(&temp);
            emitter.writer.write(" }");
        }
        "inverte" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".reverse()");
        }
        "ordinata" if args.is_empty() => {
            let temp = format!("__faber_list_{}", receiver.id.0);
            emitter.writer.write("{ let mut ");
            emitter.writer.write(&temp);
            emitter.writer.write(" = ");
            emitter.expr(receiver)?;
            emitter.writer.write(".clone(); ");
            emitter.writer.write(&temp);
            emitter.writer.write(".sort(); ");
            emitter.writer.write(&temp);
            emitter.writer.write(" }");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn generate_textus_method(
    emitter: &mut ExprEmitter<'_, '_>,
    receiver: &HirExpr,
    method_name: &str,
    args: &[HirCallArg],
) -> Result<bool, CodegenError> {
    match method_name {
        "longitudo" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".len() as i64");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn generate_tabula_method(
    emitter: &mut ExprEmitter<'_, '_>,
    receiver: &HirExpr,
    method_name: &str,
    key_ty: TypeId,
    value_ty: TypeId,
    args: &[HirCallArg],
) -> Result<bool, CodegenError> {
    match method_name {
        "pone" if args.len() == 2 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".insert(");
            emitter.expr_as_type(&args[0].expr, key_ty)?;
            emitter.writer.write(", ");
            emitter.expr_as_type(&args[1].expr, value_ty)?;
            emitter.writer.write(")");
        }
        "accipe" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".get(&");
            emitter.expr_as_type(&args[0].expr, key_ty)?;
            emitter.writer.write(").cloned()");
        }
        "habet" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".contains_key(&");
            emitter.expr_as_type(&args[0].expr, key_ty)?;
            emitter.writer.write(")");
        }
        "dele" if args.len() == 1 => {
            emitter.expr(receiver)?;
            emitter.writer.write(".remove(&");
            emitter.expr_as_type(&args[0].expr, key_ty)?;
            emitter.writer.write(").is_some()");
        }
        "longitudo" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".len() as i64");
        }
        "vacua" if args.is_empty() => {
            emitter.expr(receiver)?;
            emitter.writer.write(".is_empty()");
        }
        _ => return Ok(false),
    }
    Ok(true)
}
