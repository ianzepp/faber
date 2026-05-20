use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn numeric_bin(&mut self, lhs: TypeId, rhs: TypeId, span: crate::lexer::Span) -> TypeId {
        let lhs_resolved = self.resolve_type(lhs);
        let rhs_resolved = self.resolve_type(rhs);
        if self.is_infer(lhs_resolved) || self.is_infer(rhs_resolved) {
            let numerus = self.numerus_type();
            if self.is_infer(lhs_resolved) {
                self.unify(lhs, numerus, span, "numeric operands required");
            }
            if self.is_infer(rhs_resolved) {
                self.unify(rhs, numerus, span, "numeric operands required");
            }
        }
        if !self.is_numeric(lhs) || !self.is_numeric(rhs) {
            self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operands required", span);
            return self.error_type;
        }

        if self.is_fractus(lhs) || self.is_fractus(rhs) {
            self.fractus_type()
        } else {
            self.numerus_type()
        }
    }

    pub(super) fn common_type(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span) -> TypeId {
        let left = self.resolve_type(a);
        let right = self.resolve_type(b);
        if self.types.assignable(left, right) {
            return right;
        }
        if self.types.assignable(right, left) {
            return left;
        }
        if self.is_numeric(left) && self.is_numeric(right) {
            return self.numeric_bin(left, right, span);
        }
        self.error(SemanticErrorKind::TypeMismatch, "incompatible types", span);
        self.error_type
    }
    pub(super) fn check_assign_op(&mut self, op: HirBinOp, target: &mut HirExpr, value: &mut HirExpr) -> TypeId {
        let target_ty = self.check_lvalue(target);
        let value_ty = self.check_expr(value);
        match op {
            HirBinOp::Add if self.is_textus(target_ty) => {
                let textus = self.textus_type();
                self.unify(value_ty, textus, target.span, "string operands required");
            }
            HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => {
                self.numeric_bin(target_ty, value_ty, target.span);
            }
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "unsupported compound assignment",
                    target.span,
                );
            }
        }
        target_ty
    }
    pub(super) fn check_assign(&mut self, target: &mut HirExpr, value: &mut HirExpr) -> TypeId {
        let target_ty = self.check_lvalue(target);
        if target_ty == self.error_type {
            self.check_expr(value);
            return self.error_type;
        }
        let value_ty = self.check_expr_with_expected(value, Some(target_ty));
        self.unify(value_ty, target_ty, value.span, "assignment type mismatch");
        target_ty
    }
    pub(super) fn check_unary(&mut self, op: crate::hir::HirUnOp, operand: &mut HirExpr) -> TypeId {
        let operand_ty = self.check_expr(operand);
        match op {
            crate::hir::HirUnOp::Neg => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let numerus = self.numerus_type();
                    return self.unify(operand_ty, numerus, operand.span, "numeric operand required");
                }
                if !self.is_numeric(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operand required", operand.span);
                }
                operand_ty
            }
            crate::hir::HirUnOp::Not => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let bivalens = self.bool_type();
                    self.unify(operand_ty, bivalens, operand.span, "boolean operand required");
                    return self.bool_type();
                }
                if !self.is_bool(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "boolean operand required", operand.span);
                }
                self.bool_type()
            }
            crate::hir::HirUnOp::BitNot => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(operand_ty, numerus, operand.span, "integer operand required");
                    return self.numerus_type();
                }
                if !self.is_integer(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "integer operand required", operand.span);
                }
                self.numerus_type()
            }
            crate::hir::HirUnOp::IsNull
            | crate::hir::HirUnOp::IsNotNull
            | crate::hir::HirUnOp::IsNil
            | crate::hir::HirUnOp::IsNotNil => self.bool_type(),
            crate::hir::HirUnOp::IsNeg | crate::hir::HirUnOp::IsPos => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(operand_ty, numerus, operand.span, "numeric operand required");
                } else if !self.is_numeric(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operand required", operand.span);
                }
                self.bool_type()
            }
            crate::hir::HirUnOp::IsTrue | crate::hir::HirUnOp::IsFalse => self.bool_type(),
        }
    }
    pub(super) fn check_binary(&mut self, op: HirBinOp, lhs: &mut HirExpr, rhs: &mut HirExpr) -> TypeId {
        let lhs_ty = self.check_expr(lhs);
        let rhs_ty = self.check_expr(rhs);

        match op {
            HirBinOp::Add => {
                if self.is_infer(self.resolve_type(lhs_ty)) && self.is_textus(rhs_ty) {
                    let textus = self.textus_type();
                    self.unify(lhs_ty, textus, lhs.span, "string operands required");
                }
                if self.is_infer(self.resolve_type(rhs_ty)) && self.is_textus(lhs_ty) {
                    let textus = self.textus_type();
                    self.unify(rhs_ty, textus, rhs.span, "string operands required");
                }
                if self.is_textus(lhs_ty) || self.is_textus(rhs_ty) {
                    if !self.is_textus(lhs_ty) || !self.is_textus(rhs_ty) {
                        self.error(SemanticErrorKind::InvalidOperandTypes, "string operands required", lhs.span);
                        return self.error_type;
                    }
                    self.textus_type()
                } else {
                    self.numeric_bin(lhs_ty, rhs_ty, lhs.span)
                }
            }
            HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => self.numeric_bin(lhs_ty, rhs_ty, lhs.span),
            HirBinOp::Eq
            | HirBinOp::NotEq
            | HirBinOp::StrictEq
            | HirBinOp::StrictNotEq
            | HirBinOp::Is
            | HirBinOp::IsNot => {
                let lhs_is_nil = matches!(self.types.get(self.resolve_type(lhs_ty)), Type::Primitive(Primitive::Nihil));
                let rhs_is_nil = matches!(self.types.get(self.resolve_type(rhs_ty)), Type::Primitive(Primitive::Nihil));
                if !(lhs_is_nil || rhs_is_nil) {
                    self.unify(lhs_ty, rhs_ty, lhs.span, "incompatible operands");
                }
                self.bool_type()
            }
            HirBinOp::Lt | HirBinOp::Gt | HirBinOp::LtEq | HirBinOp::GtEq => {
                self.numeric_bin(lhs_ty, rhs_ty, lhs.span);
                self.bool_type()
            }
            HirBinOp::InRange => {
                if self.is_infer(self.resolve_type(lhs_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(lhs_ty, numerus, lhs.span, "range operand must be numeric");
                }
                if !self.is_numeric(lhs_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "range operand must be numeric",
                        lhs.span,
                    );
                }
                self.bool_type()
            }
            HirBinOp::Between => {
                match self.types.get(self.resolve_type(rhs_ty)) {
                    Type::Array(elem) | Type::Set(elem) => {
                        self.unify(lhs_ty, *elem, lhs.span, "membership operand type mismatch");
                    }
                    Type::Map(key, _) => {
                        self.unify(lhs_ty, *key, lhs.span, "membership operand type mismatch");
                    }
                    _ => {}
                }
                self.bool_type()
            }
            HirBinOp::Coalesce => {
                let lhs_kind = self.types.get(self.resolve_type(lhs_ty)).clone();
                match lhs_kind {
                    Type::Option(inner) => match self.types.get(self.resolve_type(rhs_ty)).clone() {
                        Type::Option(rhs_inner) => {
                            self.unify(rhs_inner, inner, rhs.span, "coalesce fallback type mismatch");
                            self.types.option(inner)
                        }
                        Type::Primitive(Primitive::Nihil) => self.types.option(inner),
                        _ => {
                            self.unify(rhs_ty, inner, rhs.span, "coalesce fallback type mismatch");
                            inner
                        }
                    },
                    Type::Primitive(Primitive::Nihil) => rhs_ty,
                    _ => self.common_type(lhs_ty, rhs_ty, lhs.span),
                }
            }
            HirBinOp::And | HirBinOp::Or => {
                if !self.is_bool(lhs_ty) || !self.is_bool(rhs_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "boolean operands required", lhs.span);
                }
                self.bool_type()
            }
            HirBinOp::BitAnd | HirBinOp::BitOr | HirBinOp::BitXor | HirBinOp::Shl | HirBinOp::Shr => {
                if !self.is_integer(lhs_ty) || !self.is_integer(rhs_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "integer operands required", lhs.span);
                }
                self.numerus_type()
            }
        }
    }
}
