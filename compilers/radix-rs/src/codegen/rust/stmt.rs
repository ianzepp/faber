//! Rust Statement Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Generates Rust statements (let bindings, expression statements, return, break,
//! continue) from HIR, handling error wrapping for return statements in failable
//! functions.
//!
//! COMPILER PHASE: Codegen (submodule)
//! INPUT: HirStmt nodes
//! OUTPUT: Rust statement source text
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Return wrapping: `redde expr` becomes `return Ok(expr)` in failable functions.
//!   WHY: Failable functions return Result; explicit returns must wrap with Ok.
//! - Control flow mapping: Direct translation of rumpe/perge to break/continue.
//!   WHY: Faber's loop control flow maps 1:1 to Rust.

use super::super::CodeWriter;
use super::expr::{generate_expr, generate_expr_unwrapped};
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

/// Generate a Rust statement.
///
/// TRANSFORMS:
///   fixum numerus x = 5  -> let x: i64 = 5;
///   varia textus s       -> let mut s: String;
///   redde x              -> return Ok(x); (in failable fn) or return x;
///   rumpe                -> break;
///   perge                -> continue;
///
/// TARGET: Rust-specific Ok wrapping for return in failable functions.
pub fn generate_stmt(
    codegen: &RustCodegen<'_>,
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            generate_local(codegen, local, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirStmtKind::Expr(expr) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.writeln(";");
        }
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                w.write("return ");
                if in_failable_fn && !in_entry {
                    w.write("Ok(");
                    generate_expr_unwrapped(
                        codegen,
                        expr,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    w.write(")");
                } else {
                    generate_expr_unwrapped(
                        codegen,
                        expr,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                }
                w.writeln(";");
            } else if in_failable_fn && !in_entry {
                w.writeln("return Ok(());");
            } else {
                w.writeln("return;");
            }
        }
        HirStmtKind::Rumpe => {
            w.writeln("break;");
        }
        HirStmtKind::Perge => {
            w.writeln("continue;");
        }
    }
    Ok(())
}

fn generate_local(
    codegen: &RustCodegen<'_>,
    local: &HirLocal,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("let ");
    if local.mutable {
        w.write("mut ");
    }
    w.write(codegen.resolve_symbol(local.name));

    if let Some(ty) = local.ty {
        w.write(": ");
        w.write(&type_to_rust(codegen, ty, types));
    }

    if let Some(init) = &local.init {
        w.write(" = ");
        if local_init_requires_some_wrapper(codegen, local, init, types) {
            w.write("Some(");
            generate_expr(codegen, init, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if matches!(init.kind, HirExprKind::Literal(HirLiteral::String(_))) {
                w.write(".to_string()");
            }
            w.write(")");
        } else {
            generate_expr(codegen, init, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
    }

    w.writeln(";");
    Ok(())
}

fn local_init_requires_some_wrapper(
    codegen: &RustCodegen<'_>,
    local: &HirLocal,
    init: &HirExpr,
    types: &TypeTable,
) -> bool {
    let Some(local_ty) = local.ty else {
        return false;
    };

    if !type_to_rust(codegen, local_ty, types).starts_with("Option<") {
        return false;
    }

    match &init.kind {
        HirExprKind::Literal(HirLiteral::Nil) => false,
        HirExprKind::Path(_)
        | HirExprKind::Call(_, _)
        | HirExprKind::MethodCall(_, _, _)
        | HirExprKind::Field(_, _)
        | HirExprKind::Index(_, _)
        | HirExprKind::Binary(_, _, _) => false,
        _ => true,
    }
}
