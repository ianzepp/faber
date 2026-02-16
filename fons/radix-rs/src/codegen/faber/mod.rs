//! Faber Canonical Code Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module implements a canonical pretty-printer for Faber source code. It
//! converts HIR back into well-formatted Faber source text, enabling formatting,
//! normalization, round-trip compilation validation, and AST visualization.
//!
//! COMPILER PHASE: Codegen
//! INPUT: HirProgram (fully-analyzed HIR), TypeTable, Interner
//! OUTPUT: FaberOutput (formatted Faber source text)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Canonical form: Generate a single normalized representation for any given HIR.
//!   WHY: Consistent formatting across all Faber code, enables reliable diff comparisons.
//! - Round-trip fidelity: Preserve all semantic information from the HIR.
//!   WHY: Parser(Codegen(HIR)) should produce equivalent HIR for validation testing.
//! - Minimal whitespace: Generate readable but compact output.
//!   WHY: Balance human readability with efficient storage and transmission.
//!
//! TRADE-OFFS
//! ==========
//! - Comments and original formatting are lost (HIR doesn't preserve them).
//! - Generates Latin keywords only; no target-language interop in this backend.
//! - DefId resolution requires building a names map upfront (single-pass generation).

use super::{CodeWriter, Codegen, CodegenError};
use crate::hir::{
    DefId, HirBlock, HirCasuArm, HirEnum, HirExpr, HirExprKind, HirFunction, HirInterface, HirItem, HirItemKind,
    HirLiteral, HirPattern, HirProgram, HirStmt, HirStmtKind, HirStruct,
};
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};
use crate::FaberOutput;
use rustc_hash::FxHashMap;

// =============================================================================
// CORE
// =============================================================================
//
// The FaberCodegen struct is stateless; all name resolution is performed during
// generation by building a DefId -> Symbol map from the HIR. This enables
// parallel code generation if needed in the future.

/// Faber canonical code generator.
///
/// WHY: Separate from RustCodegen to maintain clean separation between Faber
/// pretty-printing (for tooling/formatting) and target code generation.
pub struct FaberCodegen;

impl FaberCodegen {
    pub fn new() -> Self {
        Self
    }

    fn generate_item(
        &self,
        item: &HirItem,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => {
                self.generate_function(func, types, names, interner, w)?;
            }
            HirItemKind::Struct(s) => {
                self.generate_struct(s, types, names, interner, w)?;
            }
            HirItemKind::Enum(e) => {
                self.generate_enum(e, types, names, interner, w)?;
            }
            HirItemKind::Interface(i) => {
                self.generate_interface(i, types, names, interner, w)?;
            }
            HirItemKind::TypeAlias(a) => {
                w.write("typus ");
                w.write(&self.symbol_to_string(a.name, interner));
                w.write(" = ");
                w.write(&self.type_to_faber(a.ty, types, names, interner));
                w.newline();
            }
            HirItemKind::Const(c) => {
                w.write("fixum ");
                if let Some(ty) = c.ty {
                    w.write(&self.type_to_faber(ty, types, names, interner));
                    w.write(" ");
                }
                w.write(&self.symbol_to_string(c.name, interner));
                w.write(" = ");
                self.write_expr(&c.value, types, names, interner, w);
                w.newline();
            }
            HirItemKind::Import(import) => {
                w.write("importa ");
                w.write(&self.symbol_to_string(import.path, interner));
                if !import.items.is_empty() {
                    w.write(" {");
                    for (idx, item) in import.items.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        let name = self.symbol_to_string(item.name, interner);
                        if let Some(alias) = item.alias {
                            let alias = self.symbol_to_string(alias, interner);
                            w.write(&format!("{} ut {}", name, alias));
                        } else {
                            w.write(&name);
                        }
                    }
                    w.write("}");
                }
                w.newline();
            }
        }
        Ok(())
    }

    fn generate_function(
        &self,
        func: &HirFunction,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("functio ");
        w.write(&self.symbol_to_string(func.name, interner));

        if !func.type_params.is_empty() {
            w.write("(");
            for (i, param) in func.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write("prae typus ");
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(")");
        } else {
            w.write("(");
        }

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 || !func.type_params.is_empty() {
                w.write(", ");
            }
            match param.mode {
                crate::hir::HirParamMode::Ref => w.write("de "),
                crate::hir::HirParamMode::MutRef => w.write("in "),
                crate::hir::HirParamMode::Move => w.write("ex "),
                crate::hir::HirParamMode::Owned => {}
            }
            w.write(&self.type_to_faber(param.ty, types, names, interner));
            w.write(" ");
            w.write(&self.symbol_to_string(param.name, interner));
        }
        w.write(")");

        if let Some(ret) = func.ret_ty {
            w.write(" -> ");
            w.write(&self.type_to_faber(ret, types, names, interner));
        }

        if let Some(body) = &func.body {
            w.writeln(" {");
            w.indented(|w| self.write_block(body, types, names, interner, w));
            w.writeln("}");
        } else {
            w.newline();
        }

        Ok(())
    }

    fn generate_struct(
        &self,
        s: &HirStruct,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("genus ");
        w.write(&self.symbol_to_string(s.name, interner));

        if !s.type_params.is_empty() {
            w.write("<");
            for (i, param) in s.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        if let Some(parent) = s.extends {
            w.write(" sub ");
            w.write(&self.name_for_def(parent, names, interner));
        }

        if !s.implements.is_empty() {
            w.write(" implet ");
            for (i, interface) in s.implements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.name_for_def(*interface, names, interner));
            }
        }

        w.writeln(" {");
        let mut struct_result = Ok(());
        w.indented(|w| {
            for field in &s.fields {
                if field.is_static {
                    w.write("generis ");
                }
                w.write(&self.type_to_faber(field.ty, types, names, interner));
                w.write(" ");
                w.write(&self.symbol_to_string(field.name, interner));
                if let Some(init) = &field.init {
                    w.write(" = ");
                    self.write_expr(init, types, names, interner, w);
                }
                w.newline();
            }

            for method in &s.methods {
                if struct_result.is_err() {
                    return;
                }
                w.newline();
                struct_result = self.generate_function(&method.func, types, names, interner, w);
            }
        });
        struct_result?;
        w.writeln("}");

        Ok(())
    }

    fn generate_enum(
        &self,
        e: &HirEnum,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("discretio ");
        w.write(&self.symbol_to_string(e.name, interner));

        if !e.type_params.is_empty() {
            w.write("<");
            for (i, param) in e.type_params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        w.writeln(" {");
        w.indented(|w| {
            for variant in &e.variants {
                w.write(&self.symbol_to_string(variant.name, interner));
                if !variant.fields.is_empty() {
                    w.writeln(" {");
                    w.indented(|w| {
                        for field in &variant.fields {
                            w.write(&self.type_to_faber(field.ty, types, names, interner));
                            w.write(" ");
                            w.write(&self.symbol_to_string(field.name, interner));
                            w.newline();
                        }
                    });
                    w.write("}");
                }
                w.writeln(",");
            }
        });
        w.writeln("}");

        Ok(())
    }

    fn generate_interface(
        &self,
        i: &HirInterface,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> Result<(), CodegenError> {
        w.write("pactum ");
        w.write(&self.symbol_to_string(i.name, interner));

        if !i.type_params.is_empty() {
            w.write("<");
            for (idx, param) in i.type_params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(&self.symbol_to_string(param.name, interner));
            }
            w.write(">");
        }

        w.writeln(" {");
        w.indented(|w| {
            for method in &i.methods {
                w.write("functio ");
                w.write(&self.symbol_to_string(method.name, interner));
                w.write("(");
                for (idx, param) in method.params.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    w.write(&self.type_to_faber(param.ty, types, names, interner));
                    w.write(" ");
                    w.write(&self.symbol_to_string(param.name, interner));
                }
                w.write(")");
                if let Some(ret) = method.ret_ty {
                    w.write(" -> ");
                    w.write(&self.type_to_faber(ret, types, names, interner));
                }
                w.newline();
            }
        });
        w.writeln("}");

        Ok(())
    }

    /// Convert a TypeId to canonical Faber type syntax.
    ///
    /// TRANSFORMS:
    ///   Type::Primitive(Numerus) -> "numerus"
    ///   Type::Array(elem)        -> "lista<elem>"
    ///   Type::Ref(Immutable, T)  -> "de T"
    ///
    /// WHY: Type syntax must match Faber grammar exactly for round-trip validity.
    fn type_to_faber(
        &self,
        type_id: TypeId,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
    ) -> String {
        let ty = types.get(type_id);

        match ty {
            Type::Primitive(prim) => match prim {
                Primitive::Textus => "textus",
                Primitive::Numerus => "numerus",
                Primitive::Fractus => "fractus",
                Primitive::Bivalens => "bivalens",
                Primitive::Nihil => "nihil",
                Primitive::Vacuum => "vacuum",
                Primitive::Numquam => "numquam",
                Primitive::Ignotum => "ignotum",
                Primitive::Octeti => "octeti",
            }
            .to_owned(),

            Type::Array(elem) => format!("lista<{}>", self.type_to_faber(*elem, types, names, interner)),

            Type::Map(key, value) => format!(
                "tabula<{}, {}>",
                self.type_to_faber(*key, types, names, interner),
                self.type_to_faber(*value, types, names, interner)
            ),

            Type::Set(elem) => format!("copia<{}>", self.type_to_faber(*elem, types, names, interner)),

            Type::Option(inner) => {
                format!("si {}", self.type_to_faber(*inner, types, names, interner))
            }

            Type::Ref(mutability, inner) => {
                let prefix = match mutability {
                    Mutability::Immutable => "de",
                    Mutability::Mutable => "in",
                };
                format!("{} {}", prefix, self.type_to_faber(*inner, types, names, interner))
            }

            Type::Struct(def_id) | Type::Enum(def_id) | Type::Interface(def_id) => {
                self.name_for_def(*def_id, names, interner)
            }

            Type::Alias(def_id, resolved) => names
                .get(def_id)
                .map(|sym| self.symbol_to_string(*sym, interner))
                .unwrap_or_else(|| self.type_to_faber(*resolved, types, names, interner)),

            Type::Func(sig) => {
                let params: Vec<String> = sig
                    .params
                    .iter()
                    .map(|p| self.type_to_faber(p.ty, types, names, interner))
                    .collect();
                let ret = self.type_to_faber(sig.ret, types, names, interner);
                format!("({}) -> {}", params.join(", "), ret)
            }

            Type::Param(sym) => self.symbol_to_string(*sym, interner),

            Type::Applied(base, args) => {
                let base_str = self.type_to_faber(*base, types, names, interner);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.type_to_faber(*a, types, names, interner))
                    .collect();
                format!("{}<{}>", base_str, args_str.join(", "))
            }

            Type::Infer(_) => "/* unresolved */".to_owned(),
            Type::Union(_) => "unio".to_owned(),
            Type::Error => "/* error */".to_owned(),
        }
    }

    fn write_block(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        for stmt in &block.stmts {
            self.write_stmt(stmt, types, names, interner, w);
        }
        if let Some(expr) = &block.expr {
            self.write_expr(expr, types, names, interner, w);
            w.newline();
        }
    }

    fn write_stmt(
        &self,
        stmt: &HirStmt,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match &stmt.kind {
            HirStmtKind::Local(local) => {
                if local.mutable {
                    w.write("varia ");
                } else {
                    w.write("fixum ");
                }
                if let Some(ty) = local.ty {
                    w.write(&self.type_to_faber(ty, types, names, interner));
                    w.write(" ");
                }
                w.write(&self.symbol_to_string(local.name, interner));
                if let Some(init) = &local.init {
                    w.write(" = ");
                    self.write_expr(init, types, names, interner, w);
                }
                w.newline();
            }
            HirStmtKind::Expr(expr) => {
                self.write_expr(expr, types, names, interner, w);
                w.newline();
            }
            HirStmtKind::Redde(value) => {
                w.write("redde");
                if let Some(expr) = value {
                    w.write(" ");
                    self.write_expr(expr, types, names, interner, w);
                }
                w.newline();
            }
            HirStmtKind::Rumpe => {
                w.writeln("rumpe");
            }
            HirStmtKind::Perge => {
                w.writeln("perge");
            }
        }
    }

    fn write_expr(
        &self,
        expr: &HirExpr,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                w.write(&self.name_for_def(*def_id, names, interner));
            }
            HirExprKind::Literal(lit) => {
                self.write_literal(lit, interner, w);
            }
            HirExprKind::Binary(op, lhs, rhs) => {
                self.write_expr(lhs, types, names, interner, w);
                w.write(" ");
                w.write(self.binop_to_faber(*op));
                w.write(" ");
                self.write_expr(rhs, types, names, interner, w);
            }
            HirExprKind::Unary(op, operand) => {
                w.write(self.unop_to_faber(*op));
                self.write_expr(operand, types, names, interner, w);
            }
            HirExprKind::Call(callee, args) => {
                self.write_expr(callee, types, names, interner, w);
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
                self.write_expr(receiver, types, names, interner, w);
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
                self.write_expr(object, types, names, interner, w);
                w.write(".");
                w.write(&self.symbol_to_string(*name, interner));
            }
            HirExprKind::Index(object, index) => {
                self.write_expr(object, types, names, interner, w);
                w.write("[");
                self.write_expr(index, types, names, interner, w);
                w.write("]");
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.write_expr(object, types, names, interner, w);
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
                self.write_si_chain(cond, then_block, else_block.as_ref(), types, names, interner, w);
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                w.write("discerne ");
                self.write_expr(scrutinee, types, names, interner, w);
                w.writeln(" {");
                w.indented(|w| self.write_match_arms(arms, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Loop(block) => {
                w.writeln("dum verum {");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Dum(cond, block) => {
                w.write("dum ");
                self.write_expr(cond, types, names, interner, w);
                w.writeln(" {");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Itera(mode, binding, iter, block) => {
                w.write("itera ");
                let mode_text = match mode {
                    crate::hir::HirIteraMode::Ex => "ex",
                    crate::hir::HirIteraMode::De => "de",
                    crate::hir::HirIteraMode::Pro => "pro",
                };
                w.write(mode_text);
                w.write(" ");
                w.write(&self.name_for_def(*binding, names, interner));
                w.write(" ");
                self.write_expr(iter, types, names, interner, w);
                w.writeln(" {");
                w.indented(|w| self.write_block(block, types, names, interner, w));
                w.write("}");
            }
            HirExprKind::Assign(lhs, rhs) => {
                self.write_expr(lhs, types, names, interner, w);
                w.write(" = ");
                self.write_expr(rhs, types, names, interner, w);
            }
            HirExprKind::AssignOp(op, lhs, rhs) => {
                self.write_expr(lhs, types, names, interner, w);
                w.write(" ");
                w.write(self.binop_to_faber(*op));
                w.write("= ");
                self.write_expr(rhs, types, names, interner, w);
            }
            HirExprKind::Array(elements) => {
                w.write("[");
                for (idx, elem) in elements.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(elem, types, names, interner, w);
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
            HirExprKind::Scribe(args) => {
                w.write("scribe ");
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
                w.write("clausura(");
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    w.write(&self.type_to_faber(param.ty, types, names, interner));
                    w.write(" ");
                    w.write(&self.symbol_to_string(param.name, interner));
                }
                w.write(")");
                if let Some(ret) = ret {
                    w.write(" -> ");
                    w.write(&self.type_to_faber(*ret, types, names, interner));
                }
                w.writeln(" {");
                w.indented(|w| {
                    self.write_expr(body, types, names, interner, w);
                    w.newline();
                });
                w.write("}");
            }
            HirExprKind::Cede(inner) => {
                w.write("cede ");
                self.write_expr(inner, types, names, interner, w);
            }
            HirExprKind::Qua(inner, target) => {
                self.write_expr(inner, types, names, interner, w);
                w.write(" qua ");
                w.write(&self.type_to_faber(*target, types, names, interner));
            }
            HirExprKind::Innatum { source, target, .. } => {
                self.write_expr(source, types, names, interner, w);
                w.write(" innatum ");
                w.write(&self.type_to_faber(*target, types, names, interner));
            }
            HirExprKind::Ref(kind, inner) => {
                match kind {
                    crate::hir::HirRefKind::Shared => w.write("de "),
                    crate::hir::HirRefKind::Mutable => w.write("in "),
                }
                self.write_expr(inner, types, names, interner, w);
            }
            HirExprKind::Deref(inner) => {
                w.write("*");
                self.write_expr(inner, types, names, interner, w);
            }
            HirExprKind::Error => w.write("nihil"),
        }
    }

    fn write_match_arms(
        &self,
        arms: &[HirCasuArm],
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        for arm in arms {
            w.write("casu ");
            self.write_pattern(&arm.pattern, names, interner, w);
            if let Some(guard) = &arm.guard {
                w.write(" si ");
                self.write_expr(guard, types, names, interner, w);
            }
            w.writeln(" {");
            w.indented(|w| match &arm.body.kind {
                HirExprKind::Block(block) => self.write_block(block, types, names, interner, w),
                _ => {
                    self.write_expr(&arm.body, types, names, interner, w);
                    w.newline();
                }
            });
            w.writeln("}");
        }
    }

    fn write_pattern(
        &self,
        pattern: &HirPattern,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match pattern {
            HirPattern::Wildcard => w.write("_"),
            HirPattern::Binding(def_id, name) => {
                let name = names.get(def_id).copied().unwrap_or(*name);
                w.write(&self.symbol_to_string(name, interner));
            }
            HirPattern::Variant(def_id, patterns) => {
                w.write(&self.name_for_def(*def_id, names, interner));
                if !patterns.is_empty() {
                    w.write("(");
                    for (idx, pat) in patterns.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        self.write_pattern(pat, names, interner, w);
                    }
                    w.write(")");
                }
            }
            HirPattern::Literal(lit) => {
                self.write_literal(lit, interner, w);
            }
        }
    }

    fn write_literal(&self, lit: &HirLiteral, interner: &Interner, w: &mut CodeWriter) {
        match lit {
            HirLiteral::Int(value) => w.write(&value.to_string()),
            HirLiteral::Float(value) => w.write(&value.to_string()),
            HirLiteral::String(sym) => {
                w.write("\"");
                w.write(interner.resolve(*sym));
                w.write("\"");
            }
            HirLiteral::Bool(value) => w.write(if *value { "verum" } else { "falsum" }),
            HirLiteral::Nil => w.write("nihil"),
        }
    }

    fn binop_to_faber(&self, op: crate::hir::HirBinOp) -> &'static str {
        match op {
            crate::hir::HirBinOp::Add => "+",
            crate::hir::HirBinOp::Sub => "-",
            crate::hir::HirBinOp::Mul => "*",
            crate::hir::HirBinOp::Div => "/",
            crate::hir::HirBinOp::Mod => "%",
            crate::hir::HirBinOp::Eq => "==",
            crate::hir::HirBinOp::NotEq => "!=",
            crate::hir::HirBinOp::StrictEq => "===",
            crate::hir::HirBinOp::StrictNotEq => "!==",
            crate::hir::HirBinOp::Lt => "<",
            crate::hir::HirBinOp::Gt => ">",
            crate::hir::HirBinOp::LtEq => "<=",
            crate::hir::HirBinOp::GtEq => ">=",
            crate::hir::HirBinOp::And => "et",
            crate::hir::HirBinOp::Or => "aut",
            crate::hir::HirBinOp::Coalesce => "vel",
            crate::hir::HirBinOp::BitAnd => "&",
            crate::hir::HirBinOp::BitOr => "|",
            crate::hir::HirBinOp::BitXor => "^",
            crate::hir::HirBinOp::Shl => "<<",
            crate::hir::HirBinOp::Shr => ">>",
            crate::hir::HirBinOp::Is => "est",
            crate::hir::HirBinOp::IsNot => "non est",
            crate::hir::HirBinOp::InRange => "intra",
            crate::hir::HirBinOp::Between => "inter",
        }
    }

    fn unop_to_faber(&self, op: crate::hir::HirUnOp) -> &'static str {
        match op {
            crate::hir::HirUnOp::Neg => "-",
            crate::hir::HirUnOp::Not => "non ",
            crate::hir::HirUnOp::BitNot => "~",
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

    fn name_for_def(&self, def_id: DefId, names: &FxHashMap<DefId, Symbol>, interner: &Interner) -> String {
        names
            .get(&def_id)
            .map(|sym| self.symbol_to_string(*sym, interner))
            .unwrap_or_else(|| format!("def_{}", def_id.0))
    }

    fn symbol_to_string(&self, sym: Symbol, interner: &Interner) -> String {
        interner.resolve(sym).to_owned()
    }

    /// Collect DefId -> Symbol mappings for all definitions in the program.
    ///
    /// WHY: HIR uses DefIds for references; we need to map them back to their
    /// original names for source generation. This is a single upfront traversal
    /// rather than repeated lookups during generation.
    fn collect_names(&self, hir: &HirProgram) -> FxHashMap<DefId, Symbol> {
        let mut names = FxHashMap::default();
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    self.collect_function_names(&mut names, item.def_id, func);
                }
                HirItemKind::Struct(strukt) => {
                    names.insert(item.def_id, strukt.name);
                    for field in &strukt.fields {
                        names.insert(field.def_id, field.name);
                    }
                    for method in &strukt.methods {
                        self.collect_function_names(&mut names, method.def_id, &method.func);
                    }
                }
                HirItemKind::Enum(enum_item) => {
                    names.insert(item.def_id, enum_item.name);
                    for variant in &enum_item.variants {
                        names.insert(variant.def_id, variant.name);
                    }
                }
                HirItemKind::Interface(interface) => {
                    names.insert(item.def_id, interface.name);
                }
                HirItemKind::TypeAlias(alias) => {
                    names.insert(item.def_id, alias.name);
                }
                HirItemKind::Const(const_item) => {
                    names.insert(item.def_id, const_item.name);
                }
                HirItemKind::Import(import) => {
                    for item in &import.items {
                        let name = item.alias.unwrap_or(item.name);
                        names.insert(item.def_id, name);
                    }
                }
            }
        }

        if let Some(entry) = &hir.entry {
            self.collect_block_names(&mut names, Some(entry));
        }

        names
    }

    fn collect_function_names(&self, names: &mut FxHashMap<DefId, Symbol>, def_id: DefId, func: &HirFunction) {
        names.insert(def_id, func.name);
        for param in &func.params {
            names.insert(param.def_id, param.name);
        }
        self.collect_block_names(names, func.body.as_ref());
    }

    fn collect_block_names(&self, names: &mut FxHashMap<DefId, Symbol>, block: Option<&HirBlock>) {
        let Some(block) = block else {
            return;
        };
        for stmt in &block.stmts {
            match &stmt.kind {
                HirStmtKind::Local(local) => {
                    names.insert(local.def_id, local.name);
                    if let Some(init) = &local.init {
                        self.collect_expr_names(names, init);
                    }
                }
                HirStmtKind::Expr(expr) => self.collect_expr_names(names, expr),
                HirStmtKind::Redde(value) => {
                    if let Some(expr) = value {
                        self.collect_expr_names(names, expr);
                    }
                }
                HirStmtKind::Rumpe | HirStmtKind::Perge => {}
            }
        }
        if let Some(expr) = &block.expr {
            self.collect_expr_names(names, expr);
        }
    }

    fn collect_expr_names(&self, names: &mut FxHashMap<DefId, Symbol>, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.collect_expr_names(names, lhs);
                self.collect_expr_names(names, rhs);
            }
            HirExprKind::Unary(_, operand)
            | HirExprKind::Cede(operand)
            | HirExprKind::Qua(operand, _)
            | HirExprKind::Ref(_, operand)
            | HirExprKind::Deref(operand) => self.collect_expr_names(names, operand),
            HirExprKind::Innatum { source, map_entries, .. } => {
                self.collect_expr_names(names, source);
                if let Some(entries) = map_entries {
                    for (_, value) in entries {
                        self.collect_expr_names(names, value);
                    }
                }
            }
            HirExprKind::Call(callee, args) => {
                self.collect_expr_names(names, callee);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.collect_expr_names(names, receiver);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Field(object, _) | HirExprKind::Index(object, _) => {
                self.collect_expr_names(names, object);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.collect_expr_names(names, object);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(_) => {}
                    crate::hir::HirOptionalChainKind::Index(index) => self.collect_expr_names(names, index),
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_names(names, arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.collect_expr_names(names, source);
                if let Some(filter) = filter {
                    if let crate::hir::HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.collect_expr_names(names, cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.collect_expr_names(names, arg);
                    }
                }
            }
            HirExprKind::Block(block) => self.collect_block_names(names, Some(block)),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.collect_expr_names(names, cond);
                self.collect_block_names(names, Some(then_block));
                self.collect_block_names(names, else_block.as_ref());
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.collect_expr_names(names, scrutinee);
                for arm in arms {
                    self.collect_pattern_names(names, &arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_names(names, guard);
                    }
                    self.collect_expr_names(names, &arm.body);
                }
            }
            HirExprKind::Loop(block) | HirExprKind::Dum(_, block) => {
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Itera(_, _, iter, block) => {
                self.collect_expr_names(names, iter);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Array(elements) | HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.collect_expr_names(names, element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.collect_expr_names(names, cond);
                if let Some(message) = message {
                    self.collect_expr_names(names, message);
                }
            }
            HirExprKind::Panic(value) | HirExprKind::Throw(value) => self.collect_expr_names(names, value),
            HirExprKind::Tempta { body, catch, finally } => {
                self.collect_block_names(names, Some(body));
                self.collect_block_names(names, catch.as_ref());
                self.collect_block_names(names, finally.as_ref());
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.collect_expr_names(names, value);
                }
            }
            HirExprKind::Clausura(params, _, body) => {
                for param in params {
                    names.insert(param.def_id, param.name);
                }
                self.collect_expr_names(names, body);
            }
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn write_si_chain(
        &self,
        cond: &HirExpr,
        then_block: &HirBlock,
        else_block: Option<&HirBlock>,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        w.write("si ");
        self.write_expr(cond, types, names, interner, w);
        self.write_si_branch_body(then_block, types, names, interner, w);

        let mut next_else = else_block;
        while let Some(block) = next_else {
            if let Some((sin_cond, sin_then, sin_else)) = self.as_sin_branch(block) {
                w.write(" sin ");
                self.write_expr(sin_cond, types, names, interner, w);
                self.write_si_branch_body(sin_then, types, names, interner, w);
                next_else = sin_else;
            } else {
                w.write(" secus");
                self.write_si_branch_body(block, types, names, interner, w);
                break;
            }
        }
    }

    fn write_si_branch_body(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        if let Some(reddit_expr) = self.reddit_expr(block) {
            w.write(" reddit ");
            self.write_expr(reddit_expr, types, names, interner, w);
            return;
        }

        w.writeln(" {");
        w.indented(|w| self.write_block(block, types, names, interner, w));
        w.write("}");
    }

    fn as_sin_branch<'a>(&self, block: &'a HirBlock) -> Option<(&'a HirExpr, &'a HirBlock, Option<&'a HirBlock>)> {
        if !block.stmts.is_empty() {
            return None;
        }

        let expr = block.expr.as_ref()?;
        if let HirExprKind::Si(cond, then_block, else_block) = &expr.kind {
            Some((cond, then_block, else_block.as_ref()))
        } else {
            None
        }
    }

    fn reddit_expr<'a>(&self, block: &'a HirBlock) -> Option<&'a HirExpr> {
        if block.expr.is_some() || block.stmts.len() != 1 {
            return None;
        }
        match &block.stmts[0].kind {
            HirStmtKind::Redde(Some(expr)) => Some(expr),
            _ => None,
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_pattern_names(&self, names: &mut FxHashMap<DefId, Symbol>, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Binding(def_id, name) => {
                names.insert(*def_id, *name);
            }
            HirPattern::Variant(_, patterns) => {
                for pattern in patterns {
                    self.collect_pattern_names(names, pattern);
                }
            }
            HirPattern::Literal(_) => {}
        }
    }
}

impl Default for FaberCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen for FaberCodegen {
    type Output = FaberOutput;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<FaberOutput, CodegenError> {
        super::reject_hir_errors(hir)?;

        let mut w = CodeWriter::new();
        let names = self.collect_names(hir);

        for item in &hir.items {
            self.generate_item(item, types, &names, interner, &mut w)?;
            w.newline();
        }

        if let Some(entry) = &hir.entry {
            w.writeln("incipit {");
            w.indented(|w| self.write_block(entry, types, &names, interner, w));
            w.writeln("}");
        }

        Ok(FaberOutput { code: w.finish() })
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
