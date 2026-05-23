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
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{DefId, HirExpr, HirExprKind, HirItem, HirItemKind, HirProgram};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use crate::GoOutput;
use rustc_hash::FxHashMap;
use std::collections::BTreeSet;

type StructFieldTypes = FxHashMap<DefId, FxHashMap<Symbol, crate::semantic::TypeId>>;
type StructSponteFields = FxHashMap<DefId, FxHashMap<Symbol, bool>>;

pub struct GoCodegen<'a> {
    names: NameCatalog<'a>,
    use_counts: FxHashMap<DefId, usize>,
    variant_fields: FxHashMap<DefId, Vec<Symbol>>,
    struct_fields: StructFieldTypes,
    struct_sponte_fields: StructSponteFields,
}

impl<'a> GoCodegen<'a> {
    pub fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        let mut codegen = Self {
            names: NameCatalog::new(hir, interner),
            use_counts: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            struct_fields: FxHashMap::default(),
            struct_sponte_fields: FxHashMap::default(),
        };
        codegen.use_counts = codegen.collect_use_counts(hir);
        codegen.variant_fields = codegen.collect_variant_fields(hir);
        let (struct_fields, struct_sponte_fields) = codegen.collect_struct_fields(hir);
        codegen.struct_fields = struct_fields;
        codegen.struct_sponte_fields = struct_sponte_fields;
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

    pub(super) fn struct_field_is_sponte(&self, def_id: DefId, field: Symbol) -> bool {
        self.struct_sponte_fields
            .get(&def_id)
            .and_then(|fields| fields.get(&field).copied())
            .unwrap_or(false)
    }

    fn collect_use_counts(&self, hir: &HirProgram) -> FxHashMap<DefId, usize> {
        let mut collector = UseCounter { counts: FxHashMap::default() };
        collector.visit_program(hir);
        collector.counts
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

    fn collect_struct_fields(&self, hir: &HirProgram) -> (StructFieldTypes, StructSponteFields) {
        let mut fields = FxHashMap::default();
        let mut sponte_fields = FxHashMap::default();
        for item in &hir.items {
            if let HirItemKind::Struct(strukt) = &item.kind {
                let mut field_map = FxHashMap::default();
                let mut sponte_map = FxHashMap::default();
                for field in &strukt.fields {
                    field_map.insert(field.name, field.ty);
                    sponte_map.insert(field.name, field.sponte);
                }
                fields.insert(item.def_id, field_map);
                sponte_fields.insert(item.def_id, sponte_map);
            }
        }
        (fields, sponte_fields)
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
    let mut detector = AdDetector::default();
    detector.visit_program(hir);
    detector.found
}

struct UseCounter {
    counts: FxHashMap<DefId, usize>,
}

impl HirVisitor for UseCounter {
    fn visit_def(&mut self, def_id: DefId, _name: Symbol) {
        self.counts.entry(def_id).or_insert(0);
    }

    fn visit_expr(&mut self, expr: &HirExpr) {
        if let HirExprKind::Path(def_id) = &expr.kind {
            *self.counts.entry(*def_id).or_insert(0) += 1;
        }
        walk_expr(self, expr);
    }
}

#[derive(Default)]
struct AdDetector {
    found: bool,
}

impl HirVisitor for AdDetector {
    fn visit_ad(&mut self, _ad: &crate::hir::HirAd) {
        self.found = true;
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
