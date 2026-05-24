//! Branch and option-shaped `si` expression lowering.

use super::*;
use crate::codegen::rust::stmt::generate_stmt;

pub(in crate::codegen::rust::expr) fn generate_if_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    cond: &HirExpr,
    then: &HirBlock,
    else_: Option<&HirBlock>,
    result_ty: Option<TypeId>,
) -> Result<(), CodegenError> {
    emitter.writer.write("if ");
    emitter.expr_unwrapped(cond)?;
    emitter.writer.write(" ");
    if result_ty.is_some_and(|ty| matches!(resolve_type(ty, emitter.types), Type::Option(_))) {
        generate_option_branch_block_with_emitter(emitter, then)?;
    } else {
        generate_block_with_emitter(emitter, then)?;
    }
    if let Some(else_block) = else_ {
        emitter.writer.write(" else ");
        if result_ty.is_some_and(|ty| matches!(resolve_type(ty, emitter.types), Type::Option(_))) {
            generate_option_branch_block_with_emitter(emitter, else_block)?;
        } else {
            generate_block_with_emitter(emitter, else_block)?;
        }
    }
    Ok(())
}

fn generate_option_branch_block_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    block: &HirBlock,
) -> Result<(), CodegenError> {
    emitter.writer.writeln("{");
    let mut result = Ok(());
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|writer| {
        let mut inner_emitter = ExprEmitter::new(codegen, types, writer, policy);
        for stmt in &block.stmts {
            if let Err(err) = generate_stmt(
                inner_emitter.codegen,
                stmt,
                inner_emitter.types,
                inner_emitter.writer,
                policy.can_propagate_failure,
                policy.inside_entrypoint,
                policy.propagation_suppressed,
            ) {
                result = Err(err);
                return;
            }
        }
        if let Some(expr) = &block.expr {
            if let Err(err) = generate_option_branch_expr_with_emitter(&mut inner_emitter, expr) {
                result = Err(err);
                return;
            }
            inner_emitter.writer.newline();
        }
    });
    result?;
    emitter.writer.write("}");
    Ok(())
}

fn generate_option_branch_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    expr: &HirExpr,
) -> Result<(), CodegenError> {
    if option_branch_expr_produces_option(emitter.codegen, expr, emitter.types) {
        emitter.expr(expr)?;
        if matches!(expr.kind, HirExprKind::Path(_)) {
            emitter.writer.write(".clone()");
        }
        return Ok(());
    }

    emitter.writer.write("Some(");
    emitter.expr_unwrapped(expr)?;
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::String(_))) {
        emitter.writer.write(".to_string()");
    }
    emitter.writer.write(")");
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
