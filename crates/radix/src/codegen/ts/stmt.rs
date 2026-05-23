//! Statement and block emission for the TypeScript backend.
//!
//! This module owns the layout contract for HIR statements after semantic
//! analysis has already established control-flow validity and expression types.
//! Expression-specific target policy lives in `expr`; this file decides how
//! statement lists become stable TypeScript blocks, when block expressions
//! return their tail value, and which statement forms remain unsupported for
//! this backend.
//!
//! OUTPUT POLICY
//! =============
//! Blocks are emitted in two shapes. Normal blocks use newlines and indentation
//! for declarations, functions, and entry code. Inline blocks are compact and
//! are used inside expression IIFEs so branch, loop, try/catch, and block
//! expressions can still produce values. Both forms preserve the HIR tail
//! expression as an explicit `return`, matching Faber block-expression
//! semantics instead of depending on JavaScript's statement completion rules.
//!
//! ERROR BEHAVIOR
//! ==============
//! Unsupported statement surfaces return [`CodegenError`] immediately. The
//! caller stops emitting the current block on the first error so partially
//! written code is discarded with the failed backend result rather than exposed
//! as a successful compilation artifact.

use super::types::type_to_ts;
use super::{expr::generate_expr, CodeWriter, CodegenError, TsCodegen};
use crate::hir::{HirBlock, HirStmt, HirStmtKind};
use crate::semantic::TypeTable;

/// Emits a statement block using the backend's normal multiline layout.
///
/// A block tail expression is emitted as `return <expr>;` because HIR block
/// expressions carry values explicitly. This keeps branch and entry behavior
/// consistent even though TypeScript blocks themselves are statement-only.
pub fn generate_block(
    codegen: &TsCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if result.is_err() {
                return;
            }
            result = generate_stmt(codegen, stmt, types, w);
        }
        if result.is_ok() {
            if let Some(expr) = &block.expr {
                w.write("return ");
                result = generate_expr(codegen, expr, types, w);
                w.writeln(";");
            }
        }
    });
    result?;
    w.write("}");
    Ok(())
}

/// Emits a compact block for expression-position IIFEs.
///
/// Inline blocks share the same tail-return policy as [`generate_block`] but
/// avoid multiline formatting because they are embedded inside generated
/// expressions for branches, loops, `tempta`, and assertions.
pub fn generate_inline_block(
    codegen: &TsCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("{ ");
    for stmt in &block.stmts {
        generate_stmt(codegen, stmt, types, w)?;
        w.write(" ");
    }
    if let Some(expr) = &block.expr {
        w.write("return ");
        generate_expr(codegen, expr, types, w)?;
        w.write("; ");
    }
    w.write("}");
    Ok(())
}

/// Emits one HIR statement in TypeScript statement position.
///
/// Statement emission intentionally stays narrow: locals preserve the semantic
/// mutability decision as `const` or `let`, expression statements delegate back
/// to expression lowering, and control-transfer statements are printed
/// directly. Backend-specific gaps, such as `ad`, are reported as codegen
/// errors instead of approximate JavaScript.
pub fn generate_stmt(
    codegen: &TsCodegen<'_>,
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            w.write(if local.mutable { "let " } else { "const " });
            w.write(codegen.resolve_symbol(local.name));
            if let Some(ty) = local.ty {
                w.write(": ");
                w.write(&type_to_ts(codegen, ty, types));
            }
            if let Some(init) = &local.init {
                w.write(" = ");
                generate_expr(codegen, init, types, w)?;
            }
            w.writeln(";");
        }
        HirStmtKind::Expr(expr) => {
            generate_expr(codegen, expr, types, w)?;
            w.writeln(";");
        }
        HirStmtKind::Ad(_) => {
            return Err(CodegenError { message: "ad is not supported for TypeScript codegen".to_owned() });
        }
        HirStmtKind::Redde(expr) => {
            if let Some(expr) = expr {
                w.write("return ");
                generate_expr(codegen, expr, types, w)?;
                w.writeln(";");
            } else {
                w.writeln("return;");
            }
        }
        HirStmtKind::Rumpe => w.writeln("break;"),
        HirStmtKind::Perge => w.writeln("continue;"),
        HirStmtKind::Tacet => w.writeln("{ /* tacet: explicit noop */ }"),
    }
    Ok(())
}
