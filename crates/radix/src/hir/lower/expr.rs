//! Expression normalization for the AST-to-HIR boundary.
//!
//! This module lowers parser expressions into HIR nodes that later phases can
//! reason about without re-reading surface syntax. It preserves source spans and
//! HIR node identity, resolves names that must already be in scope, normalizes
//! syntactic sugar such as string-template calls and ternaries, and keeps
//! target-sensitive constructs in explicit HIR shapes for typecheck and codegen.
//!
//! INVARIANTS
//! ==========
//! - Every lowered expression receives a fresh HIR id and the original source
//!   span that should anchor later diagnostics.
//! - Lowering does not infer final expression types. The optional `ty` slot is
//!   left for typecheck except where a target type is part of the syntax itself.
//! - Unknown names or unsupported expression forms become diagnostics plus
//!   `HirExprKind::Error`, allowing downstream passes to continue collecting
//!   errors instead of crashing.
//! - Surface conveniences are normalized only when the meaning is syntactic:
//!   overload resolution, receiver typing, and backend legality remain later
//!   phase responsibilities.
//!
//! BOUNDARIES
//! ==========
//! Nullable-sensitive forms (`?.`, non-null chains, `nihil` literals, and
//! `ignotum`-adjacent conversions) are represented, not proven, here. This file
//! records the operation and source structure; typecheck decides whether the
//! operation is valid, and codegen decides how a valid operation is emitted.

use super::Lowerer;
use crate::hir::{
    HirArrayElement, HirBinOp, HirCallArg, HirExpr, HirExprKind, HirLiteral, HirNonNullKind, HirObjectField,
    HirObjectKey, HirOptionalChainKind, HirUnOp,
};
use crate::semantic::{InferVar, Primitive, Type, TypeId};
use crate::syntax::{BinaryExpr, Expr, ExprKind, Literal, UnaryExpr};

/// Lower one AST expression into the HIR expression contract.
///
/// The entry point owns the cross-cutting guarantees for this file: assign a
/// fresh HIR id, preserve the AST span for diagnostics, leave final typing to
/// typecheck, and recover unsupported forms as `Error` nodes after recording a
/// diagnostic. Nested expression lowerers return only the kind so this wrapper
/// remains the single place that stamps identity onto expression nodes.
pub fn lower_expr(lowerer: &mut Lowerer, expr: &Expr) -> HirExpr {
    let id = lowerer.next_hir_id();
    let span = expr.span;
    lowerer.current_span = span;

    let kind = match &expr.kind {
        ExprKind::Literal(lit) => lower_literal(lowerer, lit),
        ExprKind::Vacua(_) => HirExprKind::Vacua,
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
        ExprKind::Verte(verte) => lowerer.lower_verte(verte),
        ExprKind::Cede(cede_expr) => lowerer.lower_cede(cede_expr),
        ExprKind::Array(array) => lowerer.lower_serie(array),
        ExprKind::Object(obj) => lowerer.lower_objectum(obj),
        ExprKind::Finge(finge) => lowerer.lower_finge(finge),
        ExprKind::Clausura(closure) => lowerer.lower_clausura(closure),
        ExprKind::Intervallum(range) => lowerer.lower_intervallum(range),
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
fn lower_literal(_lowerer: &mut Lowerer, lit: &Literal) -> HirExprKind {
    let hir_lit = match lit {
        Literal::Integer(n) => HirLiteral::Int(*n),
        Literal::Float(n) => HirLiteral::Float(*n),
        Literal::String(s) => HirLiteral::String(*s),
        Literal::Bool(b) => HirLiteral::Bool(*b),
        Literal::Nil => HirLiteral::Nil,
    };
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

    /// Resolve an expression identifier that must already name something.
    ///
    /// Pattern and declaration lowering allocate new bindings; expression
    /// lowering only consumes resolver state. A missing name is therefore a
    /// recoverable semantic error, not an opportunity to invent a binding.
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
        HirExprKind::Si {
            cond: Box::new(cond),
            then_block: self.expr_block(then_expr),
            then_catch: None,
            else_block: Some(self.expr_block(else_expr)),
        }
    }

    /// Normalize call syntax while preserving the semantic call boundary.
    ///
    /// String literals in callee position are Faber template application, and
    /// member callees are kept as `MethodCall` so receiver typing can select the
    /// method later. Ordinary calls retain an explicit callee expression.
    fn lower_vocare(&mut self, call: &crate::syntax::CallExpr) -> HirExprKind {
        match &call.callee.kind {
            ExprKind::Literal(Literal::String(template)) => HirExprKind::Scriptum(
                *template,
                call.args
                    .iter()
                    .map(|arg| lower_expr(self, &arg.value))
                    .collect(),
            ),
            ExprKind::Member(member) => {
                let recv = lower_expr(self, &member.object);
                let name = member.member.name;
                let args = self.lower_call_args(&call.args);
                HirExprKind::MethodCall(Box::new(recv), name, args)
            }
            _ => {
                let callee = lower_expr(self, &call.callee);
                let args = self.lower_call_args(&call.args);
                HirExprKind::Call(Box::new(callee), args)
            }
        }
    }

    fn lower_call_args(&mut self, args: &[crate::syntax::Argument]) -> Vec<HirCallArg> {
        args.iter()
            .map(|arg| HirCallArg {
                name: None,
                spread: arg.spread,
                expr: lower_expr(self, &arg.value),
                span: arg.span,
            })
            .collect()
    }

    /// Lower member access, including the enum-variant shorthand.
    ///
    /// If the left side is a known enum and the member names a known variant,
    /// this is a path to the variant constructor rather than a field access.
    /// Receiver-field legality remains a typecheck concern for every other
    /// member expression.
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
            crate::syntax::OptionalChainKind::Call(args) => HirOptionalChainKind::Call(self.lower_call_args(args)),
        };
        HirExprKind::OptionalChain(Box::new(object), chain)
    }

    fn lower_non_null(&mut self, expr: &crate::syntax::NonNullExpr) -> HirExprKind {
        let object = lower_expr(self, &expr.object);
        let chain = match &expr.chain {
            crate::syntax::NonNullKind::Member(member) => HirNonNullKind::Member(member.name),
            crate::syntax::NonNullKind::Index(index) => HirNonNullKind::Index(Box::new(lower_expr(self, index))),
            crate::syntax::NonNullKind::Call(args) => HirNonNullKind::Call(self.lower_call_args(args)),
        };
        HirExprKind::NonNull(Box::new(object), chain)
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

    /// Lower static type ascription (`∷`) into target-driven HIR.
    ///
    /// The grammar exposes one conversion operator; later phases dispatch on
    /// the resolved target type rather than on a source keyword. Object literal
    /// sources are split into entries so struct and collection construction can
    /// retain named fields without requiring a dedicated object-expression HIR
    /// shape.
    fn lower_verte(&mut self, verte: &crate::syntax::VerteExpr) -> HirExprKind {
        let target = self.lower_type(&verte.ty);
        match &verte.expr.kind {
            crate::syntax::ExprKind::Object(object) => {
                let mut entries = Vec::new();
                for field in &object.fields {
                    entries.push(self.lower_object_field(field));
                }

                let placeholder = HirExpr {
                    id: self.next_hir_id(),
                    kind: HirExprKind::Tuple(Vec::new()),
                    ty: None,
                    span: self.current_span,
                };
                HirExprKind::Verte { source: Box::new(placeholder), target, entries: Some(entries) }
            }
            _ => {
                let source = lower_expr(self, &verte.expr);
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
                    elements.push(HirArrayElement::Expr(lower_expr(self, e)));
                }
                crate::syntax::ArrayElement::Spread(e) => {
                    elements.push(HirArrayElement::Spread(lower_expr(self, e)));
                }
            }
        }

        HirExprKind::Array(elements)
    }

    /// Lower object literals through the same target-shape carrier as `∷`.
    ///
    /// HIR currently represents object construction as `Verte` with extracted
    /// entries. The map target is a best-effort syntax-derived shape used to
    /// keep empty and heterogeneous objects typed enough for later analysis; it
    /// is not a substitute for typecheck's final object or struct typing.
    fn lower_objectum(&mut self, obj: &crate::syntax::ObjectExpr) -> HirExprKind {
        let textus_ty = self.types.primitive(Primitive::Textus);
        let mut value_types = Vec::new();
        let has_spread = obj
            .fields
            .iter()
            .any(|field| matches!(field.key, crate::syntax::ObjectKey::Spread(_)));
        for field in &obj.fields {
            if let Some(value) = &field.value {
                value_types.push(self.guess_expr_type(value));
            }
        }
        let value_ty = if has_spread {
            self.fresh_lower_infer_type()
        } else {
            match value_types.as_slice() {
                [] => self.types.primitive(Primitive::Ignotum),
                [single] => *single,
                _ => self.types.intern(Type::Union(value_types)),
            }
        };
        let target = self.types.map(textus_ty, value_ty);
        let mut entries = Vec::new();
        for field in &obj.fields {
            entries.push(self.lower_object_field(field));
        }
        let source =
            HirExpr { id: self.next_hir_id(), kind: HirExprKind::Tuple(Vec::new()), ty: None, span: self.current_span };
        HirExprKind::Verte { source: Box::new(source), target, entries: Some(entries) }
    }

    fn lower_object_field(&mut self, field: &crate::syntax::ObjectField) -> HirObjectField {
        let key = match &field.key {
            crate::syntax::ObjectKey::Ident(ident) => HirObjectKey::Ident(ident.name),
            crate::syntax::ObjectKey::String(string) => HirObjectKey::String(*string),
            crate::syntax::ObjectKey::Computed(expr) => HirObjectKey::Computed(lower_expr(self, expr)),
            crate::syntax::ObjectKey::Spread(expr) => HirObjectKey::Spread(lower_expr(self, expr)),
        };

        let value = match &field.value {
            Some(value) => Some(lower_expr(self, value)),
            None => match &field.key {
                crate::syntax::ObjectKey::Ident(ident) => {
                    Some(HirExpr { id: self.next_hir_id(), kind: self.lower_nomen(ident), ty: None, span: ident.span })
                }
                crate::syntax::ObjectKey::String(_)
                | crate::syntax::ObjectKey::Computed(_)
                | crate::syntax::ObjectKey::Spread(_) => None,
            },
        };

        HirObjectField { key, value }
    }

    /// Estimate only the type information that is syntactically obvious.
    ///
    /// This helper exists for object-literal lowering, where HIR needs a target
    /// carrier before typecheck has run. Anything that requires name resolution,
    /// overload logic, flow analysis, or backend knowledge becomes a fresh infer
    /// variable instead of being guessed in codegen later.
    fn guess_expr_type(&mut self, expr: &crate::syntax::Expr) -> crate::semantic::TypeId {
        match &expr.kind {
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Integer(_)) => {
                self.types.primitive(Primitive::Numerus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Float(_)) => {
                self.types.primitive(Primitive::Fractus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::String(_)) => {
                self.types.primitive(Primitive::Textus)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Bool(_)) => {
                self.types.primitive(Primitive::Bivalens)
            }
            crate::syntax::ExprKind::Literal(crate::syntax::Literal::Nil) => self.types.primitive(Primitive::Nihil),
            crate::syntax::ExprKind::Array(array) => {
                let elem_ty = match array
                    .elements
                    .iter()
                    .map(|element| match element {
                        crate::syntax::ArrayElement::Expr(expr) => self.guess_expr_type(expr),
                        crate::syntax::ArrayElement::Spread(_) => self.fresh_lower_infer_type(),
                    })
                    .collect::<Vec<_>>()
                    .as_slice()
                {
                    [] => self.fresh_lower_infer_type(),
                    [single] => *single,
                    many => self.types.intern(Type::Union(many.to_vec())),
                };
                self.types.array(elem_ty)
            }
            crate::syntax::ExprKind::Object(object) => {
                let key = self.types.primitive(Primitive::Textus);
                let value_ty = match object
                    .fields
                    .iter()
                    .filter_map(|field| field.value.as_deref())
                    .map(|value| self.guess_expr_type(value))
                    .collect::<Vec<_>>()
                    .as_slice()
                {
                    [] => self.fresh_lower_infer_type(),
                    [single] => *single,
                    many => self.types.intern(Type::Union(many.to_vec())),
                };
                self.types.map(key, value_ty)
            }
            _ => self.fresh_lower_infer_type(),
        }
    }

    /// Lower variant-construction sugar into a constructor call plus optional cast.
    ///
    /// `finge` names must already resolve to variants or type-bearing cast
    /// symbols. The lowerer records missing resolver information as diagnostics
    /// because fabricating a constructor or target type here would hide the real
    /// upstream symbol-table problem.
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
            .map(|field| HirCallArg {
                name: Some(field.name.name),
                spread: false,
                expr: lower_expr(self, &field.value),
                span: field.span,
            })
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

    /// Lower closures with a temporary scope for their parameters.
    ///
    /// Parameter bindings are available while lowering the body and disappear
    /// before the enclosing expression continues. Return type annotations remain
    /// optional HIR metadata for typecheck to reconcile with the body.
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
                sponte: false,
                fixus: false,
                default: None,
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
            crate::syntax::ClausuraBody::Fac(stmt) => self.lower_fac_expr(stmt),
        };
        self.pop_scope();

        let ret = closure.ret.as_ref().map(|ret| self.lower_type(ret));
        HirExprKind::Clausura(params, ret, Box::new(body))
    }

    /// Lower range expression (intervallum)
    fn lower_intervallum(&mut self, range: &crate::syntax::IntervallumExpr) -> HirExprKind {
        HirExprKind::Intervallum {
            start: Box::new(lower_expr(self, &range.start)),
            end: Box::new(lower_expr(self, &range.end)),
            step: range
                .step
                .as_ref()
                .map(|step| Box::new(lower_expr(self, step))),
            kind: match range.kind {
                crate::syntax::RangeKind::Exclusive => crate::hir::HirRangeKind::Exclusive,
                crate::syntax::RangeKind::Inclusive => crate::hir::HirRangeKind::Inclusive,
            },
        }
    }

    /// Lower explicit conversion syntax and collect target-shape hints.
    ///
    /// Conversion hints are symbols, not resolved types. They preserve user
    /// intent for downstream conversion analysis without forcing expression
    /// lowering to know every target-specific coercion rule.
    fn lower_conversio(&mut self, conversio: &crate::syntax::ConversioExpr) -> HirExprKind {
        let source = lower_expr(self, &conversio.expr);
        let mut params = Vec::new();
        let target = match &conversio.target {
            crate::syntax::ConversioTarget::Explicit(ty) => lower_conversio_target_type(self, ty, &mut params),
        };
        params.extend(
            conversio
                .type_params
                .iter()
                .filter_map(conversio_hint_symbol),
        );
        let fallback = conversio
            .fallback
            .as_ref()
            .map(|fb| Box::new(lower_expr(self, fb)));
        HirExprKind::Conversio { source: Box::new(source), target, params, fallback }
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

fn lower_conversio_target_type(
    lowerer: &mut Lowerer<'_>,
    ty: &crate::syntax::TypeExpr,
    params: &mut Vec<crate::lexer::Symbol>,
) -> TypeId {
    match &ty.kind {
        crate::syntax::TypeExprKind::Named(ident, target_params)
            if Primitive::from_name(lowerer.interner.resolve(ident.name)).is_some() =>
        {
            params.extend(target_params.iter().filter_map(conversio_hint_symbol));
            let bare = crate::syntax::TypeExpr {
                nullable: ty.nullable,
                mode: ty.mode,
                kind: crate::syntax::TypeExprKind::Named(
                    crate::syntax::Ident { name: ident.name, span: ident.span },
                    Vec::new(),
                ),
                span: ty.span,
            };
            lowerer.lower_type(&bare)
        }
        _ => lowerer.lower_type(ty),
    }
}

/// Extract a user-written conversion hint from a type expression.
///
/// Only bare named syntax contributes a hint. More complex type expressions are
/// still lowered as target types, but they are not collapsed into a single
/// symbol because that would lose the structure typecheck needs later.
fn conversio_hint_symbol(ty: &crate::syntax::TypeExpr) -> Option<crate::lexer::Symbol> {
    match &ty.kind {
        crate::syntax::TypeExprKind::Named(ident, _) => Some(ident.name),
        _ => None,
    }
}
