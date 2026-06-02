//! Operator expression lowering for the Go backend.
//!
//! Most Faber operators map directly to Go infix or prefix syntax, but the
//! exceptions are important target contracts: strict equality is only as strict
//! as Go comparability, coalescing has custom lowering, integer division may be
//! promoted for fractional results, and nil checks are only meaningful for
//! pointer-capable option shapes.
//!
//! TARGET CONTRACTS
//! ================
//! - `==`, strict equality, and identity-style equality all emit Go equality.
//! - `??` is not emitted through the operator table in normal expression
//!   lowering; it is routed to the coalesce helper.
//! - Numeric division promotes both integer operands to `float64` only when the
//!   resolved result type is fractional.
//! - Range and between tokens share a table entry for composite lowering paths;
//!   this file does not expand range endpoints by itself.
//! - Null tests on non-option, non-unknown, non-param values fail closed to
//!   constant `false` or `true` instead of generating invalid Go comparisons.

use super::*;
use crate::hir::{HirBinOp, HirUnOp};

pub(super) fn binary_op_to_go(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        Eq | StrictEq | Is => "==",
        NotEq | StrictNotEq | IsNot => "!=",
        Lt => "<",
        Gt => ">",
        LtEq => "<=",
        GtEq => ">=",
        And => "&&",
        Or => "||",
        // Fallback table value only; generate_binary_expr routes real coalesce
        // expressions through generate_coalesce_expr before this is used.
        Coalesce => "||",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        InRange | Between => "&&",
    }
}
pub(super) fn generate_binary_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if matches!(op, HirBinOp::Coalesce) {
        return generate_coalesce_expr(codegen, expr, lhs, rhs, types, w);
    }
    if matches!(op, HirBinOp::Div)
        && matches!(
            expr.ty
                .map(|ty| normalize_receiver_type(types.get(ty), types)),
            Some(Type::Primitive(Primitive::Fractus))
        )
        && matches!(
            lhs.ty
                .map(|ty| normalize_receiver_type(types.get(ty), types)),
            Some(Type::Primitive(Primitive::Numerus))
        )
        && matches!(
            rhs.ty
                .map(|ty| normalize_receiver_type(types.get(ty), types)),
            Some(Type::Primitive(Primitive::Numerus))
        )
    {
        // WHY: Go integer division would truncate before assignment; when
        // Faber has already resolved a fractional result, promote both sides.
        w.write("(float64(");
        generate_expr(codegen, lhs, types, w)?;
        w.write(") / float64(");
        generate_expr(codegen, rhs, types, w)?;
        w.write("))");
        return Ok(());
    }
    w.write("(");
    generate_expr(codegen, lhs, types, w)?;
    w.write(" ");
    w.write(binary_op_to_go(op));
    w.write(" ");
    generate_expr(codegen, rhs, types, w)?;
    w.write(")");
    Ok(())
}

pub(super) fn generate_binary_expr_for_go_type(
    codegen: &GoCodegen<'_>,
    expected_ty: crate::semantic::TypeId,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if !matches!(
        normalize_receiver_type(types.get(expected_ty), types),
        Type::Primitive(Primitive::Fractus)
    ) || !matches!(op, HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div)
    {
        w.write("(");
        generate_expr(codegen, lhs, types, w)?;
        w.write(" ");
        w.write(binary_op_to_go(op));
        w.write(" ");
        generate_expr(codegen, rhs, types, w)?;
        w.write(")");
        return Ok(());
    }

    w.write("(");
    generate_fractus_operand(codegen, lhs, types, w)?;
    w.write(" ");
    w.write(binary_op_to_go(op));
    w.write(" ");
    generate_fractus_operand(codegen, rhs, types, w)?;
    w.write(")");
    Ok(())
}

fn generate_fractus_operand(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    if matches!(
        expr.ty
            .map(|ty| normalize_receiver_type(types.get(ty), types)),
        Some(Type::Primitive(Primitive::Numerus))
    ) {
        w.write("float64(");
        generate_expr(codegen, expr, types, w)?;
        w.write(")");
        return Ok(());
    }

    match &expr.kind {
        HirExprKind::Binary(op, lhs, rhs) => {
            if let Some(ty) = expr.ty {
                generate_binary_expr_for_go_type(codegen, ty, *op, lhs, rhs, types, w)
            } else {
                generate_expr(codegen, expr, types, w)
            }
        }
        _ => generate_expr(codegen, expr, types, w),
    }
}

pub(super) fn generate_assign_expr(
    codegen: &GoCodegen<'_>,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_expr(codegen, lhs, types, w)?;
    w.write(" = ");
    if let Some(expected_ty) = assignment_target_type(codegen, lhs, types) {
        generate_expr_for_go_type(codegen, rhs, expected_ty, types, w)
    } else {
        generate_expr(codegen, rhs, types, w)
    }
}

pub(super) fn generate_assign_op_expr(
    codegen: &GoCodegen<'_>,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    generate_expr(codegen, lhs, types, w)?;
    w.write(" ");
    w.write(assign_op_to_go(op));
    w.write(" ");
    generate_expr(codegen, rhs, types, w)
}

fn assignment_target_type(
    codegen: &GoCodegen<'_>,
    lhs: &HirExpr,
    types: &TypeTable,
) -> Option<crate::semantic::TypeId> {
    match &lhs.kind {
        HirExprKind::Path(def_id) => codegen.binding_type(*def_id).or(lhs.ty),
        HirExprKind::Field(object, field) => match &object.kind {
            HirExprKind::Path(def_id) if codegen.is_struct_def(*def_id) => {
                codegen.struct_field_type(*def_id, *field).or(lhs.ty)
            }
            _ => object
                .ty
                .and_then(|ty| match normalize_receiver_type(types.get(ty), types) {
                    Type::Struct(def_id) => codegen.struct_field_type(*def_id, *field),
                    _ => None,
                })
                .or(lhs.ty),
        },
        _ => lhs.ty,
    }
}

pub(super) fn assign_op_to_go(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
    match op {
        Add => "+=",
        Sub => "-=",
        Mul => "*=",
        Div => "/=",
        Mod => "%=",
        BitAnd => "&=",
        BitOr => "|=",
        BitXor => "^=",
        Shl => "<<=",
        Shr => ">>=",
        _ => "=",
    }
}

pub(super) fn generate_unary_expr(
    codegen: &GoCodegen<'_>,
    op: HirUnOp,
    operand: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match op {
        HirUnOp::Neg => {
            w.write("-");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::Not => {
            w.write("!");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::BitNot => {
            w.write("^");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::IsNull | HirUnOp::IsNil => {
            if !matches!(
                operand
                    .ty
                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                Some(Type::Option(_))
                    | Some(Type::Primitive(Primitive::Ignotum))
                    | Some(Type::Primitive(Primitive::Nihil))
                    | Some(Type::Param(_))
            ) {
                w.write("false");
                return Ok(());
            }
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == nil)");
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            if !matches!(
                operand
                    .ty
                    .map(|ty| normalize_receiver_type(types.get(ty), types)),
                Some(Type::Option(_))
                    | Some(Type::Primitive(Primitive::Ignotum))
                    | Some(Type::Primitive(Primitive::Nihil))
                    | Some(Type::Param(_))
            ) {
                w.write("true");
                return Ok(());
            }
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" != nil)");
        }
        HirUnOp::IsNeg => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" < 0)");
        }
        HirUnOp::IsPos => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" > 0)");
        }
        HirUnOp::IsTrue => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == true)");
        }
        HirUnOp::IsFalse => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == false)");
        }
    }
    Ok(())
}
