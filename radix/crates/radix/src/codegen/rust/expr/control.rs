use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_tempta_expr(
    codegen: &RustCodegen<'_>,
    body: &HirBlock,
    catch: Option<&HirBlock>,
    finally: Option<&HirBlock>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut tempta_result = Ok(());
    w.indented(|w| {
        for stmt in &body.stmts {
            if tempta_result.is_err() {
                return;
            }
            tempta_result = super::super::stmt::generate_stmt(
                codegen,
                stmt,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation || catch.is_some(),
            );
        }
        if let Some(expr) = &body.expr {
            if tempta_result.is_err() {
                return;
            }
            tempta_result = generate_expr(
                codegen,
                expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation || catch.is_some(),
            );
            w.writeln(";");
        }
        if let Some(catch) = catch {
            for stmt in &catch.stmts {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = super::super::stmt::generate_stmt(
                    codegen,
                    stmt,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                );
            }
            if let Some(expr) = &catch.expr {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result =
                    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                w.writeln(";");
            }
        }
        if let Some(finally) = finally {
            for stmt in &finally.stmts {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = super::super::stmt::generate_stmt(
                    codegen,
                    stmt,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                );
            }
            if let Some(expr) = &finally.expr {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result =
                    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                w.writeln(";");
            }
        }
    });
    tempta_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_if_expr(
    codegen: &RustCodegen<'_>,
    cond: &HirExpr,
    then: &HirBlock,
    else_: Option<&HirBlock>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("if ");
    generate_expr_unwrapped(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" ");
    generate_block(codegen, then, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if let Some(else_block) = else_ {
        w.write(" else ");
        generate_block(
            codegen,
            else_block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_match_expr(
    codegen: &RustCodegen<'_>,
    scrutinees: &[HirExpr],
    arms: &[HirCasuArm],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("match ");
    if scrutinees.len() == 1 {
        generate_expr(
            codegen,
            &scrutinees[0],
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
    } else {
        w.write("(");
        for (idx, scrutinee) in scrutinees.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(
                codegen,
                scrutinee,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        w.write(")");
    }
    w.writeln(" {");
    let mut discerne_result = Ok(());
    w.indented(|w| {
        for arm in arms {
            if arm.patterns.len() == 1 {
                generate_pattern(codegen, &arm.patterns[0], w);
            } else {
                w.write("(");
                for (idx, pattern) in arm.patterns.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    generate_pattern(codegen, pattern, w);
                }
                w.write(")");
            }
            if let Some(guard) = &arm.guard {
                w.write(" if ");
                if discerne_result.is_err() {
                    return;
                }
                discerne_result =
                    generate_expr(codegen, guard, types, w, in_failable_fn, in_entry, suppress_error_propagation);
            }
            w.write(" => ");
            if discerne_result.is_err() {
                return;
            }
            discerne_result = generate_expr(
                codegen,
                &arm.body,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            w.writeln(",");
        }
    });
    discerne_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_loop_expr(
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
pub(super) fn generate_while_expr(
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
pub(super) fn generate_for_expr(
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
    w.write(" in ");
    generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" ");
    generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_range_tuple_expr(
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

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_closure_expr(
    codegen: &RustCodegen<'_>,
    params: &[HirParam],
    body: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("|");
    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        w.write(codegen.resolve_symbol(param.name));
    }
    w.write("| ");
    generate_expr(codegen, body, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_await_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(".await");
    Ok(())
}
