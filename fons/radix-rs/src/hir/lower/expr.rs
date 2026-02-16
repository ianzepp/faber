//! Expression lowering
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Transforms AST expressions into HIR expressions, resolving identifiers to
//! DefIds and desugaring method calls. Each expression kind has a dedicated
//! lowering function following Latin naming conventions.
//!
//! COMPILER PHASE: HIR Lowering (submodule)
//! INPUT: AST expressions (syntax::Expr)
//! OUTPUT: HIR expressions (HirExpr) with resolved DefIds
//!
//! WHY: Separates expression lowering logic from statement/declaration lowering,
//! keeping the codebase modular and focused.
//!
//! METHOD CALL DESUGARING
//! ======================
//! `obj.method(args)` in AST becomes:
//! - HirExprKind::MethodCall(obj, method, args) in HIR
//!
//! WHY: Distinguishes method calls from field access for type checker to look
//! up method signatures on the receiver type.
//!
//! STUB CONSTRUCTS
//! ===============
//! Several Faber features lower to placeholder HIR nodes:
//! - Non-null assertion (!) - Not yet supported, becomes Error
//! - Collection pipelines (ab) - Needs dedicated HIR node
//! - String interpolation (scriptum) - Lowered as Tuple placeholder
//! - Object literals - Lowered as Tuple (needs dedicated HIR shape)
//!
//! WHY: Allows parser to accept syntax while deferring full implementation.
//! Error nodes prevent crashes and allow continued analysis.

use super::Lowerer;
use crate::hir::{
    HirBinOp, HirCollectionFilter, HirCollectionFilterKind, HirCollectionTransform, HirExpr, HirExprKind, HirLiteral,
    HirOptionalChainKind, HirTransformKind, HirUnOp,
};
use crate::semantic::{InferVar, Primitive, Type};
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
        ExprKind::Ternary(ternary) => lowerer.lower_ternary(ternary),
        ExprKind::Call(call) => lowerer.lower_vocare(call),
        ExprKind::Member(member) => lowerer.lower_membrum(member),
        ExprKind::Index(index) => lowerer.lower_index(index),
        ExprKind::OptionalChain(optional) => lowerer.lower_optional_chain(optional),
        ExprKind::NonNull(non_null) => lowerer.lower_non_null(non_null),
        ExprKind::Assign(assign) => lowerer.lower_assign(assign),
        ExprKind::Innatum(innatum) => lowerer.lower_innatum(innatum),
        ExprKind::Cede(cede_expr) => lowerer.lower_cede(cede_expr),
        ExprKind::Array(array) => lowerer.lower_serie(array),
        ExprKind::Object(obj) => lowerer.lower_objectum(obj),
        ExprKind::Qua(cast) => lowerer.lower_qua(cast),
        ExprKind::Novum(new) => lowerer.lower_novum(new),
        ExprKind::Finge(finge) => lowerer.lower_finge(finge),
        ExprKind::Clausura(closure) => lowerer.lower_clausura(closure),
        ExprKind::Intervallum(range) => lowerer.lower_intervallum(range),
        ExprKind::Ab(ab) => lowerer.lower_ab(ab),
        ExprKind::Conversio(conversio) => lowerer.lower_conversio(conversio),
        ExprKind::Scriptum(scriptum) => lowerer.lower_scriptum(scriptum),
        ExprKind::Sed(sed) => lowerer.lower_sed(sed),
        ExprKind::Praefixum(praefixum) => lowerer.lower_praefixum(praefixum),
        ExprKind::Ego(span) => lowerer.lower_ego(*span),
        ExprKind::Paren(expr) => lower_expr(lowerer, expr).kind,
        _ => {
            lowerer.error("unsupported expression kind in lowering");
            HirExprKind::Error
        }
    };

    HirExpr { id, kind, ty: None, span }
}

/// Lower a literal
fn lower_literal(lowerer: &mut Lowerer, lit: &Literal) -> HirExprKind {
    let hir_lit = match lit {
        Literal::Integer(n) => HirLiteral::Int(*n),
        Literal::Float(n) => HirLiteral::Float(*n),
        Literal::String(s) | Literal::TemplateString(s) => HirLiteral::String(*s),
        Literal::Bool(b) => HirLiteral::Bool(*b),
        Literal::Nil => HirLiteral::Nil,
    };
    let _ = lowerer;
    HirExprKind::Literal(hir_lit)
}

impl<'a> Lowerer<'a> {
    fn fresh_lower_infer_type(&mut self) -> crate::semantic::TypeId {
        let infer_id = self.next_def_id().0;
        self.types.intern(Type::Infer(InferVar(infer_id)))
    }

    fn expr_block(&mut self, expr: HirExpr) -> crate::hir::HirBlock {
        crate::hir::HirBlock { stmts: Vec::new(), expr: Some(Box::new(expr)), span: self.current_span }
    }

    /// Lower identifier (nomen)
    fn lower_nomen(&mut self, ident: &crate::syntax::Ident) -> HirExprKind {
        match self.lookup_name(ident.name) {
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
            crate::syntax::BinOp::StrictEq => HirBinOp::StrictEq,
            crate::syntax::BinOp::StrictNotEq => HirBinOp::StrictNotEq,
            crate::syntax::BinOp::Lt => HirBinOp::Lt,
            crate::syntax::BinOp::Gt => HirBinOp::Gt,
            crate::syntax::BinOp::LtEq => HirBinOp::LtEq,
            crate::syntax::BinOp::GtEq => HirBinOp::GtEq,
            crate::syntax::BinOp::And => HirBinOp::And,
            crate::syntax::BinOp::Or => HirBinOp::Or,
            crate::syntax::BinOp::Coalesce => HirBinOp::Coalesce,
            crate::syntax::BinOp::BitAnd => HirBinOp::BitAnd,
            crate::syntax::BinOp::BitOr => HirBinOp::BitOr,
            crate::syntax::BinOp::BitXor => HirBinOp::BitXor,
            crate::syntax::BinOp::Shl => HirBinOp::Shl,
            crate::syntax::BinOp::Shr => HirBinOp::Shr,
            crate::syntax::BinOp::Is => HirBinOp::Is,
            crate::syntax::BinOp::IsNot => HirBinOp::IsNot,
            crate::syntax::BinOp::InRange => HirBinOp::InRange,
            crate::syntax::BinOp::Between => HirBinOp::Between,
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
            crate::syntax::UnOp::IsNull => HirUnOp::IsNull,
            crate::syntax::UnOp::IsNotNull => HirUnOp::IsNotNull,
            crate::syntax::UnOp::IsNil => HirUnOp::IsNil,
            crate::syntax::UnOp::IsNotNil => HirUnOp::IsNotNil,
            crate::syntax::UnOp::IsNeg => HirUnOp::IsNeg,
            crate::syntax::UnOp::IsPos => HirUnOp::IsPos,
            crate::syntax::UnOp::IsTrue => HirUnOp::IsTrue,
            crate::syntax::UnOp::IsFalse => HirUnOp::IsFalse,
        };

        HirExprKind::Unary(op, Box::new(operand))
    }

    fn lower_ternary(&mut self, ternary: &crate::syntax::TernaryExpr) -> HirExprKind {
        let cond = lower_expr(self, &ternary.cond);
        let then_expr = lower_expr(self, &ternary.then);
        let else_expr = lower_expr(self, &ternary.else_);
        HirExprKind::Si(Box::new(cond), self.expr_block(then_expr), Some(self.expr_block(else_expr)))
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
        if let crate::syntax::ExprKind::Ident(base) = &member.object.kind {
            if let Some(base_def) = self.lookup_name(base.name) {
                if let Some(base_symbol) = self.resolver.get_symbol(base_def) {
                    if base_symbol.kind == crate::semantic::SymbolKind::Enum {
                        if let Some(variant_def) = self.lookup_name(member.member.name) {
                            if let Some(variant_symbol) = self.resolver.get_symbol(variant_def) {
                                if variant_symbol.kind == crate::semantic::SymbolKind::Variant {
                                    return HirExprKind::Path(variant_def);
                                }
                            }
                        }
                    }
                }
            }
        }

        let object = lower_expr(self, &member.object);
        let field = member.member.name;

        HirExprKind::Field(Box::new(object), field)
    }

    fn lower_optional_chain(&mut self, expr: &crate::syntax::OptionalChainExpr) -> HirExprKind {
        let object = lower_expr(self, &expr.object);
        let chain = match &expr.chain {
            crate::syntax::OptionalChainKind::Member(member) => HirOptionalChainKind::Member(member.name),
            crate::syntax::OptionalChainKind::Index(index) => {
                HirOptionalChainKind::Index(Box::new(lower_expr(self, index)))
            }
            crate::syntax::OptionalChainKind::Call(args) => HirOptionalChainKind::Call(
                args.iter()
                    .map(|arg| lower_expr(self, &arg.value))
                    .collect(),
            ),
        };
        HirExprKind::OptionalChain(Box::new(object), chain)
    }

    fn lower_non_null(&mut self, expr: &crate::syntax::NonNullExpr) -> HirExprKind {
        let _ = expr;
        self.error("STUB: non-null chain lowering requires dedicated HIR node");
        HirExprKind::Error
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
            crate::syntax::AssignOp::Assign => HirExprKind::Assign(Box::new(target), Box::new(value)),
            crate::syntax::AssignOp::AddAssign => {
                HirExprKind::AssignOp(HirBinOp::Add, Box::new(target), Box::new(value))
            }
            crate::syntax::AssignOp::SubAssign => {
                HirExprKind::AssignOp(HirBinOp::Sub, Box::new(target), Box::new(value))
            }
            crate::syntax::AssignOp::MulAssign => {
                HirExprKind::AssignOp(HirBinOp::Mul, Box::new(target), Box::new(value))
            }
            crate::syntax::AssignOp::DivAssign => {
                HirExprKind::AssignOp(HirBinOp::Div, Box::new(target), Box::new(value))
            }
            crate::syntax::AssignOp::BitAndAssign => {
                HirExprKind::AssignOp(HirBinOp::BitAnd, Box::new(target), Box::new(value))
            }
            crate::syntax::AssignOp::BitOrAssign => {
                HirExprKind::AssignOp(HirBinOp::BitOr, Box::new(target), Box::new(value))
            }
        }
    }

    /// Lower await expression (cede)
    fn lower_cede(&mut self, cede_expr: &crate::syntax::CedeExpr) -> HirExprKind {
        let expr = lower_expr(self, &cede_expr.expr);
        HirExprKind::Cede(Box::new(expr))
    }

    fn lower_innatum(&mut self, innatum: &crate::syntax::InnatumExpr) -> HirExprKind {
        let target = self.lower_type(&innatum.ty);
        match &innatum.expr.kind {
            crate::syntax::ExprKind::Object(object) => {
                let mut entries = Vec::new();
                for field in &object.fields {
                    let key = match &field.key {
                        crate::syntax::ObjectKey::Ident(ident) => ident.name,
                        crate::syntax::ObjectKey::String(string) => *string,
                        _ => continue,
                    };
                    let value = match &field.value {
                        Some(value) => lower_expr(self, value),
                        None => HirExpr {
                            id: self.next_hir_id(),
                            kind: HirExprKind::Path(match self.lookup_name(key) {
                                Some(def_id) => def_id,
                                None => {
                                    self.error("undefined shorthand key in innatum object");
                                    return HirExprKind::Error;
                                }
                            }),
                            ty: None,
                            span: field.span,
                        },
                    };
                    entries.push((key, value));
                }

                HirExprKind::Verte {
                    source: Box::new(lower_expr(self, &innatum.expr)),
                    target,
                    entries: Some(entries),
                }
            }
            _ => {
                let source = lower_expr(self, &innatum.expr);
                HirExprKind::Verte { source: Box::new(source), target, entries: None }
            }
        }
    }

    /// Lower array literal (serie)
    fn lower_serie(&mut self, array: &crate::syntax::ArrayExpr) -> HirExprKind {
        let mut elements = Vec::new();
        for el in &array.elements {
            match el {
                crate::syntax::ArrayElement::Expr(e) => {
                    elements.push(lower_expr(self, e));
                }
                crate::syntax::ArrayElement::Spread(e) => {
                    elements.push(lower_expr(self, e));
                }
            }
        }

        HirExprKind::Array(elements)
    }

    /// Lower object literal (objectum)
    fn lower_objectum(&mut self, obj: &crate::syntax::ObjectExpr) -> HirExprKind {
        let textus_ty = self.types.primitive(Primitive::Textus);
        let mut value_types = Vec::new();
        for field in &obj.fields {
            if let Some(value) = &field.value {
                value_types.push(self.guess_expr_type(value));
            }
        }
        let value_ty = match value_types.as_slice() {
            [] => self.fresh_lower_infer_type(),
            [single] => *single,
            _ => self.types.intern(Type::Union(value_types)),
        };
        let target = self.types.map(textus_ty, value_ty);
        let mut entries = Vec::new();
        for field in &obj.fields {
            let key = match &field.key {
                crate::syntax::ObjectKey::Ident(ident) => ident.name,
                crate::syntax::ObjectKey::String(string) => *string,
                _ => continue,
            };
            let value = match &field.value {
                Some(value) => lower_expr(self, value),
                None => match &field.key {
                    crate::syntax::ObjectKey::Ident(ident) => {
                        HirExpr { id: self.next_hir_id(), kind: self.lower_nomen(ident), ty: None, span: ident.span }
                    }
                    _ => HirExpr { id: self.next_hir_id(), kind: HirExprKind::Error, ty: None, span: field.span },
                },
            };
            entries.push((key, value));
        }
        let source =
            HirExpr { id: self.next_hir_id(), kind: HirExprKind::Tuple(Vec::new()), ty: None, span: self.current_span };
        HirExprKind::Verte { source: Box::new(source), target, entries: Some(entries) }
    }

    fn guess_expr_type(&mut self, expr: &crate::syntax::Expr) -> crate::semantic::TypeId {
        match &expr.kind {
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Integer(_)) => {
                self.types.primitive(Primitive::Numerus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Float(_)) => {
                self.types.primitive(Primitive::Fractus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::String(_))
            | crate::syntax::ExprKind::Literal(crate::syntax::Literal::TemplateString(_)) => {
                self.types.primitive(Primitive::Textus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Bool(_)) => {
                self.types.primitive(Primitive::Bivalens)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Nil) => self.types.primitive(Primitive::Nihil),
            crate::syntax::ExprKind::Array(_) => {
                let infer = self.fresh_lower_infer_type();
                self.types.array(infer)
            }
            crate::syntax::ExprKind::Object(_) => {
                let infer = self.fresh_lower_infer_type();
                let key = self.types.primitive(Primitive::Textus);
                self.types.map(key, infer)
            }
            _ => self.fresh_lower_infer_type(),
        }
    }

    /// Lower cast expression (qua)
    fn lower_qua(&mut self, cast: &crate::syntax::QuaExpr) -> HirExprKind {
        let expr = lower_expr(self, &cast.expr);
        let target = self.lower_type(&cast.ty);
        HirExprKind::Verte { source: Box::new(expr), target, entries: None }
    }

    /// Lower struct instantiation (novum) — postfix form: `expr novum Type`
    fn lower_novum(&mut self, novum: &crate::syntax::NovumExpr) -> HirExprKind {
        let target = self.lower_type(&novum.ty);

        // Extract object literal entries from the source expression when present
        match &novum.expr.kind {
            crate::syntax::ExprKind::Object(object) => {
                let mut entries = Vec::new();
                for field in &object.fields {
                    let key = match &field.key {
                        crate::syntax::ObjectKey::Ident(ident) => ident.name,
                        crate::syntax::ObjectKey::String(string) => *string,
                        _ => continue,
                    };
                    let value = match &field.value {
                        Some(value) => lower_expr(self, value),
                        None => HirExpr {
                            id: self.next_hir_id(),
                            kind: HirExprKind::Path(match self.lookup_name(key) {
                                Some(def_id) => def_id,
                                None => {
                                    self.error("undefined shorthand key in novum object");
                                    return HirExprKind::Error;
                                }
                            }),
                            ty: None,
                            span: field.span,
                        },
                    };
                    entries.push((key, value));
                }

                HirExprKind::Verte {
                    source: Box::new(lower_expr(self, &novum.expr)),
                    target,
                    entries: Some(entries),
                }
            }
            _ => {
                let source = lower_expr(self, &novum.expr);
                HirExprKind::Verte { source: Box::new(source), target, entries: None }
            }
        }
    }

    fn lower_finge(&mut self, finge: &crate::syntax::FingeExpr) -> HirExprKind {
        let Some(variant_def_id) = self.resolver.lookup(finge.variant.name) else {
            self.error("unknown variant in finge expression");
            return HirExprKind::Error;
        };

        let callee = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Path(variant_def_id),
            ty: None,
            span: finge.variant.span,
        };
        let args = finge
            .fields
            .iter()
            .map(|field| lower_expr(self, &field.value))
            .collect();

        let call = HirExpr {
            id: self.next_hir_id(),
            kind: HirExprKind::Call(Box::new(callee), args),
            ty: None,
            span: self.current_span,
        };

        if let Some(cast) = &finge.cast {
            let Some(cast_def_id) = self.resolver.lookup(cast.name) else {
                self.error("unknown cast type in finge expression");
                return HirExprKind::Error;
            };
            let Some(symbol) = self.resolver.get_symbol(cast_def_id) else {
                self.error("missing cast symbol information");
                return HirExprKind::Error;
            };
            let cast_ty = match symbol.kind {
                crate::semantic::SymbolKind::Struct => self
                    .types
                    .intern(crate::semantic::Type::Struct(cast_def_id)),
                crate::semantic::SymbolKind::Enum => self.types.intern(crate::semantic::Type::Enum(cast_def_id)),
                crate::semantic::SymbolKind::Interface => self
                    .types
                    .intern(crate::semantic::Type::Interface(cast_def_id)),
                _ => {
                    self.error("finge cast must name a type");
                    return HirExprKind::Error;
                }
            };
            return HirExprKind::Verte { source: Box::new(call), target: cast_ty, entries: None };
        }

        call.kind
    }

    /// Lower closure (clausura)
    fn lower_clausura(&mut self, closure: &crate::syntax::ClausuraExpr) -> HirExprKind {
        let mut params = Vec::new();
        self.push_scope();
        for param in &closure.params {
            let def_id = self.next_def_id();
            params.push(crate::hir::HirParam {
                def_id,
                name: param.name.name,
                ty: self.lower_type(&param.ty),
                mode: crate::hir::HirParamMode::Owned,
                optional: false,
                span: param.span,
            });
            self.bind_local(param.name.name, def_id);
        }

        let body = match &closure.body {
            crate::syntax::ClausuraBody::Expr(expr) => lower_expr(self, expr),
            crate::syntax::ClausuraBody::Block(block) => HirExpr {
                id: self.next_hir_id(),
                kind: HirExprKind::Block(self.lower_block(block)),
                ty: None,
                span: block.span,
            },
        };
        self.pop_scope();

        let ret = closure.ret.as_ref().map(|ret| self.lower_type(ret));
        HirExprKind::Clausura(params, ret, Box::new(body))
    }

    /// Lower range expression (intervallum)
    fn lower_intervallum(&mut self, range: &crate::syntax::IntervallumExpr) -> HirExprKind {
        // STUB: lowered as tuple placeholder; needs dedicated range HIR node.
        let mut items = vec![lower_expr(self, &range.start), lower_expr(self, &range.end)];
        if let Some(step) = &range.step {
            items.push(lower_expr(self, step));
        }
        HirExprKind::Tuple(items)
    }

    fn lower_ab(&mut self, ab: &crate::syntax::AbExpr) -> HirExprKind {
        let source = Box::new(lower_expr(self, &ab.source));
        let filter = ab.filter.as_ref().map(|filter| HirCollectionFilter {
            negated: filter.negated,
            kind: match &filter.kind {
                crate::syntax::CollectionFilterKind::Condition(expr) => {
                    HirCollectionFilterKind::Condition(Box::new(lower_expr(self, expr)))
                }
                crate::syntax::CollectionFilterKind::Property(ident) => HirCollectionFilterKind::Property(ident.name),
            },
        });
        let transforms = ab
            .transforms
            .iter()
            .map(|transform| HirCollectionTransform {
                kind: match transform.kind {
                    crate::syntax::TransformKind::First => HirTransformKind::First,
                    crate::syntax::TransformKind::Last => HirTransformKind::Last,
                    crate::syntax::TransformKind::Sum => HirTransformKind::Sum,
                },
                arg: transform
                    .arg
                    .as_ref()
                    .map(|arg| Box::new(lower_expr(self, arg))),
            })
            .collect();
        HirExprKind::Ab { source, filter, transforms }
    }

    fn lower_conversio(&mut self, conversio: &crate::syntax::ConversioExpr) -> HirExprKind {
        let expr = lower_expr(self, &conversio.expr);
        let target = match conversio.kind {
            crate::syntax::ConversioKind::Numeratum => self.types.primitive(crate::semantic::Primitive::Numerus),
            crate::syntax::ConversioKind::Fractatum => self.types.primitive(crate::semantic::Primitive::Fractus),
            crate::syntax::ConversioKind::Textatum => self.types.primitive(crate::semantic::Primitive::Textus),
            crate::syntax::ConversioKind::Bivalentum => self.types.primitive(crate::semantic::Primitive::Bivalens),
        };
        HirExprKind::Verte { source: Box::new(expr), target, entries: None }
    }

    fn lower_scriptum(&mut self, scriptum: &crate::syntax::ScriptumExpr) -> HirExprKind {
        let args = scriptum
            .args
            .iter()
            .map(|arg| lower_expr(self, arg))
            .collect();
        HirExprKind::Scriptum(scriptum.template, args)
    }

    fn lower_sed(&mut self, sed: &crate::syntax::SedExpr) -> HirExprKind {
        HirExprKind::Literal(HirLiteral::Regex(sed.pattern, sed.flags))
    }

    fn lower_praefixum(&mut self, praefixum: &crate::syntax::PraefixumExpr) -> HirExprKind {
        match &praefixum.body {
            crate::syntax::PraefixumBody::Expr(expr) => lower_expr(self, expr).kind,
            crate::syntax::PraefixumBody::Block(block) => HirExprKind::Block(self.lower_block(block)),
        }
    }

    fn lower_ego(&mut self, span: crate::lexer::Span) -> HirExprKind {
        self.current_span = span;
        match self.current_ego_struct {
            Some(def_id) => HirExprKind::Path(def_id),
            None => {
                self.error("ego used outside method context");
                HirExprKind::Error
            }
        }
    }
}
