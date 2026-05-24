//! Operator expression emission for the Rust backend.
//!
//! Most operators are direct Rust token mappings, but this module also owns the
//! semantic exceptions: nil checks against `None`, coalescing over `Option`,
//! range membership expansion, and Faber predicate operators that Rust does not
//! have as tokens. The string mapping helpers are therefore only safe for the
//! simple cases that `generate_unary_expr` and `generate_binary_expr` do not
//! intercept first.
//!
//! CAVEATS
//! =======
//! - Strict equality currently shares Rust equality with ordinary equality.
//! - `??`, `intra`, and `inter` are marker spellings in the token maps; callers
//!   must use the specialized branches rather than emitting them directly.
//! - Assignment operators reuse binary token mapping, so only Rust-compatible
//!   assignment forms should reach this backend path.

use super::*;
pub(super) fn unop_to_rust(op: HirUnOp) -> &'static str {
    match op {
        HirUnOp::Neg => "-",
        HirUnOp::Not => "!",
        HirUnOp::BitNot => "~",
        HirUnOp::IsNull => "nulla ",
        HirUnOp::IsNotNull => "nonnulla ",
        HirUnOp::IsNil => "nihil ",
        HirUnOp::IsNotNil => "nonnihil ",
        HirUnOp::IsNeg => "negativum ",
        HirUnOp::IsPos => "positivum ",
        HirUnOp::IsTrue => "verum ",
        HirUnOp::IsFalse => "falsum ",
    }
}
pub(super) fn binop_to_rust(op: HirBinOp) -> &'static str {
    match op {
        HirBinOp::Add => "+",
        HirBinOp::Sub => "-",
        HirBinOp::Mul => "*",
        HirBinOp::Div => "/",
        HirBinOp::Mod => "%",
        HirBinOp::Eq => "==",
        HirBinOp::NotEq => "!=",
        HirBinOp::StrictEq => "==",
        HirBinOp::StrictNotEq => "!=",
        HirBinOp::Lt => "<",
        HirBinOp::Gt => ">",
        HirBinOp::LtEq => "<=",
        HirBinOp::GtEq => ">=",
        HirBinOp::And => "&&",
        HirBinOp::Or => "||",
        HirBinOp::Coalesce => "??",
        HirBinOp::BitAnd => "&",
        HirBinOp::BitOr => "|",
        HirBinOp::BitXor => "^",
        HirBinOp::Shl => "<<",
        HirBinOp::Shr => ">>",
        HirBinOp::Is => "==",
        HirBinOp::IsNot => "!=",
        HirBinOp::InRange => "intra",
        HirBinOp::Between => "inter",
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn generate_unary_expr(
    codegen: &RustCodegen<'_>,
    op: HirUnOp,
    operand: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    wrap: bool,
) -> Result<(), CodegenError> {
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_unary_expr_with_emitter(&mut emitter, op, operand, wrap)
}

fn generate_unary_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    op: HirUnOp,
    operand: &HirExpr,
    wrap: bool,
) -> Result<(), CodegenError> {
    match op {
        HirUnOp::IsNull | HirUnOp::IsNil => {
            if wrap {
                emitter.writer.write("(");
            }
            if operand
                .ty
                .is_some_and(|ty| type_id_is_faber_value(ty, emitter.types))
            {
                emitter.expr(operand)?;
                emitter.writer.write(" == FaberValue::Nihil");
            } else {
                emitter.expr(operand)?;
                emitter.writer.write(" == None");
            }
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            if wrap {
                emitter.writer.write("(");
            }
            if operand
                .ty
                .is_some_and(|ty| type_id_is_faber_value(ty, emitter.types))
            {
                emitter.expr(operand)?;
                emitter.writer.write(" != FaberValue::Nihil");
            } else {
                emitter.expr(operand)?;
                emitter.writer.write(" != None");
            }
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirUnOp::IsNeg => {
            if wrap {
                emitter.writer.write("(");
            }
            emitter.expr(operand)?;
            emitter.writer.write(" < 0");
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirUnOp::IsPos => {
            if wrap {
                emitter.writer.write("(");
            }
            emitter.expr(operand)?;
            emitter.writer.write(" > 0");
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirUnOp::IsTrue => {
            if wrap {
                emitter.writer.write("(");
            }
            emitter.expr(operand)?;
            if operand
                .ty
                .is_some_and(|ty| type_id_is_faber_value(ty, emitter.types))
            {
                emitter.writer.write(" == FaberValue::from(true)");
            } else {
                emitter.writer.write(" == true");
            }
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirUnOp::IsFalse => {
            if wrap {
                emitter.writer.write("(");
            }
            emitter.expr(operand)?;
            if operand
                .ty
                .is_some_and(|ty| type_id_is_faber_value(ty, emitter.types))
            {
                emitter.writer.write(" == FaberValue::from(false)");
            } else {
                emitter.writer.write(" == false");
            }
            if wrap {
                emitter.writer.write(")");
            }
        }
        _ => {
            emitter.writer.write(unop_to_rust(op));
            emitter.expr(operand)?;
        }
    }

    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn generate_binary_expr(
    codegen: &RustCodegen<'_>,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    result_ty: Option<TypeId>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    wrap: bool,
) -> Result<(), CodegenError> {
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_binary_expr_with_emitter(&mut emitter, op, lhs, rhs, result_ty, wrap)
}

fn generate_binary_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    result_ty: Option<TypeId>,
    wrap: bool,
) -> Result<(), CodegenError> {
    match op {
        HirBinOp::Add if is_text_expr(lhs, emitter.types) && is_text_expr(rhs, emitter.types) => {
            emitter.writer.write("format!(\"{}{}\", ");
            emitter.expr(lhs)?;
            emitter.writer.write(", ");
            emitter.expr(rhs)?;
            emitter.writer.write(")");
        }
        HirBinOp::Coalesce => {
            // Coalescing preserves Option shape when the right side is also an
            // Option; otherwise it unwraps with a plain fallback value.
            let lhs_ty = lhs.ty.map(|ty| resolve_type(ty, emitter.types));
            match lhs_ty {
                Some(Type::Option(_)) => {
                    emitter.writer.write("(");
                    emitter.expr(lhs)?;
                    emitter.writer.write(").clone()");
                    let rhs_ty = rhs.ty.map(|ty| resolve_type(ty, emitter.types));
                    if matches!(rhs_ty, Some(Type::Option(_))) {
                        emitter.writer.write(".or(");
                    } else {
                        emitter.writer.write(".unwrap_or(");
                    }
                    emitter.expr(rhs)?;
                    emitter.writer.write(")");
                }
                Some(Type::Primitive(Primitive::Nihil)) => {
                    emitter.expr(rhs)?;
                }
                _ => {
                    emitter.expr(lhs)?;
                }
            }
        }
        HirBinOp::InRange => {
            // `intra` is half-open: lower <= value < upper. Non-tuple or
            // malformed bounds have already lost their semantic shape, so the
            // backend emits a deterministic false expression.
            if let HirExprKind::Tuple(bounds) = &rhs.kind {
                if bounds.len() >= 2 {
                    if wrap {
                        emitter.writer.write("(");
                    }
                    emitter.expr(lhs)?;
                    emitter.writer.write(" >= ");
                    emitter.expr(&bounds[0])?;
                    emitter.writer.write(" && ");
                    emitter.expr(lhs)?;
                    emitter.writer.write(" < ");
                    emitter.expr(&bounds[1])?;
                    if wrap {
                        emitter.writer.write(")");
                    }
                } else {
                    emitter.writer.write("false");
                }
            } else {
                emitter.writer.write("false");
            }
        }
        HirBinOp::Between => {
            // `inter` delegates to Rust range/container `contains`, borrowing
            // the left operand to match the standard method signature.
            emitter.writer.write("(");
            emitter.expr(rhs)?;
            emitter.writer.write(").contains(&");
            emitter.expr(lhs)?;
            emitter.writer.write(")");
        }
        HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod
            if binary_result_is_fractus(result_ty, emitter.types) =>
        {
            if wrap {
                emitter.writer.write("(");
            }
            generate_fractus_operand_with_emitter(emitter, lhs)?;
            emitter.writer.write(" ");
            emitter.writer.write(binop_to_rust(op));
            emitter.writer.write(" ");
            generate_fractus_operand_with_emitter(emitter, rhs)?;
            if wrap {
                emitter.writer.write(")");
            }
        }
        HirBinOp::Eq
        | HirBinOp::NotEq
        | HirBinOp::StrictEq
        | HirBinOp::StrictNotEq
        | HirBinOp::Is
        | HirBinOp::IsNot
            if binary_has_faber_value_operand(lhs, rhs, emitter.types) =>
        {
            if wrap {
                emitter.writer.write("(");
            }
            generate_binary_faber_value_operand_with_emitter(emitter, lhs)?;
            emitter.writer.write(" ");
            emitter.writer.write(binop_to_rust(op));
            emitter.writer.write(" ");
            generate_binary_faber_value_operand_with_emitter(emitter, rhs)?;
            if wrap {
                emitter.writer.write(")");
            }
        }
        _ => {
            if wrap {
                emitter.writer.write("(");
            }
            emitter.expr(lhs)?;
            emitter.writer.write(" ");
            emitter.writer.write(binop_to_rust(op));
            emitter.writer.write(" ");
            emitter.expr(rhs)?;
            if wrap {
                emitter.writer.write(")");
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_fractus_operand_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    expr: &HirExpr,
) -> Result<(), CodegenError> {
    if expr_is_numerus(expr, emitter.types) {
        emitter.writer.write("(");
        emitter.expr(expr)?;
        emitter.writer.write(" as f64)");
        return Ok(());
    }

    emitter.expr(expr)
}

fn binary_result_is_fractus(result_ty: Option<TypeId>, types: &TypeTable) -> bool {
    result_ty.is_some_and(|ty| matches!(resolve_type(ty, types), Type::Primitive(Primitive::Fractus)))
}

fn binary_has_faber_value_operand(lhs: &HirExpr, rhs: &HirExpr, types: &TypeTable) -> bool {
    lhs.ty.is_some_and(|ty| type_id_is_faber_value(ty, types))
        || rhs.ty.is_some_and(|ty| type_id_is_faber_value(ty, types))
}

#[allow(clippy::too_many_arguments)]
fn generate_binary_faber_value_operand_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    expr: &HirExpr,
) -> Result<(), CodegenError> {
    if expr
        .ty
        .is_some_and(|ty| type_id_is_faber_value(ty, emitter.types))
    {
        return emitter.expr(expr);
    }

    generate_expr_as_faber_value_with_emitter(emitter, expr)
}

fn expr_is_numerus(expr: &HirExpr, types: &TypeTable) -> bool {
    expr.ty
        .is_some_and(|ty| matches!(resolve_type(ty, types), Type::Primitive(Primitive::Numerus)))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_assign_expr(
    codegen: &RustCodegen<'_>,
    target: &HirExpr,
    value: &HirExpr,
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
    generate_assign_expr_with_emitter(&mut emitter, target, value)
}

fn generate_assign_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    target: &HirExpr,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    if let HirExprKind::Index(object, index) = &target.kind {
        if matches!(object.ty.map(|ty| resolve_type(ty, emitter.types)), Some(Type::Map(_, _))) {
            emitter.expr(object)?;
            emitter.writer.write(".insert(");
            emitter.expr_unwrapped(index)?;
            emitter.writer.write(", ");
            match object.ty.map(|ty| resolve_type(ty, emitter.types)) {
                Some(Type::Map(_, value_ty)) => {
                    emitter.expr_as_type(value, value_ty)?;
                }
                _ => {
                    emitter.expr_unwrapped(value)?;
                }
            }
            emitter.writer.write(")");
            return Ok(());
        }
    }

    let target_ty = match &target.kind {
        HirExprKind::Path(def_id) => emitter
            .codegen
            .binding_type(*def_id)
            .or_else(|| emitter.codegen.binding_type_by_generated_name(*def_id))
            .or(target.ty),
        _ => target.ty,
    };

    emitter.expr(target)?;
    emitter.writer.write(" = ");

    if let Some(target_ty) = target_ty {
        emitter.expr_as_type(value, target_ty)
    } else {
        emitter.expr_unwrapped(value)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_assign_op_expr(
    codegen: &RustCodegen<'_>,
    op: HirBinOp,
    target: &HirExpr,
    value: &HirExpr,
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
    generate_assign_op_expr_with_emitter(&mut emitter, op, target, value)
}

fn generate_assign_op_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    op: HirBinOp,
    target: &HirExpr,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    if matches!(op, HirBinOp::Add) && is_text_expr(target, emitter.types) && is_text_expr(value, emitter.types) {
        emitter.expr(target)?;
        emitter.writer.write(".push_str(&");
        emitter.expr_unwrapped(value)?;
        emitter.writer.write(")");
        return Ok(());
    }

    emitter.expr(target)?;
    emitter.writer.write(" ");
    emitter.writer.write(binop_to_rust(op));
    emitter.writer.write("= ");
    emitter.expr_unwrapped(value)
}

fn is_text_expr(expr: &HirExpr, types: &TypeTable) -> bool {
    expr.ty
        .map(|ty| matches!(resolve_type(ty, types), Type::Primitive(Primitive::Textus)))
        .unwrap_or(false)
}

#[allow(clippy::too_many_arguments)]
fn generate_expr_as_faber_value_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    expr: &HirExpr,
) -> Result<(), CodegenError> {
    if matches!(expr.kind, HirExprKind::Literal(HirLiteral::Nil)) {
        emitter.writer.write("FaberValue::Nihil");
        return Ok(());
    }

    emitter.writer.write("FaberValue::from(");
    emitter.expr_unwrapped(expr)?;
    emitter.writer.write(")");
    Ok(())
}
