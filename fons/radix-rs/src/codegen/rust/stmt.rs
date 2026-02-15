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
) -> Result<(), CodegenError> {
    match &stmt.kind {
        HirStmtKind::Local(local) => {
            generate_local(codegen, local, types, w)?;
        }
        HirStmtKind::Expr(expr) => {
            generate_expr(codegen, expr, types, w)?;
            w.writeln(";");
        }
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                w.write("return ");
                generate_expr(codegen, expr, types, w)?;
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
    codegen: &RustCodegen<'_>,
    local: &HirLocal,
    types: &TypeTable,
    w: &mut CodeWriter,
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
        generate_expr(codegen, init, types, w)?;
    }

    w.writeln(";");
    Ok(())
}
