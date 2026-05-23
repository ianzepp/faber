//! Operator spelling and precedence policy for canonical Faber output.
//!
//! The expression writer delegates operator decisions here so syntax validity is
//! kept in one place. The numeric precedence table is not a parser table copy;
//! it is the printer's contract for when child expressions need parentheses to
//! remain parseable after HIR has normalized the original surface form.
//!
//! The backend emits canonical Faber operator spellings. That means source
//! aliases and target-language conveniences are not preserved here. Assignment
//! operators use Faber's compound glyphs when the HIR operator has a dedicated
//! assignment spelling; otherwise the binary spelling is reused so output stays
//! syntactically recognizable rather than inventing new tokens. Equal-precedence
//! child expressions are parenthesized by the expression writer, which favors
//! unambiguous reparsing over preserving associativity-derived minimality.

use crate::hir::{HirExpr, HirExprKind};

impl super::FaberCodegen {
    /// Return the binding strength used by the pretty-printer.
    ///
    /// Lower numbers bind more weakly. The expression writer parenthesizes a
    /// child when its precedence would otherwise be ambiguous under the current
    /// parent expression.
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

    /// Canonical Faber spelling for unary operators.
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

    /// Canonical Faber spelling for compound assignment operators.
    ///
    /// Only operators with assignment glyphs get special spellings. Unsupported
    /// compound forms fall back to the binary spelling so this table remains a
    /// syntax policy surface rather than a semantic validator.
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

    /// Binding strength for canonical binary operator emission.
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

    /// Canonical Faber spelling for binary operators.
    ///
    /// Strict and non-strict equality currently share glyphs in canonical Faber
    /// output because HIR has already carried the semantic distinction.
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
