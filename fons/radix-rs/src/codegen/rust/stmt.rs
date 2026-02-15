//! Rust statement generation

use super::super::CodeWriter;
use super::expr::generate_expr;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

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
                    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                } else {
                    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
        generate_expr(codegen, init, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }

    w.writeln(";");
    Ok(())
}
