//! Expression lowering
//!
//! Lowers AST expressions to HIR expressions.

use super::Lowerer;
use crate::hir::{HirBinOp, HirExpr, HirExprKind, HirLiteral, HirUnOp};
use crate::syntax::{BinaryExpr, Expr, ExprKind, Literal, UnaryExpr};

/// Lower an expression
pub fn lower_expr(lowerer: &mut Lowerer, expr: &Expr) -> HirExpr {
    let id = lowerer.next_hir_id();
    let span = expr.span;
    lowerer.current_span = span;

    let kind = match &expr.kind {
        ExprKind::Literal(lit) => lower_literal(lowerer, lit),
        ExprKind::Ident(ident) => lowerer.lower_nomen(ident),
        ExprKind::Binary(bin) => lowerer.lower_binarius(bin),
        ExprKind::Unary(unary) => lowerer.lower_unarius(unary),
        ExprKind::Call(call) => lowerer.lower_vocare(call),
        ExprKind::Member(member) => lowerer.lower_membrum(member),
        ExprKind::Index(index) => lowerer.lower_index(index),
        ExprKind::Assign(assign) => lowerer.lower_assign(assign),
        ExprKind::Cede(cede_expr) => lowerer.lower_cede(cede_expr),
        ExprKind::Array(array) => lowerer.lower_serie(array),
        ExprKind::Object(obj) => lowerer.lower_objectum(obj),
        ExprKind::Qua(cast) => lowerer.lower_qua(cast),
        ExprKind::Novum(new) => lowerer.lower_novum(new),
        ExprKind::Clausura(closure) => lowerer.lower_clausura(closure),
        ExprKind::Intervallum(range) => lowerer.lower_intervallum(range),
        _ => {
            lowerer.error("unhandled expression kind in lowering");
            HirExprKind::Error
        }
    };

    HirExpr {
        id,
        kind,
        ty: None,
        span,
    }
}

/// Lower a literal
fn lower_literal(lowerer: &mut Lowerer, lit: &Literal) -> HirExprKind {
    let hir_lit = match lit {
        Literal::Integer(n) => HirLiteral::Int(*n),
        Literal::Float(n) => HirLiteral::Float(*n),
        Literal::String(s) => HirLiteral::String(*s),
        Literal::Bool(b) => HirLiteral::Bool(*b),
        Literal::Nil => HirLiteral::Nil,
        _ => {
            lowerer.error("unsupported literal in lowering");
            return HirExprKind::Error;
        }
    };
    HirExprKind::Literal(hir_lit)
}

impl<'a> Lowerer<'a> {
    /// Lower identifier (nomen)
    fn lower_nomen(&mut self, ident: &crate::syntax::Ident) -> HirExprKind {
        match self.resolver.lookup(ident.name) {
            Some(def_id) => HirExprKind::Path(def_id),
            None => {
                self.error("undefined name in lowering");
                HirExprKind::Error
            }
        }
    }

    /// Lower binary expression (binarius)
    fn lower_binarius(&mut self, bin: &BinaryExpr) -> HirExprKind {
        let lhs = lower_expr(self, &bin.lhs);
        let rhs = lower_expr(self, &bin.rhs);

        let op = match bin.op {
            crate::syntax::BinOp::Add => HirBinOp::Add,
            crate::syntax::BinOp::Sub => HirBinOp::Sub,
            crate::syntax::BinOp::Mul => HirBinOp::Mul,
            crate::syntax::BinOp::Div => HirBinOp::Div,
            crate::syntax::BinOp::Mod => HirBinOp::Mod,
            crate::syntax::BinOp::Eq => HirBinOp::Eq,
            crate::syntax::BinOp::NotEq => HirBinOp::NotEq,
            crate::syntax::BinOp::Lt => HirBinOp::Lt,
            crate::syntax::BinOp::Gt => HirBinOp::Gt,
            crate::syntax::BinOp::LtEq => HirBinOp::LtEq,
            crate::syntax::BinOp::GtEq => HirBinOp::GtEq,
            crate::syntax::BinOp::And => HirBinOp::And,
            crate::syntax::BinOp::Or => HirBinOp::Or,
            crate::syntax::BinOp::BitAnd => HirBinOp::BitAnd,
            crate::syntax::BinOp::BitOr => HirBinOp::BitOr,
            crate::syntax::BinOp::BitXor => HirBinOp::BitXor,
            crate::syntax::BinOp::Shl => HirBinOp::Shl,
            crate::syntax::BinOp::Shr => HirBinOp::Shr,
            _ => {
                self.error("unsupported binary operator");
                return HirExprKind::Error;
            }
        };

        HirExprKind::Binary(op, Box::new(lhs), Box::new(rhs))
    }

    /// Lower unary expression (unarius)
    fn lower_unarius(&mut self, unary: &UnaryExpr) -> HirExprKind {
        let operand = lower_expr(self, &unary.operand);

        let op = match unary.op {
            crate::syntax::UnOp::Neg => HirUnOp::Neg,
            crate::syntax::UnOp::Not => HirUnOp::Not,
            crate::syntax::UnOp::BitNot => HirUnOp::BitNot,
            _ => {
                self.error("unsupported unary operator");
                return HirExprKind::Error;
            }
        };

        HirExprKind::Unary(op, Box::new(operand))
    }

    /// Lower call expression (vocare)
    fn lower_vocare(&mut self, call: &crate::syntax::CallExpr) -> HirExprKind {
        let args = call
            .args
            .iter()
            .map(|arg| lower_expr(self, &arg.value))
            .collect();

        match &call.callee.kind {
            ExprKind::Member(member) => {
                let recv = lower_expr(self, &member.object);
                let name = member.member.name;
                HirExprKind::MethodCall(Box::new(recv), name, args)
            }
            _ => {
                let callee = lower_expr(self, &call.callee);
                HirExprKind::Call(Box::new(callee), args)
            }
        }
    }

    /// Lower member access (membrum)
    fn lower_membrum(&mut self, member: &crate::syntax::MemberExpr) -> HirExprKind {
        let object = lower_expr(self, &member.object);
        let field = member.member.name;

        HirExprKind::Field(Box::new(object), field)
    }

    /// Lower index access
    fn lower_index(&mut self, index: &crate::syntax::IndexExpr) -> HirExprKind {
        let object = lower_expr(self, &index.object);
        let idx = lower_expr(self, &index.index);

        HirExprKind::Index(Box::new(object), Box::new(idx))
    }

    /// Lower assignment
    fn lower_assign(&mut self, assign: &crate::syntax::AssignExpr) -> HirExprKind {
        let target = lower_expr(self, &assign.target);
        let value = lower_expr(self, &assign.value);

        match assign.op {
            crate::syntax::AssignOp::Assign => {
                HirExprKind::Assign(Box::new(target), Box::new(value))
            }
            _ => {
                self.error("compound assignment lowering not implemented");
                HirExprKind::Error
            }
        }
    }

    /// Lower await expression (cede)
    fn lower_cede(&mut self, cede_expr: &crate::syntax::CedeExpr) -> HirExprKind {
        let expr = lower_expr(self, &cede_expr.expr);
        HirExprKind::Cede(Box::new(expr))
    }

    /// Lower array literal (serie)
    fn lower_serie(&mut self, array: &crate::syntax::ArrayExpr) -> HirExprKind {
        let mut elements = Vec::new();
        for el in &array.elements {
            match el {
                crate::syntax::ArrayElement::Expr(e) => {
                    elements.push(lower_expr(self, e));
                }
                _ => {
                    self.error("array spread lowering not implemented");
                    elements.push(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Error,
                        ty: None,
                        span: self.current_span,
                    });
                }
            }
        }

        HirExprKind::Array(elements)
    }

    /// Lower object literal (objectum)
    fn lower_objectum(&mut self, obj: &crate::syntax::ObjectExpr) -> HirExprKind {
        self.error("object literal lowering not implemented");
        HirExprKind::Error
    }

    /// Lower cast expression (qua)
    fn lower_qua(&mut self, cast: &crate::syntax::QuaExpr) -> HirExprKind {
        let expr = lower_expr(self, &cast.expr);
        // TODO: Resolve type and get TypeId
        HirExprKind::Qua(Box::new(expr), crate::semantic::TypeId(0))
    }

    /// Lower new expression (novum)
    fn lower_novum(&mut self, new: &crate::syntax::NovumExpr) -> HirExprKind {
        self.error("new expression lowering not implemented");
        HirExprKind::Error
    }

    /// Lower closure (clausura)
    fn lower_clausura(&mut self, closure: &crate::syntax::ClausuraExpr) -> HirExprKind {
        self.error("closure lowering not implemented");
        HirExprKind::Error
    }

    /// Lower range expression (intervallum)
    fn lower_intervallum(&mut self, range: &crate::syntax::IntervallumExpr) -> HirExprKind {
        self.error("range lowering not implemented");
        HirExprKind::Error
    }
}
