//! Loop, iterator, and range expression lowering.

use super::*;
use crate::codegen::rust::stmt::generate_stmt;

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_loop_expr(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("loop ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_while_expr(
    codegen: &RustCodegen<'_>,
    cond: &HirExpr,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("while ");
    generate_expr_unwrapped(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_for_expr(
    codegen: &RustCodegen<'_>,
    mode: HirIteraMode,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if matches!(mode, HirIteraMode::Ex) && matches!(iter.ty.map(|ty| resolve_type(ty, types)), Some(Type::Array(_))) {
        return generate_borrowed_array_for_expr(
            codegen,
            binding,
            iter,
            block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    }

    if matches!(mode, HirIteraMode::De) {
        match iter.ty.map(|ty| resolve_type(ty, types)) {
            Some(Type::Array(_)) => {
                return generate_array_index_for_expr(
                    codegen,
                    binding,
                    iter,
                    block,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                );
            }
            Some(Type::Map(_, _)) => {
                return generate_map_key_for_expr(
                    codegen,
                    binding,
                    iter,
                    block,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                );
            }
            _ => {}
        }
    }

    w.write("for ");
    w.write(codegen.resolve_def(binding));
    w.write(" in ");
    if matches!(mode, HirIteraMode::Pro) {
        if let HirExprKind::Intervallum { start, end, step, kind } = &iter.kind {
            generate_range_iter_expr(
                codegen,
                start,
                end,
                step.as_deref(),
                *kind,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
            generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
    } else {
        generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(" ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
fn generate_array_index_for_expr(
    codegen: &RustCodegen<'_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("for ");
    w.write(codegen.resolve_def(binding));
    w.write(" in 0..((");
    generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(").len() as i64) ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
fn generate_map_key_for_expr(
    codegen: &RustCodegen<'_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("for ");
    w.write(codegen.resolve_def(binding));
    w.write(" in (");
    generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(").keys().cloned() ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
fn generate_borrowed_array_for_expr(
    codegen: &RustCodegen<'_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let item = format!("__faber_item_{}", binding.0);
    w.write("for ");
    w.write(&item);
    w.write(" in &(");
    generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.writeln(") {");
    let mut block_result = Ok(());
    w.indented(|w| {
        w.write("let ");
        w.write(codegen.resolve_def(binding));
        w.write(" = ");
        w.write(&item);
        w.writeln(".clone();");
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = generate_stmt(codegen, stmt, types, w, in_failable_fn, in_entry, suppress_error_propagation);
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            block_result = generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
        }
    });
    block_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_range_iter_expr(
    codegen: &RustCodegen<'_>,
    start: &HirExpr,
    end: &HirExpr,
    step: Option<&HirExpr>,
    kind: HirRangeKind,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("{ let __faber_start: i64 = ");
    generate_expr(codegen, start, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("; let __faber_end: i64 = ");
    generate_expr(codegen, end, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("; let __faber_step: i64 = ");
    if let Some(step) = step {
        generate_expr(codegen, step, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    } else {
        w.write("if __faber_start <= __faber_end { 1 } else { -1 }");
    }
    w.write("; let __faber_limit: i64 = ");
    match kind {
        HirRangeKind::Exclusive => w.write("__faber_end"),
        HirRangeKind::Inclusive => w.write("__faber_end + __faber_step.signum()"),
    }
    w.write("; let mut __faber_values = Vec::new(); let mut __faber_i = __faber_start; ");
    w.write("if __faber_step > 0 { while __faber_i < __faber_limit { __faber_values.push(__faber_i); __faber_i += __faber_step; } } ");
    w.write("else if __faber_step < 0 { while __faber_i > __faber_limit { __faber_values.push(__faber_i); __faber_i += __faber_step; } } ");
    w.write("__faber_values }");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_range_tuple_expr(
    codegen: &RustCodegen<'_>,
    start: &HirExpr,
    end: &HirExpr,
    step: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("(");
    generate_expr(codegen, start, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(", ");
    generate_expr(codegen, end, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if let Some(step) = step {
        w.write(", ");
        generate_expr(codegen, step, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}
