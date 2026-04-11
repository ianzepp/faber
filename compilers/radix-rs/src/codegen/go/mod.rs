//! Go Code Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This module implements Faber-to-Go transpilation. It transforms HIR into
//! idiomatic Go source code, mapping Faber's error semantics to Go's multi-return
//! (T, error) convention and Faber's structs/enums to Go structs and interfaces.
//!
//! COMPILER PHASE: Codegen
//! INPUT: HirProgram (fully-analyzed HIR), TypeTable, Interner
//! OUTPUT: GoOutput (compilable Go source code)
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Idiomatic Go: Generate code that a Go programmer would write.
//!   WHY: Output should integrate seamlessly with existing Go ecosystems.
//! - Multi-return errors: Faber's `iace` (throw) maps to Go's (T, error) returns.
//!   WHY: Go lacks exceptions; error values are the idiomatic pattern.
//! - No borrow analysis: Go is garbage-collected, so de/in/ex modes are no-ops.
//!   WHY: Go uses pointers for mutability, not Rust-style borrows.
//!
//! TRADE-OFFS
//! ==========
//! - Enums map to interfaces + concrete struct variants (Go lacks sum types).
//! - Generics use Go 1.18+ type parameters where possible.
//! - Optional types map to pointers (*T).

mod decl;
mod expr;
mod stmt;
mod types;

use super::{names::NameCatalog, CodeWriter, Codegen, CodegenError};
use crate::hir::{
    DefId, HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirItem, HirItemKind, HirOptionalChainKind,
    HirPattern, HirProgram, HirStmtKind,
};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use crate::GoOutput;
use rustc_hash::FxHashMap;
use std::collections::BTreeSet;

pub struct GoCodegen<'a> {
    names: NameCatalog<'a>,
    use_counts: FxHashMap<DefId, usize>,
    variant_fields: FxHashMap<DefId, Vec<Symbol>>,
    struct_fields: FxHashMap<DefId, FxHashMap<Symbol, crate::semantic::TypeId>>,
}

impl<'a> GoCodegen<'a> {
    pub fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        let mut codegen = Self {
            names: NameCatalog::new(hir, interner),
            use_counts: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            struct_fields: FxHashMap::default(),
        };
        codegen.use_counts = codegen.collect_use_counts(hir);
        codegen.variant_fields = codegen.collect_variant_fields(hir);
        codegen.struct_fields = codegen.collect_struct_fields(hir);
        codegen
    }

    pub(super) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.names.resolve_symbol(sym)
    }

    pub(super) fn resolve_def(&self, def_id: DefId) -> &str {
        self.names.resolve_def(def_id)
    }

    pub(super) fn is_used(&self, def_id: DefId) -> bool {
        self.use_counts.get(&def_id).copied().unwrap_or(0) > 0
    }

    pub(super) fn variant_fields(&self, def_id: DefId) -> Option<&[Symbol]> {
        self.variant_fields.get(&def_id).map(Vec::as_slice)
    }

    pub(super) fn is_variant_def(&self, def_id: DefId) -> bool {
        self.variant_fields.contains_key(&def_id)
    }

    pub(super) fn is_struct_def(&self, def_id: DefId) -> bool {
        self.struct_fields.contains_key(&def_id)
    }

    pub(super) fn struct_field_type(&self, def_id: DefId, field: Symbol) -> Option<crate::semantic::TypeId> {
        self.struct_fields
            .get(&def_id)
            .and_then(|fields| fields.get(&field).copied())
    }

    fn collect_use_counts(&self, hir: &HirProgram) -> FxHashMap<DefId, usize> {
        let mut counts = FxHashMap::default();
        for item in &hir.items {
            self.collect_item_use_counts(&mut counts, item);
        }
        if let Some(entry) = &hir.entry {
            self.collect_block_use_counts(&mut counts, entry);
        }
        counts
    }

    fn collect_variant_fields(&self, hir: &HirProgram) -> FxHashMap<DefId, Vec<Symbol>> {
        let mut fields = FxHashMap::default();
        for item in &hir.items {
            if let HirItemKind::Enum(enum_item) = &item.kind {
                for variant in &enum_item.variants {
                    fields.insert(variant.def_id, variant.fields.iter().map(|field| field.name).collect());
                }
            }
        }
        fields
    }

    fn collect_struct_fields(&self, hir: &HirProgram) -> FxHashMap<DefId, FxHashMap<Symbol, crate::semantic::TypeId>> {
        let mut fields = FxHashMap::default();
        for item in &hir.items {
            if let HirItemKind::Struct(strukt) = &item.kind {
                let mut field_map = FxHashMap::default();
                for field in &strukt.fields {
                    field_map.insert(field.name, field.ty);
                }
                fields.insert(item.def_id, field_map);
            }
        }
        fields
    }

    fn collect_item_use_counts(&self, counts: &mut FxHashMap<DefId, usize>, item: &HirItem) {
        match &item.kind {
            HirItemKind::Function(func) => {
                if let Some(body) = &func.body {
                    self.collect_block_use_counts(counts, body);
                }
            }
            HirItemKind::Struct(strukt) => {
                for field in &strukt.fields {
                    if let Some(init) = &field.init {
                        self.collect_expr_use_counts(counts, init);
                    }
                }
                for method in &strukt.methods {
                    if let Some(body) = &method.func.body {
                        self.collect_block_use_counts(counts, body);
                    }
                }
            }
            HirItemKind::Const(constant) => self.collect_expr_use_counts(counts, &constant.value),
            HirItemKind::Enum(_) | HirItemKind::Interface(_) | HirItemKind::TypeAlias(_) | HirItemKind::Import(_) => {}
        }
    }

    fn collect_block_use_counts(&self, counts: &mut FxHashMap<DefId, usize>, block: &HirBlock) {
        for stmt in &block.stmts {
            match &stmt.kind {
                HirStmtKind::Local(local) => {
                    if let Some(init) = &local.init {
                        self.collect_expr_use_counts(counts, init);
                    }
                }
                HirStmtKind::Ad(ad) => {
                    for arg in &ad.args {
                        self.collect_expr_use_counts(counts, arg);
                    }
                    if let Some(body) = &ad.body {
                        self.collect_block_use_counts(counts, body);
                    }
                    if let Some(catch) = &ad.catch {
                        self.collect_block_use_counts(counts, catch);
                    }
                }
                HirStmtKind::Expr(expr) => self.collect_expr_use_counts(counts, expr),
                HirStmtKind::Redde(expr) => {
                    if let Some(expr) = expr {
                        self.collect_expr_use_counts(counts, expr);
                    }
                }
                HirStmtKind::Rumpe | HirStmtKind::Perge => {}
            }
        }
        if let Some(expr) = &block.expr {
            self.collect_expr_use_counts(counts, expr);
        }
    }

    fn collect_expr_use_counts(&self, counts: &mut FxHashMap<DefId, usize>, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                *counts.entry(*def_id).or_insert(0) += 1;
            }
            HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.collect_expr_use_counts(counts, lhs);
                self.collect_expr_use_counts(counts, rhs);
            }
            HirExprKind::Unary(_, operand)
            | HirExprKind::Cede(operand)
            | HirExprKind::Ref(_, operand)
            | HirExprKind::Deref(operand)
            | HirExprKind::Panic(operand)
            | HirExprKind::Throw(operand) => self.collect_expr_use_counts(counts, operand),
            HirExprKind::Verte { source, entries, .. } => {
                self.collect_expr_use_counts(counts, source);
                if let Some(entries) = entries {
                    for field in entries {
                        match &field.key {
                            crate::hir::HirObjectKey::Computed(expr) | crate::hir::HirObjectKey::Spread(expr) => {
                                self.collect_expr_use_counts(counts, expr)
                            }
                            crate::hir::HirObjectKey::Ident(_) | crate::hir::HirObjectKey::String(_) => {}
                        }
                        if let Some(value) = &field.value {
                            self.collect_expr_use_counts(counts, value);
                        }
                    }
                }
            }
            HirExprKind::Conversio { source, fallback, .. } => {
                self.collect_expr_use_counts(counts, source);
                if let Some(fallback) = fallback {
                    self.collect_expr_use_counts(counts, fallback);
                }
            }
            HirExprKind::Call(callee, args) => {
                self.collect_expr_use_counts(counts, callee);
                for arg in args {
                    self.collect_expr_use_counts(counts, arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.collect_expr_use_counts(counts, receiver);
                for arg in args {
                    self.collect_expr_use_counts(counts, arg);
                }
            }
            HirExprKind::Field(object, _) => self.collect_expr_use_counts(counts, object),
            HirExprKind::Index(object, index) => {
                self.collect_expr_use_counts(counts, object);
                self.collect_expr_use_counts(counts, index);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.collect_expr_use_counts(counts, object);
                match chain {
                    HirOptionalChainKind::Member(_) => {}
                    HirOptionalChainKind::Index(index) => self.collect_expr_use_counts(counts, index),
                    HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_use_counts(counts, arg);
                        }
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.collect_expr_use_counts(counts, object);
                match chain {
                    crate::hir::HirNonNullKind::Member(_) => {}
                    crate::hir::HirNonNullKind::Index(index) => self.collect_expr_use_counts(counts, index),
                    crate::hir::HirNonNullKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_use_counts(counts, arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.collect_expr_use_counts(counts, source);
                if let Some(filter) = filter {
                    if let HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.collect_expr_use_counts(counts, cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.collect_expr_use_counts(counts, arg);
                    }
                }
            }
            HirExprKind::Block(block) | HirExprKind::Loop(block) => self.collect_block_use_counts(counts, block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.collect_expr_use_counts(counts, cond);
                self.collect_block_use_counts(counts, then_block);
                if let Some(else_block) = else_block {
                    self.collect_block_use_counts(counts, else_block);
                }
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                for scrutinee in scrutinees {
                    self.collect_expr_use_counts(counts, scrutinee);
                }
                for arm in arms {
                    for pattern in &arm.patterns {
                        self.collect_pattern_use_counts(counts, pattern);
                    }
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_use_counts(counts, guard);
                    }
                    self.collect_expr_use_counts(counts, &arm.body);
                }
            }
            HirExprKind::Dum(cond, block) => {
                self.collect_expr_use_counts(counts, cond);
                self.collect_block_use_counts(counts, block);
            }
            HirExprKind::Itera(_, _, _, iter, block) => {
                self.collect_expr_use_counts(counts, iter);
                self.collect_block_use_counts(counts, block);
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                self.collect_expr_use_counts(counts, start);
                self.collect_expr_use_counts(counts, end);
                if let Some(step) = step {
                    self.collect_expr_use_counts(counts, step);
                }
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    match element {
                        crate::hir::HirArrayElement::Expr(expr) | crate::hir::HirArrayElement::Spread(expr) => {
                            self.collect_expr_use_counts(counts, expr)
                        }
                    }
                }
            }
            HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.collect_expr_use_counts(counts, element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.collect_expr_use_counts(counts, arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.collect_expr_use_counts(counts, cond);
                if let Some(message) = message {
                    self.collect_expr_use_counts(counts, message);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.collect_expr_use_counts(counts, value);
                }
            }
            HirExprKind::Tempta { body, catch, finally } => {
                self.collect_block_use_counts(counts, body);
                if let Some(catch) = catch {
                    self.collect_block_use_counts(counts, catch);
                }
                if let Some(finally) = finally {
                    self.collect_block_use_counts(counts, finally);
                }
            }
            HirExprKind::Clausura(params, _, body) => {
                for param in params {
                    counts.entry(param.def_id).or_insert(0);
                }
                self.collect_expr_use_counts(counts, body);
            }
            HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    fn collect_pattern_use_counts(&self, counts: &mut FxHashMap<DefId, usize>, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard | HirPattern::Literal(_) => {}
            HirPattern::Binding(def_id, _) => {
                counts.entry(*def_id).or_insert(0);
            }
            HirPattern::Alias(def_id, _, pattern) => {
                counts.entry(*def_id).or_insert(0);
                self.collect_pattern_use_counts(counts, pattern);
            }
            HirPattern::Variant(_, patterns) => {
                for pattern in patterns {
                    self.collect_pattern_use_counts(counts, pattern);
                }
            }
        }
    }

    fn generate_item(&self, item: &HirItem, types: &TypeTable, w: &mut CodeWriter) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => decl::generate_function(self, func, types, w)?,
            HirItemKind::Struct(strukt) => decl::generate_struct(self, strukt, types, w)?,
            HirItemKind::Enum(enum_item) => decl::generate_enum(self, enum_item, types, w)?,
            HirItemKind::Interface(interface) => decl::generate_interface(self, interface, types, w)?,
            HirItemKind::TypeAlias(alias) => decl::generate_type_alias(self, alias, types, w)?,
            HirItemKind::Const(constant) => decl::generate_const(self, constant, types, w)?,
            HirItemKind::Import(import) => decl::generate_import(self, import, w)?,
        }
        Ok(())
    }
}

impl Codegen for GoCodegen<'_> {
    type Output = GoOutput;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, _interner: &Interner) -> Result<GoOutput, CodegenError> {
        let mut body = CodeWriter::new();

        for item in &hir.items {
            self.generate_item(item, types, &mut body)?;
            body.newline();
        }

        if let Some(entry) = &hir.entry {
            if program_contains_ad(hir) {
                body.writeln("func radixAd[T any](endpoint string, args ...any) (T, error) {");
                body.indented(|w| {
                    w.writeln("_ = args");
                    w.writeln("var zero T");
                    w.writeln(
                        r#"return zero, fmt.Errorf("ad dispatch is not implemented for Go codegen: %s", endpoint)"#,
                    );
                });
                body.writeln("}");
                body.newline();
            }

            body.writeln("func main() {");
            let mut block_result = Ok(());
            body.indented(|w| {
                block_result = stmt::generate_block_stmts(self, entry, types, w);
            });
            block_result?;
            body.writeln("}");
        }
        let body = body.finish();
        let imports = collect_imports(&body);
        let mut w = CodeWriter::new();
        w.writeln("// Generated by radix - do not edit");
        w.newline();
        w.writeln("package main");
        if !imports.is_empty() {
            w.newline();
            if imports.len() == 1 {
                w.write("import ");
                w.writeln(&format!("{:?}", imports.iter().next().expect("single import")));
            } else {
                w.writeln("import (");
                w.indented(|w| {
                    for import in &imports {
                        w.writeln(&format!("{:?}", import));
                    }
                });
                w.writeln(")");
            }
        }
        w.newline();
        w.write(&body);

        Ok(GoOutput { code: w.finish() })
    }
}

fn collect_imports(code: &str) -> BTreeSet<&'static str> {
    let mut imports = BTreeSet::new();
    if code.contains("fmt.") {
        imports.insert("fmt");
    }
    if code.contains("strconv.") {
        imports.insert("strconv");
    }
    if code.contains("regexp.") {
        imports.insert("regexp");
    }
    if code.contains("os.") {
        imports.insert("os");
    }
    if code.contains("sort.") {
        imports.insert("sort");
    }
    imports
}

fn program_contains_ad(hir: &HirProgram) -> bool {
    hir.items.iter().any(item_contains_ad) || hir.entry.as_ref().is_some_and(block_contains_ad)
}

fn item_contains_ad(item: &HirItem) -> bool {
    match &item.kind {
        HirItemKind::Function(func) => func.body.as_ref().is_some_and(block_contains_ad),
        HirItemKind::Struct(strukt) => strukt
            .methods
            .iter()
            .any(|method| method.func.body.as_ref().is_some_and(block_contains_ad)),
        _ => false,
    }
}

fn block_contains_ad(block: &HirBlock) -> bool {
    block.stmts.iter().any(stmt_contains_ad)
        || block
            .expr
            .as_ref()
            .is_some_and(|expr| expr_contains_ad(expr))
}

fn stmt_contains_ad(stmt: &crate::hir::HirStmt) -> bool {
    match &stmt.kind {
        HirStmtKind::Ad(_) => true,
        HirStmtKind::Local(local) => local.init.as_ref().is_some_and(expr_contains_ad),
        HirStmtKind::Expr(expr) => expr_contains_ad(expr),
        HirStmtKind::Redde(expr) => expr.as_ref().is_some_and(expr_contains_ad),
        HirStmtKind::Rumpe | HirStmtKind::Perge => false,
    }
}

fn expr_contains_ad(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Block(block) | HirExprKind::Loop(block) => block_contains_ad(block),
        HirExprKind::Si(cond, then_block, else_block) => {
            expr_contains_ad(cond)
                || block_contains_ad(then_block)
                || else_block.as_ref().is_some_and(block_contains_ad)
        }
        HirExprKind::Dum(cond, block) => expr_contains_ad(cond) || block_contains_ad(block),
        HirExprKind::Tempta { body, catch, finally } => {
            block_contains_ad(body)
                || catch.as_ref().is_some_and(block_contains_ad)
                || finally.as_ref().is_some_and(block_contains_ad)
        }
        HirExprKind::Itera(_, _, _, iter, block) => expr_contains_ad(iter) || block_contains_ad(block),
        HirExprKind::Intervallum { start, end, step, .. } => {
            expr_contains_ad(start) || expr_contains_ad(end) || step.as_ref().is_some_and(|step| expr_contains_ad(step))
        }
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            expr_contains_ad(lhs) || expr_contains_ad(rhs)
        }
        HirExprKind::Adfirma(condition, message) => {
            expr_contains_ad(condition)
                || message
                    .as_ref()
                    .is_some_and(|message| expr_contains_ad(message))
        }
        HirExprKind::Unary(_, inner)
        | HirExprKind::Field(inner, _)
        | HirExprKind::Panic(inner)
        | HirExprKind::Throw(inner)
        | HirExprKind::Cede(inner)
        | HirExprKind::Ref(_, inner)
        | HirExprKind::Deref(inner) => expr_contains_ad(inner),
        HirExprKind::Call(callee, args) | HirExprKind::MethodCall(callee, _, args) => {
            expr_contains_ad(callee) || args.iter().any(expr_contains_ad)
        }
        HirExprKind::Index(object, index) => expr_contains_ad(object) || expr_contains_ad(index),
        HirExprKind::OptionalChain(object, chain) => {
            expr_contains_ad(object)
                || match chain {
                    HirOptionalChainKind::Member(_) => false,
                    HirOptionalChainKind::Index(index) => expr_contains_ad(index),
                    HirOptionalChainKind::Call(args) => args.iter().any(expr_contains_ad),
                }
        }
        HirExprKind::NonNull(object, chain) => {
            expr_contains_ad(object)
                || match chain {
                    crate::hir::HirNonNullKind::Member(_) => false,
                    crate::hir::HirNonNullKind::Index(index) => expr_contains_ad(index),
                    crate::hir::HirNonNullKind::Call(args) => args.iter().any(expr_contains_ad),
                }
        }
        HirExprKind::Array(elements) => elements.iter().any(|element| match element {
            crate::hir::HirArrayElement::Expr(expr) | crate::hir::HirArrayElement::Spread(expr) => {
                expr_contains_ad(expr)
            }
        }),
        HirExprKind::Struct(_, fields) => fields.iter().any(|(_, expr)| expr_contains_ad(expr)),
        HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => elements.iter().any(expr_contains_ad),
        HirExprKind::Scriptum(_, args) => args.iter().any(expr_contains_ad),
        HirExprKind::Clausura(_, _, body) => expr_contains_ad(body),
        HirExprKind::Verte { source, entries, .. } => {
            expr_contains_ad(source)
                || entries.as_ref().is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|field| field.value.as_ref().is_some_and(expr_contains_ad))
                })
        }
        HirExprKind::Conversio { source, fallback, .. } => {
            expr_contains_ad(source)
                || fallback
                    .as_ref()
                    .is_some_and(|fallback| expr_contains_ad(fallback))
        }
        HirExprKind::Ab { source, filter, transforms } => {
            expr_contains_ad(source)
                || filter.as_ref().is_some_and(|filter| match &filter.kind {
                    HirCollectionFilterKind::Condition(expr) => expr_contains_ad(expr),
                    HirCollectionFilterKind::Property(_) => false,
                })
                || transforms.iter().any(|transform| {
                    transform
                        .arg
                        .as_ref()
                        .is_some_and(|arg| expr_contains_ad(arg))
                })
        }
        HirExprKind::Discerne(scrutinees, arms) => {
            scrutinees.iter().any(expr_contains_ad)
                || arms
                    .iter()
                    .any(|arm| arm.guard.as_ref().is_some_and(expr_contains_ad) || expr_contains_ad(&arm.body))
        }
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => false,
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
