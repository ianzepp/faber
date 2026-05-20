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
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    wrap: bool,
) -> Result<(), CodegenError> {
    match op {
        HirBinOp::Coalesce => {
            let lhs_ty = lhs.ty.map(|ty| resolve_type(ty, types));
            match lhs_ty {
                Some(Type::Option(_)) => {
                    w.write("(");
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    let rhs_ty = rhs.ty.map(|ty| resolve_type(ty, types));
                    if matches!(rhs_ty, Some(Type::Option(_))) {
                        w.write(").or(");
                    } else {
                        w.write(").unwrap_or(");
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
            w.write("(");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(").contains(&");
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
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
