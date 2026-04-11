use super::{analyze_source, Config, Session};
use crate::codegen::{self, Target};
use crate::diagnostics::Diagnostic;
use crate::parser;
use crate::syntax::{DirectiveArg, ImportDecl, ImportKind, Program, StmtKind};
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
    _program: Program,
}

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
        Err(diag) => return CompileResult { output: None, diagnostics: vec![diag] },
    };

    let files = match load_package(&spec) {
        Ok(files) => files,
        Err(diagnostics) => return CompileResult { output: None, diagnostics },
    };

    let session = Session::new(config.clone());
    let mut entry_code = None;
    let mut module_tree = ModuleNode::default();
    let mut diagnostics = Vec::new();

    for file in files {
        let mut analysis = match analyze_source(&session, &file.path.display().to_string(), &file.source) {
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

        let rust = if is_entry {
            match codegen::generate(Target::Rust, &analysis.hir, &analysis.types, &analysis.interner) {
                Ok(Output::Rust(output)) => output.code,
                Ok(_) => unreachable!("Rust target must emit Rust output"),
                Err(err) => {
                    diagnostics
                        .push(Diagnostic::codegen_error(&err.message).with_file(file.path.display().to_string()));
                    continue;
                }
            }
        } else {
            match codegen::rust::generate_module(&analysis.hir, &analysis.types, &analysis.interner) {
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

fn discover_package(input: &Path) -> Result<PackageSpec, Diagnostic> {
    let input = absolutize_path(input);
    if !input.exists() {
        return Err(Diagnostic::io_error(&input, std::io::Error::from(std::io::ErrorKind::NotFound)));
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

fn parse_manifest(path: &Path) -> Result<PackageSpec, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|err| Diagnostic::io_error(path, err))?;
    let parse = parser::parse(crate::lexer::lex(&source));
    let program = parse
        .program
        .ok_or_else(|| Diagnostic::error("manifest parse failed").with_file(path.display().to_string()))?;
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
                return Err(
                    Diagnostic::error("package compilation does not support manifest dependencies yet")
                        .with_file(path.display().to_string())
                        .with_span(directive.span),
                );
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

        let program = parse
            .program
            .expect("successful package parse must contain a program");

        for stmt in &program.stmts {
            let StmtKind::Import(decl) = &stmt.kind else {
                continue;
            };
            let import_path = parse.interner.resolve(decl.path);
            match resolve_local_import(spec, &canonical, import_path) {
                Some(target) => queue.push_back(target),
                None => diagnostics.push(import_unsupported_diagnostic(&canonical, decl, import_path)),
            }
        }

        files.push(PackageFile {
            module_segments: module_segments(&spec.source_root, &canonical),
            path: canonical,
            source,
            _program: program,
        });
    }

    if diagnostics.iter().any(|diag| diag.is_error()) {
        Err(diagnostics)
    } else {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
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
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("radix-project-{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn compile_package_reports_unresolved_external_imports() {
        let dir = temp_dir("external-import");
        let entry = dir.join("main.fab");
        fs::write(&entry, "importa ex \"lodash\" privata map\nincipit { scribe \"x\" }").expect("write entry");

        let result = compile_package(&Config::default(), &entry);
        assert!(result.output.is_none());
        assert!(result.diagnostics.iter().any(|diag| diag
            .message
            .contains("only supports local intra-package imports")));
    }

    #[test]
    fn compile_package_resolves_relative_input_from_current_working_directory() {
        let dir = temp_dir("relative-input");
        let project_dir = dir.join("project");
        fs::create_dir_all(&project_dir).expect("create project dir");
        fs::write(project_dir.join("main.fab"), "incipit { scribe \"salve\" }").expect("write entry");

        let original_cwd = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(&dir).expect("set current dir");

        let result = compile_package(&Config::default(), Path::new("./project/main.fab"));

        std::env::set_current_dir(original_cwd).expect("restore current dir");

        assert!(result.success(), "expected relative package compile success");
    }
}
