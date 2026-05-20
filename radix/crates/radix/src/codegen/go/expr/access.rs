use super::*;
pub(super) fn generate_field_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    object: &HirExpr,
    field: crate::lexer::Symbol,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let field_name = codegen.resolve_symbol(field);
    if let HirExprKind::Path(def_id) = object.kind {
        if codegen.is_struct_def(def_id) {
            w.write("self.");
            w.write(&capitalize(field_name));
            return Ok(());
        }
    }
    let object_ty = object
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types));
    if matches!(field_name, "length" | "longitudo")
        && matches!(object_ty, Some(Type::Array(_)) | Some(Type::Primitive(Primitive::Textus)))
    {
        w.write("len(");
        generate_expr(codegen, object, types, w)?;
        w.write(")");
        return Ok(());
    }
    if let Some(Type::Map(_, value_ty)) = object_ty {
        return write_map_member_expr(codegen, object, field_name, *value_ty, expr.ty, types, w);
    }
    if matches!(object_ty, Some(Type::Primitive(Primitive::Ignotum))) {
        w.write("func() any { if m, ok := ");
        generate_expr(codegen, object, types, w)?;
        w.write(".(map[string]any); ok { return m[");
        w.write(&format!("{:?}", field_name));
        w.write("] }; return nil }()");
        return Ok(());
    }
    generate_expr(codegen, object, types, w)?;
    w.write(".");
    w.write(&capitalize(field_name));
    Ok(())
}

pub(super) fn generate_index_expr(
    codegen: &GoCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_expr(codegen, object, types, w)?;
    w.write("[");
    generate_expr(codegen, index, types, w)?;
    w.write("]");
    Ok(())
}

pub(super) fn write_map_member_expr(
    codegen: &GoCodegen<'_>,
    object: &HirExpr,
    field_name: &str,
    value_ty: crate::semantic::TypeId,
    result_ty: Option<crate::semantic::TypeId>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_expr(codegen, object, types, w)?;
    w.write("[");
    w.write(&format!("{:?}", field_name));
    w.write("]");
    if let Some(assert_ty) = asserted_map_value_type(value_ty, result_ty, types) {
        w.write(".(");
        w.write(&types::type_to_go(codegen, assert_ty, types));
        w.write(")");
    }
    Ok(())
}

pub(super) fn asserted_map_value_type(
    value_ty: crate::semantic::TypeId,
    result_ty: Option<crate::semantic::TypeId>,
    types: &TypeTable,
) -> Option<crate::semantic::TypeId> {
    if !matches!(
        normalize_receiver_type(types.get(value_ty), types),
        Type::Primitive(Primitive::Ignotum)
    ) {
        return None;
    }

    result_ty.filter(|ty| {
        !matches!(
            normalize_receiver_type(types.get(*ty), types),
            Type::Primitive(Primitive::Ignotum) | Type::Primitive(Primitive::Nihil) | Type::Option(_)
        )
    })
}
