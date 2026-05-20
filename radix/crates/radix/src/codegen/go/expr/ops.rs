use super::*;
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
        Coalesce => "||", // WHY: Go has no ?? — || is the closest for booleans
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        InRange | Between => "&&",
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
