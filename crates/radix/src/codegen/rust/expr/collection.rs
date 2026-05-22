use super::*;
use rustc_hash::FxHashSet;

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
    let provided: FxHashSet<Symbol> = fields.iter().map(|(n, _)| *n).collect();
    w.indented(|w| {
        for (name, value) in fields {
            w.write(codegen.resolve_symbol(*name));
            w.write(": ");
            if struct_result.is_err() {
                return;
            }
            struct_result = generate_struct_field_value(
                codegen,
                def_id,
                *name,
                value,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            w.writeln(",");
        }
        if struct_result.is_err() {
            return;
        }
        struct_result = generate_omitted_struct_fields(
            codegen,
            def_id,
            &provided,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    });
    struct_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_struct_field_value(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if codegen.struct_field_stores_option(def_id, name, types) && expr_requires_some_wrapper(value, types) {
        w.write("Some(");
        generate_struct_value_expr(
            codegen,
            def_id,
            name,
            value,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(")");
        return Ok(());
    }

    generate_struct_value_expr(
        codegen,
        def_id,
        name,
        value,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_omitted_struct_fields(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    provided: &FxHashSet<Symbol>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    for field in codegen.sorted_struct_omittable_fields(def_id) {
        if provided.contains(&field.name) {
            continue;
        }
        w.write(codegen.resolve_symbol(field.name));
        w.write(": ");
        if let Some(init) = field.init {
            generate_struct_field_value(
                codegen,
                def_id,
                field.name,
                init,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
            w.write("None");
        }
        w.writeln(",");
    }
    Ok(())
}

fn expr_requires_some_wrapper(expr: &HirExpr, types: &TypeTable) -> bool {
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::Nil)) {
        return false;
    }

    match expr.ty {
        Some(ty) => !type_is_option_or_nihil(ty, types),
        None => true,
    }
}

fn type_is_option_or_nihil(type_id: TypeId, types: &TypeTable) -> bool {
    match types.get(type_id) {
        Type::Option(_) | Type::Primitive(Primitive::Nihil) => true,
        Type::Alias(_, resolved) => type_is_option_or_nihil(*resolved, types),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_struct_value_expr(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_)))
        && value.ty.is_none()
        && struct_field_value_is_textus(codegen, def_id, name, types)
    {
        w.write(".to_string()");
    }
    Ok(())
}

fn struct_field_value_is_textus(codegen: &RustCodegen<'_>, def_id: DefId, name: Symbol, types: &TypeTable) -> bool {
    let Some(field) = codegen.struct_field_info(def_id, name) else {
        return false;
    };

    let value_type = match types.get(field.ty) {
        Type::Option(inner) => *inner,
        Type::Alias(_, resolved) => option_inner_or_self(*resolved, types),
        _ => field.ty,
    };
    matches!(types.get(value_type), Type::Primitive(Primitive::Textus))
}

fn option_inner_or_self(type_id: TypeId, types: &TypeTable) -> TypeId {
    match types.get(type_id) {
        Type::Option(inner) => *inner,
        Type::Alias(_, resolved) => option_inner_or_self(*resolved, types),
        _ => type_id,
    }
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
