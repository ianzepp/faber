//! Loop, iterator, and range expression lowering.

use super::*;
use crate::codegen::rust::stmt::generate_stmt;

pub(in crate::codegen::rust::expr) fn generate_loop_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    emitter.writer.write("loop ");
    generate_block_with_emitter(emitter, block)
}

pub(in crate::codegen::rust::expr) fn generate_while_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    cond: &HirExpr,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    emitter.writer.write("while ");
    emitter.expr_unwrapped(cond)?;
    emitter.writer.write(" ");
    generate_block_with_emitter(emitter, block)
}

pub(in crate::codegen::rust::expr) fn generate_for_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    mode: HirIteraMode,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    if matches!(mode, HirIteraMode::Ex)
        && matches!(iter.ty.map(|ty| resolve_type(ty, emitter.types)), Some(Type::Array(_)))
    {
        return generate_borrowed_array_for_expr_with_emitter(emitter, binding, iter, block);
    }

    if matches!(mode, HirIteraMode::De) {
        match iter.ty.map(|ty| resolve_type(ty, emitter.types)) {
            Some(Type::Array(_)) => {
                return generate_array_index_for_expr_with_emitter(emitter, binding, iter, block);
            }
            Some(Type::Map(_, _)) => {
                return generate_map_key_for_expr_with_emitter(emitter, binding, iter, block);
            }
            _ => {}
        }
    }

    emitter.writer.write("for ");
    emitter.writer.write(emitter.codegen.resolve_def(binding));
    emitter.writer.write(" in ");
    if matches!(mode, HirIteraMode::Pro) {
        if let HirExprKind::Intervallum { start, end, step, kind } = &iter.kind {
            generate_range_iter_expr_with_emitter(emitter, start, end, step.as_deref(), *kind)?;
        } else {
            emitter.expr(iter)?;
        }
    } else {
        emitter.expr(iter)?;
    }
    emitter.writer.write(" ");
    generate_block_with_emitter(emitter, block)
}

fn generate_array_index_for_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    emitter.writer.write("for ");
    emitter.writer.write(emitter.codegen.resolve_def(binding));
    emitter.writer.write(" in 0..((");
    emitter.expr_unwrapped(iter)?;
    emitter.writer.write(").len() as i64) ");
    generate_block_with_emitter(emitter, block)
}

fn generate_map_key_for_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    emitter.writer.write("for ");
    emitter.writer.write(emitter.codegen.resolve_def(binding));
    emitter.writer.write(" in (");
    emitter.expr_unwrapped(iter)?;
    emitter.writer.write(").keys().cloned() ");
    generate_block_with_emitter(emitter, block)
}

fn generate_borrowed_array_for_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    binding: DefId,
    iter: &HirExpr,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    let item = format!("__faber_item_{}", binding.0);
    emitter.writer.write("for ");
    emitter.writer.write(&item);
    emitter.writer.write(" in &(");
    emitter.expr_unwrapped(iter)?;
    emitter.writer.writeln(") {");
    let mut block_result = Ok(());
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|writer| {
        let mut inner_emitter = ExprEmitter::new(codegen, types, writer, policy);
        inner_emitter.writer.write("let ");
        inner_emitter.writer.write(codegen.resolve_def(binding));
        inner_emitter.writer.write(" = ");
        inner_emitter.writer.write(&item);
        inner_emitter.writer.writeln(".clone();");
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = generate_stmt(
                codegen,
                stmt,
                types,
                inner_emitter.writer,
                policy.can_propagate_failure,
                policy.inside_entrypoint,
                policy.propagation_suppressed,
            );
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            block_result = inner_emitter.expr(expr);
        }
    });
    block_result?;
    emitter.writer.write("}");
    Ok(())
}

fn generate_range_iter_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    start: &HirExpr,
    end: &HirExpr,
    step: Option<&HirExpr>,
    kind: HirRangeKind,
) -> Result<(), CodegenError> {
    emitter.writer.write("{ let __faber_start: i64 = ");
    emitter.expr_unwrapped(start)?;
    emitter.writer.write("; let __faber_end: i64 = ");
    emitter.expr_unwrapped(end)?;
    emitter.writer.write("; let __faber_step: i64 = ");
    if let Some(step) = step {
        emitter.expr_unwrapped(step)?;
    } else {
        emitter
            .writer
            .write("if __faber_start <= __faber_end { 1 } else { -1 }");
    }
    emitter.writer.write("; let __faber_limit: i64 = ");
    match kind {
        HirRangeKind::Exclusive => emitter.writer.write("__faber_end"),
        HirRangeKind::Inclusive => emitter.writer.write("__faber_end + __faber_step.signum()"),
    }
    emitter
        .writer
        .write("; let mut __faber_values = Vec::new(); let mut __faber_i = __faber_start; ");
    emitter
        .writer
        .write("if __faber_step > 0 { while __faber_i < __faber_limit { __faber_values.push(__faber_i); __faber_i += __faber_step; } } ");
    emitter
        .writer
        .write("else if __faber_step < 0 { while __faber_i > __faber_limit { __faber_values.push(__faber_i); __faber_i += __faber_step; } } ");
    emitter.writer.write("__faber_values }");
    Ok(())
}

pub(in crate::codegen::rust::expr) fn generate_range_tuple_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    start: &HirExpr,
    end: &HirExpr,
    step: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    emitter.writer.write("(");
    emitter.expr_unwrapped(start)?;
    emitter.writer.write(", ");
    emitter.expr_unwrapped(end)?;
    if let Some(step) = step {
        emitter.writer.write(", ");
        emitter.expr_unwrapped(step)?;
    }
    emitter.writer.write(")");
    Ok(())
}
