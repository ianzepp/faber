use crate::hir::{HirExpr, HirExprKind};

impl super::FaberCodegen {
    pub(super) fn expr_precedence(&self, expr: &HirExpr) -> u8 {
        match &expr.kind {
            HirExprKind::Assign(_, _) | HirExprKind::AssignOp(_, _, _) => 1,
            HirExprKind::Si { .. } | HirExprKind::Conversio { .. } => 2,
            HirExprKind::Binary(op, _, _) => self.binop_precedence(*op),
            HirExprKind::Unary(_, _) | HirExprKind::Ref(_, _) | HirExprKind::Deref(_) => 12,
            HirExprKind::Call(_, _)
            | HirExprKind::MethodCall(_, _, _)
            | HirExprKind::Field(_, _)
            | HirExprKind::Index(_, _)
            | HirExprKind::OptionalChain(_, _)
            | HirExprKind::NonNull(_, _) => 13,
            _ => 14,
        }
    }
    pub(super) fn unop_to_faber(&self, op: crate::hir::HirUnOp) -> &'static str {
        match op {
            crate::hir::HirUnOp::Neg => "-",
            crate::hir::HirUnOp::Not => "non ",
            crate::hir::HirUnOp::BitNot => "¬",
            crate::hir::HirUnOp::IsNull => "nulla ",
            crate::hir::HirUnOp::IsNotNull => "nonnulla ",
            crate::hir::HirUnOp::IsNil => "nihil ",
            crate::hir::HirUnOp::IsNotNil => "nonnihil ",
            crate::hir::HirUnOp::IsNeg => "negativum ",
            crate::hir::HirUnOp::IsPos => "positivum ",
            crate::hir::HirUnOp::IsTrue => "verum ",
            crate::hir::HirUnOp::IsFalse => "falsum ",
        }
    }
    pub(super) fn assignop_to_faber(&self, op: crate::hir::HirBinOp) -> &'static str {
        match op {
            crate::hir::HirBinOp::Add => "⊕",
            crate::hir::HirBinOp::Sub => "⊖",
            crate::hir::HirBinOp::Mul => "⊛",
            crate::hir::HirBinOp::Div => "⊘",
            crate::hir::HirBinOp::BitAnd => "⊜",
            crate::hir::HirBinOp::BitOr => "⊚",
            _ => self.binop_to_faber(op),
        }
    }
    pub(super) fn binop_precedence(&self, op: crate::hir::HirBinOp) -> u8 {
        match op {
            crate::hir::HirBinOp::Coalesce => 2,
            crate::hir::HirBinOp::Or => 3,
            crate::hir::HirBinOp::And => 4,
            crate::hir::HirBinOp::Eq
            | crate::hir::HirBinOp::NotEq
            | crate::hir::HirBinOp::StrictEq
            | crate::hir::HirBinOp::StrictNotEq
            | crate::hir::HirBinOp::Is
            | crate::hir::HirBinOp::IsNot => 5,
            crate::hir::HirBinOp::Lt
            | crate::hir::HirBinOp::Gt
            | crate::hir::HirBinOp::LtEq
            | crate::hir::HirBinOp::GtEq
            | crate::hir::HirBinOp::InRange
            | crate::hir::HirBinOp::Between => 6,
            crate::hir::HirBinOp::BitOr => 7,
            crate::hir::HirBinOp::BitXor => 8,
            crate::hir::HirBinOp::BitAnd => 9,
            crate::hir::HirBinOp::Shl | crate::hir::HirBinOp::Shr => 10,
            crate::hir::HirBinOp::Add | crate::hir::HirBinOp::Sub => 11,
            crate::hir::HirBinOp::Mul | crate::hir::HirBinOp::Div | crate::hir::HirBinOp::Mod => 12,
        }
    }
    pub(super) fn binop_to_faber(&self, op: crate::hir::HirBinOp) -> &'static str {
        match op {
            crate::hir::HirBinOp::Add => "+",
            crate::hir::HirBinOp::Sub => "-",
            crate::hir::HirBinOp::Mul => "*",
            crate::hir::HirBinOp::Div => "/",
            crate::hir::HirBinOp::Mod => "%",
            crate::hir::HirBinOp::Eq => "≡",
            crate::hir::HirBinOp::NotEq => "≠",
            crate::hir::HirBinOp::StrictEq => "≡",
            crate::hir::HirBinOp::StrictNotEq => "≠",
            crate::hir::HirBinOp::Lt => "<",
            crate::hir::HirBinOp::Gt => ">",
            crate::hir::HirBinOp::LtEq => "≤",
            crate::hir::HirBinOp::GtEq => "≥",
            crate::hir::HirBinOp::And => "et",
            crate::hir::HirBinOp::Or => "aut",
            crate::hir::HirBinOp::Coalesce => "vel",
            crate::hir::HirBinOp::BitAnd => "∧",
            crate::hir::HirBinOp::BitOr => "∨",
            crate::hir::HirBinOp::BitXor => "⊻",
            crate::hir::HirBinOp::Shl => "≪",
            crate::hir::HirBinOp::Shr => "≫",
            crate::hir::HirBinOp::Is => "est",
            crate::hir::HirBinOp::IsNot => "non est",
            crate::hir::HirBinOp::InRange => "intra",
            crate::hir::HirBinOp::Between => "inter",
        }
    }
}
