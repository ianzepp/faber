use super::*;
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
