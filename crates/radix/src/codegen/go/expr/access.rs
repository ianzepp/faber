//! Member and index lowering for Go expressions.
//!
//! Faber fields are source-language names, while generated Go fields must be
//! exported when structs cross helper boundaries. This file owns that naming
//! bridge, the special `self` path used while emitting struct methods, and the
//! places where map and dynamic `ignotum` access intentionally use Go runtime
//! checks instead of pretending the value has a static shape.
//!
//! TARGET COMPROMISES
//! ==================
//! - Struct field access capitalizes source names to match generated exported
//!   Go fields.
//! - `length`/`longitudo` lower to `len` for arrays and `textus`.
//! - `textus` indexing walks runes and returns `""` for scalar out-of-bounds
//!   access; this avoids byte indexing but does not claim full grapheme
//!   semantics.
//! - Dynamic `ignotum` member access only succeeds for `map[string]any` and
//!   otherwise returns `nil`.

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
            // A struct definition path inside its own methods names the Go
            // receiver, not a package-level value.
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
        // Dynamic member access is deliberately narrow: only string-keyed
        // dynamic maps participate, and all other runtime values produce nil.
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
    if matches!(object.ty.map(|ty| types.get(ty)), Some(Type::Primitive(Primitive::Textus))) {
        return generate_textus_index_expr(codegen, object, index, types, w);
    }

    generate_expr(codegen, object, types, w)?;
    w.write("[");
    generate_expr(codegen, index, types, w)?;
    w.write("]");
    Ok(())
}

fn generate_textus_index_expr(
    codegen: &GoCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // Go strings index bytes. Faber `textus` indexing is expressed through
    // runes here so common Unicode scalar values do not split into bytes.
    match &index.kind {
        HirExprKind::Intervallum { start, end, step, kind } => {
            w.write("func() string { __faber_runes := []rune(");
            generate_expr(codegen, object, types, w)?;
            w.write("); __faber_start := int(");
            generate_expr(codegen, start, types, w)?;
            w.write("); __faber_end := int(");
            generate_expr(codegen, end, types, w)?;
            if matches!(kind, crate::hir::HirRangeKind::Inclusive) {
                w.write(") + 1");
            } else {
                w.write(")");
            }
            w.write("; __faber_step := ");
            if let Some(step) = step {
                w.write("int(");
                generate_expr(codegen, step, types, w)?;
                w.write(")");
            } else {
                w.write("1");
            }
            w.write("; if __faber_step < 1 { __faber_step = 1 }; var __faber_out []rune; for __faber_i := __faber_start; __faber_i < __faber_end && __faber_i < len(__faber_runes); __faber_i += __faber_step { if __faber_i >= 0 { __faber_out = append(__faber_out, __faber_runes[__faber_i]) } }; return string(__faber_out) }()");
        }
        _ => {
            w.write("func() string { __faber_runes := []rune(");
            generate_expr(codegen, object, types, w)?;
            w.write("); __faber_index := int(");
            generate_expr(codegen, index, types, w)?;
            w.write("); if __faber_index < 0 || __faber_index >= len(__faber_runes) { return \"\" }; return string(__faber_runes[__faber_index]) }()");
        }
    }
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
    // Dot-like map member access is sugar for a string key. If the map carries
    // `ignotum` values and semantic analysis knows a narrower result, assert at
    // the boundary so later Go operations see the intended static type.
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
