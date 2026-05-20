use super::*;
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
