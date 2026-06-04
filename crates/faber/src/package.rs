//! Package build orchestration for the user-facing `faber` CLI.
//!
//! This module is the boundary between Faber source packages and the generated
//! Rust crate that Cargo builds. It owns package discovery, manifest policy,
//! import graph loading, built-in library binding, mounted CLI command analysis,
//! generated-crate layout, and Cargo invocation. Compiler parsing, semantic
//! analysis, and backend code generation remain in `radix`; this file decides
//! how many source files and package-level policies become one compiler input
//! and one generated build artifact.
//!
//! INVARIANTS
//! ==========
//! - Package mode is filesystem-backed; stdin cannot represent imports,
//!   manifests, or generated layouts.
//! - Generated Rust crates live under `<package>/target/faber/`.
//! - Cargo artifacts live under sibling `<package>/target/{debug,release}/`.
//! - Manifest, import, mount, and package-policy errors are diagnostics, not
//!   silent fallbacks to single-file compilation.
//! - Built-in library interfaces are parsed as Faber source so package builds
//!   do not need compiler-only special cases for stdlib APIs.
//!
//! COMPATIBILITY
//! =============
//! Legacy direct-file and directory inputs are still accepted where possible.
//! Those paths deliberately share layout discovery with manifest-backed
//! packages so old examples keep deterministic binary names and target paths
//! while `faber.toml` remains the preferred package surface.

use crate::library::{LibraryResolveError, LibraryResolver, ResolvedLibraryModule};
use radix::codegen::rust::TestSelection as RustTestSelection;
use radix::codegen::Target;
use radix::diagnostics::Diagnostic;
use radix::driver::{analyze_source_with_cli_program, Config, Session};
use radix::hir::{
    HirItemKind, LibraryBinding, LibraryIdentity, LibraryItem, LibraryItemKind, LibraryProvider,
};
use radix::lexer::{Interner, Span, TokenKind};
use radix::parser;
use radix::syntax::{AnnotationKind, ImportDecl, ImportKind, Program, StmtKind};
use radix::{CompileResult, Output, RustOutput};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

const MANIFEST_FILE: &str = "faber.toml";

/// Package entrypoints resolved from either `faber.toml` or legacy path input.
///
/// The compiler keeps package discovery separate from build layout discovery:
/// this type describes the Faber source graph only, not the generated Cargo
/// crate that may later be emitted under `target/faber/`.
struct PackageSpec {
    source_root: PathBuf,
    entry: PathBuf,
}

/// Layout for a package build: generated Rust crate under `target/faber/`,
/// Cargo artifacts under sibling `target/debug/` and `target/release/`.
///
/// This model is path-only and is the single source of truth for the package
/// build directory contract. Callers should derive all generated Cargo paths
/// from it instead of rebuilding paths ad hoc.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct BuildLayout {
    /// Directory containing `faber.toml`, or the legacy package root when no
    /// manifest exists.
    pub package_root: PathBuf,

    /// Manifest path for manifest-backed packages; may not exist for legacy
    /// direct-file or directory inputs.
    pub manifest_path: PathBuf,

    /// Root of the generated Rust crate.
    pub generated_crate_root: PathBuf,

    /// Cargo manifest written for the generated Rust crate.
    pub generated_cargo_manifest: PathBuf,

    /// Rust entrypoint written from the assembled package code.
    pub generated_rust_entry: PathBuf,

    /// Cargo target directory shared by generated build/test invocations.
    pub cargo_target_dir: PathBuf,

    /// Expected debug binary path produced by Cargo.
    pub debug_binary: PathBuf,

    /// Expected release binary path produced by Cargo.
    pub release_binary: PathBuf,
}

impl BuildLayout {
    /// Build a layout from an explicit package root directory and the package name
    /// declared in its faber.toml (or a provided name for legacy cases).
    ///
    /// The supplied `package_name` is sanitized for use as a Rust crate/binary name.
    #[allow(dead_code)]
    pub fn from_package_root(root: impl AsRef<Path>, package_name: &str) -> Self {
        let package_root = normalize_path(root.as_ref());
        let manifest_path = package_root.join(MANIFEST_FILE);
        let target_base = package_root.join("target");
        let generated_root = target_base.join("faber");
        let binary = sanitize_crate_name(package_name);

        let debug_binary = target_base.join("debug").join(&binary);
        let release_binary = target_base.join("release").join(&binary);

        Self {
            package_root,
            manifest_path,
            generated_crate_root: generated_root.clone(),
            generated_cargo_manifest: generated_root.join("Cargo.toml"),
            generated_rust_entry: generated_root.join("src").join("main.rs"),
            cargo_target_dir: target_base,
            debug_binary,
            release_binary,
        }
    }

    /// Returns the sanitized name used for the generated binary and crate.
    #[allow(dead_code)]
    pub fn binary_name(&self) -> &str {
        self.debug_binary
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("package")
    }
}

/// Sanitize a Faber package name into a valid Rust/Cargo crate and binary name.
///
/// The policy is intentionally conservative and Cargo-compatible:
/// - lowercase ASCII letters and digits
/// - keep `-` and `_`
/// - other characters become `-`
/// - trim leading/trailing separators
/// - if result empty, fallback to "package"
/// - if starts with a digit, prefix "p-" (Cargo prefers letter or _ start for some contexts)
#[allow(dead_code)]
pub fn sanitize_crate_name(name: &str) -> String {
    if name.trim().is_empty() {
        return "package".to_owned();
    }
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c == '-' || c == '_' {
            out.push(c);
        } else {
            out.push('-');
        }
    }
    let mut s = out.trim_matches(|c: char| c == '-' || c == '_').to_owned();
    if s.is_empty() {
        s = "package".to_owned();
    }
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        s = format!("p-{}", s);
    }
    s
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaberManifest {
    /// Package identity used for generated crate metadata and binary naming.
    pub package: ManifestPackage,

    /// Source-layout settings for package graph discovery.
    #[serde(default)]
    pub paths: ManifestPaths,

    /// Build settings accepted by the current package compiler.
    #[serde(default)]
    pub build: ManifestBuild,
}

/// `[package]` metadata from `faber.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestPackage {
    /// Human-authored package name; sanitized before it becomes a Cargo name.
    pub name: String,

    /// Package version copied into the generated Cargo manifest.
    #[serde(default = "default_version")]
    pub version: String,

    /// Faber source edition, distinct from the generated Rust edition.
    #[serde(default = "default_edition")]
    pub edition: String,
}

/// `[paths]` metadata that anchors package source discovery.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestPaths {
    /// Directory containing package source files, relative to the manifest.
    #[serde(default = "default_source_path")]
    pub source: String,

    /// Entry module path, relative to `source`.
    #[serde(default = "default_entry_path")]
    pub entry: String,
}

/// `[build]` metadata accepted by the package command surface.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestBuild {
    /// Backend target requested by the package.
    #[serde(default = "default_build_target")]
    pub target: String,

    /// Package output kind; currently only binary crates are supported.
    #[serde(default = "default_build_kind")]
    pub kind: String,
}

impl Default for ManifestPaths {
    fn default() -> Self {
        Self {
            source: default_source_path(),
            entry: default_entry_path(),
        }
    }
}

impl Default for ManifestBuild {
    fn default() -> Self {
        Self {
            target: default_build_target(),
            kind: default_build_kind(),
        }
    }
}

fn default_version() -> String {
    "0.1.0".to_owned()
}

fn default_edition() -> String {
    "2026".to_owned()
}

fn default_source_path() -> String {
    "src".to_owned()
}

fn default_entry_path() -> String {
    "main.fab".to_owned()
}

fn default_build_target() -> String {
    "rust".to_owned()
}

fn default_build_kind() -> String {
    "bin".to_owned()
}

struct PackageFile {
    path: PathBuf,
    module_segments: Vec<String>,
    source: String,
    program: Program,
    interner: Interner,
    library_imports: Vec<LibraryImportBinding>,
}

struct LibraryImportBinding {
    binding: String,
    import_span: Span,
    module: ResolvedLibraryModule,
}

struct LibraryInterfaceItem {
    exported_name: String,
    local_name: String,
    kind: LibraryItemKind,
}

type PackageDiscoveryResult = Result<PackageSpec, Box<Diagnostic>>;

/// Compile a package source graph into one backend output.
///
/// Package compilation currently targets Rust only because it must assemble
/// multiple modules and generated CLI surfaces into a single crate-shaped
/// backend result. Unsupported targets are reported as diagnostics instead of
/// falling back to single-file compilation.
pub fn compile_package(config: &Config, input: &Path) -> CompileResult {
    compile_package_internal(config, input, None)
}

/// Compile a package while forwarding a Rust test-selection policy to codegen.
///
/// This is used by the package test command path so module and entry code are
/// generated under the same test filtering contract.
pub fn compile_package_with_test_selection(
    config: &Config,
    input: &Path,
    test_selection: Option<&RustTestSelection>,
) -> CompileResult {
    compile_package_internal(config, input, test_selection)
}

fn compile_package_internal(
    config: &Config,
    input: &Path,
    test_selection: Option<&RustTestSelection>,
) -> CompileResult {
    if config.target != Target::Rust {
        return CompileResult {
            output: None,
            diagnostics: vec![Diagnostic::error(
                "package compilation currently supports Rust target only",
            )
            .with_file(input.display().to_string())],
        };
    }

    let spec = match discover_package(input) {
        Ok(spec) => spec,
        Err(diag) => {
            return CompileResult {
                output: None,
                diagnostics: vec![*diag],
            }
        }
    };

    let library_resolver = library_resolver_from_config(config);
    let files = match load_package(&spec, &library_resolver) {
        Ok(files) => files,
        Err(diagnostics) => {
            return CompileResult {
                output: None,
                diagnostics,
            }
        }
    };

    let session = Session::new(config.clone());
    let mount_plan = match build_mount_plan(&spec, &files) {
        Ok(plan) => plan,
        Err(diagnostics) => {
            return CompileResult {
                output: None,
                diagnostics,
            }
        }
    };
    let mut entry_code = None;
    let mut module_tree = ModuleNode::default();
    let mut diagnostics = Vec::new();

    for file in files {
        let file_cli = mount_plan.module_cli.get(&file.path).cloned();
        let analysis_source = match analysis_source_for_file(&file) {
            Ok(source) => source,
            Err(diag) => {
                diagnostics.push(diag);
                continue;
            }
        };
        let mut analysis = match analyze_source_with_cli_program(
            &session,
            &file.path.display().to_string(),
            &analysis_source,
            file_cli,
        ) {
            Ok(analysis) => analysis,
            Err(file_diagnostics) => {
                diagnostics.extend(file_diagnostics);
                continue;
            }
        };
        if let Err(diag) = attach_library_provenance(&mut analysis, &file.library_imports) {
            diagnostics.push(diag);
            continue;
        }

        let is_entry = file.path == spec.entry;
        if !is_entry {
            analysis.hir.entry = None;
        }
        if is_entry {
            if let Some(root_cli) = mount_plan.root_cli.clone() {
                analysis.cli_program = Some(root_cli);
            }
        }

        let rust = match generate_rust_code_for_analysis(&analysis, is_entry, test_selection) {
            Ok(output) => output,
            Err(err) => {
                diagnostics.push(
                    Diagnostic::codegen_error(&err.message)
                        .with_file(file.path.display().to_string()),
                );
                continue;
            }
        };

        diagnostics.extend(analysis.diagnostics);

        if rust.contains("unresolved_def") {
            diagnostics.push(
                Diagnostic::error("project compilation produced unresolved Rust backend names")
                    .with_file(file.path.display().to_string()),
            );
            continue;
        }

        if is_entry {
            entry_code = Some(rust);
        } else {
            module_tree.insert(&file.module_segments, rust);
        }
    }

    if diagnostics.iter().any(|diag| diag.is_error()) {
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    let Some(entry_code) = entry_code else {
        return CompileResult {
            output: None,
            diagnostics: vec![Diagnostic::error(
                "package compilation did not produce an entry module",
            )
            .with_file(spec.entry.display().to_string())],
        };
    };

    let crate_code = assemble_crate(&entry_code, &module_tree.render(0));
    CompileResult {
        output: Some(Output::Rust(RustOutput { code: crate_code })),
        diagnostics,
    }
}

fn generate_rust_code_for_analysis(
    analysis: &radix::driver::AnalyzedUnit,
    is_entry: bool,
    test_selection: Option<&RustTestSelection>,
) -> Result<String, radix::codegen::CodegenError> {
    if is_entry {
        if let Some(cli_program) = analysis.cli_program.as_ref() {
            let codegen =
                radix::codegen::rust::RustCodegen::new_with_library_registry_and_test_selection(
                    &analysis.hir,
                    &analysis.interner,
                    &analysis.libraries,
                    test_selection.cloned(),
                );
            return codegen
                .generate_cli(&analysis.hir, &analysis.types, cli_program)
                .map(|output| output.code);
        }

        let codegen =
            radix::codegen::rust::RustCodegen::new_with_library_registry_and_test_selection(
                &analysis.hir,
                &analysis.interner,
                &analysis.libraries,
                test_selection.cloned(),
            );
        return radix::codegen::Codegen::generate(
            &codegen,
            &analysis.hir,
            &analysis.types,
            &analysis.interner,
        )
        .map(|output| output.code);
    }

    if let Some(cli_program) = analysis.cli_program.as_ref() {
        return radix::codegen::rust::generate_module_with_cli_library_registry_and_test_selection(
            &analysis.hir,
            &analysis.types,
            &analysis.interner,
            cli_program,
            &analysis.libraries,
            test_selection.cloned(),
        )
        .map(|output| output.code);
    }

    radix::codegen::rust::generate_module_with_library_registry_and_test_selection(
        &analysis.hir,
        &analysis.types,
        &analysis.interner,
        &analysis.libraries,
        test_selection.cloned(),
    )
    .map(|output| output.code)
}

/// Check every loaded package module and return diagnostics without codegen.
///
/// The checker mirrors package compilation discovery and CLI mount analysis so
/// `faber check` reports the same import, manifest, and mounted-command policy
/// errors that a package build would encounter.
pub fn check_package(config: &Config, input: &Path) -> Vec<Diagnostic> {
    let spec = match discover_package(input) {
        Ok(spec) => spec,
        Err(diag) => return vec![*diag],
    };

    let library_resolver = library_resolver_from_config(config);
    let files = match load_package(&spec, &library_resolver) {
        Ok(files) => files,
        Err(diagnostics) => return diagnostics,
    };

    let mount_plan = match build_mount_plan(&spec, &files) {
        Ok(plan) => plan,
        Err(diagnostics) => return diagnostics,
    };

    let session = Session::new(config.clone());
    let mut diagnostics = Vec::new();

    for file in &files {
        let file_cli = if file.path == spec.entry {
            mount_plan.root_cli.clone()
        } else {
            mount_plan.module_cli.get(&file.path).cloned()
        };
        let analysis_source = match analysis_source_for_file(file) {
            Ok(source) => source,
            Err(diag) => {
                diagnostics.push(diag);
                continue;
            }
        };

        match analyze_source_with_cli_program(
            &session,
            &file.path.display().to_string(),
            &analysis_source,
            file_cli,
        ) {
            Ok(mut analysis) => {
                match attach_library_provenance(&mut analysis, &file.library_imports) {
                    Ok(()) => diagnostics.extend(analysis.diagnostics),
                    Err(diag) => diagnostics.push(diag),
                }
            }
            Err(file_diagnostics) => diagnostics.extend(file_diagnostics),
        }
    }

    diagnostics
}

fn discover_package(input: &Path) -> PackageDiscoveryResult {
    let input = absolutize_path(input);
    if !input.exists() {
        return Err(Box::new(Diagnostic::io_error(
            &input,
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )));
    }

    if input.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE) {
        return parse_manifest(&input);
    }

    if input.is_dir() {
        let root = normalize_path(&input);
        let manifest = root.join(MANIFEST_FILE);
        if manifest.exists() {
            return parse_manifest(&manifest);
        }

        return Ok(PackageSpec {
            entry: root.join("main.fab"),
            source_root: root,
        });
    }

    let entry = normalize_path(&input);
    let root = entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(PackageSpec {
        source_root: root,
        entry,
    })
}

fn parse_manifest(path: &Path) -> PackageDiscoveryResult {
    let manifest = read_manifest(path)?;
    let package_root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    validate_manifest(&manifest, path)?;

    let source_root = package_root.join(&manifest.paths.source);
    let entry = source_root.join(&manifest.paths.entry);
    Ok(PackageSpec { source_root, entry })
}

fn library_resolver_from_config(config: &Config) -> LibraryResolver {
    config
        .stdlib_path
        .as_ref()
        .map(|path| LibraryResolver::new(path.clone()))
        .unwrap_or_else(LibraryResolver::default)
}

/// Read and deserialize a `faber.toml` manifest.
///
/// Unknown manifest fields are rejected by the manifest structs so spelling
/// mistakes become diagnostics rather than silently ignored configuration.
pub fn read_manifest(path: &Path) -> Result<FaberManifest, Box<Diagnostic>> {
    let source =
        fs::read_to_string(path).map_err(|err| Box::new(Diagnostic::io_error(path, err)))?;
    toml::from_str::<FaberManifest>(&source).map_err(|err| {
        Box::new(
            Diagnostic::error(format!("invalid faber.toml manifest: {err}"))
                .with_file(path.display().to_string()),
        )
    })
}

fn validate_manifest(manifest: &FaberManifest, path: &Path) -> Result<(), Box<Diagnostic>> {
    if manifest.package.name.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::error("faber.toml package.name must not be empty")
                .with_file(path.display().to_string()),
        ));
    }

    if manifest.package.version.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::error("faber.toml package.version must not be empty")
                .with_file(path.display().to_string()),
        ));
    }

    if manifest.package.edition.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::error("faber.toml package.edition must not be empty")
                .with_file(path.display().to_string()),
        ));
    }

    if manifest.paths.source.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::error("faber.toml paths.source must not be empty")
                .with_file(path.display().to_string()),
        ));
    }

    if manifest.paths.entry.trim().is_empty() {
        return Err(Box::new(
            Diagnostic::error("faber.toml paths.entry must not be empty")
                .with_file(path.display().to_string()),
        ));
    }

    if manifest.build.target != "rust" {
        return Err(Box::new(
            Diagnostic::error(format!(
                "faber.toml build.target '{}' is not supported for package compilation yet",
                manifest.build.target
            ))
            .with_file(path.display().to_string()),
        ));
    }

    if manifest.build.kind != "bin" {
        return Err(Box::new(
            Diagnostic::error(format!(
                "faber.toml build.kind '{}' is not supported yet",
                manifest.build.kind
            ))
            .with_file(path.display().to_string()),
        ));
    }

    Ok(())
}

/// Discover a `BuildLayout` for the given input (directory, manifest file, or entry file).
///
/// Mirrors the resolution rules of `discover_package`, then adds the package
/// name needed for generated crate and binary paths. Manifest-backed packages
/// use `package.name`; legacy non-manifest inputs fall back to their directory
/// name so old direct-file workflows still have deterministic output paths.
#[allow(dead_code)]
pub fn discover_build_layout(input: &Path) -> Result<BuildLayout, Box<Diagnostic>> {
    let input = absolutize_path(input);
    if !input.exists() {
        return Err(Box::new(Diagnostic::io_error(
            &input,
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )));
    }

    if input.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE) {
        let manifest = read_manifest(&input)?;
        let root = normalize_path(input.parent().unwrap_or_else(|| Path::new(".")));
        let name = manifest.package.name.clone();
        return Ok(BuildLayout::from_package_root(root, &name));
    }

    if input.is_dir() {
        let root = normalize_path(&input);
        let manifest = root.join(MANIFEST_FILE);
        if manifest.exists() {
            let m = read_manifest(&manifest)?;
            return Ok(BuildLayout::from_package_root(root, &m.package.name));
        }
        let name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("package")
            .to_owned();
        return Ok(BuildLayout::from_package_root(root, &name));
    }

    let entry = normalize_path(&input);
    let root = entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let manifest = root.join(MANIFEST_FILE);
    if manifest.exists() {
        let m = read_manifest(&manifest)?;
        return Ok(BuildLayout::from_package_root(root, &m.package.name));
    }
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("package")
        .to_owned();
    Ok(BuildLayout::from_package_root(root, &name))
}

fn load_package(
    spec: &PackageSpec,
    library_resolver: &LibraryResolver,
) -> Result<Vec<PackageFile>, Vec<Diagnostic>> {
    let mut queue = VecDeque::from([spec.entry.clone()]);
    let mut seen = BTreeSet::new();
    let mut files = Vec::new();
    let mut diagnostics = Vec::new();

    while let Some(path) = queue.pop_front() {
        let canonical = normalize_path(&path);
        if !seen.insert(canonical.clone()) {
            continue;
        }

        let source = match fs::read_to_string(&canonical) {
            Ok(source) => source,
            Err(err) => {
                diagnostics.push(Diagnostic::io_error(&canonical, err));
                continue;
            }
        };

        let parse = parser::parse(radix::lexer::lex(&source));
        if !parse.success() {
            diagnostics.extend(parse.errors.iter().map(|err| {
                Diagnostic::from_parse_error(&canonical.display().to_string(), &source, err)
            }));
            continue;
        }

        let radix::parser::ParseResult {
            program, interner, ..
        } = parse;
        let Some(program) = program else {
            diagnostics.push(
                Diagnostic::error("successful package parse result missing program")
                    .with_file(canonical.display().to_string()),
            );
            continue;
        };

        let mut library_imports = Vec::new();
        for stmt in &program.stmts {
            let StmtKind::Import(decl) = &stmt.kind else {
                continue;
            };
            let import_path = interner.resolve(decl.path);
            match resolve_import(spec, library_resolver, &canonical, import_path) {
                ImportResolution::Local(target) => queue.push_back(target),
                ImportResolution::Library(module) => {
                    if let Some(binding) = library_import_binding(&interner, decl, module) {
                        library_imports.push(binding);
                    } else {
                        diagnostics.push(library_import_kind_diagnostic(
                            &canonical,
                            decl,
                            import_path,
                        ));
                    }
                }
                ImportResolution::Unsupported => {
                    diagnostics.push(import_unsupported_diagnostic(&canonical, decl, import_path));
                }
                ImportResolution::Error(diag) => {
                    diagnostics.push(diag.with_span(decl.span));
                }
            }
        }

        files.push(PackageFile {
            module_segments: module_segments(&spec.source_root, &canonical),
            path: canonical,
            source,
            program,
            interner,
            library_imports,
        });
    }

    if diagnostics.iter().any(|diag| diag.is_error()) {
        Err(diagnostics)
    } else {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        diagnostics.extend(detect_import_cycles(spec, &files));
        if diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(diagnostics);
        }
        Ok(files)
    }
}

#[allow(clippy::result_large_err)]
fn analysis_source_for_file(file: &PackageFile) -> Result<String, Diagnostic> {
    if file.library_imports.is_empty() {
        return Ok(file.source.clone());
    }

    let mut source = strip_library_imports(&file.source, &file.library_imports);
    for import in &file.library_imports {
        source.push('\n');
        source.push_str(&library_interface_source(import)?);
    }
    Ok(source)
}

fn strip_library_imports(source: &str, imports: &[LibraryImportBinding]) -> String {
    let mut bytes = source.as_bytes().to_vec();
    for import in imports {
        let start = import.import_span.start as usize;
        let end = import.import_span.end as usize;
        for byte in bytes.iter_mut().take(end.min(source.len())).skip(start) {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| source.to_owned())
}

#[allow(clippy::result_large_err)]
fn library_interface_source(import: &LibraryImportBinding) -> Result<String, Diagnostic> {
    let source = fs::read_to_string(&import.module.interface_path)
        .map_err(|err| Diagnostic::io_error(&import.module.interface_path, err))?;
    rename_library_interface(&source, import)
}

#[allow(clippy::result_large_err)]
fn attach_library_provenance(
    analysis: &mut radix::driver::AnalyzedUnit,
    imports: &[LibraryImportBinding],
) -> Result<(), Diagnostic> {
    if imports.is_empty() {
        return Ok(());
    }

    let hir_items = analysis
        .hir
        .items
        .iter()
        .filter_map(|item| {
            hir_item_name_and_kind(item, &analysis.interner)
                .map(|(name, kind)| (name, kind, item.def_id))
        })
        .collect::<Vec<_>>();

    for import in imports {
        let identity = library_identity(&import.module);
        let interface_items = library_interface_items(import)?;

        let Some((_, _, binding_def_id)) = hir_items
            .iter()
            .find(|(name, _, _)| name.as_str() == import.binding)
        else {
            return Err(Diagnostic::error(format!(
                "library import `{}` did not produce binding `{}` in analyzed HIR",
                import.module.package, import.binding
            ))
            .with_span(import.import_span));
        };

        analysis.libraries.bindings.insert(
            *binding_def_id,
            LibraryBinding {
                local_def_id: *binding_def_id,
                identity: identity.clone(),
            },
        );

        for interface_item in interface_items {
            let Some((_, _, def_id)) = hir_items.iter().find(|(name, kind, _)| {
                name == &interface_item.local_name && kind == &interface_item.kind
            }) else {
                return Err(Diagnostic::error(format!(
                    "library import `{}` did not produce item `{}` in analyzed HIR",
                    import.module.package, interface_item.local_name
                ))
                .with_span(import.import_span));
            };

            analysis.libraries.items.insert(
                *def_id,
                LibraryItem {
                    def_id: *def_id,
                    identity: identity.clone(),
                    exported_name: interface_item.exported_name,
                    kind: interface_item.kind,
                },
            );
        }
    }

    Ok(())
}

fn library_identity(module: &ResolvedLibraryModule) -> LibraryIdentity {
    let provider = if module.package == "norma"
        && module.provider == crate::library::LibraryProviderKind::Builtin
    {
        LibraryProvider::BuiltinNorma
    } else {
        LibraryProvider::Package(module.package.clone())
    };
    LibraryIdentity {
        provider,
        module_path: module.module_path.clone(),
    }
}

#[allow(clippy::result_large_err)]
fn library_interface_items(
    import: &LibraryImportBinding,
) -> Result<Vec<LibraryInterfaceItem>, Diagnostic> {
    let source = fs::read_to_string(&import.module.interface_path)
        .map_err(|err| Diagnostic::io_error(&import.module.interface_path, err))?;
    let parse = parser::parse(radix::lexer::lex(&source));
    if !parse.success() {
        let mut message = format!(
            "library interface `{}` failed to parse",
            import.module.interface_path.display()
        );
        if let Some(err) = parse.errors.first() {
            message.push_str(&format!(": {}", err.message));
        }
        return Err(Diagnostic::error(message)
            .with_file(import.module.interface_path.display().to_string()));
    }

    let Some(program) = parse.program else {
        return Err(
            Diagnostic::error("successful library interface parse result missing program")
                .with_file(import.module.interface_path.display().to_string()),
        );
    };

    let module_name = import.module.module_name().unwrap_or_default();
    let items = program
        .stmts
        .iter()
        .filter_map(|stmt| {
            library_interface_item(stmt, &parse.interner, module_name, &import.binding)
        })
        .collect::<Vec<_>>();
    Ok(items)
}

fn library_interface_item(
    stmt: &radix::syntax::Stmt,
    interner: &Interner,
    module_name: &str,
    binding: &str,
) -> Option<LibraryInterfaceItem> {
    let (name, kind) = match &stmt.kind {
        StmtKind::Interface(interface) => (
            interner.resolve(interface.name.name),
            LibraryItemKind::Interface,
        ),
        StmtKind::Func(func) => (interner.resolve(func.name.name), LibraryItemKind::Function),
        StmtKind::TypeAlias(alias) => (
            interner.resolve(alias.name.name),
            LibraryItemKind::TypeAlias,
        ),
        StmtKind::Class(class) => (interner.resolve(class.name.name), LibraryItemKind::Struct),
        StmtKind::Enum(enm) => (interner.resolve(enm.name.name), LibraryItemKind::Enum),
        StmtKind::Union(union) => (interner.resolve(union.name.name), LibraryItemKind::Enum),
        StmtKind::Var(var) => {
            let radix::syntax::BindingPattern::Ident(ident) = &var.binding else {
                return None;
            };
            (interner.resolve(ident.name), LibraryItemKind::Const)
        }
        _ => return None,
    };
    let local_name = if name == module_name { binding } else { name };
    Some(LibraryInterfaceItem {
        exported_name: name.to_owned(),
        local_name: local_name.to_owned(),
        kind,
    })
}

fn hir_item_name_and_kind(
    item: &radix::hir::HirItem,
    interner: &Interner,
) -> Option<(String, LibraryItemKind)> {
    match &item.kind {
        HirItemKind::Interface(interface) => Some((
            interner.resolve(interface.name).to_owned(),
            LibraryItemKind::Interface,
        )),
        HirItemKind::Function(func) => Some((
            interner.resolve(func.name).to_owned(),
            LibraryItemKind::Function,
        )),
        HirItemKind::TypeAlias(alias) => Some((
            interner.resolve(alias.name).to_owned(),
            LibraryItemKind::TypeAlias,
        )),
        HirItemKind::Struct(strukt) => Some((
            interner.resolve(strukt.name).to_owned(),
            LibraryItemKind::Struct,
        )),
        HirItemKind::Enum(enm) => {
            Some((interner.resolve(enm.name).to_owned(), LibraryItemKind::Enum))
        }
        HirItemKind::Const(konst) => Some((
            interner.resolve(konst.name).to_owned(),
            LibraryItemKind::Const,
        )),
        HirItemKind::Import(_) => None,
    }
}

#[allow(clippy::result_large_err)]
fn rename_library_interface(
    source: &str,
    import: &LibraryImportBinding,
) -> Result<String, Diagnostic> {
    let parse = parser::parse(radix::lexer::lex(source));
    if !parse.success() {
        let mut message = format!(
            "library interface `{}` failed to parse",
            import.module.interface_path.display()
        );
        if let Some(err) = parse.errors.first() {
            message.push_str(&format!(": {}", err.message));
        }
        return Err(Diagnostic::error(message)
            .with_file(import.module.interface_path.display().to_string()));
    }

    let Some(program) = parse.program else {
        return Err(
            Diagnostic::error("successful library interface parse result missing program")
                .with_file(import.module.interface_path.display().to_string()),
        );
    };

    let Some(interface_name_span) = program.stmts.iter().find_map(|stmt| {
        let StmtKind::Interface(interface) = &stmt.kind else {
            return None;
        };
        Some(interface.name.span)
    }) else {
        return Err(Diagnostic::error(format!(
            "library interface `{}` does not declare a pactum",
            import.module.interface_path.display()
        ))
        .with_file(import.module.interface_path.display().to_string()));
    };

    let start = interface_name_span.start as usize;
    let end = interface_name_span.end as usize;
    // WHY: imported built-in interfaces are rebound to the user's module alias
    // so ordinary package analysis can typecheck calls without a special
    // library namespace in the parser or resolver.
    let mut renamed = String::with_capacity(source.len() + import.binding.len());
    renamed.push_str(&source[..start]);
    renamed.push_str(&import.binding);
    renamed.push_str(&source[end..]);
    Ok(renamed)
}

#[derive(Default)]
struct MountPlan {
    root_cli: Option<radix::cli::CliProgram>,
    module_cli: BTreeMap<PathBuf, radix::cli::CliProgram>,
}

struct MountSpec {
    prefix: Vec<String>,
    alias: String,
    span: Span,
}

fn build_mount_plan(
    spec: &PackageSpec,
    files: &[PackageFile],
) -> Result<MountPlan, Vec<Diagnostic>> {
    let Some(entry_file) = files.iter().find(|file| file.path == spec.entry) else {
        return Ok(MountPlan::default());
    };

    let root_analysis = radix::cli::analyze(&entry_file.program, &entry_file.interner);
    let mut diagnostics = root_analysis
        .errors
        .iter()
        .map(|err| {
            Diagnostic::from_semantic_error(
                &entry_file.path.display().to_string(),
                &entry_file.source,
                err,
            )
        })
        .collect::<Vec<_>>();
    let Some(mut root_cli) = root_analysis.program else {
        if diagnostics.iter().any(Diagnostic::is_error) {
            return Err(diagnostics);
        }
        return Ok(MountPlan::default());
    };

    let imports = import_aliases(spec, entry_file);
    let mounts = collect_root_mounts(entry_file, &mut diagnostics);
    let files_by_path = files
        .iter()
        .map(|file| (file.path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let mut module_cli = BTreeMap::<PathBuf, radix::cli::CliProgram>::new();
    let mut command_origins = root_cli
        .commands
        .iter()
        .map(|command| (command.clone(), entry_file.path.clone()))
        .collect::<Vec<_>>();

    for mount in mounts {
        if imports.named_aliases.contains(&mount.alias) {
            diagnostics.push(
                Diagnostic::error(format!(
                    "@ imperia target '{}' must be a wildcard import alias, not a named import",
                    mount.alias
                ))
                .with_file(entry_file.path.display().to_string())
                .with_span(mount.span),
            );
            continue;
        }

        let Some(module_path) = imports.wildcard_aliases.get(&mount.alias) else {
            diagnostics.push(
                Diagnostic::error(format!(
                    "@ imperia target '{}' does not name a package-local wildcard import alias",
                    mount.alias
                ))
                .with_file(entry_file.path.display().to_string())
                .with_span(mount.span),
            );
            continue;
        };
        let Some(module_file) = files_by_path.get(module_path) else {
            diagnostics.push(
                Diagnostic::error(format!(
                    "@ imperia target '{}' resolved to a module that was not loaded",
                    mount.alias
                ))
                .with_file(entry_file.path.display().to_string())
                .with_span(mount.span),
            );
            continue;
        };

        let module_analysis = radix::cli::analyze_mounted_module(
            &module_file.program,
            &module_file.interner,
            &mount.prefix,
        );
        diagnostics.extend(module_analysis.errors.iter().map(|err| {
            Diagnostic::from_semantic_error(
                &module_file.path.display().to_string(),
                &module_file.source,
                err,
            )
        }));
        let Some(mut mounted_cli) = module_analysis.program else {
            continue;
        };
        mounted_cli.global_options = root_cli.global_options.clone();
        mounted_cli.global_operands = root_cli.global_operands.clone();
        diagnostics.extend(validate_mounted_global_collisions(
            &mounted_cli.commands,
            &root_cli,
            &module_file.path,
        ));

        for command in &mut mounted_cli.commands {
            let mut root_command = command.clone();
            root_command.module_path = Some(module_file.module_segments.clone());
            root_cli.commands.push(root_command.clone());
            command_origins.push((root_command, module_file.path.clone()));
        }
        module_cli.insert(module_file.path.clone(), mounted_cli);
    }

    diagnostics.extend(validate_mounted_command_collisions(&command_origins));
    if !root_cli.commands.is_empty() {
        root_cli.mode = radix::cli::CliMode::Subcommand;
    }
    if diagnostics.iter().any(Diagnostic::is_error) {
        Err(diagnostics)
    } else {
        Ok(MountPlan {
            root_cli: Some(root_cli),
            module_cli,
        })
    }
}

#[derive(Default)]
struct ImportAliases {
    wildcard_aliases: BTreeMap<String, PathBuf>,
    named_aliases: BTreeSet<String>,
}

fn import_aliases(spec: &PackageSpec, file: &PackageFile) -> ImportAliases {
    let mut aliases = ImportAliases::default();
    for stmt in &file.program.stmts {
        let StmtKind::Import(decl) = &stmt.kind else {
            continue;
        };
        let import_path = file.interner.resolve(decl.path);
        let Some(target) = resolve_local_import(spec, &file.path, import_path) else {
            continue;
        };
        match &decl.kind {
            ImportKind::Wildcard { alias } => {
                aliases.wildcard_aliases.insert(
                    file.interner.resolve(alias.name).to_owned(),
                    normalize_path(&target),
                );
            }
            ImportKind::Named { name, alias } => {
                let visible = alias.as_ref().unwrap_or(name);
                aliases
                    .named_aliases
                    .insert(file.interner.resolve(visible.name).to_owned());
            }
        }
    }
    aliases
}

fn collect_root_mounts(file: &PackageFile, diagnostics: &mut Vec<Diagnostic>) -> Vec<MountSpec> {
    let mut mounts = Vec::new();
    for stmt in &file.program.stmts {
        let is_cli_entry = stmt
            .annotations
            .iter()
            .any(|annotation| matches!(annotation.kind, AnnotationKind::Cli(_)));
        for annotation in &stmt.annotations {
            let AnnotationKind::Statement(annotation_stmt) = &annotation.kind else {
                continue;
            };
            if file.interner.resolve(annotation_stmt.name.name) != "imperia" {
                continue;
            }
            if !is_cli_entry {
                diagnostics.push(
                    Diagnostic::error(
                        "@ imperia module mounts must annotate the root @ cli entry point",
                    )
                    .with_file(file.path.display().to_string())
                    .with_span(annotation.span),
                );
                continue;
            }
            match parse_mount_annotation(file, annotation_stmt, annotation.span) {
                Some(mount) => mounts.push(mount),
                None => diagnostics.push(
                    Diagnostic::error(
                        "@ imperia must use '@ imperia \"path\" ex <wildcard_alias>'",
                    )
                    .with_file(file.path.display().to_string())
                    .with_span(annotation.span),
                ),
            }
        }
    }
    mounts
}

fn parse_mount_annotation(
    file: &PackageFile,
    annotation: &radix::syntax::AnnotationStmt,
    span: Span,
) -> Option<MountSpec> {
    if annotation.args.len() != 3 {
        return None;
    }
    let TokenKind::String(path) = annotation.args[0].kind else {
        return None;
    };
    match annotation.args[1].kind {
        TokenKind::Ex => {}
        TokenKind::Ident(sym) if file.interner.resolve(sym) == "ex" => {}
        _ => return None,
    }
    let TokenKind::Ident(alias) = annotation.args[2].kind else {
        return None;
    };
    let raw_path = file.interner.resolve(path);
    let prefix = raw_path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    // Policy: mounted command prefixes are logical CLI paths, not filesystem
    // paths, so absolute paths and empty segments are rejected at annotation
    // parse time.
    if prefix.is_empty()
        || raw_path.starts_with('/')
        || raw_path.ends_with('/')
        || raw_path.contains("//")
    {
        return None;
    }
    Some(MountSpec {
        prefix,
        alias: file.interner.resolve(alias).to_owned(),
        span,
    })
}

fn validate_mounted_command_collisions(
    commands: &[(radix::cli::CliCommand, PathBuf)],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut paths = BTreeMap::<String, Span>::new();
    let mut aliases = BTreeMap::<String, Span>::new();

    for (command, file) in commands {
        let path = command.path.join("/");
        if paths.insert(path.clone(), command.span).is_some() {
            diagnostics.push(
                Diagnostic::error(format!("duplicate command path '{path}'"))
                    .with_file(file.display().to_string())
                    .with_span(command.span),
            );
        }
    }

    for (command, file) in commands {
        for alias in &command.aliases {
            if aliases.insert(alias.clone(), command.span).is_some() {
                diagnostics.push(
                    Diagnostic::error(format!("duplicate command alias '{alias}'"))
                        .with_file(file.display().to_string())
                        .with_span(command.span),
                );
            }
            if paths.contains_key(alias) {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "command alias '{alias}' collides with a command path"
                    ))
                    .with_file(file.display().to_string())
                    .with_span(command.span),
                );
            }
        }
    }

    diagnostics
}

fn validate_mounted_global_collisions(
    commands: &[radix::cli::CliCommand],
    root_cli: &radix::cli::CliProgram,
    file: &Path,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut globals = BTreeSet::<&str>::new();
    for option in &root_cli.global_options {
        globals.insert(option.binding.as_str());
    }
    for operand in &root_cli.global_operands {
        globals.insert(operand.binding.as_str());
    }

    for command in commands {
        let label = command.path.join("/");
        for option in &command.options {
            if globals.contains(option.binding.as_str()) {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "command '{label}' option '{}' collides with a global CLI binding",
                        option.binding
                    ))
                    .with_file(file.display().to_string())
                    .with_span(option.span),
                );
            }
        }
        for operand in &command.operands {
            if globals.contains(operand.binding.as_str()) {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "command '{label}' operand '{}' collides with a global CLI binding",
                        operand.binding
                    ))
                    .with_file(file.display().to_string())
                    .with_span(operand.span),
                );
            }
        }
    }

    diagnostics
}

fn detect_import_cycles(spec: &PackageSpec, files: &[PackageFile]) -> Vec<Diagnostic> {
    let by_path = files
        .iter()
        .map(|file| (file.path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let mut graph = BTreeMap::<PathBuf, Vec<(PathBuf, Span)>>::new();
    for file in files {
        let mut edges = Vec::new();
        for stmt in &file.program.stmts {
            let StmtKind::Import(decl) = &stmt.kind else {
                continue;
            };
            let import_path = file.interner.resolve(decl.path);
            if let Some(target) = resolve_local_import(spec, &file.path, import_path) {
                edges.push((normalize_path(&target), decl.span));
            }
        }
        graph.insert(file.path.clone(), edges);
    }

    let mut diagnostics = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::<PathBuf>::new();
    for file in files {
        detect_import_cycles_from(
            &file.path,
            &graph,
            &by_path,
            &mut visiting,
            &mut visited,
            &mut stack,
            &mut diagnostics,
        );
    }
    diagnostics
}

enum ImportResolution {
    Local(PathBuf),
    Library(ResolvedLibraryModule),
    Unsupported,
    Error(Diagnostic),
}

fn resolve_import(
    spec: &PackageSpec,
    library_resolver: &LibraryResolver,
    from_file: &Path,
    import_path: &str,
) -> ImportResolution {
    match library_resolver.resolve(import_path) {
        Ok(Some(module)) => return ImportResolution::Library(module),
        Ok(None) => {}
        Err(err) => return ImportResolution::Error(library_resolve_diagnostic(from_file, err)),
    }

    if let Some(target) = resolve_local_import(spec, from_file, import_path) {
        return ImportResolution::Local(target);
    }

    ImportResolution::Unsupported
}

fn detect_import_cycles_from(
    path: &PathBuf,
    graph: &BTreeMap<PathBuf, Vec<(PathBuf, Span)>>,
    by_path: &BTreeMap<PathBuf, &PackageFile>,
    visiting: &mut BTreeSet<PathBuf>,
    visited: &mut BTreeSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if visited.contains(path) {
        return;
    }
    if !visiting.insert(path.clone()) {
        return;
    }
    stack.push(path.clone());

    for (next, span) in graph.get(path).into_iter().flatten() {
        if visiting.contains(next) {
            let cycle_start = stack.iter().position(|item| item == next).unwrap_or(0);
            let mut cycle = stack[cycle_start..]
                .iter()
                .map(|item| item.display().to_string())
                .collect::<Vec<_>>();
            cycle.push(next.display().to_string());
            diagnostics.push(
                Diagnostic::error(format!("import cycle detected: {}", cycle.join(" -> ")))
                    .with_file(path.display().to_string())
                    .with_span(*span),
            );
            continue;
        }
        if by_path.contains_key(next) {
            detect_import_cycles_from(next, graph, by_path, visiting, visited, stack, diagnostics);
        }
    }

    stack.pop();
    visiting.remove(path);
    visited.insert(path.clone());
}

fn resolve_local_import(
    spec: &PackageSpec,
    from_file: &Path,
    import_path: &str,
) -> Option<PathBuf> {
    if import_path.starts_with('.') {
        return resolve_module_candidates(&from_file.parent()?.join(import_path));
    }

    if import_path.starts_with('@') || import_path.contains("://") {
        return None;
    }

    resolve_module_candidates(&spec.source_root.join(import_path))
}

fn resolve_module_candidates(base: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if base.extension().is_some() {
        candidates.push(base.to_path_buf());
    } else {
        candidates.push(base.with_extension("fab"));
        candidates.push(base.join("main.fab"));
        candidates.push(base.join("mod.fab"));
    }
    candidates.into_iter().find(|candidate| candidate.exists())
}

fn library_import_binding(
    interner: &Interner,
    decl: &ImportDecl,
    module: ResolvedLibraryModule,
) -> Option<LibraryImportBinding> {
    let module_name = module.module_name()?;
    match &decl.kind {
        ImportKind::Named { name, alias } => {
            if interner.resolve(name.name) != module_name {
                return None;
            }
            let binding = alias.as_ref().unwrap_or(name);
            Some(LibraryImportBinding {
                binding: interner.resolve(binding.name).to_owned(),
                import_span: decl.span,
                module,
            })
        }
        ImportKind::Wildcard { .. } => None,
    }
}

fn library_resolve_diagnostic(file: &Path, err: LibraryResolveError) -> Diagnostic {
    match err {
        LibraryResolveError::OldBuiltinNormaSpecifier {
            specifier: _,
            replacement,
        } => Diagnostic::error(format!(
            "built-in Norma imports use provider syntax; write \"{replacement}\""
        ))
        .with_file(file.display().to_string()),
        LibraryResolveError::InvalidProviderSpecifier { specifier, reason } => Diagnostic::error(format!(
            "invalid library import specifier `{specifier}`: {reason}"
        ))
        .with_file(file.display().to_string()),
        LibraryResolveError::UnknownProvider {
            specifier,
            provider,
        } => Diagnostic::error(format!(
            "unknown library provider `{provider}` in import `{specifier}`"
        ))
        .with_file(file.display().to_string()),
        LibraryResolveError::UnknownBuiltinModule {
            specifier,
            package,
            known_modules,
        } => Diagnostic::error(format!(
            "unknown built-in library module `{specifier}` for provider `{package}`; known modules: {}",
            known_modules.join(", ")
        ))
        .with_file(file.display().to_string()),
    }
}

fn library_import_kind_diagnostic(file: &Path, decl: &ImportDecl, import_path: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "library import `{import_path}` must import its module name as a module alias"
    ))
    .with_file(file.display().to_string())
    .with_span(decl.span)
}

fn import_unsupported_diagnostic(file: &Path, decl: &ImportDecl, import_path: &str) -> Diagnostic {
    let kind = match &decl.kind {
        ImportKind::Named { .. } => "import",
        ImportKind::Wildcard { .. } => "wildcard import",
    };
    Diagnostic::error(format!(
        "package compilation only supports local intra-package imports; unsupported {kind} path `{import_path}`"
    ))
    .with_file(file.display().to_string())
    .with_span(decl.span)
}

fn module_segments(source_root: &Path, file: &Path) -> Vec<String> {
    let relative = file.strip_prefix(source_root).unwrap_or(file);
    let mut parts: Vec<String> = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    if let Some(last) = parts.last_mut() {
        if last == "main.fab" || last == "mod.fab" {
            parts.pop();
        } else if let Some(stripped) = last.strip_suffix(".fab") {
            *last = stripped.to_string();
        }
    }

    parts
}

/// Normalize lexical path components without consulting the filesystem.
///
/// Package compilation uses normalized paths as stable graph keys, but must not
/// require `std::fs::canonicalize` because missing files should become compiler
/// diagnostics rather than path-resolution panics.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Convert a possibly relative path into the normalized form used for package
/// graph keys.
///
/// This deliberately avoids filesystem canonicalization: package checks should
/// be able to report diagnostics for paths that do not exist yet without
/// requiring every parent directory to resolve through the OS.
pub(crate) fn absolutize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return normalize_path(path);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    normalize_path(&cwd.join(path))
}

fn assemble_crate(entry_code: &str, module_code: &str) -> String {
    let mut output = String::new();
    let mut inserted_modules = false;

    for line in entry_code.lines() {
        output.push_str(line);
        output.push('\n');

        if !inserted_modules && line.starts_with("#![allow(dead_code)]") {
            output.push('\n');
            if !module_code.trim().is_empty() {
                output.push_str(module_code);
                if !module_code.ends_with('\n') {
                    output.push('\n');
                }
                output.push('\n');
            }
            inserted_modules = true;
        }
    }

    if !inserted_modules && !module_code.trim().is_empty() {
        output.push('\n');
        output.push_str(module_code);
    }

    output
}

#[derive(Default)]
struct ModuleNode {
    code: Option<String>,
    children: BTreeMap<String, ModuleNode>,
}

impl ModuleNode {
    fn insert(&mut self, path: &[String], code: String) {
        if path.is_empty() {
            self.code = Some(code);
            return;
        }

        let child = self.children.entry(path[0].clone()).or_default();
        child.insert(&path[1..], code);
    }

    fn render(&self, indent: usize) -> String {
        let mut rendered = String::new();

        if let Some(code) = &self.code {
            for line in code.lines() {
                rendered.push_str(&" ".repeat(indent));
                rendered.push_str(line);
                rendered.push('\n');
            }
        }

        for (name, child) in &self.children {
            rendered.push_str(&" ".repeat(indent));
            rendered.push_str("pub mod ");
            rendered.push_str(name);
            rendered.push_str(" {\n");
            rendered.push_str(&child.render(indent + 4));
            rendered.push_str(&" ".repeat(indent));
            rendered.push_str("}\n");
        }

        rendered
    }
}

/// Generate a minimal, deterministic `Cargo.toml` for the emitted Rust crate.
///
/// The Rust edition is fixed at 2021 for backend output; Faber source edition
/// is manifest metadata for the language frontend and does not imply a Rust
/// edition. `binary_name` must already be sanitized for Cargo.
fn generate_cargo_toml(meta: &FaberManifest, binary_name: &str) -> String {
    let version = if meta.package.version.trim().is_empty() {
        "0.1.0"
    } else {
        meta.package.version.trim()
    };
    let norma_path = norma_runtime_path();
    format!(
        r#"[package]
name = "{name}"
version = "{version}"
edition = "2021"

# This crate was generated by `faber build` from the package's faber.toml.
# Source of truth: faber.toml at the package root.
# Do not edit this file by hand.

[workspace]
# Empty workspace table keeps this generated crate independent when the
# package lives inside the faber repository workspace tree (e.g. examples/).
# Prevents "current package believes it's in a workspace" errors for
# `cargo build/test --manifest-path target/faber/Cargo.toml`.

[dependencies]
norma = {{ path = "{norma_path}" }}
tokio = {{ version = "1", features = ["rt", "net", "time"] }}
"#,
        name = binary_name,
        version = version,
        norma_path = norma_path.display()
    )
}

fn norma_runtime_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("norma")
}

/// Write the generated Rust crate tree under the layout's `target/faber/` directory.
///
/// The generated crate owns exactly `target/faber/Cargo.toml` and
/// `target/faber/src/main.rs`. Other files under `target/` are Cargo artifacts
/// or user-adjacent build output and are intentionally left alone.
#[allow(dead_code)]
pub fn emit_generated_crate(
    layout: &BuildLayout,
    rust_code: &str,
    meta: Option<&FaberManifest>,
) -> Result<PathBuf, Box<Diagnostic>> {
    use std::fs;

    let src_dir = layout.generated_crate_root.join("src");
    if let Err(err) = fs::create_dir_all(&src_dir) {
        return Err(Box::new(Diagnostic::io_error(&src_dir, err)));
    }

    let cargo_src = if let Some(m) = meta {
        generate_cargo_toml(m, layout.binary_name())
    } else {
        let norma_path = norma_runtime_path();
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

# Generated crate (no faber.toml metadata available)

[workspace]
# Empty workspace table keeps this generated crate independent when the
# package lives inside the faber repository workspace tree (e.g. examples/).

[dependencies]
norma = {{ path = "{norma_path}" }}
tokio = {{ version = "1", features = ["rt", "net", "time"] }}
"#,
            name = layout.binary_name(),
            norma_path = norma_path.display()
        )
    };
    if let Err(err) = fs::write(&layout.generated_cargo_manifest, &cargo_src) {
        return Err(Box::new(Diagnostic::io_error(
            &layout.generated_cargo_manifest,
            err,
        )));
    }

    // Policy: keep an outer generated marker even when backend codegen already
    // writes its own header, because this file belongs to the package builder's
    // generated crate contract.
    let final_code = format!(
        "// Generated by faber build — do not edit by hand.\n\
         // Crate layout: target/faber/  (see plan.md)\n\
         // Run with: cargo build --manifest-path target/faber/Cargo.toml --target-dir target\n\n{}",
        rust_code
    );
    if let Err(err) = fs::write(&layout.generated_rust_entry, &final_code) {
        return Err(Box::new(Diagnostic::io_error(
            &layout.generated_rust_entry,
            err,
        )));
    }

    Ok(layout.generated_crate_root.clone())
}

/// Invoke Cargo to build the generated crate and return the expected binary path.
///
/// Uses the layout's paths so that artifacts land in `<pkg>/target/debug/<name>`
/// (sibling to `target/faber/`, never nested).
///
/// Cargo's stdout/stderr are inherited to preserve native compiler progress and
/// diagnostics.
#[allow(dead_code)]
pub fn invoke_cargo_build(layout: &BuildLayout, release: bool) -> Result<PathBuf, Box<Diagnostic>> {
    use std::process::Command;

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(&layout.generated_cargo_manifest)
        .arg("--target-dir")
        .arg(&layout.cargo_target_dir);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status().map_err(|e| {
        Box::new(Diagnostic::error(format!(
            "failed to spawn cargo (ensure cargo is installed and on PATH): {e}"
        )))
    })?;

    if !status.success() {
        return Err(Box::new(Diagnostic::error(format!(
            "cargo build exited with status {status}"
        ))));
    }

    let bin = if release {
        &layout.release_binary
    } else {
        &layout.debug_binary
    };
    Ok(bin.clone())
}

/// Invoke `cargo test` against the generated Rust crate.
///
/// Uses the package build directory contract:
///   --manifest-path <pkg>/target/faber/Cargo.toml
///   --target-dir <pkg>/target
///
/// The optional `filter` is passed before `--` as Cargo's Rust test name
/// filter. `harness_args` are forwarded after `--`. Test failures are not
/// converted into diagnostics; the harness exit status is returned verbatim so
/// the CLI can preserve Cargo's semantics.
#[allow(dead_code)]
pub fn invoke_cargo_test(
    layout: &BuildLayout,
    filter: Option<&str>,
    harness_args: &[String],
) -> Result<std::process::ExitStatus, Box<Diagnostic>> {
    use std::process::Command;

    let mut cmd = Command::new("cargo");
    cmd.arg("test")
        .arg("--manifest-path")
        .arg(&layout.generated_cargo_manifest)
        .arg("--target-dir")
        .arg(&layout.cargo_target_dir);

    if let Some(f) = filter {
        cmd.arg(f);
    }

    if !harness_args.is_empty() {
        cmd.arg("--");
        for arg in harness_args {
            cmd.arg(arg);
        }
    }

    let status = cmd.status().map_err(|e| {
        Box::new(Diagnostic::error(format!(
            "failed to spawn cargo (ensure cargo is installed and on PATH): {e}"
        )))
    })?;

    Ok(status)
}

#[cfg(test)]
#[path = "package_test.rs"]
mod tests;

/// Execute the user-facing `faber build` command.
///
/// Package Rust builds emit a generated Cargo crate and then delegate binary
/// production to Cargo. Direct-file builds and non-Rust targets keep the legacy
/// single-output behavior so package ergonomics do not change unrelated command
/// paths.
pub fn cmd_build(command: radix::tool::BuildCommand) {
    use std::fs;
    use std::path::PathBuf;

    let input_path = PathBuf::from(&command.input);
    let is_package = command.package || should_treat_as_package(&input_path);
    let config = Config::default().with_target(command.target);
    let result = if is_package {
        compile_package(&config, &input_path)
    } else {
        let compiler = radix::Compiler::new(config);
        compiler.compile(&input_path)
    };

    for diag in &result.diagnostics {
        if diag.is_error() {
            eprintln!("error: {}", diag.message);
        } else {
            eprintln!("warning: {}", diag.message);
        }
    }

    let Some(output) = result.output else {
        eprintln!("compilation failed");
        std::process::exit(1);
    };

    // Package Rust builds own a generated crate under target/faber/ and let
    // Cargo place artifacts in sibling debug/release directories.
    if is_package && command.target == radix::codegen::Target::Rust {
        let layout = match discover_build_layout(&input_path) {
            Ok(l) => l,
            Err(d) => {
                eprintln!("error: {}", d.message);
                std::process::exit(1);
            }
        };
        let meta = if layout.manifest_path.exists() {
            read_manifest(&layout.manifest_path).ok()
        } else {
            None
        };
        match emit_generated_crate(&layout, &output_code(output), meta.as_ref()) {
            Ok(_crate_root) => {
                let binary_path = match invoke_cargo_build(&layout, command.release) {
                    Ok(p) => p,
                    Err(d) => {
                        eprintln!("error: {}", d.message);
                        std::process::exit(1);
                    }
                };
                println!("{}", binary_path.display());
                return;
            }
            Err(d) => {
                eprintln!("error: {}", d.message);
                std::process::exit(1);
            }
        }
    }

    // Legacy single-file path (direct .fab files, other targets, or --out-dir override cases)
    let output_path =
        radix::tool::build_output_path(&command.out_dir, &input_path, command.target, is_package);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            eprintln!("error: failed to create '{}': {}", parent.display(), err);
            std::process::exit(1);
        });
    }

    fs::write(&output_path, output_code(output)).unwrap_or_else(|err| {
        eprintln!(
            "error: failed to write '{}': {}",
            output_path.display(),
            err
        );
        std::process::exit(1);
    });

    println!("{}", output_path.display());
}

/// Decide whether an input path should enter package-mode command handling.
///
/// Directory and manifest inputs are package-shaped by default; direct `.fab`
/// files remain eligible for legacy single-file commands unless the caller
/// forces package mode.
fn should_treat_as_package(path: &std::path::Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE)
}

/// CLI-argument variant of package-mode detection.
///
/// Standard-input builds (`-`) cannot be package builds because package
/// discovery needs filesystem paths for imports, manifests, and generated
/// layouts.
pub fn should_treat_as_package_from_args(input: &[String]) -> bool {
    if input.is_empty() || input[0] == "-" {
        return false;
    }
    let path = std::path::Path::new(&input[0]);
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE)
}

/// Execute the package-aware `faber check` command.
///
/// The permissive mode intentionally downgrades only unresolved/import-driven
/// semantic errors; manifest, I/O, parse, and package-policy errors remain
/// fatal because they prevent reliable package graph construction.
pub fn cmd_check_package(command: radix::tool::CheckCommand) {
    if command.input.is_empty() || command.input[0] == "-" {
        eprintln!("error: package checking requires a path input");
        std::process::exit(1);
    }

    let input_path = std::path::PathBuf::from(&command.input[0]);
    let config = Config::default().with_target(Target::Rust);
    let diagnostics = check_package(&config, &input_path);

    let mut fatal_errors = 0usize;
    let mut downgraded = 0usize;
    for diag in &diagnostics {
        let downgraded_error =
            command.permissive && diag.is_error() && is_permissive_check_code(diag.code);
        let prefix = if diag.is_error() && !downgraded_error {
            "error"
        } else {
            "warning"
        };
        eprintln!("{}: {}", prefix, diagnostic_summary(diag));
        if diag.is_error() {
            if downgraded_error {
                downgraded += 1;
            } else {
                fatal_errors += 1;
            }
        }
    }

    if command.permissive && downgraded > 0 {
        eprintln!(
            "warning:{}: downgraded {} unresolved/import-driven semantic error(s) in permissive mode",
            input_path.display(),
            downgraded
        );
    }

    if fatal_errors == 0 {
        eprintln!("ok: {}", input_path.display());
    } else {
        std::process::exit(1);
    }
}

/// Execute package emission and print generated code to stdout.
///
/// Unlike `cmd_build`, this command does not materialize the generated Cargo
/// crate. It is a compiler-inspection surface for the assembled backend output.
pub fn cmd_emit_package(command: radix::tool::EmitCommand) {
    let result = compile_package_input(&command.input, command.package, command.target);

    for diag in &result.diagnostics {
        if diag.is_error() {
            eprintln!("error: {}", diag.message);
        } else {
            eprintln!("warning: {}", diag.message);
        }
    }

    let Some(output) = result.output else {
        eprintln!("compilation failed");
        std::process::exit(1);
    };

    println!("{}", output_code(output));
}

fn compile_package_input(input: &[String], force_package: bool, target: Target) -> CompileResult {
    if input.is_empty() || input[0] == "-" {
        eprintln!("error: package compilation requires a path input");
        std::process::exit(1);
    }

    let path = std::path::PathBuf::from(&input[0]);
    let package = force_package || should_treat_as_package_from_args(input);
    if !package {
        eprintln!("error: expected a package directory, manifest, or entry file");
        std::process::exit(1);
    }

    let config = Config::default().with_target(target);
    compile_package(&config, &path)
}

fn diagnostic_summary(diag: &Diagnostic) -> String {
    if diag.file.is_empty() {
        diag.message.clone()
    } else {
        format!("{}: {}", diag.file, diag.message)
    }
}

fn is_permissive_check_code(code: Option<&'static str>) -> bool {
    matches!(
        code,
        Some("SEM001" | "SEM002" | "SEM003" | "SEM004" | "SEM006")
    )
}

fn output_code(output: Output) -> String {
    match output {
        Output::Rust(out) => out.code,
        Output::Faber(out) => out.code,
        Output::TypeScript(out) => out.code,
        Output::Go(out) => out.code,
        Output::WasmText(out) => out.code,
        Output::LlvmText(out) => out.code,
    }
}
