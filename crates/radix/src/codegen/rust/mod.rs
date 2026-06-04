//! Rust backend orchestration for `radix` code generation.
//!
//! This module is the target boundary between analyzed Faber HIR and generated
//! Rust source. It owns Rust backend construction, target dispatch entrypoints,
//! generated-file framing, import collection, `incipit` entry emission, CLI
//! attachment, and backend-wide state that declaration/expression/statement
//! emitters need while walking HIR.
//!
//! The backend does not redo semantic checking. It assumes earlier compiler
//! phases have produced HIR, types, resolved definition ids, and diagnostics;
//! this module validates that no recorded HIR errors cross public module
//! generation boundaries and then translates the already-analyzed program into
//! a [`RustOutput`] string.
//!
//! INVARIANTS
//! ==========
//! - `NameCatalog` and failable-function state are precomputed before emission
//!   so every sub-emitter sees one stable name and error-propagation policy.
//! - Rust fallibility is represented as `Result<T, String>` for generated
//!   functions that can throw or call throwing code.
//! - `incipit` emits `fn main()` and is an output boundary: it executes entry
//!   statements but does not itself become a failable function.
//! - Imports are collected after body emission from explicit HIR imports plus
//!   generated Rust surface requirements, then written before the body.
//! - `module_mode` emits embeddable module code; normal mode emits a generated
//!   file prelude with crate-level allowances.
//!
//! TRADE-OFFS
//! ==========
//! `Result<T, String>` keeps throw lowering simple and target-local while the
//! language error model is still untyped. Ownership and reference mapping are
//! also Rust backend policy: Faber reference modes lower to Rust borrow/ownership
//! forms in `types`, `decl`, and `expr` rather than being encoded as a
//! target-neutral runtime abstraction. Import collection deliberately avoids a
//! second mutable side channel through every emitter; the cost is that prelude
//! imports depend on the generated Rust surface, not only the HIR type graph.

mod cli;
mod decl;
mod expr;
mod failable;
mod prelude;
mod stmt;
mod type_shape;
mod types;

use super::{names::NameCatalog, CodeWriter, Codegen, CodegenError};
use crate::hir::{
    DefId, HirExpr, HirFunction, HirItem, HirItemKind, HirProgram, HirTestMetadata, HirTestModifier, LibraryRegistry,
};
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Type, TypeId, TypeTable};
use crate::RustOutput;
use prelude::{
    block_contains_await, collect_hir_imports, collect_prelude_imports, generate_block_on_helper, generate_prelude,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::{Cell, RefCell};

/// User-facing test filter requested by the package or CLI layer.
///
/// Selection is captured before Rust emission so ignored tests can be rendered
/// with deterministic Rust `#[ignore = "..."]` reasons instead of leaving test
/// filtering to Cargo runtime behavior.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestSelection {
    /// Exact test name to keep, if one was requested.
    pub name: Option<String>,

    /// Slash-joined suite path to keep, if one was requested.
    pub suite: Option<String>,

    /// Metadata tag to keep, if one was requested.
    pub tag: Option<String>,
}

#[derive(Debug, Clone)]
struct TestSelectionState {
    query: TestSelection,
    has_solum_tests: bool,
}

#[derive(Clone)]
pub(super) struct VariantInfo {
    pub(super) enum_def: DefId,
    pub(super) fields: Vec<Symbol>,
}

#[derive(Clone, Copy)]
pub(super) struct StructFieldInfo<'a> {
    pub(super) name: Symbol,
    pub(super) ty: TypeId,
    sponte: bool,
    pub(super) init: Option<&'a HirExpr>,
}

#[derive(Clone, Copy)]
pub(super) struct FunctionParamInfo<'a> {
    pub(super) def_id: DefId,
    pub(super) ty: TypeId,
    pub(super) optional: bool,
    pub(super) default: Option<&'a HirExpr>,
}

#[derive(Clone, Copy)]
struct RuntimeInterfaceInfo {
    rust_type: Option<&'static str>,
    elide_decl: bool,
}

/// Rust backend state shared by all Rust sub-emitters.
///
/// Construction precomputes the name catalog, failable-function closure, test
/// selection state, and struct-field metadata before any Rust text is emitted.
/// That makes emission mostly a deterministic HIR walk instead of a set of
/// local guesses about names, `Result` wrapping, or omitted Faber fields.
pub struct RustCodegen<'a> {
    /// Canonical names for symbols and definitions visible to generated Rust.
    names: NameCatalog<'a>,

    /// Functions that lower to `Result<_, String>`.
    ///
    /// This is a transitive codegen policy computed from HIR calls and throws;
    /// it is not a second semantic validation pass.
    failable_defs: FxHashSet<DefId>,

    /// Optional Faber test selection used to emit ignore reasons.
    test_selection: Option<TestSelectionState>,

    /// DefId -> field metadata for struct construction.
    ///
    /// Enables Rust struct literals to emit complete initializers for omitted `sponte` fields
    /// and fields with genus defaults, while preserving nullable storage as `Option<T>`.
    struct_fields: FxHashMap<DefId, Vec<StructFieldInfo<'a>>>,

    /// Struct definitions that declare a `creo` post-construction hook.
    struct_creo_hooks: FxHashSet<DefId>,

    /// Current function return type while emitting a body.
    ///
    /// Statement lowering uses this to bridge nullable Faber returns to Rust
    /// `Option<T>` without re-inferring expression types in codegen.
    current_return_ty: Cell<Option<TypeId>>,

    /// Current generator yield type while emitting a cursor function body.
    current_generator_yield_ty: Cell<Option<TypeId>>,

    /// Current struct receiver while emitting an inherent method body.
    ///
    /// HIR lowers `ego` to the enclosing struct definition path. Rust method
    /// emission remaps that path to `self` only while this context is active.
    current_self_def: Cell<Option<DefId>>,

    /// DefId -> declared or derived binding type.
    ///
    /// Expression annotations can be widened by contextual checks. For path
    /// emission decisions such as `return Some(local)`, the original binding
    /// storage type is the target contract we need.
    binding_types: RefCell<FxHashMap<DefId, TypeId>>,

    /// DefId -> function parameter metadata for direct-call argument recovery.
    function_params: FxHashMap<DefId, Vec<FunctionParamInfo<'a>>>,

    /// Parameter definitions whose Rust storage type is `Option<T>`.
    option_param_defs: FxHashSet<DefId>,

    /// Variant DefId -> parent enum and payload field names.
    ///
    /// Rust requires enum variants to be qualified at expression and pattern
    /// sites. HIR stores the resolved variant identity, so codegen keeps the
    /// parent relationship here instead of asking expression emitters to infer
    /// it from names.
    variant_info: FxHashMap<DefId, VariantInfo>,

    /// Runtime-owned stdlib interfaces keyed by their HIR definition.
    ///
    /// The HTTP HAL source pacta are contracts for the Norma runtime, not local
    /// Rust traits. Codegen records exact-shape matches here so type rendering
    /// can name concrete runtime structs without guessing from unresolved names.
    runtime_interfaces: FxHashMap<DefId, RuntimeInterfaceInfo>,

    /// Target-neutral library provenance supplied by package analysis.
    libraries: LibraryRegistry,
}

impl<'a> RustCodegen<'a> {
    pub fn new(hir: &'a HirProgram, interner: &'a Interner) -> Self {
        Self::new_with_test_selection(hir, interner, None)
    }

    pub fn new_with_test_selection(
        hir: &'a HirProgram,
        interner: &'a Interner,
        test_selection: Option<TestSelection>,
    ) -> Self {
        Self::new_with_library_registry_and_test_selection(hir, interner, &LibraryRegistry::default(), test_selection)
    }

    pub fn new_with_library_registry_and_test_selection(
        hir: &'a HirProgram,
        interner: &'a Interner,
        libraries: &LibraryRegistry,
        test_selection: Option<TestSelection>,
    ) -> Self {
        let mut codegen = Self {
            names: NameCatalog::new(hir, interner),
            failable_defs: FxHashSet::default(),
            test_selection: None,
            struct_fields: FxHashMap::default(),
            struct_creo_hooks: FxHashSet::default(),
            current_return_ty: Cell::new(None),
            current_generator_yield_ty: Cell::new(None),
            current_self_def: Cell::new(None),
            binding_types: RefCell::new(FxHashMap::default()),
            function_params: FxHashMap::default(),
            option_param_defs: FxHashSet::default(),
            variant_info: FxHashMap::default(),
            runtime_interfaces: FxHashMap::default(),
            libraries: libraries.clone(),
        };
        codegen.failable_defs = codegen.collect_failable_functions(hir);
        codegen.test_selection = Some(TestSelectionState {
            query: test_selection.unwrap_or_default(),
            has_solum_tests: hir
                .items
                .iter()
                .any(|item| matches!(&item.kind, HirItemKind::Function(func) if func.test.as_ref().is_some_and(|test| test.modifiers.iter().any(|modifier| matches!(modifier, HirTestModifier::Solum))))),
        });
        codegen.struct_fields = codegen.collect_struct_fields(hir);
        codegen.struct_creo_hooks = codegen.collect_struct_creo_hooks(hir);
        codegen.function_params = codegen.collect_function_params(hir);
        codegen.option_param_defs = codegen.collect_option_param_defs();
        codegen.variant_info = codegen.collect_variant_info(hir);
        codegen.runtime_interfaces = codegen.collect_runtime_interfaces(hir);
        codegen
    }

    pub(super) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.names.resolve_symbol(sym)
    }

    pub(super) fn resolve_def(&self, def_id: DefId) -> &str {
        self.names.resolve_def(def_id)
    }

    pub(super) fn variant_info(&self, def_id: DefId) -> Option<&VariantInfo> {
        self.variant_info.get(&def_id)
    }

    pub(super) fn runtime_interface_type(&self, def_id: DefId) -> Option<&'static str> {
        self.runtime_interfaces
            .get(&def_id)
            .and_then(|info| info.rust_type)
    }

    pub(super) fn should_elide_runtime_interface_decl(&self, def_id: DefId) -> bool {
        self.runtime_interfaces
            .get(&def_id)
            .is_some_and(|info| info.elide_decl)
    }

    pub(super) fn is_failable_def(&self, def_id: DefId) -> bool {
        self.failable_defs.contains(&def_id)
    }

    pub(super) fn is_failable_method_name(&self, method: Symbol) -> bool {
        self.names
            .iter()
            .any(|(def_id, name)| *name == method && self.failable_defs.contains(def_id))
    }

    pub(super) fn current_return_ty(&self) -> Option<TypeId> {
        self.current_return_ty.get()
    }

    pub(super) fn replace_current_return_ty(&self, ty: Option<TypeId>) -> Option<TypeId> {
        self.current_return_ty.replace(ty)
    }

    pub(super) fn current_generator_yield_ty(&self) -> Option<TypeId> {
        self.current_generator_yield_ty.get()
    }

    pub(super) fn replace_current_generator_yield_ty(&self, ty: Option<TypeId>) -> Option<TypeId> {
        self.current_generator_yield_ty.replace(ty)
    }

    pub(super) fn current_self_def(&self) -> Option<DefId> {
        self.current_self_def.get()
    }

    pub(super) fn replace_current_self_def(&self, def_id: Option<DefId>) -> Option<DefId> {
        self.current_self_def.replace(def_id)
    }

    pub(super) fn binding_type(&self, def_id: DefId) -> Option<TypeId> {
        self.binding_types.borrow().get(&def_id).copied()
    }

    pub(super) fn binding_type_by_generated_name(&self, def_id: DefId) -> Option<TypeId> {
        let name = self.resolve_def(def_id);
        self.binding_types
            .borrow()
            .iter()
            .find_map(|(binding_def, ty)| (self.resolve_def(*binding_def) == name).then_some(*ty))
    }

    pub(super) fn function_param_count(&self, def_id: DefId) -> Option<usize> {
        self.function_params.get(&def_id).map(Vec::len)
    }

    pub(super) fn function_params(&self, def_id: DefId) -> Option<&[FunctionParamInfo<'a>]> {
        self.function_params.get(&def_id).map(Vec::as_slice)
    }

    pub(super) fn binding_stores_option(&self, def_id: DefId) -> bool {
        self.option_param_defs.contains(&def_id)
    }

    pub(super) fn test_ignore_reason(&self, func: &HirFunction) -> Option<String> {
        let test = func.test.as_ref()?;
        let state = self.test_selection.as_ref()?;
        if let Some(reason) = self.selection_ignore_reason(test, &state.query, state.has_solum_tests) {
            return Some(reason);
        }
        self.source_ignore_reason(test)
    }

    fn selection_ignore_reason(
        &self,
        test: &HirTestMetadata,
        query: &TestSelection,
        has_solum_tests: bool,
    ) -> Option<String> {
        if has_solum_tests && !self.test_has_modifier(test, |modifier| matches!(modifier, HirTestModifier::Solum)) {
            return Some("faber: not selected by solum".to_owned());
        }

        if let Some(name) = query.name.as_deref() {
            if self.resolve_symbol(test.name) != name {
                return Some(format!("faber: not selected by name {}", name));
            }
        }

        if let Some(suite) = query.suite.as_deref() {
            if self.test_suite_path(test) != suite {
                return Some(format!("faber: not selected by suite {}", suite));
            }
        }

        if let Some(tag) = query.tag.as_deref() {
            if !self.test_has_tag(test, tag) {
                return Some(format!("faber: not selected by tag {}", tag));
            }
        }

        None
    }

    fn source_ignore_reason(&self, test: &HirTestMetadata) -> Option<String> {
        for modifier in &test.modifiers {
            match modifier {
                HirTestModifier::Omitte(reason) => {
                    return Some(format!("faber: omitte - {}", self.resolve_symbol(*reason)));
                }
                HirTestModifier::Futurum(reason) => {
                    return Some(format!("faber: futurum - {}", self.resolve_symbol(*reason)));
                }
                _ => {}
            }
        }
        None
    }

    fn test_suite_path(&self, test: &HirTestMetadata) -> String {
        test.suite_path
            .iter()
            .map(|sym| self.resolve_symbol(*sym))
            .collect::<Vec<_>>()
            .join("/")
    }

    fn test_has_tag(&self, test: &HirTestMetadata, tag: &str) -> bool {
        test.modifiers
            .iter()
            .any(|modifier| matches!(modifier, HirTestModifier::Tag(value) if self.resolve_symbol(*value) == tag))
    }

    fn test_has_modifier(&self, test: &HirTestMetadata, predicate: impl Fn(&HirTestModifier) -> bool) -> bool {
        test.modifiers.iter().any(predicate)
    }

    fn generate_item(
        &self,
        item: &HirItem,
        types: &TypeTable,
        writer: &mut CodeWriter,
        cli_program: Option<&crate::cli::CliProgram>,
        cli_args_type_prefix: &str,
    ) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => {
                if let Some(command) = cli_program.and_then(|program| {
                    program
                        .commands
                        .iter()
                        .find(|command| command.module_path.is_none() && command.function_symbol == func.name)
                }) {
                    let args_type = format!("{cli_args_type_prefix}{}", cli::command_args_struct_name(command));
                    decl::generate_function_with_cli_args_type(
                        self,
                        item.def_id,
                        func,
                        types,
                        writer,
                        Some(&args_type),
                    )?;
                } else {
                    decl::generate_function(self, item.def_id, func, types, writer)?;
                }
            }
            HirItemKind::Struct(s) => {
                decl::generate_struct(self, item.def_id, s, types, writer)?;
            }
            HirItemKind::Enum(e) => {
                decl::generate_enum(self, e, types, writer)?;
            }
            HirItemKind::Interface(i) => {
                if !self.should_elide_runtime_interface_decl(item.def_id) {
                    decl::generate_trait(self, i, types, writer)?;
                }
            }
            HirItemKind::TypeAlias(a) => {
                decl::generate_type_alias(self, a, types, writer)?;
            }
            HirItemKind::Const(c) => {
                decl::generate_const(self, c, types, writer)?;
            }
            HirItemKind::Import(_) => {
                // Handled in prelude or use statements
            }
        }
        Ok(())
    }

    /// Transitively compute which functions use the Rust failable signature.
    ///
    /// WHY: a function that calls a throwing function must be emitted with
    /// `Result<_, String>` even when it does not contain `iace` directly. The
    /// fixed-point analysis is precomputed once so declaration, statement, and
    /// expression emitters agree on `?`, `Ok(...)`, and return signatures.
    ///
    /// EDGE: `tempta` with a catch block suppresses propagation from its body.
    fn collect_failable_functions(&self, hir: &HirProgram) -> FxHashSet<DefId> {
        failable::collect_failable_functions(self, hir)
    }

    fn collect_runtime_interfaces(&self, hir: &HirProgram) -> FxHashMap<DefId, RuntimeInterfaceInfo> {
        let mut interfaces = FxHashMap::default();

        for item in &hir.items {
            let HirItemKind::Interface(interface) = &item.kind else {
                continue;
            };

            let _library_item = self.libraries.items.get(&item.def_id);
            let Some(info) = self.http_runtime_interface_info(interface) else {
                continue;
            };

            interfaces.insert(item.def_id, info);
        }

        interfaces
    }

    fn http_runtime_interface_info(&self, interface: &crate::hir::HirInterface) -> Option<RuntimeInterfaceInfo> {
        let name = self.resolve_symbol(interface.name);
        let method_names = interface
            .methods
            .iter()
            .map(|method| self.resolve_symbol(method.name))
            .collect::<Vec<_>>();

        match name {
            "http"
                if method_names_match(
                    &method_names,
                    &[
                        "petet",
                        "mittet",
                        "ponet",
                        "delet",
                        "mutabit",
                        "rogabit",
                        "exspectabit",
                        "replica",
                        "scribe",
                        "funde",
                        "json",
                        "redirige",
                    ],
                ) =>
            {
                Some(RuntimeInterfaceInfo { rust_type: None, elide_decl: true })
            }
            "Replicatio"
                if method_names_match(
                    &method_names,
                    &[
                        "status",
                        "corpus",
                        "corpus_octeti",
                        "corpus_json",
                        "capita",
                        "caput",
                        "bene",
                    ],
                ) =>
            {
                Some(RuntimeInterfaceInfo { rust_type: Some("norma::hal::http::Replicatio"), elide_decl: true })
            }
            "Rogatio"
                if method_names_match(
                    &method_names,
                    &["modus", "via", "corpus", "corpus_json", "capita", "caput", "param"],
                ) =>
            {
                Some(RuntimeInterfaceInfo { rust_type: None, elide_decl: true })
            }
            "Servitor" if method_names_match(&method_names, &["siste", "portus"]) => {
                Some(RuntimeInterfaceInfo { rust_type: None, elide_decl: true })
            }
            _ => None,
        }
    }

    /// Precompute struct field metadata needed by Rust declaration and literal emission.
    ///
    /// WHY: Rust struct literals require all fields. Faber fields with `sponte` or genus
    /// defaults may be omitted at construction, so codegen must fill them explicitly.
    /// `fixus` is recorded in HIR but produces no target-level immutability in this phase.
    fn collect_struct_fields(&self, hir: &'a HirProgram) -> FxHashMap<DefId, Vec<StructFieldInfo<'a>>> {
        let mut fields = FxHashMap::default();
        for item in &hir.items {
            if let HirItemKind::Struct(strukt) = &item.kind {
                fields.insert(
                    item.def_id,
                    strukt
                        .fields
                        .iter()
                        .filter(|field| !field.is_static)
                        .map(|field| StructFieldInfo {
                            name: field.name,
                            ty: field.ty,
                            sponte: field.sponte,
                            init: field.init.as_ref(),
                        })
                        .collect(),
                );
            }
        }
        fields
    }

    fn collect_struct_creo_hooks(&self, hir: &'a HirProgram) -> FxHashSet<DefId> {
        hir.items
            .iter()
            .filter_map(|item| {
                let HirItemKind::Struct(strukt) = &item.kind else {
                    return None;
                };
                strukt
                    .methods
                    .iter()
                    .any(|method| {
                        self.resolve_symbol(method.func.name) == "creo"
                            && method.func.params.is_empty()
                            && method.func.body.is_some()
                    })
                    .then_some(item.def_id)
            })
            .collect()
    }

    fn collect_function_params(&self, hir: &'a HirProgram) -> FxHashMap<DefId, Vec<FunctionParamInfo<'a>>> {
        let mut params = FxHashMap::default();
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    params.insert(item.def_id, function_param_info(func));
                }
                HirItemKind::Struct(strukt) => {
                    for method in &strukt.methods {
                        params.insert(method.def_id, function_param_info(&method.func));
                    }
                }
                _ => {}
            }
        }
        params
    }

    fn collect_option_param_defs(&self) -> FxHashSet<DefId> {
        self.function_params
            .values()
            .flat_map(|params| params.iter())
            .filter(|param| param.optional && param.default.is_none())
            .map(|param| param.def_id)
            .collect()
    }

    fn collect_variant_info(&self, hir: &HirProgram) -> FxHashMap<DefId, VariantInfo> {
        let mut info = FxHashMap::default();
        for item in &hir.items {
            let HirItemKind::Enum(enum_item) = &item.kind else {
                continue;
            };
            for variant in &enum_item.variants {
                info.insert(
                    variant.def_id,
                    VariantInfo {
                        enum_def: item.def_id,
                        fields: variant.fields.iter().map(|field| field.name).collect(),
                    },
                );
            }
        }
        info
    }

    fn collect_binding_types(&self, hir: &HirProgram, types: &TypeTable) -> FxHashMap<DefId, TypeId> {
        struct BindingTypeCollector<'a> {
            types: &'a TypeTable,
            bindings: FxHashMap<DefId, TypeId>,
        }

        impl<'a> crate::hir::visit::HirVisitor for BindingTypeCollector<'a> {
            fn visit_param(&mut self, param: &crate::hir::HirParam) {
                self.bindings.insert(param.def_id, param.ty);
            }

            fn visit_local(&mut self, local: &crate::hir::HirLocal) {
                if let Some(ty) = local.ty {
                    self.bindings.insert(local.def_id, ty);
                }
                crate::hir::visit::walk_local(self, local);
            }

            fn visit_expr(&mut self, expr: &HirExpr) {
                if let crate::hir::HirExprKind::Itera(mode, def_id, _, iter, _) = &expr.kind {
                    if let Some(binding_ty) = itera_binding_type(*mode, iter.ty, self.types) {
                        self.bindings.insert(*def_id, binding_ty);
                    }
                }
                crate::hir::visit::walk_expr(self, expr);
            }
        }

        let mut collector = BindingTypeCollector { types, bindings: FxHashMap::default() };
        crate::hir::visit::HirVisitor::visit_program(&mut collector, hir);
        collector.bindings
    }

    pub(super) fn struct_field_info(&self, def_id: DefId, field: Symbol) -> Option<StructFieldInfo<'a>> {
        self.struct_fields
            .get(&def_id)?
            .iter()
            .copied()
            .find(|info| info.name == field)
    }

    /// Returns true when the Rust storage type for this field is `Option<_>`.
    pub(super) fn struct_field_stores_option(&self, def_id: DefId, field: Symbol, types: &TypeTable) -> bool {
        self.struct_field_info(def_id, field)
            .is_some_and(|info| info.sponte || type_shape::type_id_is_option(info.ty, types))
    }

    pub(super) fn sorted_struct_omittable_fields(&self, def_id: DefId) -> Vec<StructFieldInfo<'a>> {
        let mut fields = self
            .struct_fields
            .get(&def_id)
            .map(|fields| {
                fields
                    .iter()
                    .copied()
                    .filter(|field| field.sponte || field.init.is_some())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        fields.sort_by(|a, b| self.resolve_symbol(a.name).cmp(self.resolve_symbol(b.name)));
        fields
    }

    pub(super) fn struct_has_creo_hook(&self, def_id: DefId) -> bool {
        self.struct_creo_hooks.contains(&def_id)
    }
}

fn method_names_match(actual: &[&str], expected: &[&str]) -> bool {
    actual == expected
}

fn function_param_info(func: &HirFunction) -> Vec<FunctionParamInfo<'_>> {
    func.params
        .iter()
        .map(|param| FunctionParamInfo {
            def_id: param.def_id,
            ty: param.ty,
            optional: param.optional,
            default: param.default.as_ref(),
        })
        .collect()
}

fn itera_binding_type(mode: crate::hir::HirIteraMode, iter_ty: Option<TypeId>, types: &TypeTable) -> Option<TypeId> {
    let iter_ty = iter_ty?;
    match (mode, type_shape::resolve_type(iter_ty, types)) {
        (crate::hir::HirIteraMode::Ex, Type::Array(inner)) => Some(inner),
        (crate::hir::HirIteraMode::De, Type::Array(_)) | (crate::hir::HirIteraMode::Range, _) => {
            Some(types.primitive(crate::semantic::Primitive::Numerus))
        }
        (crate::hir::HirIteraMode::De, Type::Map(key, _)) => Some(key),
        _ => None,
    }
}

impl Codegen for RustCodegen<'_> {
    type Output = RustOutput;

    fn generate(&self, hir: &HirProgram, types: &TypeTable, _interner: &Interner) -> Result<RustOutput, CodegenError> {
        self.generate_output(hir, types, false, None)
    }
}

impl RustCodegen<'_> {
    /// Generate a complete Rust source file with Rust-owned CLI support.
    ///
    /// CLI lowering is target-owned because the parser, process exits, help
    /// printing, and argument storage types are concrete Rust code rather than
    /// target-neutral HIR constructs.
    pub fn generate_cli(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
        cli_program: &crate::cli::CliProgram,
    ) -> Result<RustOutput, CodegenError> {
        self.generate_output(hir, types, false, Some(cli_program))
    }

    fn generate_output(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
        module_mode: bool,
        cli_program: Option<&crate::cli::CliProgram>,
    ) -> Result<RustOutput, CodegenError> {
        self.binding_types
            .replace(self.collect_binding_types(hir, types));
        let mut body = CodeWriter::new();

        // Item emission intentionally precedes the prelude. Imports include
        // both source-level HIR imports and Rust helper types discovered from
        // the generated body.
        for item in &hir.items {
            let cli_args_type_prefix = if module_mode { "crate::" } else { "" };
            self.generate_item(item, types, &mut body, cli_program, cli_args_type_prefix)?;
            body.newline();
        }

        if let Some(cli_program) = cli_program.filter(|_| !module_mode) {
            cli::generate_cli_support(cli_program, &mut body);
            body.newline();
        }

        // `incipit` is emitted as Rust `main`. It is a generated-output
        // boundary, so statements run in entry context instead of inheriting
        // the failable function model used for ordinary Faber functions.
        if let Some(entry) = &hir.entry {
            let entry_needs_block_on = block_contains_await(entry);
            if entry_needs_block_on {
                generate_block_on_helper(&mut body);
                body.newline();
            }
            body.writeln("fn main() {");
            let mut entry_result = Ok(());
            body.indented(|writer| {
                if entry_needs_block_on {
                    writer.writeln("__faber_block_on(async {");
                }
                writer.indented(|writer| {
                    if let Some(cli_program) = cli_program {
                        if cli_program.mode == crate::cli::CliMode::Subcommand {
                            cli::generate_command_dispatch(cli_program, writer);
                            return;
                        }
                    }
                    if let Some(cli_program) = cli_program {
                        writer.write("let ");
                        writer.write(&cli_program.entry_args);
                        writer.writeln(" = parse_cli_args_or_exit();");
                    }
                    for stmt in &entry.stmts {
                        if entry_result.is_err() {
                            return;
                        }
                        if cli_program.is_some_and(|cli| is_cli_args_local(stmt, &cli.entry_args, self)) {
                            continue;
                        }
                        entry_result = stmt::generate_stmt(self, stmt, types, writer, false, true, false);
                    }
                    if let Some(expr) = &entry.expr {
                        if entry_result.is_err() {
                            return;
                        }
                        entry_result = expr::generate_expr(self, expr, types, writer, false, true, false);
                        writer.writeln(";");
                    }
                    if let Some(cli_program) = cli_program {
                        if let Some(exit) = &cli_program.exit {
                            cli::generate_cli_exit(exit, writer);
                        }
                    }
                });
                if entry_needs_block_on {
                    writer.writeln("});");
                }
            });
            entry_result?;
            body.writeln("}");
        }

        let body_code = body.finish();
        let mut imports = collect_hir_imports(self, hir);
        imports.extend(collect_prelude_imports(&body_code));

        let mut writer = CodeWriter::new();
        if module_mode {
            for import in &imports {
                writer.write("use ");
                writer.write(import);
                writer.writeln(";");
            }
            if !imports.is_empty() {
                writer.newline();
            }
        } else {
            generate_prelude(&mut writer, &imports);
        }
        writer.write(&body_code);

        Ok(RustOutput { code: writer.finish() })
    }
}

/// Generate embeddable Rust module source for a checked HIR program.
///
/// Module generation skips the file-level generated prelude and `main`
/// contract used by full executable output. Callers get a [`RustOutput`] whose
/// `code` field is the complete generated module text.
pub fn generate_module(hir: &HirProgram, types: &TypeTable, interner: &Interner) -> Result<RustOutput, CodegenError> {
    generate_module_with_test_selection(hir, types, interner, None)
}

/// Generate module source while applying Faber test-selection metadata.
///
/// `reject_hir_errors` guards this public backend boundary from programs that
/// earlier phases already marked erroneous; it does not rerun parsing,
/// resolution, or semantic analysis.
pub fn generate_module_with_test_selection(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    test_selection: Option<TestSelection>,
) -> Result<RustOutput, CodegenError> {
    generate_module_with_library_registry_and_test_selection(
        hir,
        types,
        interner,
        &LibraryRegistry::default(),
        test_selection,
    )
}

pub fn generate_module_with_library_registry_and_test_selection(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    libraries: &LibraryRegistry,
    test_selection: Option<TestSelection>,
) -> Result<RustOutput, CodegenError> {
    super::reject_hir_errors(hir)?;
    RustCodegen::new_with_library_registry_and_test_selection(hir, interner, libraries, test_selection)
        .generate_output(hir, types, true, None)
}

pub fn generate_module_with_cli(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    cli_program: &crate::cli::CliProgram,
) -> Result<RustOutput, CodegenError> {
    generate_module_with_cli_and_test_selection(hir, types, interner, cli_program, None)
}

/// Generate module source for package CLI commands.
///
/// Subcommand support can reference functions from mounted modules, so this
/// path keeps CLI argument types addressable as `crate::...` while still using
/// the same Rust backend item emitters as ordinary module generation.
pub fn generate_module_with_cli_and_test_selection(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    cli_program: &crate::cli::CliProgram,
    test_selection: Option<TestSelection>,
) -> Result<RustOutput, CodegenError> {
    generate_module_with_cli_library_registry_and_test_selection(
        hir,
        types,
        interner,
        cli_program,
        &LibraryRegistry::default(),
        test_selection,
    )
}

pub fn generate_module_with_cli_library_registry_and_test_selection(
    hir: &HirProgram,
    types: &TypeTable,
    interner: &Interner,
    cli_program: &crate::cli::CliProgram,
    libraries: &LibraryRegistry,
    test_selection: Option<TestSelection>,
) -> Result<RustOutput, CodegenError> {
    super::reject_hir_errors(hir)?;
    RustCodegen::new_with_library_registry_and_test_selection(hir, interner, libraries, test_selection).generate_output(
        hir,
        types,
        true,
        Some(cli_program),
    )
}

fn is_cli_args_local(stmt: &crate::hir::HirStmt, entry_args: &str, codegen: &RustCodegen<'_>) -> bool {
    let crate::hir::HirStmtKind::Local(local) = &stmt.kind else {
        return false;
    };
    local.init.is_none() && codegen.resolve_symbol(local.name) == entry_args
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
