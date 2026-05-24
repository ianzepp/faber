//! Branch and option-shaped `si` expression lowering.

use super::*;
use crate::codegen::rust::stmt::generate_stmt;

#[allow(clippy::too_many_arguments)]
pub(in crate::codegen::rust::expr) fn generate_if_expr(
    codegen: &RustCodegen<'_>,
    cond: &HirExpr,
    then: &HirBlock,
    else_: Option<&HirBlock>,
    result_ty: Option<TypeId>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("if ");
    generate_expr_unwrapped(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" ");
    if result_ty.is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_))) {
        generate_option_branch_block(codegen, then, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    } else {
        generate_block(codegen, then, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    if let Some(else_block) = else_ {
        w.write(" else ");
        if result_ty.is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_))) {
            generate_option_branch_block(
                codegen,
                else_block,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
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
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_option_branch_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if let Err(err) =
                generate_stmt(codegen, stmt, types, w, in_failable_fn, in_entry, suppress_error_propagation)
            {
                result = Err(err);
                return;
            }
        }
        if let Some(expr) = &block.expr {
            if let Err(err) = generate_option_branch_expr(
                codegen,
                expr,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            ) {
                result = Err(err);
                return;
            }
            w.newline();
        }
    });
    result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_option_branch_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if option_branch_expr_produces_option(codegen, expr, types) {
        generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        if matches!(expr.kind, HirExprKind::Path(_)) {
            w.write(".clone()");
        }
        return Ok(());
    }

    w.write("Some(");
    generate_expr_unwrapped(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::String(_))) {
        w.write(".to_string()");
    }
    w.write(")");
    Ok(())
}

fn option_branch_expr_produces_option(codegen: &RustCodegen<'_>, expr: &HirExpr, types: &TypeTable) -> bool {
    match &expr.kind {
        HirExprKind::Literal(HirLiteral::Nil) | HirExprKind::OptionalChain(_, _) | HirExprKind::Si { .. } => true,
        HirExprKind::Path(def_id) => codegen
            .binding_type(*def_id)
            .or(expr.ty)
            .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_) | Type::Primitive(Primitive::Nihil))),
        HirExprKind::Call(_, _) | HirExprKind::MethodCall(_, _, _) | HirExprKind::Field(_, _) => expr
            .ty
            .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Option(_) | Type::Primitive(Primitive::Nihil))),
        _ => false,
    }
}
