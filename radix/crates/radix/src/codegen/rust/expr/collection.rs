use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_ab_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    source: &HirExpr,
    filter: Option<&HirCollectionFilter>,
    transforms: &[HirCollectionTransform],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let suffix = expr_id.0;
    let vec_name = format!("__faber_ab_vec_{}", suffix);
    let n_name = format!("__faber_ab_n_{}", suffix);
    let len_name = format!("__faber_ab_len_{}", suffix);
    let keep_name = format!("__faber_ab_keep_{}", suffix);
    let sum_name = format!("__faber_ab_sum_{}", suffix);
    let item_name = format!("__faber_ab_item_{}", suffix);

    w.writeln("{");
    let mut ab_result = Ok(());
    w.indented(|w| {
        w.write("let mut ");
        w.write(&vec_name);
        w.write(" = (");
        if ab_result.is_err() {
            return;
        }
        ab_result = generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation);
        w.write(").iter()");
        if let Some(filter) = filter {
            match &filter.kind {
                HirCollectionFilterKind::Property(name) => {
                    w.write(".filter(|");
                    w.write(&item_name);
                    w.write("| ");
                    if filter.negated {
                        w.write("!");
                    }
                    w.write(&item_name);
                    w.write(".");
                    w.write(codegen.resolve_symbol(*name));
                    w.write(")");
                }
                HirCollectionFilterKind::Condition(cond) => {
                    w.write(".filter(|_| ");
                    if ab_result.is_err() {
                        return;
                    }
                    ab_result =
                        generate_expr(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                    w.write(")");
                }
            }
        }
        w.writeln(".collect::<Vec<_>>();");

        let mut terminal_sum = false;
        for transform in transforms {
            if terminal_sum {
                break;
            }
            match transform.kind {
                HirTransformKind::First => {
                    w.write("let ");
                    w.write(&n_name);
                    w.write(" = ");
                    if let Some(arg) = &transform.arg {
                        if ab_result.is_err() {
                            return;
                        }
                        ab_result =
                            generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                    } else {
                        w.write("1");
                    }
                    w.writeln(" as usize;");
                    w.write(&vec_name);
                    w.write(" = ");
                    w.write(&vec_name);
                    w.write(".into_iter().take(");
                    w.write(&n_name);
                    w.writeln(").collect::<Vec<_>>();");
                }
                HirTransformKind::Last => {
                    w.write("let ");
                    w.write(&n_name);
                    w.write(" = ");
                    if let Some(arg) = &transform.arg {
                        if ab_result.is_err() {
                            return;
                        }
                        ab_result =
                            generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                    } else {
                        w.write("1");
                    }
                    w.writeln(" as usize;");
                    w.write("let ");
                    w.write(&len_name);
                    w.write(" = ");
                    w.write(&vec_name);
                    w.writeln(".len();");
                    w.write("let ");
                    w.write(&keep_name);
                    w.write(" = ");
                    w.write(&n_name);
                    w.write(".min(");
                    w.write(&len_name);
                    w.writeln(");");
                    w.write(&vec_name);
                    w.write(" = ");
                    w.write(&vec_name);
                    w.write(".into_iter().skip(");
                    w.write(&len_name);
                    w.write(".saturating_sub(");
                    w.write(&keep_name);
                    w.writeln(")).collect::<Vec<_>>();");
                }
                HirTransformKind::Sum => {
                    w.write("let ");
                    w.write(&sum_name);
                    w.write(" = ");
                    w.write(&vec_name);
                    w.writeln(".into_iter().copied().sum::<i64>();");
                    terminal_sum = true;
                }
            }
        }

        if transforms
            .iter()
            .any(|t| matches!(t.kind, HirTransformKind::Sum))
        {
            w.write(&sum_name);
            w.newline();
        } else {
            w.write(&vec_name);
            w.newline();
        }
    });
    ab_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_array_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        let temp = format!("__faber_vec_{}", expr_id.0);
        w.writeln("{");
        let mut result = Ok(());
        w.indented(|w| {
            w.write("let mut ");
            w.write(&temp);
            w.writeln(" = Vec::new();");
            for element in elements {
                if result.is_err() {
                    return;
                }
                match element {
                    HirArrayElement::Expr(elem) => {
                        w.write(&temp);
                        w.write(".push(");
                        result = generate_expr(
                            codegen,
                            elem,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(");");
                    }
                    HirArrayElement::Spread(elem) => {
                        w.write(&temp);
                        w.write(".extend(");
                        result = generate_expr(
                            codegen,
                            elem,
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
        result?;
        w.write("}");
    } else {
        w.write("vec![");
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            let HirArrayElement::Expr(elem) = elem else {
                continue;
            };
            generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        w.write("]");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_struct_expr(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    fields: &[(Symbol, HirExpr)],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.writeln(" {");
    let mut struct_result = Ok(());
    w.indented(|w| {
        for (name, value) in fields {
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
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_tuple_expr(
    codegen: &RustCodegen<'_>,
    elements: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("(");
    for (i, elem) in elements.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn write_object_map_key(
    codegen: &RustCodegen<'_>,
    types: &TypeTable,
    key: &HirObjectKey,
    key_ty: TypeId,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match key {
        HirObjectKey::Ident(key) | HirObjectKey::String(key) => {
            write_innatum_map_key(codegen, types, *key, key_ty, w);
        }
        HirObjectKey::Computed(expr) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirObjectKey::Spread(_) => {}
    }
    Ok(())
}
pub(super) fn write_innatum_map_key(
    codegen: &RustCodegen<'_>,
    types: &TypeTable,
    key: Symbol,
    key_ty: TypeId,
    w: &mut CodeWriter,
) {
    if matches!(types.get(key_ty), Type::Primitive(Primitive::Textus)) {
        w.write("\"");
        for ch in codegen.resolve_symbol(key).chars() {
            match ch {
                '\\' => w.write("\\\\"),
                '"' => w.write("\\\""),
                '\n' => w.write("\\n"),
                '\r' => w.write("\\r"),
                '\t' => w.write("\\t"),
                _ => w.write(&ch.to_string()),
            }
        }
        w.write("\".to_string()");
        return;
    }

    w.write(codegen.resolve_symbol(key));
}
