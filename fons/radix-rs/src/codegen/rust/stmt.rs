//! Rust statement generation

use super::super::CodeWriter;
use super::expr::generate_expr;
use super::types::type_to_rust;
use super::CodegenError;
use crate::hir::*;
use crate::semantic::TypeTable;

pub fn generate_stmt(
    stmt: &HirStmt,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            generate_local(local, types, w)?;
        }
        HirStmtKind::Expr(expr) => {
            generate_expr(expr, types, w)?;
            w.writeln(";");
        }
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                w.write("return ");
                generate_expr(expr, types, w)?;
                w.writeln(";");
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
    local: &HirLocal,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("let ");
    if local.mutable {
        w.write("mut ");
    }
    // TODO: Write variable name
    w.write("var");

    if let Some(ty) = local.ty {
        w.write(": ");
        w.write(&type_to_rust(ty, types));
    }

    if let Some(init) = &local.init {
        w.write(" = ");
        generate_expr(init, types, w)?;
    }

    w.writeln(";");
    Ok(())
}
