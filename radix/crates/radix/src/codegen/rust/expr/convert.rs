use super::super::types::type_to_rust;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_conversio_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    target: TypeId,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let target_resolved = types.get(target);
    let source_ty = source.ty.map(|t| types.get(t));
    match (source_ty, target_resolved) {
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Numerus)) => generate_parse_expr(
            codegen,
            source,
            "i64",
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Fractus)) => generate_parse_expr(
            codegen,
            source,
            "f64",
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Bivalens)) => generate_text_bool_expr(
            codegen,
            source,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Textus))
        | (Some(Type::Primitive(Primitive::Fractus)), Type::Primitive(Primitive::Textus)) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(".to_string()");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Fractus)) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" as f64");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Bivalens)) => generate_number_bool_expr(
            codegen,
            source,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (_, Type::Primitive(Primitive::Textus)) => {
            w.write("format!(\"{}\", ");
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
            Ok(())
        }
        _ => match target_resolved {
            Type::Primitive(Primitive::Numerus) | Type::Primitive(Primitive::Fractus) => {
                let target_text = type_to_rust(codegen, target, types);
                generate_parse_expr(
                    codegen,
                    source,
                    &target_text,
                    fallback,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )
            }
            _ => {
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" as ");
                w.write(&type_to_rust(codegen, target, types));
                Ok(())
            }
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_parse_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    target: &str,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(".parse::<");
    w.write(target);
    if let Some(fallback) = fallback {
        w.write(">().unwrap_or(");
        generate_expr(
            codegen,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(")");
    } else {
        w.write(">().unwrap()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_text_bool_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        w.write("if ");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".is_empty() { ");
        generate_expr(codegen, fb, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" } else { true }");
    } else {
        w.write("!");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".is_empty()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_number_bool_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        w.write("if ");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" == 0 { ");
        generate_expr(codegen, fb, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" } else { true }");
    } else {
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" != 0");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_ref_expr(
    codegen: &RustCodegen<'_>,
    kind: HirRefKind,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match kind {
        HirRefKind::Shared => w.write("&"),
        HirRefKind::Mutable => w.write("&mut "),
    }
    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_deref_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("*");
    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}
