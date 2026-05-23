//! Block expression emission.
//!
//! Blocks are the common container used by control flow, loops, closures, and
//! `tempta` fragments. This file deliberately keeps block lowering mechanical:
//! statements are emitted in order, then the optional trailing expression is
//! emitted without forcing a semicolon so Rust can preserve expression-valued
//! blocks when the surrounding construct expects one.

use super::*;

/// Emit a Rust block while preserving the caller's propagation context.
///
/// Blocks do not decide whether failures propagate. They are context carriers:
/// call sites such as `tempta`, entry generation, and failable function bodies
/// choose the flags, and every statement or trailing expression observes the
/// same policy unless a nested construct overrides it.
pub(super) fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut block_result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::super::stmt::generate_stmt(
                codegen,
                stmt,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
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
