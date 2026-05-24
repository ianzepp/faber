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

pub(super) fn generate_field_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    field: Symbol,
) -> Result<(), CodegenError> {
    if let Some(Type::Map(key_ty, _)) = object.ty.map(|ty| resolve_type(ty, emitter.types)) {
        if matches!(resolve_type(key_ty, emitter.types), Type::Primitive(Primitive::Textus)) {
            emitter.expr(object)?;
            emitter.writer.write(".get(\"");
            emitter.writer.write(emitter.codegen.resolve_symbol(field));
            emitter.writer.write("\").cloned().unwrap_or_default()");
            return Ok(());
        }
    }

    emitter.expr(object)?;
    emitter.writer.write(".");
    emitter.writer.write(emitter.codegen.resolve_symbol(field));
    Ok(())
}

pub(super) fn generate_index_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    index: &HirExpr,
) -> Result<(), CodegenError> {
    let object_ty = object.ty.map(|ty| resolve_type(ty, emitter.types));
    if matches!(object_ty, Some(Type::Primitive(Primitive::Textus))) {
        return generate_textus_index_expr(emitter, object, index);
    }

    emitter.expr(object)?;
    emitter.writer.write("[");
    if matches!(object_ty, Some(Type::Array(_))) {
        emitter.writer.write("(");
        emitter.expr(index)?;
        emitter.writer.write(") as usize");
    } else if matches!(object_ty, Some(Type::Map(_, _))) {
        emitter.writer.write("&");
        emitter.expr(index)?;
    } else {
        emitter.expr(index)?;
    }
    emitter.writer.write("]");
    Ok(())
}

fn generate_textus_index_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    object: &HirExpr,
    index: &HirExpr,
) -> Result<(), CodegenError> {
    match &index.kind {
        HirExprKind::Intervallum { start, end, step, kind } => {
            emitter.writer.write("({ let __faber_text = ");
            emitter.expr(object)?;
            emitter.writer.write("; let __faber_start = (");
            emitter.expr(start)?;
            emitter.writer.write(") as usize; let __faber_end = (");
            emitter.expr(end)?;
            match kind {
                HirRangeKind::Exclusive => emitter.writer.write(") as usize; "),
                HirRangeKind::Inclusive => emitter.writer.write(") as usize + 1; "),
            }
            emitter.writer.write("let __faber_step = ");
            if let Some(step) = step {
                emitter.writer.write("((");
                emitter.expr(step)?;
                emitter.writer.write(") as usize).max(1)");
            } else {
                emitter.writer.write("1usize");
            }
            emitter.writer.write("; __faber_text.chars().skip(__faber_start).take(__faber_end.saturating_sub(__faber_start)).step_by(__faber_step).collect::<String>() })");
        }
        _ => {
            emitter.writer.write("(");
            emitter.expr(object)?;
            emitter.writer.write(".chars().nth((");
            emitter.expr(index)?;
            emitter
                .writer
                .write(") as usize).map(|ch| ch.to_string()).unwrap_or_default())");
        }
    }
    Ok(())
}
