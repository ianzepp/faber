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
    match op {
        HirUnOp::IsNull | HirUnOp::IsNil => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == None");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" != None");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsNeg => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" < 0");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsPos => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" > 0");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsTrue => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == true");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsFalse => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == false");
            if wrap {
                w.write(")");
            }
        }
        _ => {
            w.write(unop_to_rust(op));
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
    match op {
        HirBinOp::Add if is_text_expr(lhs, types) && is_text_expr(rhs, types) => {
            w.write("format!(\"{}{}\", ");
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(", ");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        }
        HirBinOp::Coalesce => {
            // Coalescing preserves Option shape when the right side is also an
            // Option; otherwise it unwraps with a plain fallback value.
            let lhs_ty = lhs.ty.map(|ty| resolve_type(ty, types));
            match lhs_ty {
                Some(Type::Option(_)) => {
                    w.write("(");
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(").clone()");
                    let rhs_ty = rhs.ty.map(|ty| resolve_type(ty, types));
                    if matches!(rhs_ty, Some(Type::Option(_))) {
                        w.write(".or(");
                    } else {
                        w.write(".unwrap_or(");
                    }
                    generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                }
                Some(Type::Primitive(Primitive::Nihil)) => {
                    generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                }
                _ => {
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
                        w.write("(");
                    }
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(" >= ");
                    generate_expr(
                        codegen,
                        &bounds[0],
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    w.write(" && ");
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(" < ");
                    generate_expr(
                        codegen,
                        &bounds[1],
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    if wrap {
                        w.write(")");
                    }
                } else {
                    w.write("false");
                }
            } else {
                w.write("false");
            }
        }
        HirBinOp::Between => {
            // `inter` delegates to Rust range/container `contains`, borrowing
            // the left operand to match the standard method signature.
            w.write("(");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(").contains(&");
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        }
        HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod
            if binary_result_is_fractus(result_ty, types) =>
        {
            if wrap {
                w.write("(");
            }
            generate_fractus_operand(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            w.write(binop_to_rust(op));
            w.write(" ");
            generate_fractus_operand(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if wrap {
                w.write(")");
            }
        }
        _ => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            w.write(binop_to_rust(op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if wrap {
                w.write(")");
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_fractus_operand(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if expr_is_numerus(expr, types) {
        w.write("(");
        generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" as f64)");
        return Ok(());
    }

    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

fn binary_result_is_fractus(result_ty: Option<TypeId>, types: &TypeTable) -> bool {
    result_ty.is_some_and(|ty| matches!(resolve_type(ty, types), Type::Primitive(Primitive::Fractus)))
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
    generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" = ");
    generate_expr_unwrapped(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)
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
    if matches!(op, HirBinOp::Add) && is_text_expr(target, types) && is_text_expr(value, types) {
        generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".push_str(&");
        generate_expr_unwrapped(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(")");
        return Ok(());
    }

    generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(" ");
    w.write(binop_to_rust(op));
    w.write("= ");
    generate_expr_unwrapped(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

fn is_text_expr(expr: &HirExpr, types: &TypeTable) -> bool {
    expr.ty
        .map(|ty| matches!(resolve_type(ty, types), Type::Primitive(Primitive::Textus)))
        .unwrap_or(false)
}
