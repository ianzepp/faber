use super::{analyze_source_with_cli_program, Config, Session};
use crate::codegen::{self, Target};
use crate::diagnostics::Diagnostic;
use crate::lexer::{Interner, Span, TokenKind};
use crate::parser;
use crate::syntax::{AnnotationKind, DirectiveArg, ImportDecl, ImportKind, Program, StmtKind};
use crate::{CompileResult, Output, RustOutput};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

struct PackageSpec {
    source_root: PathBuf,
    entry: PathBuf,
}

struct PackageFile {
    path: PathBuf,
    module_segments: Vec<String>,
    source: String,
    program: Program,
    interner: Interner,
}

type PackageDiscoveryResult = Result<PackageSpec, Box<Diagnostic>>;

pub fn compile_package(config: &Config, input: &Path) -> CompileResult {
    if config.target != Target::Rust {
        return CompileResult {
            output: None,
            diagnostics: vec![
                Diagnostic::error("package compilation currently supports Rust target only")
                    .with_file(input.display().to_string()),
            ],
        };
    }

    let spec = match discover_package(input) {
        Ok(spec) => spec,
        Err(diag) => return CompileResult { output: None, diagnostics: vec![*diag] },
    };

    let files = match load_package(&spec) {
        Ok(files) => files,
        Err(diagnostics) => return CompileResult { output: None, diagnostics },
    };

    let session = Session::new(config.clone());
    let mount_plan = match build_mount_plan(&spec, &files) {
        Ok(plan) => plan,
        Err(diagnostics) => return CompileResult { output: None, diagnostics },
    };
    let mut entry_code = None;
    let mut module_tree = ModuleNode::default();
    let mut diagnostics = Vec::new();

    for file in files {
        let file_cli = mount_plan.module_cli.get(&file.path).cloned();
        let mut analysis =
            match analyze_source_with_cli_program(&session, &file.path.display().to_string(), &file.source, file_cli) {
                Ok(analysis) => analysis,
                Err(file_diagnostics) => {
                    diagnostics.extend(file_diagnostics);
                    continue;
                }
            };

        let is_entry = file.path == spec.entry;
        if !is_entry {
            analysis.hir.entry = None;
        }
        if is_entry {
            if let Some(root_cli) = mount_plan.root_cli.clone() {
                analysis.cli_program = Some(root_cli);
            }
        }

        let rust = if is_entry {
            let generated = if let Some(cli_program) = analysis.cli_program.as_ref() {
                codegen::generate_rust_cli(&analysis.hir, &analysis.types, &analysis.interner, cli_program)
                    .map(Output::Rust)
            } else {
                codegen::generate(Target::Rust, &analysis.hir, &analysis.types, &analysis.interner)
            };
            match generated {
                Ok(Output::Rust(output)) => output.code,
                Ok(_) => {
                    diagnostics.push(
                        Diagnostic::error("Rust target emitted unexpected non-Rust output variant")
                            .with_file(file.path.display().to_string()),
                    );
                    continue;
                }
                Err(err) => {
                    diagnostics
                        .push(Diagnostic::codegen_error(&err.message).with_file(file.path.display().to_string()));
                    continue;
                }
            }
        } else {
            let generated = if let Some(cli_program) = analysis.cli_program.as_ref() {
                codegen::rust::generate_module_with_cli(&analysis.hir, &analysis.types, &analysis.interner, cli_program)
            } else {
                codegen::rust::generate_module(&analysis.hir, &analysis.types, &analysis.interner)
            };
            match generated {
                Ok(output) => output.code,
                Err(err) => {
                    diagnostics
                        .push(Diagnostic::codegen_error(&err.message).with_file(file.path.display().to_string()));
                    continue;
                }
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
        return CompileResult { output: None, diagnostics };
    }

    let Some(entry_code) = entry_code else {
        return CompileResult {
            output: None,
            diagnostics: vec![Diagnostic::error("package compilation did not produce an entry module")
                .with_file(spec.entry.display().to_string())],
        };
    };

    let crate_code = assemble_crate(&entry_code, &module_tree.render(0));
    CompileResult { output: Some(Output::Rust(RustOutput { code: crate_code })), diagnostics }
}

pub fn check_package(config: &Config, input: &Path) -> Vec<Diagnostic> {
    let spec = match discover_package(input) {
        Ok(spec) => spec,
        Err(diag) => return vec![*diag],
    };

    let files = match load_package(&spec) {
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

        match analyze_source_with_cli_program(&session, &file.path.display().to_string(), &file.source, file_cli) {
            Ok(analysis) => diagnostics.extend(analysis.diagnostics),
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

    if input.file_name().and_then(|name| name.to_str()) == Some("faber.fab") {
        return parse_manifest(&input);
    }

    if input.is_dir() {
        let root = normalize_path(&input);
        return Ok(PackageSpec { entry: root.join("main.fab"), source_root: root });
    }

    let entry = normalize_path(&input);
    let root = entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(PackageSpec { source_root: root, entry })
}

fn parse_manifest(path: &Path) -> PackageDiscoveryResult {
    let source = fs::read_to_string(path).map_err(|err| Box::new(Diagnostic::io_error(path, err)))?;
    let parse = parser::parse(crate::lexer::lex(&source));
    let program = parse
        .program
        .ok_or_else(|| Box::new(Diagnostic::error("manifest parse failed").with_file(path.display().to_string())))?;
    let package_root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut source_root = package_root.clone();
    let mut entry = source_root.join("main.fab");

    for directive in &program.directives {
        let name = parse.interner.resolve(directive.name.name);
        match name {
            "fons" => {
                if let Some(DirectiveArg::String(value)) = directive.args.first() {
                    source_root = package_root.join(parse.interner.resolve(*value));
                }
            }
            "ingressus" => {
                if let Some(DirectiveArg::String(value)) = directive.args.first() {
                    entry = source_root.join(parse.interner.resolve(*value));
                }
            }
            "dependentia" => {
                return Err(Box::new(
                    Diagnostic::error("package compilation does not support manifest dependencies yet")
                        .with_file(path.display().to_string())
                        .with_span(directive.span),
                ));
            }
            _ => {}
        }
    }

    Ok(PackageSpec { source_root, entry })
}

fn load_package(spec: &PackageSpec) -> Result<Vec<PackageFile>, Vec<Diagnostic>> {
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

        let parse = parser::parse(crate::lexer::lex(&source));
        if !parse.success() {
            diagnostics.extend(
                parse
                    .errors
                    .iter()
                    .map(|err| Diagnostic::from_parse_error(&canonical.display().to_string(), &source, err)),
            );
            continue;
        }

        let crate::parser::ParseResult { program, interner, .. } = parse;
        let Some(program) = program else {
            diagnostics.push(
                Diagnostic::error("successful package parse result missing program")
                    .with_file(canonical.display().to_string()),
            );
            continue;
        };

        for stmt in &program.stmts {
            let StmtKind::Import(decl) = &stmt.kind else {
                continue;
            };
            let import_path = interner.resolve(decl.path);
            match resolve_local_import(spec, &canonical, import_path) {
                Some(target) => queue.push_back(target),
                None => diagnostics.push(import_unsupported_diagnostic(&canonical, decl, import_path)),
            }
        }

        files.push(PackageFile {
            module_segments: module_segments(&spec.source_root, &canonical),
            path: canonical,
            source,
            program,
            interner,
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

#[derive(Default)]
struct MountPlan {
    root_cli: Option<crate::cli::CliProgram>,
    module_cli: BTreeMap<PathBuf, crate::cli::CliProgram>,
}

struct MountSpec {
    prefix: Vec<String>,
    alias: String,
    span: Span,
}

fn build_mount_plan(spec: &PackageSpec, files: &[PackageFile]) -> Result<MountPlan, Vec<Diagnostic>> {
    let Some(entry_file) = files.iter().find(|file| file.path == spec.entry) else {
        return Ok(MountPlan::default());
    };

    let root_analysis = crate::cli::analyze(&entry_file.program, &entry_file.interner);
    let mut diagnostics = root_analysis
        .errors
        .iter()
        .map(|err| Diagnostic::from_semantic_error(&entry_file.path.display().to_string(), &entry_file.source, err))
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
    let mut module_cli = BTreeMap::<PathBuf, crate::cli::CliProgram>::new();
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

        let module_analysis =
            crate::cli::analyze_mounted_module(&module_file.program, &module_file.interner, &mount.prefix);
        diagnostics.extend(module_analysis.errors.iter().map(|err| {
            Diagnostic::from_semantic_error(&module_file.path.display().to_string(), &module_file.source, err)
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
        root_cli.mode = crate::cli::CliMode::Subcommand;
    }
    if diagnostics.iter().any(Diagnostic::is_error) {
        Err(diagnostics)
    } else {
        Ok(MountPlan { root_cli: Some(root_cli), module_cli })
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
                aliases
                    .wildcard_aliases
                    .insert(file.interner.resolve(alias.name).to_owned(), normalize_path(&target));
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
                    Diagnostic::error("@ imperia module mounts must annotate the root @ cli entry point")
                        .with_file(file.path.display().to_string())
                        .with_span(annotation.span),
                );
                continue;
            }
            match parse_mount_annotation(file, annotation_stmt, annotation.span) {
                Some(mount) => mounts.push(mount),
                None => diagnostics.push(
                    Diagnostic::error("@ imperia must use '@ imperia \"path\" ex <wildcard_alias>'")
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
    annotation: &crate::syntax::AnnotationStmt,
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
    if prefix.is_empty() || raw_path.starts_with('/') || raw_path.ends_with('/') || raw_path.contains("//") {
        return None;
    }
    Some(MountSpec { prefix, alias: file.interner.resolve(alias).to_owned(), span })
}

fn validate_mounted_command_collisions(commands: &[(crate::cli::CliCommand, PathBuf)]) -> Vec<Diagnostic> {
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
                    Diagnostic::error(format!("command alias '{alias}' collides with a command path"))
                        .with_file(file.display().to_string())
                        .with_span(command.span),
                );
            }
        }
    }

    diagnostics
}

fn validate_mounted_global_collisions(
    commands: &[crate::cli::CliCommand],
    root_cli: &crate::cli::CliProgram,
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

fn resolve_local_import(spec: &PackageSpec, from_file: &Path, import_path: &str) -> Option<PathBuf> {
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

fn normalize_path(path: &Path) -> PathBuf {
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

fn absolutize_path(path: &Path) -> PathBuf {
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

#[cfg(test)]
#[path = "project_test.rs"]
mod tests;
