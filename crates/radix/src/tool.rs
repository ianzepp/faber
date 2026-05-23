//! radix CLI - Command-line interface for the Faber compiler
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! This binary provides the current `radix` command surface for `radix-rs`.
//! It now spans both product-facing compilation commands and compiler
//! inspection commands:
//!
//! - product-facing: `build`, `targets`, `check`
//! - inspection-oriented: `lex`, `parse`, `hir`, `emit`
//!
//! COMMANDS
//! ========
//! - `build`: Compile a file or package and write output to disk
//! - `targets`: Show supported targets and capability notes
//! - `check`: Run semantic analysis (with optional `--permissive`)
//! - `lex`: Tokenize source and emit JSON
//! - `parse`: Parse source and emit AST as JSON
//! - `hir`: Lower AST to HIR and emit JSON
//! - `mir`: Lower checked HIR to MIR and emit a deterministic text dump
//! - `emit`: Compile to target for stdout-oriented and debug workflows
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Two-layer surface: The same binary supports ordinary compile/check flows
//!   and lower-level compiler inspection without teaching a separate tool.
//!
//! - Phase introspection: Inspection commands still expose specific compiler
//!   phases, allowing developers to debug lexing, parsing, or semantic issues
//!   in isolation.
//!
//! - JSON output: Machine-readable output enables automated testing and tooling
//!   integration (e.g., language servers, formatters).
//!
//! - Stdin support: All commands accept stdin when no file is given, enabling
//!   pipeline composition and REPL-style workflows.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "faber", bin_name = "faber", about = "Faber compiler", version)]
pub struct FaberCli {
    #[command(subcommand)]
    pub command: FaberCommand,
}

#[derive(Subcommand, Debug)]
pub enum FaberCommand {
    /// Compile a file or package and write output to disk
    Build(BuildArgs),
    /// Show supported targets and current capability notes
    Targets,
    /// Tokenize source and output JSON
    Lex(InputArgs),
    /// Parse source and output AST as JSON
    Parse(InputArgs),
    /// Lower AST to HIR and output as JSON
    Hir(InputArgs),
    /// Validate and output normalized CLI IR as JSON
    CliIr(InputArgs),
    /// Run semantic analysis
    Check(CheckArgs),
    /// Compile to target (rust, faber, ts, go)
    Emit(EmitArgs),
}

#[derive(Parser, Debug)]
#[command(name = "radix", bin_name = "radix", about = "Faber compiler developer tool", version)]
pub struct RadixCli {
    #[command(subcommand)]
    pub command: RadixCommand,
}

#[derive(Subcommand, Debug)]
pub enum RadixCommand {
    /// Tokenize source and output JSON
    Lex(InputArgs),
    /// Parse source and output AST as JSON
    Parse(InputArgs),
    /// Lower AST to HIR and output as JSON
    Hir(InputArgs),
    /// Lower checked HIR to MIR and output a deterministic text dump
    Mir(InputArgs),
    /// Validate and output normalized CLI IR as JSON
    CliIr(InputArgs),
    /// Run semantic analysis
    Check(CheckArgs),
    /// Compile to target (rust, faber, ts, go)
    Emit(EmitArgs),
    /// Show supported targets and current capability notes
    Targets,
}

#[derive(Args, Debug)]
pub struct InputArgs {
    /// Input file path, or '-' / omitted for stdin
    pub input: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Downgrade unresolved/import-driven semantic errors to warnings
    #[arg(long)]
    pub permissive: bool,

    /// Force package checking mode
    #[arg(long)]
    pub package: bool,

    /// Input file or package path, or '-' / omitted for stdin
    pub input: Vec<String>,
}

#[derive(Args, Debug)]
pub struct EmitArgs {
    /// Output target language
    #[arg(short = 't', long = "target", value_enum, default_value_t = CliTarget::Rust)]
    pub target: CliTarget,

    /// Force package compilation mode
    #[arg(long)]
    pub package: bool,

    /// Run the target language's formatter on the emitted code (requires the formatter to be installed: rustfmt, gofmt, prettier, etc.)
    #[arg(long)]
    pub format: bool,

    /// Run a linter and auto-fix issues where possible.
    /// This is independent of --format; use both flags if you want formatting + linting.
    #[arg(long)]
    pub linter: bool,

    /// Input file or package path, or '-' / omitted for stdin
    pub input: Vec<String>,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    /// Output target language
    #[arg(short = 't', long = "target", value_enum, default_value_t = CliTarget::Rust)]
    pub target: CliTarget,

    /// Output directory for generated files
    #[arg(short = 'o', long = "out-dir", default_value = ".")]
    pub out_dir: PathBuf,

    /// Force package compilation mode
    #[arg(long)]
    pub package: bool,

    /// Build release profile instead of debug
    #[arg(long)]
    pub release: bool,

    /// Run the target language's formatter on the emitted code before writing files
    #[arg(long)]
    pub format: bool,

    /// Run a linter and auto-fix issues where possible before writing files.
    /// This is independent of --format; use both flags if you want formatting + linting.
    #[arg(long)]
    pub linter: bool,

    /// Input file or package path
    pub input: String,
}

#[derive(Debug)]
pub struct CheckCommand {
    pub input: Vec<String>,
    pub package: bool,
    pub permissive: bool,
}

#[derive(Debug)]
pub struct EmitCommand {
    pub input: Vec<String>,
    pub package: bool,
    pub target: crate::codegen::Target,
    pub format: bool,
    pub linter: bool,
}

#[derive(Debug)]
pub struct BuildCommand {
    pub input: String,
    pub out_dir: PathBuf,
    pub package: bool,
    pub release: bool,
    pub target: crate::codegen::Target,
    pub format: bool,
    pub linter: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum CliTarget {
    #[default]
    Rust,
    #[value(alias = "fab")]
    Faber,
    #[value(name = "ts", alias = "typescript")]
    TypeScript,
    Go,
}

impl From<CliTarget> for crate::codegen::Target {
    fn from(value: CliTarget) -> Self {
        match value {
            CliTarget::Rust => crate::codegen::Target::Rust,
            CliTarget::Faber => crate::codegen::Target::Faber,
            CliTarget::TypeScript => crate::codegen::Target::TypeScript,
            CliTarget::Go => crate::codegen::Target::Go,
        }
    }
}

pub struct TargetCapabilities {
    check: bool,
    build: bool,
    run: bool,
    package: bool,
    note: &'static str,
}

/// Read source from file argument or stdin.
///
/// WHY: Centralizes source reading logic for all commands. Stdin support
/// enables piping from other tools.
pub fn read_source(args: &[String]) -> (String, String) {
    if args.is_empty() || args[0] == "-" {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .unwrap_or_else(|err| {
                eprintln!("error: failed to read stdin: {err}");
                std::process::exit(1);
            });
        ("<stdin>".to_owned(), source)
    } else {
        let path = PathBuf::from(&args[0]);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            std::process::exit(1);
        });
        (args[0].clone(), source)
    }
}

fn source_file_from_input(name: String, source: String) -> crate::driver::SourceFile {
    crate::driver::SourceFile::inline(name, source)
}

pub fn format_location(source_file: &crate::driver::SourceFile, offset: u32) -> String {
    let (line, column) = source_file.offset_to_line_col(offset);
    format!("{}:{}:{}", source_file.name.as_str(), line, column)
}

fn format_optional_location(source_file: &crate::driver::SourceFile, span: Option<crate::lexer::Span>) -> String {
    span.map(|span| format_location(source_file, span.start))
        .unwrap_or_else(|| source_file.name.clone())
}

/// Tokenize source and emit JSON.
///
/// WHY: Enables inspection of lexer output for debugging tokenization issues.
pub fn cmd_lex(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let result = crate::lexer::lex(source_file.content.as_str());

    // WHY: JSON output for machine readability
    println!("{{");
    println!("  \"file\": \"{}\",", escape_json(&source_file.name));
    println!("  \"success\": {},", result.success());
    println!("  \"tokens\": [");

    for (i, token) in result.tokens.iter().enumerate() {
        let comma = if i + 1 < result.tokens.len() { "," } else { "" };
        let kind = format!("{:?}", token.kind);
        // WHY: Truncate long token representations to keep output readable
        let kind_display = if kind.len() > 60 {
            format!("{}...", &kind[..57])
        } else {
            kind
        };
        println!(
            "    {{ \"kind\": \"{}\", \"span\": [{}, {}] }}{}",
            escape_json(&kind_display),
            token.span.start,
            token.span.end,
            comma
        );
    }

    println!("  ],");
    println!("  \"errors\": [");

    for (i, err) in result.errors.iter().enumerate() {
        let comma = if i + 1 < result.errors.len() { "," } else { "" };
        println!(
            "    {{ \"message\": \"{}\", \"span\": [{}, {}] }}{}",
            escape_json(&err.message),
            err.span.start,
            err.span.end,
            comma
        );
    }

    println!("  ]");
    println!("}}");

    if !result.success() {
        std::process::exit(1);
    }
}

/// Parse source and emit AST JSON.
///
/// WHY: Enables inspection of parser output for debugging AST structure.
pub fn cmd_parse(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let lex_result = crate::lexer::lex(source_file.content.as_str());

    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);

    println!("{{");
    println!("  \"file\": \"{}\",", escape_json(&source_file.name));
    println!("  \"success\": {},", parse_result.success());

    if let Some(program) = &parse_result.program {
        println!("  \"statements\": {},", program.stmts.len());
        println!("  \"ast\": [");

        for (i, stmt) in program.stmts.iter().enumerate() {
            let comma = if i + 1 < program.stmts.len() { "," } else { "" };
            let kind = format!("{:?}", stmt.kind);
            // WHY: Extract variant name only to avoid huge debug output
            let kind_name = kind.split('(').next().unwrap_or(&kind);
            println!(
                "    {{ \"id\": {}, \"kind\": \"{}\", \"span\": [{}, {}], \"annotations\": [{}] }}{}",
                stmt.id,
                kind_name,
                stmt.span.start,
                stmt.span.end,
                annotation_json(&stmt.annotations),
                comma
            );
        }

        println!("  ],");
    } else {
        println!("  \"ast\": null,");
    }

    println!("  \"errors\": [");
    for (i, err) in parse_result.errors.iter().enumerate() {
        let comma = if i + 1 < parse_result.errors.len() { "," } else { "" };
        println!(
            "    {{ \"message\": \"{}\", \"span\": [{}, {}] }}{}",
            escape_json(&err.message),
            err.span.start,
            err.span.end,
            comma
        );
    }
    println!("  ]");
    println!("}}");

    if !parse_result.success() {
        std::process::exit(1);
    }
}

/// Lower AST to HIR and emit JSON.
///
/// WHY: Enables inspection of HIR lowering for debugging name resolution
/// and type assignment issues.
pub fn cmd_hir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    // WHY: HIR lowering requires lexing, parsing, and name resolution
    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let mut resolver = crate::semantic::Resolver::new();
    let mut types = crate::semantic::TypeTable::new();

    if let Err(e) = crate::semantic::passes::collect::collect(&program, &mut resolver, &mut types) {
        eprintln!("collection errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    if let Err(e) = crate::semantic::passes::resolve::resolve(&program, &mut resolver, &interner, &mut types) {
        eprintln!("resolution errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    let (hir, errors) = crate::hir::lower(&program, &resolver, &mut types, &interner);

    println!("{{");
    println!("  \"file\": \"{}\",", escape_json(&source_file.name));
    println!("  \"success\": {},", errors.is_empty());
    println!("  \"items\": {},", hir.items.len());
    println!("  \"hir\": [");

    for (i, item) in hir.items.iter().enumerate() {
        let comma = if i + 1 < hir.items.len() { "," } else { "" };
        let kind = format!("{:?}", item.kind);
        let kind_name = kind.split('(').next().unwrap_or(&kind);
        println!(
            "    {{ \"id\": {:?}, \"def_id\": {:?}, \"kind\": \"{}\", \"span\": [{}, {}] }}{}",
            item.id.0, item.def_id.0, kind_name, item.span.start, item.span.end, comma
        );
    }

    println!("  ],");
    println!("  \"errors\": [");

    for (i, err) in errors.iter().enumerate() {
        let comma = if i + 1 < errors.len() { "," } else { "" };
        println!(
            "    {{ \"message\": \"{}\", \"span\": [{}, {}] }}{}",
            escape_json(&err.message),
            err.span.start,
            err.span.end,
            comma
        );
    }

    println!("  ]");
    println!("}}");

    if !errors.is_empty() {
        std::process::exit(1);
    }
}

pub fn cmd_mir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    match mir_output_for_source(&source_file.name, &source_file.content) {
        Ok(output) => print!("{output}"),
        Err(messages) => {
            for message in messages {
                eprintln!("{message}");
            }
            std::process::exit(1);
        }
    }
}

pub fn mir_output_for_source(name: &str, source: &str) -> Result<String, Vec<String>> {
    let source_file = source_file_from_input(name.to_owned(), source.to_owned());
    let session =
        crate::driver::Session::new(crate::driver::Config::default().with_target(crate::codegen::Target::Faber));

    let analysis = match crate::driver::analyze_source(&session, &source_file.name, &source_file.content) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            return Err(diagnostics
                .into_iter()
                .map(|diagnostic| {
                    let prefix = if diagnostic.is_error() { "error" } else { "warning" };
                    format!(
                        "{}: {}: {}",
                        prefix,
                        format_optional_location(&source_file, diagnostic.span),
                        diagnostic.message
                    )
                })
                .collect())
        }
    };

    match crate::mir::dump_analyzed_unit(&analysis) {
        Ok(output) => Ok(output),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| format!("error: {}: {}", format_location(&source_file, err.span.start), err.message))
            .collect()),
    }
}

pub fn cmd_cli_ir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    println!("{}", cli_analysis_json(&cli_analysis));

    if !cli_analysis.errors.is_empty() {
        std::process::exit(1);
    }
}

/// Run semantic analysis.
///
/// WHY: --permissive mode allows checking files with unresolved imports,
/// useful for partial compilation or library development.
pub fn cmd_check(command: CheckCommand) {
    if command.package || should_treat_as_package_from_input(&command.input) {
        eprintln!("error: package checking is owned by the `faber` tool; rerun with `faber check --package`");
        std::process::exit(1);
    }

    let (name, source) = read_source(&command.input);
    let source_file = source_file_from_input(name, source);

    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        for err in &lex_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    let mut cli_fatal_errors = 0usize;
    for err in &cli_analysis.errors {
        eprintln!("error: {}: {}", format_location(&source_file, err.span.start), err.message);
        cli_fatal_errors += 1;
    }
    if cli_fatal_errors > 0 {
        std::process::exit(1);
    }

    let pass_config = crate::semantic::PassConfig::for_target(crate::codegen::Target::Rust);
    let semantic_result =
        crate::semantic::analyze_with_cli(&program, &pass_config, &interner, cli_analysis.program.as_ref());

    let mut fatal_errors = 0usize;
    let mut downgraded = 0usize;
    for err in &semantic_result.errors {
        let downgraded_error = command.permissive && err.kind.is_permissive_check_downgrade();
        let prefix = if err.is_error() && !downgraded_error {
            "error"
        } else {
            "warning"
        };
        eprintln!("{}: {}: {}", prefix, format_location(&source_file, err.span.start), err.message);
        if err.is_error() {
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
            source_file.name.as_str(),
            downgraded
        );
    }

    if semantic_result.success() || (command.permissive && fatal_errors == 0) {
        eprintln!("ok: {}", source_file.name.as_str());
    } else {
        std::process::exit(1);
    }
}

fn should_treat_as_package_from_input(input: &[String]) -> bool {
    if input.is_empty() || input[0] == "-" {
        return false;
    }
    let path = std::path::Path::new(&input[0]);
    should_treat_as_package(path)
}

/// Compile to target language.
///
/// WHY: End-to-end compilation command. Accepts -t flag to select Rust or
/// Faber pretty-print output.
pub fn cmd_emit(command: EmitCommand) {
    let result = compile_cli_input(&command.input, command.package, command.target);

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

    let mut code = output_code(output);

    if command.format {
        match format_generated_code(command.target, &code) {
            Ok(formatted) => code = formatted,
            Err(err) => {
                eprintln!("warning: formatting failed: {err}");
            }
        }
    }

    if command.linter {
        match lint_generated_code(command.target, &code) {
            Ok(fixed) => code = fixed,
            Err(err) => {
                eprintln!("warning: linter failed: {err}");
            }
        }
    }

    println!("{}", code);
}

pub fn cmd_build(command: BuildCommand) {
    let input_path = PathBuf::from(&command.input);
    let is_package = resolve_package_mode(&input_path, command.package);
    let result = compile_cli_path(&input_path, is_package, command.target);

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

    let output_path = build_output_path(&command.out_dir, &input_path, command.target, is_package);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            eprintln!("error: failed to create '{}': {}", parent.display(), err);
            std::process::exit(1);
        });
    }

    let mut code = output_code(output);

    if command.format {
        match format_generated_code(command.target, &code) {
            Ok(formatted) => code = formatted,
            Err(err) => {
                eprintln!("warning: formatting failed: {err}");
            }
        }
    }

    if command.linter {
        match lint_generated_code(command.target, &code) {
            Ok(fixed) => code = fixed,
            Err(err) => {
                eprintln!("warning: linter failed: {err}");
            }
        }
    }

    fs::write(&output_path, code).unwrap_or_else(|err| {
        eprintln!("error: failed to write '{}': {}", output_path.display(), err);
        std::process::exit(1);
    });

    println!("{}", output_path.display());
}

pub fn cmd_targets() {
    for target in [
        crate::codegen::Target::Rust,
        crate::codegen::Target::Go,
        crate::codegen::Target::TypeScript,
        crate::codegen::Target::Faber,
    ] {
        let capabilities = target_capabilities(target);
        println!(
            "{} check={} build={} run={} package={} note={}",
            target_name(target),
            yes_no(capabilities.check),
            yes_no(capabilities.build),
            yes_no(capabilities.run),
            yes_no(capabilities.package),
            capabilities.note
        );
    }
}

pub fn should_treat_as_package(path: &std::path::Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("faber.toml")
}

pub fn resolve_package_mode(path: &std::path::Path, force_package: bool) -> bool {
    force_package || should_treat_as_package(path)
}

pub fn compile_cli_input(input: &[String], package: bool, target: crate::codegen::Target) -> crate::CompileResult {
    if input.is_empty() || input[0] == "-" {
        if package {
            eprintln!("error: package compilation requires a path input");
            std::process::exit(1);
        }

        let (name, source) = read_source(input);
        return compile_cli_source(&name, &source, target);
    }

    let path = PathBuf::from(&input[0]);
    compile_cli_path(&path, resolve_package_mode(&path, package), target)
}

pub fn compile_cli_path(path: &std::path::Path, package: bool, target: crate::codegen::Target) -> crate::CompileResult {
    if package || should_treat_as_package(path) {
        eprintln!("error: package compilation is owned by the `faber` tool; rerun with `faber build` or `faber emit`");
        std::process::exit(1);
    }

    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile(path)
}

pub fn compile_cli_source(name: &str, source: &str, target: crate::codegen::Target) -> crate::CompileResult {
    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile_str(name, source)
}

pub fn build_output_path(
    out_dir: &std::path::Path,
    input_path: &std::path::Path,
    target: crate::codegen::Target,
    is_package: bool,
) -> PathBuf {
    let base_name = if is_package {
        "main".to_owned()
    } else {
        input_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("out")
            .to_owned()
    };
    out_dir.join(format!("{}.{}", base_name, target_extension(target)))
}

fn target_extension(target: crate::codegen::Target) -> &'static str {
    match target {
        crate::codegen::Target::Rust => "rs",
        crate::codegen::Target::Faber => "fab",
        crate::codegen::Target::TypeScript => "ts",
        crate::codegen::Target::Go => "go",
    }
}

fn target_name(target: crate::codegen::Target) -> &'static str {
    match target {
        crate::codegen::Target::Rust => "rust",
        crate::codegen::Target::Faber => "faber",
        crate::codegen::Target::TypeScript => "ts",
        crate::codegen::Target::Go => "go",
    }
}

pub fn target_capabilities(target: crate::codegen::Target) -> TargetCapabilities {
    match target {
        crate::codegen::Target::Rust => TargetCapabilities {
            check: true,
            build: true,
            run: true,
            package: true,
            note: "primary backend; full package build + run via `faber`",
        },
        crate::codegen::Target::Go => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        crate::codegen::Target::TypeScript => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        crate::codegen::Target::Faber => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "canonical pretty-print target; package compilation not yet supported",
        },
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

pub fn output_code(output: crate::Output) -> String {
    match output {
        crate::Output::Rust(out) => out.code,
        crate::Output::Faber(out) => out.code,
        crate::Output::TypeScript(out) => out.code,
        crate::Output::Go(out) => out.code,
    }
}

/// Run the appropriate formatter for the generated code, if available.
///
/// Returns the formatted code on success, or an error message if the formatter
/// could not be executed or failed.
pub fn format_generated_code(target: crate::codegen::Target, code: &str) -> Result<String, String> {
    match target {
        crate::codegen::Target::Rust => run_formatter("rustfmt", &["--edition", "2021"], code),
        crate::codegen::Target::Go => run_formatter("gofmt", &[], code),
        crate::codegen::Target::TypeScript => {
            // Try prettier first (most common), fall back to deno fmt if available
            if let Ok(formatted) = run_formatter("prettier", &["--parser", "typescript"], code) {
                return Ok(formatted);
            }
            run_formatter("deno", &["fmt", "--ext", "ts", "-"], code)
        }
        crate::codegen::Target::Faber => {
            // The Faber emitter is already the pretty-printer.
            // In the future we can hook a dedicated `faber fmt` here if one is added.
            Ok(code.to_string())
        }
    }
}

/// Run a linter with auto-fix on the generated code where possible.
///
/// This is intentionally best-effort. If the linter is not installed or fails,
/// we return an error and the caller can decide to keep the original code.
pub fn lint_generated_code(target: crate::codegen::Target, code: &str) -> Result<String, String> {
    match target {
        crate::codegen::Target::Rust => lint_rust_code(code),
        crate::codegen::Target::Go => {
            // Go has limited auto-fix linters. For now we can run `golangci-lint --fix` if present,
            // but a simple first version is to just return the code (or run gofmt again).
            // Future: implement proper golangci-lint support.
            Ok(code.to_string())
        }
        crate::codegen::Target::TypeScript => {
            // Try biome or eslint --fix
            if let Ok(fixed) = run_formatter("biome", &["check", "--apply", "--stdin-file-path", "main.ts"], code) {
                return Ok(fixed);
            }
            run_formatter(
                "eslint",
                &[
                    "--fix-dry-run",
                    "--stdin",
                    "--stdin-filename",
                    "main.ts",
                    "--format",
                    "json",
                ],
                code,
            )
            .map(|_| code.to_string()) // eslint --fix-dry-run doesn't rewrite; real --fix needs files
        }
        crate::codegen::Target::Faber => Ok(code.to_string()),
    }
}

/// Rust-specific linter using a temporary Cargo project + clippy --fix.
/// This is the most powerful auto-fix we can currently offer.
fn lint_rust_code(code: &str) -> Result<String, String> {
    use std::fs;
    use std::process::Command;

    // Create a unique temp directory (similar to make_temp_root)
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("radix-lint-{}", nanos));
    let src_dir = temp_dir.join("src");

    fs::create_dir_all(&src_dir).map_err(|e| format!("failed to create temp src dir: {e}"))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, code).map_err(|e| format!("failed to write temp main.rs: {e}"))?;

    let cargo_toml = temp_dir.join("Cargo.toml");
    fs::write(
        &cargo_toml,
        "[package]\nname = \"lint-target\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .map_err(|e| format!("failed to write Cargo.toml: {e}"))?;

    // Run cargo clippy --fix (best effort)
    let status = Command::new("cargo")
        .args([
            "clippy",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--allow-no-vcs",
            "--quiet",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(&temp_dir)
        .status();

    // Clean up temp dir (ignore errors)
    let _ = fs::remove_dir_all(&temp_dir);

    match status {
        Ok(s) if s.success() => {
            let fixed = fs::read_to_string(&main_rs).map_err(|e| format!("failed to read fixed code: {e}"))?;
            Ok(fixed)
        }
        Ok(s) => Err(format!("cargo clippy --fix exited with status {s}")),
        Err(e) => Err(format!("failed to run cargo clippy: {e} (is clippy installed?)")),
    }
}

/// Helper to invoke an external formatter via stdin/stdout.
fn run_formatter(cmd: &str, args: &[&str], input: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not spawn {cmd}: {e} (is it installed?)"))?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "failed to open stdin".to_string())?;
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("failed to write to {cmd} stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for {cmd}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn annotation_json(annotations: &[crate::syntax::Annotation]) -> String {
    annotations
        .iter()
        .map(|annotation| {
            let kind = match &annotation.kind {
                crate::syntax::AnnotationKind::Cli(_) => "Cli",
                crate::syntax::AnnotationKind::Imperium(_) => "Imperium",
                crate::syntax::AnnotationKind::Optio(_) => "Optio",
                crate::syntax::AnnotationKind::Operandus(_) => "Operandus",
                crate::syntax::AnnotationKind::Statement(_) => "Statement",
                crate::syntax::AnnotationKind::Innatum(_) => "Innatum",
                crate::syntax::AnnotationKind::Subsidia(_) => "Subsidia",
                crate::syntax::AnnotationKind::Radix(_) => "Radix",
                crate::syntax::AnnotationKind::Verte(_) => "Verte",
                crate::syntax::AnnotationKind::Externa => "Externa",
                crate::syntax::AnnotationKind::Futura => "Futura",
                crate::syntax::AnnotationKind::Cursor => "Cursor",
                crate::syntax::AnnotationKind::Tag => "Tag",
                crate::syntax::AnnotationKind::Solum => "Solum",
                crate::syntax::AnnotationKind::Omitte => "Omitte",
                crate::syntax::AnnotationKind::Metior => "Metior",
                crate::syntax::AnnotationKind::Publica => "Publica",
                crate::syntax::AnnotationKind::Protecta => "Protecta",
                crate::syntax::AnnotationKind::Privata => "Privata",
            };
            format!(
                "{{ \"kind\": \"{}\", \"span\": [{}, {}] }}",
                kind, annotation.span.start, annotation.span.end
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cli_analysis_json(analysis: &crate::cli::CliAnalysis) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"mode\": \"{}\",\n", cli_mode_name(&analysis.mode)));
    out.push_str(&format!("  \"success\": {},\n", analysis.errors.is_empty()));
    if let Some(program) = &analysis.program {
        out.push_str("  \"program\": ");
        out.push_str(&cli_program_json(program, 2));
        out.push_str(",\n");
    } else {
        out.push_str("  \"program\": null,\n");
    }
    out.push_str("  \"errors\": [");
    for (i, err) in analysis.errors.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{ \"message\": \"{}\", \"span\": [{}, {}] }}",
            escape_json(&err.message),
            err.span.start,
            err.span.end
        ));
    }
    out.push_str("]\n}");
    out
}

fn cli_program_json(program: &crate::cli::CliProgram, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("{inner}\"name\": \"{}\",\n", escape_json(&program.name)));
    out.push_str(&format!("{inner}\"entry_args\": \"{}\",\n", escape_json(&program.entry_args)));
    out.push_str(&format!("{inner}\"mode\": \"{}\",\n", cli_mode_name(&program.mode)));
    out.push_str(&format!(
        "{inner}\"version\": {},\n",
        json_string_opt(program.version.as_deref())
    ));
    out.push_str(&format!(
        "{inner}\"description\": {},\n",
        json_string_opt(program.description.as_deref())
    ));
    out.push_str(&format!(
        "{inner}\"global_options\": {},\n",
        cli_options_json(&program.global_options)
    ));
    out.push_str(&format!(
        "{inner}\"global_operands\": {},\n",
        cli_operands_json(&program.global_operands)
    ));
    out.push_str(&format!("{inner}\"options\": {},\n", cli_options_json(&program.options)));
    out.push_str(&format!("{inner}\"operands\": {},\n", cli_operands_json(&program.operands)));
    out.push_str(&format!("{inner}\"commands\": {}\n", cli_commands_json(&program.commands)));
    out.push_str(&format!("{pad}}}"));
    out
}

fn cli_commands_json(commands: &[crate::cli::CliCommand]) -> String {
    format!(
        "[{}]",
        commands
            .iter()
            .map(|command| {
                format!(
                    "{{ \"path\": [{}], \"function\": \"{}\", \"aliases\": [{}], \"description\": {}, \"options\": {}, \"operands\": {} }}",
                    command
                        .path
                        .iter()
                        .map(|part| format!("\"{}\"", escape_json(part)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    escape_json(&command.function),
                    command
                        .aliases
                        .iter()
                        .map(|alias| format!("\"{}\"", escape_json(alias)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    json_string_opt(command.description.as_deref()),
                    cli_options_json(&command.options),
                    cli_operands_json(&command.operands)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn cli_options_json(options: &[crate::cli::CliOption]) -> String {
    format!(
        "[{}]",
        options
            .iter()
            .map(|option| {
                format!(
                    "{{ \"binding\": \"{}\", \"type\": \"{}\", \"short\": {}, \"long\": {}, \"global\": {}, \"flag\": {}, \"default\": {} }}",
                    escape_json(&option.binding),
                    cli_type_name(&option.ty),
                    json_string_opt(option.short.as_deref()),
                    json_string_opt(option.long.as_deref()),
                    option.global,
                    option.flag,
                    cli_default_json(option.default.as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn cli_operands_json(operands: &[crate::cli::CliOperand]) -> String {
    format!(
        "[{}]",
        operands
            .iter()
            .map(|operand| {
                format!(
                    "{{ \"binding\": \"{}\", \"type\": \"{}\", \"rest\": {}, \"global\": {}, \"default\": {} }}",
                    escape_json(&operand.binding),
                    cli_type_name(&operand.ty),
                    operand.rest,
                    operand.global,
                    cli_default_json(operand.default.as_ref())
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn cli_default_json(default: Option<&crate::cli::CliDefault>) -> String {
    match default {
        Some(crate::cli::CliDefault::Text(value)) => {
            format!("{{ \"kind\": \"text\", \"value\": \"{}\" }}", escape_json(value))
        }
        Some(crate::cli::CliDefault::Integer(value)) => format!("{{ \"kind\": \"integer\", \"value\": {} }}", value),
        Some(crate::cli::CliDefault::Float(value)) => format!("{{ \"kind\": \"float\", \"value\": {} }}", value),
        Some(crate::cli::CliDefault::Bool(value)) => format!("{{ \"kind\": \"bool\", \"value\": {} }}", value),
        Some(crate::cli::CliDefault::Nil) => "{ \"kind\": \"nil\" }".to_owned(),
        Some(crate::cli::CliDefault::Expr(value)) => {
            format!("{{ \"kind\": \"expr\", \"value\": \"{}\" }}", escape_json(value))
        }
        None => "null".to_owned(),
    }
}

fn cli_mode_name(mode: &crate::cli::CliMode) -> &'static str {
    match mode {
        crate::cli::CliMode::NotCli => "not-cli",
        crate::cli::CliMode::SingleCommand => "single-command",
        crate::cli::CliMode::Subcommand => "subcommand",
    }
}

fn cli_type_name(ty: &crate::cli::CliType) -> &'static str {
    match ty {
        crate::cli::CliType::Textus => "textus",
        crate::cli::CliType::Numerus => "numerus",
        crate::cli::CliType::Fractus => "fractus",
        crate::cli::CliType::Bivalens => "bivalens",
        crate::cli::CliType::Octeti => "octeti",
        crate::cli::CliType::Ignotum => "ignotum",
        crate::cli::CliType::ListaTextus => "lista<textus>",
        crate::cli::CliType::ListaNumerus => "lista<numerus>",
    }
}

fn json_string_opt(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_owned(),
    }
}

/// Escape special characters for JSON strings.
pub fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[path = "tool_test.rs"]
mod tests;
