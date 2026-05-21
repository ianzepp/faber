use super::CodeWriter;
use crate::hir::{DefId, HirArrayElement, HirExpr, HirExprKind};
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Type, TypeTable};
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    pub(super) fn write_expr_prec(
        &self,
        expr: &HirExpr,
        parent_prec: u8,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        let expr_prec = self.expr_precedence(expr);
        let needs_parens = expr_prec <= parent_prec && parent_prec != 0;
        if needs_parens {
            w.write("(");
        }

        match &expr.kind {
            HirExprKind::Path(def_id) => w.write(&self.name_for_def(*def_id, names, interner)),
            HirExprKind::Literal(lit) => self.write_literal(lit, interner, w),
            HirExprKind::Binary(op, lhs, rhs) => {
                let op_prec = self.binop_precedence(*op);
                self.write_expr_prec(lhs, op_prec, types, names, interner, w);
                w.write(" ");
                w.write(self.binop_to_faber(*op));
                w.write(" ");
                self.write_expr_prec(rhs, op_prec, types, names, interner, w);
            }
            HirExprKind::Unary(op, operand) => {
                w.write(self.unop_to_faber(*op));
                self.write_expr_prec(operand, 12, types, names, interner, w);
            }
            HirExprKind::Call(callee, args) => {
                self.write_expr_prec(callee, 13, types, names, interner, w);
                w.write("(");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(arg, types, names, interner, w);
                }
                w.write(")");
            }
            HirExprKind::MethodCall(receiver, name, args) => {
                self.write_expr_prec(receiver, 13, types, names, interner, w);
                w.write(".");
                w.write(&self.symbol_to_string(*name, interner));
                w.write("(");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(arg, types, names, interner, w);
                }
                w.write(")");
            }
            HirExprKind::Field(object, name) => {
                self.write_expr_prec(object, 13, types, names, interner, w);
                w.write(".");
                w.write(&self.symbol_to_string(*name, interner));
            }
            HirExprKind::Index(object, index) => {
                self.write_expr_prec(object, 13, types, names, interner, w);
                w.write("[");
                self.write_expr(index, types, names, interner, w);
                w.write("]");
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.write_expr_prec(object, 13, types, names, interner, w);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(name) => {
                        w.write("?.");
                        w.write(&self.symbol_to_string(*name, interner));
                    }
                    crate::hir::HirOptionalChainKind::Index(index) => {
                        w.write("?[");
                        self.write_expr(index, types, names, interner, w);
                        w.write("]");
                    }
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        w.write("?(");
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                w.write(", ");
                            }
                            self.write_expr(arg, types, names, interner, w);
                        }
                        w.write(")");
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.write_expr_prec(object, 13, types, names, interner, w);
                match chain {
                    crate::hir::HirNonNullKind::Member(name) => {
                        w.write("!.");
                        w.write(&self.symbol_to_string(*name, interner));
                    }
                    crate::hir::HirNonNullKind::Index(index) => {
                        w.write("![");
                        self.write_expr(index, types, names, interner, w);
                        w.write("]");
                    }
                    crate::hir::HirNonNullKind::Call(args) => {
                        w.write("!(");
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                w.write(", ");
                            }
                            self.write_expr(arg, types, names, interner, w);
                        }
                        w.write(")");
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                w.write("ab ");
                self.write_expr(source, types, names, interner, w);
                if let Some(filter) = filter {
                    w.write(" ");
                    if filter.negated {
                        w.write("non ");
                    }
                    match &filter.kind {
                        crate::hir::HirCollectionFilterKind::Condition(cond) => {
                            self.write_expr(cond, types, names, interner, w);
                        }
                        crate::hir::HirCollectionFilterKind::Property(name) => {
                            w.write(&self.symbol_to_string(*name, interner));
                        }
                    }
                }
                for transform in transforms {
                    w.write(", ");
                    match transform.kind {
                        crate::hir::HirTransformKind::First => w.write("prima"),
                        crate::hir::HirTransformKind::Last => w.write("ultima"),
                        crate::hir::HirTransformKind::Sum => w.write("summa"),
                    }
                    if let Some(arg) = &transform.arg {
                        w.write(" ");
                        self.write_expr(arg, types, names, interner, w);
                    }
                }
            }
            HirExprKind::Block(block) => {
                w.writeln("{");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Si(cond, then_block, else_block) => {
                if self.write_sic_secus_chain(cond, then_block, else_block.as_ref(), types, names, interner, w) {
                    if needs_parens {
                        w.write(")");
                    }
                    return;
                }
                self.write_si_chain(cond, then_block, else_block.as_ref(), types, names, interner, w);
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                w.write("discerne ");
                for (idx, scrutinee) in scrutinees.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(scrutinee, types, names, interner, w);
                }
                w.writeln(" {");
                w.indented(|w| self.write_match_arms(arms, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Loop(block) => {
                if let Some((body_stmts, cond)) = self.as_fac_loop(block) {
                    w.writeln("fac {");
                    w.indented(|w| {
                        for stmt in body_stmts {
                            self.write_stmt(stmt, types, names, interner, w);
                        }
                    });
                    w.write("} dum ");
                    self.write_expr(cond, types, names, interner, w);
                } else {
                    w.writeln("dum verum {");
                    w.indented(|w| self.write_block(block, types, names, interner, w));
                    w.write("}");
                }
            }
            HirExprKind::Dum(cond, block) => {
                w.write("dum ");
                self.write_expr(cond, types, names, interner, w);
                w.writeln(" {");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Itera(mode, _binding, binding_name, iter, block) => {
                w.write("itera ");
                let mode_text = match mode {
                    crate::hir::HirIteraMode::Ex => "ex",
                    crate::hir::HirIteraMode::De => "de",
                    crate::hir::HirIteraMode::Pro => "pro",
                };
                w.write(mode_text);
                w.write(" ");
                self.write_expr(iter, types, names, interner, w);
                w.write(" fixum ");
                w.write(&self.symbol_to_string(*binding_name, interner));
                w.writeln(" {");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Intervallum { start, end, step, kind } => {
                self.write_expr(start, types, names, interner, w);
                w.write(match kind {
                    crate::hir::HirRangeKind::Exclusive => "‥",
                    crate::hir::HirRangeKind::Inclusive => "…",
                });
                self.write_expr(end, types, names, interner, w);
                if let Some(step) = step {
                    w.write(" per ");
                    self.write_expr(step, types, names, interner, w);
                }
            }
            HirExprKind::Assign(lhs, rhs) => {
                self.write_expr_prec(lhs, 1, types, names, interner, w);
                w.write(" ← ");
                self.write_expr_prec(rhs, 1, types, names, interner, w);
            }
            HirExprKind::AssignOp(op, lhs, rhs) => {
                self.write_expr_prec(lhs, 1, types, names, interner, w);
                w.write(" ");
                w.write(self.assignop_to_faber(*op));
                w.write(" ");
                self.write_expr_prec(rhs, 1, types, names, interner, w);
            }
            HirExprKind::Array(elements) => {
                w.write("[");
                for (idx, elem) in elements.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    match elem {
                        HirArrayElement::Expr(expr) => self.write_expr(expr, types, names, interner, w),
                        HirArrayElement::Spread(expr) => {
                            w.write("sparge ");
                            self.write_expr(expr, types, names, interner, w);
                        }
                    }
                }
                w.write("]");
            }
            HirExprKind::Struct(def_id, fields) => {
                w.write(&self.name_for_def(*def_id, names, interner));
                w.write(" {");
                if !fields.is_empty() {
                    w.newline();
                    w.indented(|w| {
                        for (idx, (name, value)) in fields.iter().enumerate() {
                            if idx > 0 {
                                w.newline();
                            }
                            w.write(&self.symbol_to_string(*name, interner));
                            w.write(": ");
                            self.write_expr(value, types, names, interner, w);
                        }
                    });
                    w.newline();
                }
                w.write("}");
            }
            HirExprKind::Tuple(items) => {
                w.write("(");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(item, types, names, interner, w);
                }
                w.write(")");
            }
            HirExprKind::Scribe(kind, args) => {
                let keyword = match kind {
                    crate::hir::HirScribeKind::Vide => "vide",
                    crate::hir::HirScribeKind::Mone => "mone",
                    crate::hir::HirScribeKind::Nota | crate::hir::HirScribeKind::Scribe => "nota",
                };
                w.write(keyword);
                w.write(" ");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(arg, types, names, interner, w);
                }
            }
            HirExprKind::Scriptum(template, args) => {
                w.write("scriptum(\"");
                w.write(&self.symbol_to_string(*template, interner));
                w.write("\"");
                for arg in args {
                    w.write(", ");
                    self.write_expr(arg, types, names, interner, w);
                }
                w.write(")");
            }
            HirExprKind::Adfirma(cond, message) => {
                w.write("adfirma ");
                self.write_expr(cond, types, names, interner, w);
                if let Some(message) = message {
                    w.write(", ");
                    self.write_expr(message, types, names, interner, w);
                }
            }
            HirExprKind::Panic(value) => {
                w.write("mori ");
                self.write_expr(value, types, names, interner, w);
            }
            HirExprKind::Throw(value) => {
                w.write("iace ");
                self.write_expr(value, types, names, interner, w);
            }
            HirExprKind::Tempta { body, catch, finally } => {
                w.writeln("{");
                w.indented(|w| {
                    self.write_block(body, types, names, interner, w);
                    if let Some(catch) = catch {
                        self.write_block(catch, types, names, interner, w);
                    }
                    if let Some(finally) = finally {
                        self.write_block(finally, types, names, interner, w);
                    }
                });
                w.write("}");
            }
            HirExprKind::Clausura(params, ret, body) => {
                w.write("clausura ");
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    w.write(&self.type_to_faber(param.ty, types, names, interner));
                    w.write(" ");
                    w.write(&self.symbol_to_string(param.name, interner));
                }
                if let Some(ret) = ret {
                    w.write(" → ");
                    w.write(&self.type_to_faber(*ret, types, names, interner));
                }
                w.write(": ");
                self.write_expr(body, types, names, interner, w);
            }
            HirExprKind::Cede(inner) => {
                w.write("cede ");
                self.write_expr(inner, types, names, interner, w);
            }
            HirExprKind::Verte { source, target, entries } => match types.get(*target) {
                Type::Struct(_) => {
                    if let Some(entries) = entries {
                        w.write("{");
                        for (idx, field) in entries.iter().enumerate() {
                            if idx > 0 {
                                w.write(", ");
                            }
                            self.write_object_field(field, types, names, interner, w);
                        }
                        w.write("} ⇢ ");
                        w.write(&self.type_to_faber(*target, types, names, interner));
                    } else {
                        self.write_expr(source, types, names, interner, w);
                        w.write(" ⇢ ");
                        w.write(&self.type_to_faber(*target, types, names, interner));
                    }
                }
                Type::Array(_) | Type::Map(_, _) | Type::Set(_) => {
                    if let Some(entries) = entries {
                        w.write("{");
                        for (idx, field) in entries.iter().enumerate() {
                            if idx > 0 {
                                w.write(", ");
                            }
                            self.write_object_field(field, types, names, interner, w);
                        }
                        w.write("} ⇢ ");
                    } else {
                        self.write_expr(source, types, names, interner, w);
                        w.write(" ⇢ ");
                    }
                    w.write(&self.type_to_faber(*target, types, names, interner));
                }
                _ => {
                    self.write_expr(source, types, names, interner, w);
                    w.write(" ⇢ ");
                    w.write(&self.type_to_faber(*target, types, names, interner));
                }
            },
            HirExprKind::Conversio { source, target, params, fallback } => {
                self.write_expr_prec(source, 2, types, names, interner, w);
                w.write(" ⇒ ");
                w.write(&self.type_to_faber(*target, types, names, interner));
                if !params.is_empty() {
                    w.write("<");
                    for (idx, param) in params.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        w.write(&self.symbol_to_string(*param, interner));
                    }
                    w.write(">");
                }
                if let Some(fallback) = fallback {
                    w.write(" vel ");
                    self.write_expr_prec(fallback, 2, types, names, interner, w);
                }
            }
            HirExprKind::Ref(kind, inner) => {
                match kind {
                    crate::hir::HirRefKind::Shared => w.write("de "),
                    crate::hir::HirRefKind::Mutable => w.write("in "),
                }
                self.write_expr_prec(inner, 12, types, names, interner, w);
            }
            HirExprKind::Deref(inner) => {
                w.write("*");
                self.write_expr_prec(inner, 12, types, names, interner, w);
            }
            HirExprKind::Error => w.write("nihil"),
        }

        if needs_parens {
            w.write(")");
        }
    }
    pub(super) fn write_expr(
        &self,
        expr: &HirExpr,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        self.write_expr_prec(expr, 0, types, names, interner, w);
    }
}
