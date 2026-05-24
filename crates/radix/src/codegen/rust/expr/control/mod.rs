//! Control-flow expression lowering for the Rust backend.
//!
//! Faber control expressions are emitted as direct Rust control constructs when
//! their HIR shape already matches what this phase supports: `si` to `if`,
//! `discerne` to `match`, `dum` to `while`, `itera` to `for`, and bare loops to
//! `loop`. Branch bodies reuse block lowering so expression-valued tails remain
//! visible to Rust where the surrounding context allows them.
//!
//! TEMPTA CONTRACT
//! ===============
//! `tempta` currently lowers as a scoped sequence: body, optional catch block,
//! then optional finally block. When a catch exists, body emission suppresses
//! `?` insertion so handled failure points do not propagate out through Rust
//! syntax. This file does not claim full exception semantics; structured cape
//! handler HIR is rejected by the dispatcher in `mod.rs`.
//!
//! OUTPUT POLICY
//! =============
//! Loops and branches preserve Rust's native value rules. This backend does not
//! synthesize hidden accumulator variables or branch coercions for unsupported
//! HIR shapes.

mod branch;
mod iteration;
mod match_expr;

pub(super) use branch::generate_if_expr;
pub(super) use iteration::{generate_for_expr, generate_loop_expr, generate_range_tuple_expr, generate_while_expr};
pub(super) use match_expr::generate_match_expr;

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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    emit_tempta_expr(&mut emitter, body, catch, finally)
}

#[allow(clippy::too_many_arguments)]
fn emit_tempta_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    body: &HirBlock,
    catch: Option<&HirBlock>,
    finally: Option<&HirBlock>,
) -> Result<(), CodegenError> {
    // WHY: A catch-bearing body is locally handled in Faber terms, so direct
    // Rust `?` would skip the catch block. The current lowering suppresses
    // propagation in that body and emits the catch/finally fragments in order.
    emitter.writer.writeln("{");
    let mut tempta_result = Ok(());
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|w| {
        let mut inner = ExprEmitter::new(codegen, types, w, policy);
        for stmt in &body.stmts {
            if tempta_result.is_err() {
                return;
            }
            tempta_result = super::super::stmt::generate_stmt(
                inner.codegen,
                stmt,
                inner.types,
                inner.writer,
                policy.can_propagate_failure,
                policy.inside_entrypoint,
                policy.propagation_suppressed || catch.is_some(),
            );
        }
        if let Some(expr) = &body.expr {
            if tempta_result.is_err() {
                return;
            }
            tempta_result = inner.expr(expr);
            inner.writer.writeln(";");
        }
        if let Some(catch) = catch {
            for stmt in &catch.stmts {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = super::super::stmt::generate_stmt(
                    inner.codegen,
                    stmt,
                    inner.types,
                    inner.writer,
                    policy.can_propagate_failure,
                    policy.inside_entrypoint,
                    policy.propagation_suppressed,
                );
            }
            if let Some(expr) = &catch.expr {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = inner.expr(expr);
                inner.writer.writeln(";");
            }
        }
        if let Some(finally) = finally {
            for stmt in &finally.stmts {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = super::super::stmt::generate_stmt(
                    inner.codegen,
                    stmt,
                    inner.types,
                    inner.writer,
                    policy.can_propagate_failure,
                    policy.inside_entrypoint,
                    policy.propagation_suppressed,
                );
            }
            if let Some(expr) = &finally.expr {
                if tempta_result.is_err() {
                    return;
                }
                tempta_result = inner.expr(expr);
                inner.writer.writeln(";");
            }
        }
    });
    tempta_result?;
    emitter.writer.write("}");
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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    emit_closure_expr(&mut emitter, params, body)
}

#[allow(clippy::too_many_arguments)]
fn emit_closure_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    params: &[HirParam],
    body: &HirExpr,
) -> Result<(), CodegenError> {
    emitter.writer.write("|");
    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            emitter.writer.write(", ");
        }
        emitter
            .writer
            .write(emitter.codegen.resolve_symbol(param.name));
    }
    emitter.writer.write("| ");
    emitter.expr(body)
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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    emit_await_expr(&mut emitter, expr)
}

#[allow(clippy::too_many_arguments)]
fn emit_await_expr(emitter: &mut ExprEmitter<'_, '_>, expr: &HirExpr) -> Result<(), CodegenError> {
    emitter.expr(expr)?;
    emitter.writer.write(".await");
    Ok(())
}
