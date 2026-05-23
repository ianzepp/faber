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
mod stmt;
mod types;

use super::{names::NameCatalog, CodeWriter, Codegen, CodegenError};
use crate::hir::{DefId, HirExpr, HirFunction, HirItem, HirItemKind, HirProgram, HirTestMetadata, HirTestModifier};
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Type, TypeId, TypeTable};
use crate::RustOutput;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;

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

#[derive(Clone, Copy)]
pub(super) struct StructFieldInfo<'a> {
    pub(super) name: Symbol,
    pub(super) ty: TypeId,
    sponte: bool,
    pub(super) init: Option<&'a HirExpr>,
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
        let mut codegen = Self {
            names: NameCatalog::new(hir, interner),
            failable_defs: FxHashSet::default(),
            test_selection: None,
            struct_fields: FxHashMap::default(),
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
        codegen
    }

    fn generate_prelude(&self, w: &mut CodeWriter, imports: &BTreeSet<String>) {
        w.writeln("// Generated by radix - do not edit");
        w.newline();
        w.writeln("#![allow(unused_imports)]");
        w.writeln("#![allow(unused_variables)]");
        w.writeln("#![allow(dead_code)]");
        w.newline();

        for import in imports {
            w.write("use ");
            w.write(import);
            w.writeln(";");
        }

        if !imports.is_empty() {
            w.newline();
        }
    }

    pub(super) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.names.resolve_symbol(sym)
    }

    pub(super) fn resolve_def(&self, def_id: DefId) -> &str {
        self.names.resolve_def(def_id)
    }

    pub(super) fn is_failable_def(&self, def_id: DefId) -> bool {
        self.failable_defs.contains(&def_id)
    }

    pub(super) fn is_failable_method_name(&self, method: Symbol) -> bool {
        self.names
            .iter()
            .any(|(def_id, name)| *name == method && self.failable_defs.contains(def_id))
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
        w: &mut CodeWriter,
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
                    decl::generate_function_with_cli_args_type(self, item.def_id, func, types, w, Some(&args_type))?;
                } else {
                    decl::generate_function(self, item.def_id, func, types, w)?;
                }
            }
            HirItemKind::Struct(s) => {
                decl::generate_struct(self, s, types, w)?;
            }
            HirItemKind::Enum(e) => {
                decl::generate_enum(self, e, types, w)?;
            }
            HirItemKind::Interface(i) => {
                decl::generate_trait(self, i, types, w)?;
            }
            HirItemKind::TypeAlias(a) => {
                decl::generate_type_alias(self, a, types, w)?;
            }
            HirItemKind::Const(c) => {
                decl::generate_const(self, c, types, w)?;
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
            .is_some_and(|info| info.sponte || type_id_is_option(info.ty, types))
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
}

fn type_id_is_option(type_id: TypeId, types: &TypeTable) -> bool {
    match types.get(type_id) {
        Type::Option(_) => true,
        Type::Alias(_, resolved) => type_id_is_option(*resolved, types),
        _ => false,
    }
}

fn normalize_import_path(path: &str) -> String {
    let canonical = path.replace("::", "/");
    if canonical.starts_with('@') {
        return String::new();
    }

    let mut segments = Vec::new();
    for segment in canonical.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            continue;
        }
        segments.push(segment);
    }

    if segments.is_empty() {
        return String::new();
    }

    if matches!(segments[0], "crate" | "self" | "super" | "std" | "core" | "alloc") {
        return segments.join("::");
    }

    format!("crate::{}", segments.join("::"))
}

/// Scan generated code for Rust types that require imports.
///
/// WHY: this backend keeps import emission at the generated-output boundary.
/// Explicit source imports are available from HIR, but helper types such as
/// `HashMap`, `HashSet`, and `Future` arise from Rust lowering decisions in
/// sub-emitters. Scanning the completed body keeps that target policy local
/// without threading an import accumulator through every expression path.
fn collect_prelude_imports(code: &str) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();
    if code.contains("HashMap<") {
        imports.insert("std::collections::HashMap".to_owned());
    }
    if code.contains("HashSet<") {
        imports.insert("std::collections::HashSet".to_owned());
    }
    if code.contains("Future<Output =") {
        imports.insert("std::future::Future".to_owned());
    }
    imports
}

fn collect_hir_imports(codegen: &RustCodegen<'_>, hir: &HirProgram) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();
    for item in &hir.items {
        let HirItemKind::Import(import) = &item.kind else {
            continue;
        };

        let path = normalize_import_path(codegen.resolve_symbol(import.path));
        if path.is_empty() {
            continue;
        }

        for import_item in &import.items {
            let name = codegen.resolve_symbol(import_item.name);
            if let Some(alias) = import_item.alias {
                let alias_name = codegen.resolve_symbol(alias);
                if alias_name == name {
                    if path == format!("crate::{alias_name}") {
                        continue;
                    }
                    imports.insert(format!("{path} as {alias_name}"));
                } else {
                    imports.insert(format!("{path}::{name} as {alias_name}"));
                }
            } else {
                imports.insert(format!("{path}::{name}"));
            }
        }
    }
    imports
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
            body.writeln("fn main() {");
            let mut entry_result = Ok(());
            body.indented(|w| {
                if let Some(cli_program) = cli_program {
                    if cli_program.mode == crate::cli::CliMode::Subcommand {
                        cli::generate_command_dispatch(cli_program, w);
                        return;
                    }
                }
                if let Some(cli_program) = cli_program {
                    w.write("let ");
                    w.write(&cli_program.entry_args);
                    w.writeln(" = parse_cli_args_or_exit();");
                }
                for stmt in &entry.stmts {
                    if entry_result.is_err() {
                        return;
                    }
                    if cli_program.is_some_and(|cli| is_cli_args_local(stmt, &cli.entry_args, self)) {
                        continue;
                    }
                    entry_result = stmt::generate_stmt(self, stmt, types, w, false, true, false);
                }
                if let Some(expr) = &entry.expr {
                    if entry_result.is_err() {
                        return;
                    }
                    entry_result = expr::generate_expr(self, expr, types, w, false, true, false);
                    w.writeln(";");
                }
                if let Some(cli_program) = cli_program {
                    if let Some(exit) = &cli_program.exit {
                        cli::generate_cli_exit(exit, w);
                    }
                }
            });
            entry_result?;
            body.writeln("}");
        }

        let body_code = body.finish();
        let mut imports = collect_hir_imports(self, hir);
        imports.extend(collect_prelude_imports(&body_code));

        let mut w = CodeWriter::new();
        if module_mode {
            for import in &imports {
                w.write("use ");
                w.write(import);
                w.writeln(";");
            }
            if !imports.is_empty() {
                w.newline();
            }
        } else {
            self.generate_prelude(&mut w, &imports);
        }
        w.write(&body_code);

        Ok(RustOutput { code: w.finish() })
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
    super::reject_hir_errors(hir)?;
    RustCodegen::new_with_test_selection(hir, interner, test_selection).generate_output(hir, types, true, None)
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
    super::reject_hir_errors(hir)?;
    RustCodegen::new_with_test_selection(hir, interner, test_selection).generate_output(
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
