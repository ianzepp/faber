use super::super::types::type_to_rust;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_verte_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    source: &HirExpr,
    target: TypeId,
    entries: Option<&[HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match types.get(target) {
        Type::Struct(def_id) => generate_verte_struct_expr(
            codegen,
            source,
            *def_id,
            entries,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        Type::Array(elem) => generate_verte_array_expr(
            codegen,
            expr_id,
            source,
            *elem,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        Type::Map(key_ty, value_ty) => generate_verte_map_expr(
            codegen,
            expr_id,
            *key_ty,
            *value_ty,
            entries,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        Type::Primitive(Primitive::Numerus)
        | Type::Primitive(Primitive::Fractus)
        | Type::Primitive(Primitive::Bivalens) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" as ");
            w.write(&type_to_rust(codegen, target, types));
            Ok(())
        }
        Type::Primitive(Primitive::Textus) => {
            w.write("format!(\"{}\", ");
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
            Ok(())
        }
        Type::Enum(_) | Type::Interface(_) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)
        }
        _ => generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation),
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_verte_struct_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    def_id: DefId,
    entries: Option<&[HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(entries) = entries {
        w.write(codegen.resolve_def(def_id));
        w.writeln(" {");
        let mut struct_result = Ok(());
        w.indented(|w| {
            for field in entries {
                let (name, value) = match (&field.key, &field.value) {
                    (HirObjectKey::Ident(name) | HirObjectKey::String(name), Some(value)) => (name, value),
                    _ => continue,
                };
                w.write(codegen.resolve_symbol(*name));
                w.write(": ");
                if struct_result.is_err() {
                    return;
                }
                struct_result =
                    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                w.writeln(",");
            }
        });
        struct_result?;
        w.write("}");
        return Ok(());
    }

    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
fn generate_verte_array_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    source: &HirExpr,
    elem: TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let HirExprKind::Array(elements) = &source.kind {
        if elements.is_empty() {
            w.write("Vec::<");
            w.write(&type_to_rust(codegen, elem, types));
            w.write(">::new()");
            return Ok(());
        }
        if elements
            .iter()
            .any(|element| matches!(element, HirArrayElement::Spread(_)))
        {
            let temp = format!("__faber_verte_vec_{}", expr_id.0);
            w.writeln("{");
            let mut array_result = Ok(());
            w.indented(|w| {
                w.write("let mut ");
                w.write(&temp);
                w.writeln(" = Vec::new();");
                for element in elements {
                    if array_result.is_err() {
                        return;
                    }
                    match element {
                        HirArrayElement::Expr(elem_expr) => {
                            w.write(&temp);
                            w.write(".push(");
                            array_result = generate_expr(
                                codegen,
                                elem_expr,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            );
                            w.writeln(");");
                        }
                        HirArrayElement::Spread(elem_expr) => {
                            w.write(&temp);
                            w.write(".extend(");
                            array_result = generate_expr(
                                codegen,
                                elem_expr,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            );
                            w.writeln(");");
                        }
                    }
                }
                w.write(&temp);
                w.newline();
            });
            array_result?;
            w.write("}");
            return Ok(());
        }

        w.write("vec![");
        for (i, elem_expr) in elements.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            let HirArrayElement::Expr(elem_expr) = elem_expr else {
                continue;
            };
            generate_expr(
                codegen,
                elem_expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        w.write("]");
        return Ok(());
    }

    w.write("Vec::<");
    w.write(&type_to_rust(codegen, elem, types));
    w.write(">::new()");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_verte_map_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    key_ty: TypeId,
    value_ty: TypeId,
    entries: Option<&[HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(entries) = entries {
        let map_name = format!("__faber_verte_map_{}", expr_id.0);
        w.writeln("{");
        let mut map_result = Ok(());
        w.indented(|w| {
            w.write("let mut ");
            w.write(&map_name);
            w.write(" = std::collections::HashMap::<");
            w.write(&type_to_rust(codegen, key_ty, types));
            w.write(", ");
            w.write(&type_to_rust(codegen, value_ty, types));
            w.writeln(">::new();");
            for field in entries {
                match (&field.key, &field.value) {
                    (HirObjectKey::Spread(expr), _) => {
                        w.write(&map_name);
                        w.write(".extend(");
                        if map_result.is_err() {
                            return;
                        }
                        map_result = generate_expr(
                            codegen,
                            expr,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(");");
                    }
                    (_, Some(value)) => {
                        w.write(&map_name);
                        w.write(".insert(");
                        if map_result.is_err() {
                            return;
                        }
                        map_result = write_object_map_key(
                            codegen,
                            types,
                            &field.key,
                            key_ty,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        if map_result.is_err() {
                            return;
                        }
                        w.write(", ");
                        map_result = generate_expr(
                            codegen,
                            value,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(");");
                    }
                    (_, None) => {}
                }
            }
            w.write(&map_name);
            w.newline();
        });
        map_result?;
        w.write("}");
        return Ok(());
    }

    w.write("std::collections::HashMap::<");
    w.write(&type_to_rust(codegen, key_ty, types));
    w.write(", ");
    w.write(&type_to_rust(codegen, value_ty, types));
    w.write(">::new()");
    Ok(())
}
