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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_block_with_emitter(&mut emitter, block)
}

fn generate_block_with_emitter(emitter: &mut ExprEmitter<'_, '_>, block: &HirBlock) -> Result<(), CodegenError> {
    emitter.writer.writeln("{");
    let mut block_result = Ok(());
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|writer| {
        let mut inner_emitter = ExprEmitter::new(codegen, types, writer, policy);
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::super::stmt::generate_stmt(
                inner_emitter.codegen,
                stmt,
                inner_emitter.types,
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
