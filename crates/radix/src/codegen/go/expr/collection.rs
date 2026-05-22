use super::*;
use crate::hir::HirObjectKey;
pub(super) fn generate_struct_expr(
    codegen: &GoCodegen<'_>,
    def_id: crate::hir::DefId,
    fields: &[(crate::lexer::Symbol, HirExpr)],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.write("{");
    for (idx, (name, value)) in fields.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(&capitalize(codegen.resolve_symbol(*name)));
        w.write(": ");
        if let Some(field_ty) = codegen.struct_field_type(def_id, *name) {
            if codegen.struct_field_is_sponte(def_id, *name) {
                generate_option_wrapped_expr(codegen, value, field_ty, types, w)?;
            } else {
                generate_expr_for_go_type(codegen, value, field_ty, types, w)?;
            }
        } else {
            generate_expr(codegen, value, types, w)?;
        }
    }
    w.write("}");
    Ok(())
}

pub(super) fn generate_tuple_expr(
    codegen: &GoCodegen<'_>,
    elements: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go has no tuples; a slice preserves multi-value expression shape.
    w.write("[]any{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, element, types, w)?;
    }
    w.write("}");
    Ok(())
}

pub(super) fn generate_array_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let elem_ty = expr
        .ty
        .and_then(|ty| match normalize_receiver_type(types.get(ty), types) {
            Type::Array(elem) => Some(types::type_to_go(codegen, *elem, types)),
            _ => None,
        })
        .unwrap_or_else(|| "any".to_owned());

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        w.write("func() []");
        w.write(&elem_ty);
        w.write(" { acc := []");
        w.write(&elem_ty);
        w.write("{}; ");
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("); ");
                }
                HirArrayElement::Spread(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("...); ");
                }
            }
        }
        w.write("return acc }()");
        return Ok(());
    }

    w.write("[]");
    w.write(&elem_ty);
    w.write("{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        let HirArrayElement::Expr(expr) = element else { continue };
        generate_expr(codegen, expr, types, w)?;
    }
    w.write("}");
    Ok(())
}
pub(super) fn generate_typed_array_expr(
    codegen: &GoCodegen<'_>,
    elem_ty: crate::semantic::TypeId,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let elem_go_ty = types::type_to_go(codegen, elem_ty, types);

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { acc := []");
        w.write(&elem_go_ty);
        w.write("{}; ");
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr_for_go_type(codegen, expr, elem_ty, types, w)?;
                    w.write("); ");
                }
                HirArrayElement::Spread(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("...); ");
                }
            }
        }
        w.write("return acc }()");
        return Ok(());
    }

    w.write("[]");
    w.write(&elem_go_ty);
    w.write("{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        let HirArrayElement::Expr(expr) = element else { continue };
        generate_expr_for_go_type(codegen, expr, elem_ty, types, w)?;
    }
    w.write("}");
    Ok(())
}
pub(super) fn generate_ab_filter(
    codegen: &GoCodegen<'_>,
    filter: &crate::hir::HirCollectionFilter,
    source_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("filtered := make(");
    w.write(&types::type_to_go(codegen, source_ty, types));
    w.write(", 0, len(current)); for _, value := range current { if ");

    if filter.negated {
        w.write("!(");
    }
    generate_ab_filter_predicate(codegen, &filter.kind, w)?;
    if filter.negated {
        w.write(")");
    }

    w.write(" { filtered = append(filtered, value) } }; current = filtered; ");
    Ok(())
}

pub(super) fn generate_ab_filter_predicate(
    codegen: &GoCodegen<'_>,
    kind: &crate::hir::HirCollectionFilterKind,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match kind {
        crate::hir::HirCollectionFilterKind::Property(name) => {
            let field_name = codegen.resolve_symbol(*name);
            w.write("func() bool { m, ok := any(value).(map[string]any); if !ok { return false }; raw, ok := m[");
            w.write(&format!("{:?}", field_name));
            w.write("]; if !ok { return false }; typed, ok := raw.(bool); return ok && typed }()");
        }
        crate::hir::HirCollectionFilterKind::Condition(_) => {
            return Err(CodegenError { message: "Go ab condition filters are not implemented yet".to_owned() });
        }
    }
    Ok(())
}
pub(super) fn generate_ab_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    source: &HirExpr,
    filter: Option<&crate::hir::HirCollectionFilter>,
    transforms: &[crate::hir::HirCollectionTransform],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let Some(source_ty) = source.ty else {
        return Err(CodegenError { message: "ab source missing Go type".to_owned() });
    };
    let ret_ty = expr_return_type(expr, types, codegen);
    w.write("func() ");
    w.write(&ret_ty);
    w.write(" { current := ");
    generate_expr_for_go_type(codegen, source, source_ty, types, w)?;
    w.write("; ");

    if let Some(filter) = filter {
        generate_ab_filter(codegen, filter, source_ty, types, w)?;
    }

    let mut terminal_sum = false;
    for (idx, transform) in transforms.iter().enumerate() {
        if terminal_sum {
            break;
        }

        match transform.kind {
            crate::hir::HirTransformKind::First => {
                let limit_name = format!("__radixAbFirst{}", idx);
                let clamp_name = format!("__radixAbFirstClamped{}", idx);
                w.write(&limit_name);
                w.write(" := ");
                if let Some(arg) = &transform.arg {
                    generate_expr(codegen, arg, types, w)?;
                } else {
                    w.write("1");
                }
                w.write("; ");
                w.write(&clamp_name);
                w.write(" := ");
                w.write(&limit_name);
                w.write("; if ");
                w.write(&clamp_name);
                w.write(" < 0 { ");
                w.write(&clamp_name);
                w.write(" = 0 }; if ");
                w.write(&clamp_name);
                w.write(" > len(current) { ");
                w.write(&clamp_name);
                w.write(" = len(current) }; current = current[:");
                w.write(&clamp_name);
                w.write("]; ");
            }
            crate::hir::HirTransformKind::Last => {
                let limit_name = format!("__radixAbLast{}", idx);
                let clamp_name = format!("__radixAbLastClamped{}", idx);
                w.write(&limit_name);
                w.write(" := ");
                if let Some(arg) = &transform.arg {
                    generate_expr(codegen, arg, types, w)?;
                } else {
                    w.write("1");
                }
                w.write("; ");
                w.write(&clamp_name);
                w.write(" := ");
                w.write(&limit_name);
                w.write("; if ");
                w.write(&clamp_name);
                w.write(" < 0 { ");
                w.write(&clamp_name);
                w.write(" = 0 }; if ");
                w.write(&clamp_name);
                w.write(" > len(current) { ");
                w.write(&clamp_name);
                w.write(" = len(current) }; current = current[len(current)-");
                w.write(&clamp_name);
                w.write(":]; ");
            }
            crate::hir::HirTransformKind::Sum => {
                let value_name = format!("__radixAbValue{}", idx);
                w.write("var total ");
                w.write(&ret_ty);
                w.write("; for _, ");
                w.write(&value_name);
                w.write(" := range current { total += ");
                w.write(&value_name);
                w.write(" }; return total");
                terminal_sum = true;
            }
        }
    }

    if !terminal_sum {
        w.write("return current");
    }
    w.write(" }()");
    Ok(())
}
pub(super) fn generate_map_literal(
    codegen: &GoCodegen<'_>,
    key_ty: crate::semantic::TypeId,
    value_ty: crate::semantic::TypeId,
    entries: Option<&[crate::hir::HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("map[");
    w.write(&types::type_to_go(codegen, key_ty, types));
    w.write("]");
    w.write(&types::type_to_go(codegen, value_ty, types));
    w.write("{");
    if let Some(entries) = entries {
        let mut wrote_any = false;
        for field in entries {
            let Some(value) = &field.value else { continue };
            if wrote_any {
                w.write(", ");
            }
            match &field.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    w.write(&format!("{:?}", codegen.resolve_symbol(*name)));
                }
                HirObjectKey::Computed(expr) => generate_expr(codegen, expr, types, w)?,
                HirObjectKey::Spread(_) => continue,
            }
            w.write(": ");
            generate_expr_for_go_type(codegen, value, value_ty, types, w)?;
            wrote_any = true;
        }
    }
    w.write("}");
    Ok(())
}
