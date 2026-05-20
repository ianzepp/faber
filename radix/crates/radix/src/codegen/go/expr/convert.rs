use super::*;
use crate::hir::HirObjectKey;
pub(super) fn generate_value_conversion(
    _codegen: &GoCodegen<'_>,
    value_expr: &str,
    target_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match normalize_receiver_type(types.get(target_ty), types) {
        Type::Primitive(Primitive::Textus) => {
            w.write("fmt.Sprint(");
            w.write(value_expr);
            w.write(")");
        }
        Type::Primitive(Primitive::Numerus) => {
            w.write("func() int { v, _ := strconv.Atoi(fmt.Sprint(");
            w.write(value_expr);
            w.write(")); return v }()");
        }
        Type::Primitive(Primitive::Fractus) => {
            w.write("func() float64 { v, _ := strconv.ParseFloat(fmt.Sprint(");
            w.write(value_expr);
            w.write("), 64); return v }()");
        }
        Type::Primitive(Primitive::Bivalens) => {
            w.write("(");
            w.write(value_expr);
            w.write(" != nil)");
        }
        _ => w.write(value_expr),
    }
    Ok(())
}
pub(super) fn generate_bool_conversion_expr(
    codegen: &GoCodegen<'_>,
    source: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match source
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types))
    {
        Some(Type::Primitive(Primitive::Bivalens)) => generate_expr(codegen, source, types, w)?,
        Some(Type::Primitive(Primitive::Textus)) => {
            w.write("(");
            generate_expr(codegen, source, types, w)?;
            w.write(" != \"\")");
        }
        Some(Type::Primitive(Primitive::Numerus)) | Some(Type::Primitive(Primitive::Fractus)) => {
            w.write("(");
            generate_expr(codegen, source, types, w)?;
            w.write(" != 0)");
        }
        Some(Type::Option(_)) | Some(Type::Primitive(Primitive::Nihil)) => {
            w.write("(");
            generate_expr(codegen, source, types, w)?;
            w.write(" != nil)");
        }
        _ => {
            w.write("func() bool { v, _ := strconv.ParseBool(fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write(")); return v }()");
        }
    }
    Ok(())
}
pub(super) fn generate_verte_array_expr(
    codegen: &GoCodegen<'_>,
    source: &HirExpr,
    elem_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if let HirExprKind::Array(elements) = &source.kind {
        return generate_typed_array_expr(codegen, elem_ty, elements, types, w);
    }

    let elem_go_ty = types::type_to_go(codegen, elem_ty, types);
    w.write("func() []");
    w.write(&elem_go_ty);
    w.write(" { src := ");
    generate_expr(codegen, source, types, w)?;
    w.write("; out := make([]");
    w.write(&elem_go_ty);
    w.write(", len(src)); for i, value := range src { out[i] = ");
    generate_value_conversion(codegen, "value", elem_ty, types, w)?;
    w.write(" }; return out }()");
    Ok(())
}

pub(super) fn generate_verte_expr(
    codegen: &GoCodegen<'_>,
    source: &HirExpr,
    target: crate::semantic::TypeId,
    entries: Option<&[crate::hir::HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match types.get(target) {
        Type::Struct(def_id) => {
            if let Some(entries) = entries {
                w.write(codegen.resolve_def(*def_id));
                w.write("{");
                let mut wrote_any = false;
                for field in entries {
                    let Some(value) = &field.value else { continue };
                    if wrote_any {
                        w.write(", ");
                    }
                    match &field.key {
                        HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                            w.write(&capitalize(codegen.resolve_symbol(*name)));
                        }
                        HirObjectKey::Computed(_) | HirObjectKey::Spread(_) => continue,
                    }
                    w.write(": ");
                    if let Some(field_ty) = match &field.key {
                        HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                            codegen.struct_field_type(*def_id, *name)
                        }
                        HirObjectKey::Computed(_) | HirObjectKey::Spread(_) => None,
                    } {
                        generate_expr_for_go_type(codegen, value, field_ty, types, w)?;
                    } else {
                        generate_expr(codegen, value, types, w)?;
                    }
                    wrote_any = true;
                }
                w.write("}");
            } else {
                generate_expr(codegen, source, types, w)?;
            }
        }
        Type::Map(key_ty, value_ty) => {
            generate_map_literal(codegen, *key_ty, *value_ty, entries, types, w)?;
        }
        Type::Array(elem_ty) => {
            generate_verte_array_expr(codegen, source, *elem_ty, types, w)?;
        }
        Type::Option(inner_ty) => {
            generate_option_wrapped_expr(codegen, source, *inner_ty, types, w)?;
        }
        Type::Primitive(Primitive::Textus) => {
            w.write("fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write(")");
        }
        Type::Primitive(Primitive::Numerus) => {
            w.write("func() int { v, _ := strconv.Atoi(fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write(")); return v }()");
        }
        Type::Primitive(Primitive::Fractus) => {
            w.write("func() float64 { v, _ := strconv.ParseFloat(fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write("), 64); return v }()");
        }
        Type::Primitive(Primitive::Bivalens) => {
            generate_bool_conversion_expr(codegen, source, types, w)?;
        }
        Type::Enum(_) | Type::Interface(_) if variant_value_expr(source, codegen) => {
            generate_expr(codegen, source, types, w)?;
        }
        Type::Enum(_) | Type::Interface(_) => {
            generate_expr(codegen, source, types, w)?;
            w.write(".(");
            w.write(&types::type_to_go(codegen, target, types));
            w.write(")");
        }
        _ => {
            generate_expr(codegen, source, types, w)?;
            w.write(".(");
            w.write(&types::type_to_go(codegen, target, types));
            w.write(")");
        }
    }
    Ok(())
}

pub(super) fn generate_conversio_expr(
    codegen: &GoCodegen<'_>,
    source: &HirExpr,
    target: crate::semantic::TypeId,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match types.get(target) {
        Type::Primitive(Primitive::Numerus) => {
            // WHY: strconv.Atoi is the idiomatic Go int parse.
            if let Some(fb) = fallback {
                w.write("func() int { v, err := strconv.Atoi(fmt.Sprint(");
                generate_expr(codegen, source, types, w)?;
                w.write(")); if err != nil { return ");
                generate_expr(codegen, fb, types, w)?;
                w.write(" }; return v }()");
            } else {
                w.write("func() int { v, _ := strconv.Atoi(fmt.Sprint(");
                generate_expr(codegen, source, types, w)?;
                w.write(")); return v }()");
            }
        }
        Type::Primitive(Primitive::Fractus) => {
            if let Some(fb) = fallback {
                w.write("func() float64 { v, err := strconv.ParseFloat(fmt.Sprint(");
                generate_expr(codegen, source, types, w)?;
                w.write("), 64); if err != nil { return ");
                generate_expr(codegen, fb, types, w)?;
                w.write(" }; return v }()");
            } else {
                w.write("func() float64 { v, _ := strconv.ParseFloat(fmt.Sprint(");
                generate_expr(codegen, source, types, w)?;
                w.write("), 64); return v }()");
            }
        }
        Type::Primitive(Primitive::Textus) => {
            w.write("fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write(")");
        }
        Type::Primitive(Primitive::Bivalens) => {
            w.write("func() bool { v, _ := strconv.ParseBool(fmt.Sprint(");
            generate_expr(codegen, source, types, w)?;
            w.write(")); return v }()");
        }
        _ => {
            generate_expr(codegen, source, types, w)?;
            w.write(".(");
            w.write(&types::type_to_go(codegen, target, types));
            w.write(")");
        }
    }
    Ok(())
}
