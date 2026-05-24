//! Field and index access lowering for Rust expressions.
//!
//! Plain field and collection indexing intentionally mirror Rust syntax. The
//! notable exception is `textus`: Faber indexes text by character position,
//! while Rust string indexing is byte-oriented and rejects direct `str[index]`
//! access. Text indexing is therefore lowered through `.chars()` and returns an
//! owned `String` for scalar access or range collection.
//!
//! EDGE CASES
//! ==========
//! - Out-of-range scalar `textus` access currently yields an empty string.
//! - Text ranges use saturating length arithmetic and clamp a missing or zero
//!   step to one.
//! - List indexes use Faber `numerus` (`i64`) in source but Rust slice indexes
//!   are `usize`, so scalar list indexes are cast at the backend boundary.
//! - Other non-text index expressions stay direct Rust indexing and may panic
//!   exactly as the generated Rust collection access would.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_field_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    field: Symbol,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(".");
    w.write(codegen.resolve_symbol(field));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_index_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let object_ty = object.ty.map(|ty| resolve_type(ty, types));
    if matches!(object_ty, Some(Type::Primitive(Primitive::Textus))) {
        return generate_textus_index_expr(
            codegen,
            object,
            index,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    }

    generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("[");
    if matches!(object_ty, Some(Type::Array(_))) {
        w.write("(");
        generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(") as usize");
    } else if matches!(object_ty, Some(Type::Map(_, _))) {
        w.write("&");
        generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    } else {
        generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write("]");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_textus_index_expr(
    codegen: &RustCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &index.kind {
        HirExprKind::Intervallum { start, end, step, kind } => {
            w.write("({ let __faber_text = ");
            generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("; let __faber_start = (");
            generate_expr(codegen, start, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(") as usize; let __faber_end = (");
            generate_expr(codegen, end, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            match kind {
                HirRangeKind::Exclusive => w.write(") as usize; "),
                HirRangeKind::Inclusive => w.write(") as usize + 1; "),
            }
            w.write("let __faber_step = ");
            if let Some(step) = step {
                w.write("((");
                generate_expr(codegen, step, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(") as usize).max(1)");
            } else {
                w.write("1usize");
            }
            w.write("; __faber_text.chars().skip(__faber_start).take(__faber_end.saturating_sub(__faber_start)).step_by(__faber_step).collect::<String>() })");
        }
        _ => {
            w.write("(");
            generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(".chars().nth((");
            generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(") as usize).map(|ch| ch.to_string()).unwrap_or_default())");
        }
    }
    Ok(())
}
