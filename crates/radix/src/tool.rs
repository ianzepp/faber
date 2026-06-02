//! Command implementation layer for Faber CLI surfaces.
//!
//! This module is the executable boundary around the `radix` compiler library.
//! It owns clap command shapes, stdin/file source loading, terminal diagnostics,
//! JSON-ish inspection output, target formatting/linting hooks, and the policy
//! split between the developer `radix` binary and the user-facing `faber`
//! package tool.
//!
//! `radix` remains a single-file compiler and phase-inspection tool. Package
//! compilation is intentionally rejected here and delegated to `crates/faber`,
//! where manifests, import graphs, stdlib binding, and generated Cargo layouts
//! are available. That separation keeps compiler phase debugging lightweight
//! while preventing the developer tool from growing a second package policy.
//!
//! ERROR STRATEGY
//! ==============
//! The command functions are process-facing: they print diagnostics and call
//! `std::process::exit` on fatal errors. Reusable helpers such as
//! [`mir_output_for_source`], [`compile_cli_source`], and formatter/linter
//! wrappers return values so tests and wrappers can exercise the same policy
//! without spawning a binary.
//!
//! INVARIANTS
//! ==========
//! - Stdin is valid for single-file commands and invalid for package mode.
//! - `radix` package requests fail fast with a message pointing to `faber`.
//! - Inspection commands expose deterministic, machine-readable output for
//!   tests and tools rather than pretty terminal prose.
//! - Formatting and linting are best-effort post-processing steps; failures are
//!   warnings that leave generated compiler output available.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Clap parser for the user-facing `faber` binary.
///
/// Execution is intentionally split into `crates/faber`; this type remains here
/// so both binaries share flag spelling and target parsing.
#[derive(Parser, Debug)]
#[command(name = "faber", bin_name = "faber", about = "Faber compiler", version)]
pub struct FaberCli {
    /// User-facing command selected by the `faber` binary.
    #[command(subcommand)]
    pub command: FaberCommand,
}

/// User-facing CLI command grammar.
///
/// The shape lives beside `RadixCommand` because both binaries share parsing
/// types and command payloads, but package-aware execution is owned by
/// `crates/faber` rather than this module.
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
    /// Compile to target (rust, faber, ts, go, wasm, llvm-text)
    Emit(EmitArgs),
}

/// Clap parser for the developer-facing `radix` binary.
#[derive(Parser, Debug)]
#[command(name = "radix", bin_name = "radix", about = "Faber compiler developer tool", version)]
pub struct RadixCli {
    /// Developer command selected by the `radix` binary.
    #[command(subcommand)]
    pub command: RadixCommand,
}

/// Developer CLI command grammar for direct compiler phase inspection.
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
    /// Compile to target (rust, faber, ts, go, wasm, llvm-text)
    Emit(EmitArgs),
    /// Show supported targets and current capability notes
    Targets,
}

/// Shared source-input grammar for phase inspection commands.
#[derive(Args, Debug)]
pub struct InputArgs {
    /// Input file path, or '-' / omitted for stdin
    pub input: Vec<String>,
}

/// Parsed `check` command payload after clap normalization.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Print expanded phase-aware diagnostics instead of normal check output
    #[arg(long)]
    pub diagnostics: bool,

    /// Downgrade unresolved/import-driven semantic errors to warnings
    #[arg(long)]
    pub permissive: bool,

    /// Force package checking mode
    #[arg(long)]
    pub package: bool,

    /// Input file or package path, or '-' / omitted for stdin
    pub input: Vec<String>,
}

/// Parsed `emit` command payload after clap normalization.
#[derive(Args, Debug)]
pub struct EmitArgs {
    /// Print expanded phase-aware diagnostics instead of normal emit diagnostics
    #[arg(long)]
    pub diagnostics: bool,

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

/// Parsed `build` command payload after clap normalization.
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

/// Diagnostic reporting style for command paths that support expanded records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticMode {
    /// Preserve the command's existing human terminal output.
    Normal,

    /// Print one expanded phase-aware record per normalized diagnostic.
    Diagnostics,
}

/// Process-independent check command contract used by command dispatch.
#[derive(Debug)]
pub struct CheckCommand {
    /// Input file path, `-`, or empty for stdin.
    pub input: Vec<String>,

    /// Whether the caller requested package mode.
    pub package: bool,

    /// Whether selected unresolved/import-driven semantic errors become warnings.
    pub permissive: bool,

    /// Reporting style for normalized diagnostics.
    pub diagnostic_mode: DiagnosticMode,
}

/// Process-independent emit command contract used by command dispatch.
#[derive(Debug)]
pub struct EmitCommand {
    /// Input file path, `-`, or empty for stdin.
    pub input: Vec<String>,

    /// Whether the caller requested package mode.
    pub package: bool,

    /// Backend target for emitted source.
    pub target: crate::codegen::Target,

    /// Whether to run the target formatter before printing.
    pub format: bool,

    /// Whether to run target lint auto-fix before printing.
    pub linter: bool,

    /// Reporting style for normalized diagnostics.
    pub diagnostic_mode: DiagnosticMode,
}

/// Process-independent build command contract used by command dispatch.
#[derive(Debug)]
pub struct BuildCommand {
    /// Input file or package path.
    pub input: String,

    /// Directory where generated source should be written.
    pub out_dir: PathBuf,

    /// Whether the caller requested package mode.
    pub package: bool,

    /// Whether to request a release build profile when applicable.
    pub release: bool,

    /// Backend target for generated source.
    pub target: crate::codegen::Target,

    /// Whether to run the target formatter before writing.
    pub format: bool,

    /// Whether to run target lint auto-fix before writing.
    pub linter: bool,
}

/// Target names accepted by CLI flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum CliTarget {
    /// Rust backend.
    #[default]
    Rust,

    /// Canonical Faber pretty-printer.
    #[value(alias = "fab")]
    Faber,

    /// TypeScript backend.
    #[value(name = "ts", alias = "typescript")]
    TypeScript,

    /// Go backend.
    Go,

    /// Experimental MIR-backed WebAssembly text target.
    #[value(name = "wasm", alias = "wat")]
    Wasm,

    /// Experimental MIR-backed LLVM text target.
    #[value(name = "llvm-text", alias = "llvm-ir", alias = "llvm")]
    LlvmText,
}

impl From<CliTarget> for crate::codegen::Target {
    fn from(value: CliTarget) -> Self {
        match value {
            CliTarget::Rust => crate::codegen::Target::Rust,
            CliTarget::Faber => crate::codegen::Target::Faber,
            CliTarget::TypeScript => crate::codegen::Target::TypeScript,
            CliTarget::Go => crate::codegen::Target::Go,
            CliTarget::Wasm => crate::codegen::Target::Wasm,
            CliTarget::LlvmText => crate::codegen::Target::LlvmText,
        }
    }
}

/// Capability row printed by `targets`.
///
/// The fields are private because callers should use `target_capabilities` and
/// `cmd_targets` rather than depending on this display schema as a stable API.
pub struct TargetCapabilities {
    check: bool,
    build: bool,
    run: bool,
    package: bool,
    note: &'static str,
}

/// Read source from a file argument or stdin.
///
/// This is the process-facing loader for single-file commands. It exits on I/O
/// failure because command handlers already report diagnostics through stderr,
/// while library callers should use `Compiler` directly for non-exiting flows.
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

/// Format an offset in a loaded source file for terminal diagnostics.
pub fn format_location(source_file: &crate::driver::SourceFile, offset: u32) -> String {
    let (line, column) = source_file.offset_to_line_col(offset);
    format!("{}:{}:{}", source_file.name.as_str(), line, column)
}

fn format_optional_location(source_file: &crate::driver::SourceFile, span: Option<crate::lexer::Span>) -> String {
    span.map(|span| format_location(source_file, span.start))
        .unwrap_or_else(|| source_file.name.clone())
}

/// Tokenize source and emit a compact JSON inspection payload.
///
/// The output intentionally exposes token kinds and spans rather than formatted
/// source so lexer regressions can be tested without backend involvement.
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

/// Parse source and emit a compact AST inspection payload.
///
/// This command stops after parsing and reports lexer/parser diagnostics
/// directly, making it useful for grammar work where semantic phases would add
/// distracting failures.
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

/// Lower parsed source to HIR and emit a compact inspection payload.
///
/// HIR inspection runs the prerequisite collection and resolution passes before
/// lowering so IDs and spans reflect the same semantic inputs used by codegen.
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

/// Lower checked source to MIR and print the deterministic MIR dump.
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

/// Produce MIR inspection text without exiting the process.
///
/// Errors are returned already formatted for terminal display because MIR
/// inspection is primarily a developer-tool surface, not a library data model.
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

/// Analyze CLI annotations and print the normalized CLI IR.
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

/// Run semantic analysis for a single-file input.
///
/// `radix check` rejects package mode because package graph loading and stdlib
/// binding belong to the `faber` tool. `--permissive` exists for partial files
/// and library-development workflows where unresolved/import-driven errors
/// should be visible but not fatal.
pub fn cmd_check(command: CheckCommand) {
    if command.package || should_treat_as_package_from_input(&command.input) {
        eprintln!("error: package checking is owned by the `faber` tool; rerun with `faber check --package`");
        std::process::exit(1);
    }

    let (name, source) = read_source(&command.input);
    let source_file = source_file_from_input(name, source);

    if command.diagnostic_mode == DiagnosticMode::Diagnostics {
        let result = check_diagnostics_for_source(&source_file.name, &source_file.content, command.permissive);
        if !result.diagnostics.is_empty() {
            eprintln!("{}", crate::diagnostics::render_expanded_diagnostics(&result.diagnostics));
        }
        if result.success {
            eprintln!("ok: {}", source_file.name.as_str());
        } else {
            std::process::exit(1);
        }
        return;
    }

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

/// Result of running check through normalized diagnostics.
#[derive(Debug)]
pub struct CheckDiagnosticsResult {
    /// Normalized diagnostics collected before check completion.
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,

    /// Whether check policy accepts the input after diagnostics are considered.
    pub success: bool,
}

/// Run single-file check and collect normalized diagnostics without exiting.
pub fn check_diagnostics_for_source(name: &str, source: &str, permissive: bool) -> CheckDiagnosticsResult {
    let mut diagnostics = Vec::new();

    let lex_result = crate::lexer::lex(source);
    if !lex_result.success() {
        diagnostics.extend(
            lex_result
                .errors
                .iter()
                .map(|err| crate::diagnostics::Diagnostic::from_lex_error(name, source, err)),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        diagnostics.extend(
            parse_result
                .errors
                .iter()
                .map(|err| crate::diagnostics::Diagnostic::from_parse_error(name, source, err)),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        diagnostics.push(
            crate::diagnostics::Diagnostic::error("successful parse result missing program")
                .with_phase(crate::diagnostics::DiagnosticPhase::Parse)
                .with_file(name.to_owned()),
        );
        return CheckDiagnosticsResult { diagnostics, success: false };
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    diagnostics.extend(
        cli_analysis
            .errors
            .iter()
            .map(|err| crate::diagnostics::Diagnostic::from_semantic_error(name, source, err)),
    );
    if diagnostics
        .iter()
        .any(crate::diagnostics::Diagnostic::is_error)
    {
        return CheckDiagnosticsResult { diagnostics, success: false };
    }

    let pass_config = crate::semantic::PassConfig::for_target(crate::codegen::Target::Rust);
    let semantic_result =
        crate::semantic::analyze_with_cli(&program, &pass_config, &interner, cli_analysis.program.as_ref());

    let mut fatal_errors = 0usize;
    let mut downgraded = 0usize;
    for err in &semantic_result.errors {
        let downgraded_error = permissive && err.kind.is_permissive_check_downgrade();
        let mut diagnostic = crate::diagnostics::Diagnostic::from_semantic_error(name, source, err);
        if downgraded_error {
            diagnostic = diagnostic.with_severity(crate::diagnostics::Severity::Warning);
            downgraded += 1;
        } else if err.is_error() {
            fatal_errors += 1;
        }
        diagnostics.push(diagnostic);
    }

    if permissive && downgraded > 0 {
        diagnostics.push(
            crate::diagnostics::Diagnostic::warning(format!(
                "downgraded {} unresolved/import-driven semantic error(s) in permissive mode",
                downgraded
            ))
            .with_phase(crate::diagnostics::DiagnosticPhase::Tool)
            .with_file(name.to_owned()),
        );
    }

    CheckDiagnosticsResult { diagnostics, success: semantic_result.success() || (permissive && fatal_errors == 0) }
}

fn should_treat_as_package_from_input(input: &[String]) -> bool {
    if input.is_empty() || input[0] == "-" {
        return false;
    }
    let path = std::path::Path::new(&input[0]);
    should_treat_as_package(path)
}

/// Compile single-file input to a target language and print generated source.
///
/// Package inputs are rejected before compilation so users do not accidentally
/// get a single-file interpretation of a package directory or manifest.
pub fn cmd_emit(command: EmitCommand) {
    let result = compile_cli_input(&command.input, command.package, command.target);

    if command.diagnostic_mode == DiagnosticMode::Diagnostics {
        if !result.diagnostics.is_empty() {
            eprintln!("{}", crate::diagnostics::render_expanded_diagnostics(&result.diagnostics));
        }
    } else {
        for diag in &result.diagnostics {
            if diag.is_error() {
                eprintln!("error: {}", diag.message);
            } else {
                eprintln!("warning: {}", diag.message);
            }
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

/// Compile single-file input and write generated source to disk.
///
/// This is intentionally source-emission only inside `radix`; executable
/// package builds are routed through `faber`, where generated Cargo layout
/// policy is available.
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

/// Print backend capability rows for terminal discovery.
pub fn cmd_targets() {
    for target in [
        crate::codegen::Target::Rust,
        crate::codegen::Target::Go,
        crate::codegen::Target::Wasm,
        crate::codegen::Target::LlvmText,
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

/// Return whether an input path syntactically names package mode.
pub fn should_treat_as_package(path: &std::path::Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("faber.toml")
}

/// Combine explicit package mode with path-based package detection.
pub fn resolve_package_mode(path: &std::path::Path, force_package: bool) -> bool {
    force_package || should_treat_as_package(path)
}

/// Compile command input after applying stdin and package-mode policy.
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

/// Compile a single file path through the public compiler API.
pub fn compile_cli_path(path: &std::path::Path, package: bool, target: crate::codegen::Target) -> crate::CompileResult {
    if package || should_treat_as_package(path) {
        eprintln!("error: package compilation is owned by the `faber` tool; rerun with `faber build` or `faber emit`");
        std::process::exit(1);
    }

    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile(path)
}

/// Compile in-memory command source through the public compiler API.
pub fn compile_cli_source(name: &str, source: &str, target: crate::codegen::Target) -> crate::CompileResult {
    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile_str(name, source)
}

/// Derive the output path for source-emission builds.
///
/// Package mode currently uses a stable `main.<ext>` placeholder here because
/// real package build layout is outside `radix` and owned by `faber`.
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
        crate::codegen::Target::Wasm => "wat",
        crate::codegen::Target::LlvmText => "ll",
    }
}

fn target_name(target: crate::codegen::Target) -> &'static str {
    match target {
        crate::codegen::Target::Rust => "rust",
        crate::codegen::Target::Faber => "faber",
        crate::codegen::Target::TypeScript => "ts",
        crate::codegen::Target::Go => "go",
        crate::codegen::Target::Wasm => "wasm",
        crate::codegen::Target::LlvmText => "llvm-text",
    }
}

/// Return current command-surface support for a backend target.
///
/// These rows describe CLI capability, not necessarily backend maturity. For
/// example, a target can support file emission while package compilation still
/// remains unavailable from `radix`.
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
        crate::codegen::Target::Wasm => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "experimental MIR-backed WAT probe; not a binary WASM backend yet",
        },
        crate::codegen::Target::LlvmText => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "experimental MIR-backed LLVM text probe; not native codegen yet",
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

/// Extract generated source from a target-specific compiler output.
pub fn output_code(output: crate::Output) -> String {
    match output {
        crate::Output::Rust(out) => out.code,
        crate::Output::Faber(out) => out.code,
        crate::Output::TypeScript(out) => out.code,
        crate::Output::Go(out) => out.code,
        crate::Output::Wasm(out) => out.code,
        crate::Output::LlvmText(out) => out.code,
    }
}

/// Run the appropriate formatter for generated target code, if available.
///
/// Returns the formatted code on success, or an error message if the formatter
/// could not be executed or failed. The command layer treats those errors as
/// warnings so formatter availability never changes compiler correctness.
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
        crate::codegen::Target::Wasm | crate::codegen::Target::LlvmText => Ok(code.to_string()),
    }
}

/// Run a linter with auto-fix on generated target code where possible.
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
        crate::codegen::Target::Wasm | crate::codegen::Target::LlvmText => Ok(code.to_string()),
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
    let output = Command::new("cargo")
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
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let fixed = fs::read_to_string(&main_rs).map_err(|e| format!("failed to read fixed code: {e}"))?;
            let _ = fs::remove_dir_all(&temp_dir);
            Ok(fixed)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let _ = fs::remove_dir_all(&temp_dir);
            Err(format!("cargo clippy --fix exited with status {}: {stderr}", output.status))
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&temp_dir);
            Err(format!("failed to run cargo clippy: {e} (is clippy installed?)"))
        }
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

/// Escape special characters for hand-written JSON inspection strings.
///
/// The inspection commands use narrow, deterministic JSON builders instead of a
/// public serialization contract. This helper keeps those surfaces valid for
/// strings that come from source text or diagnostics.
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
