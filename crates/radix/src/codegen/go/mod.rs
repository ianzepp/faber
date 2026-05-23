//! Go backend orchestration for HIR-to-Go emission.
//!
//! This module owns the outer shape of generated Go files: backend-wide name
//! lookup, metadata catalogs that downstream expression/statement emitters need,
//! declaration ordering, package/import emission, and optional `main` entry
//! emission. It receives analyzed HIR and a [`TypeTable`]; semantic validation
//! remains an earlier compiler phase, while this backend either emits Go for the
//! HIR shape it understands or returns a codegen error for unsupported surfaces.
//!
//! CONTRACTS
//! =========
//! - Input is HIR after collection, resolution, lowering, typecheck, and
//!   analysis; this backend does not redo those checks.
//! - [`NameCatalog`] is the only name source used by the Go backend so
//!   declaration, expression, and statement emitters agree on symbol spelling.
//! - Backend metadata catalogs are collected once before emission. Statement
//!   paths that require complete variant binding metadata fail closed, while
//!   expression constructors stay deterministic when metadata is absent.
//! - Package and import output is synthesized after the body is generated so
//!   imports reflect the helper packages that actually appear in emitted code.
//!
//! TARGET COMPROMISES
//! ==================
//! - Enums map to interfaces + concrete struct variants (Go lacks sum types).
//! - Generics use Go 1.18+ type parameters with `any` constraints.
//! - Optional values map to pointers (`*T`) and `nil`.
//! - Faber borrow modes are not represented because Go has no equivalent
//!   ownership/lifetime surface.
//! - Direct Go codegen has a generated `ad` stub that returns an error, while
//!   normal driver diagnostics currently reject `ad` for Go targets before
//!   successful output is exposed.

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

/// Shared context for all Go backend emitters.
///
/// The struct deliberately keeps codegen-only metadata beside the canonical
/// name catalog. Expression and statement emission need access to declaration
/// facts that are awkward to recover locally in Go, such as variant field order
/// for type-switch bindings and `sponte` field status for pointer optional
/// initialization. These catalogs are derived from HIR only; they are not a
/// substitute for semantic analysis.
pub struct GoCodegen<'a> {
    /// Canonical spelling for symbols and definitions in generated Go.
    names: NameCatalog<'a>,

    /// Path-reference counts used to suppress Go's unused-variable errors.
    use_counts: FxHashMap<DefId, usize>,

    /// Variant definition id to source-order field names for constructors and
    /// `discerne` bindings.
    variant_fields: FxHashMap<DefId, Vec<Symbol>>,

    /// Struct definition id to declared field types, used by object literals
    /// and optional field access where Go needs concrete pointer shape.
    struct_fields: StructFieldTypes,

    /// Struct definition id to per-field `sponte` flags so emitters know which
    /// fields are represented as pointers in Go struct values.
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

    /// Return the source-order field catalog for a generated variant struct.
    ///
    /// Constructors and variant `discerne` arms depend on this order to map
    /// positional Faber variant bindings onto exported Go struct fields.
    pub(super) fn variant_fields(&self, def_id: DefId) -> Option<&[Symbol]> {
        self.variant_fields.get(&def_id).map(Vec::as_slice)
    }

    pub(super) fn is_variant_def(&self, def_id: DefId) -> bool {
        self.variant_fields.contains_key(&def_id)
    }

    pub(super) fn is_struct_def(&self, def_id: DefId) -> bool {
        self.struct_fields.contains_key(&def_id)
    }

    /// Return the declared type for a struct field when the emitter must choose
    /// between a plain value and a pointer optional.
    pub(super) fn struct_field_type(&self, def_id: DefId, field: Symbol) -> Option<crate::semantic::TypeId> {
        self.struct_fields
            .get(&def_id)
            .and_then(|fields| fields.get(&field).copied())
    }

    /// Report whether a struct field was declared `sponte`.
    ///
    /// Missing metadata defaults to `false` for compatibility with non-struct
    /// paths. Call sites that require the field's concrete type must query
    /// [`Self::struct_field_type`] separately.
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
    // Go imports are emitted from actual generated references, not from source
    // imports. That keeps helper imports deterministic while stdlib Faber
    // imports remain a declaration-layer concern until the backend grows real
    // package/module output.
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
