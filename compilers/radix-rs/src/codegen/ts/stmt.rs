use super::types::type_to_ts;
use super::{expr::generate_expr, CodeWriter, CodegenError, TsCodegen};
use crate::hir::{HirBlock, HirStmt, HirStmtKind};
use crate::semantic::TypeTable;

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
    }
    Ok(())
}
