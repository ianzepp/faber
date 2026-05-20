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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Build(args) => cmd_build(BuildCommand {
            input: args.input,
            out_dir: args.out_dir,
            package: args.package,
            target: args.target.into(),
        }),
        Command::Targets => cmd_targets(),
        Command::Lex(args) => cmd_lex(&args.input),
        Command::Parse(args) => cmd_parse(&args.input),
        Command::Hir(args) => cmd_hir(&args.input),
        Command::CliIr(args) => cmd_cli_ir(&args.input),
        Command::Check(args) => cmd_check(CheckCommand { input: args.input, permissive: args.permissive }),
        Command::Emit(args) => {
            cmd_emit(EmitCommand { input: args.input, package: args.package, target: args.target.into() })
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "radix", about = "Faber compiler", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
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

#[derive(Args, Debug)]
struct InputArgs {
    /// Input file path, or '-' / omitted for stdin
    input: Vec<String>,
}

#[derive(Args, Debug)]
struct CheckArgs {
    /// Downgrade unresolved/import-driven semantic errors to warnings
    #[arg(long)]
    permissive: bool,

    /// Input file path, or '-' / omitted for stdin
    input: Vec<String>,
}

#[derive(Args, Debug)]
struct EmitArgs {
    /// Output target language
    #[arg(short = 't', long = "target", value_enum, default_value_t = CliTarget::Rust)]
    target: CliTarget,

    /// Force package compilation mode
    #[arg(long)]
    package: bool,

    /// Input file or package path, or '-' / omitted for stdin
    input: Vec<String>,
}

#[derive(Args, Debug)]
struct BuildArgs {
    /// Output target language
    #[arg(short = 't', long = "target", value_enum, default_value_t = CliTarget::Rust)]
    target: CliTarget,

    /// Output directory for generated files
    #[arg(short = 'o', long = "out-dir", default_value = ".")]
    out_dir: PathBuf,

    /// Force package compilation mode
    #[arg(long)]
    package: bool,

    /// Input file or package path
    input: String,
}

#[derive(Debug)]
struct CheckCommand {
    input: Vec<String>,
    permissive: bool,
}

#[derive(Debug)]
struct EmitCommand {
    input: Vec<String>,
    package: bool,
    target: radix::codegen::Target,
}

#[derive(Debug)]
struct BuildCommand {
    input: String,
    out_dir: PathBuf,
    package: bool,
    target: radix::codegen::Target,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum CliTarget {
    #[default]
    Rust,
    #[value(alias = "fab")]
    Faber,
    #[value(name = "ts", alias = "typescript")]
    TypeScript,
    Go,
}

impl From<CliTarget> for radix::codegen::Target {
    fn from(value: CliTarget) -> Self {
        match value {
            CliTarget::Rust => radix::codegen::Target::Rust,
            CliTarget::Faber => radix::codegen::Target::Faber,
            CliTarget::TypeScript => radix::codegen::Target::TypeScript,
            CliTarget::Go => radix::codegen::Target::Go,
        }
    }
}

struct TargetCapabilities {
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
fn read_source(args: &[String]) -> (String, String) {
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

fn source_file_from_input(name: String, source: String) -> radix::driver::SourceFile {
    radix::driver::SourceFile::inline(name, source)
}

fn format_location(source_file: &radix::driver::SourceFile, offset: u32) -> String {
    let (line, column) = source_file.offset_to_line_col(offset);
    format!("{}:{}:{}", source_file.name.as_str(), line, column)
}

/// Tokenize source and emit JSON.
///
/// WHY: Enables inspection of lexer output for debugging tokenization issues.
fn cmd_lex(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let result = radix::lexer::lex(source_file.content.as_str());

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
fn cmd_parse(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let lex_result = radix::lexer::lex(source_file.content.as_str());

    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);

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
fn cmd_hir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    // WHY: HIR lowering requires lexing, parsing, and name resolution
    let lex_result = radix::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let radix::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let mut resolver = radix::semantic::Resolver::new();
    let mut types = radix::semantic::TypeTable::new();

    if let Err(e) = radix::semantic::passes::collect::collect(&program, &mut resolver, &mut types) {
        eprintln!("collection errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    if let Err(e) = radix::semantic::passes::resolve::resolve(&program, &mut resolver, &interner, &mut types) {
        eprintln!("resolution errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    let (hir, errors) = radix::hir::lower(&program, &resolver, &mut types, &interner);

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

fn cmd_cli_ir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    let lex_result = radix::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let radix::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = radix::cli::analyze(&program, &interner);
    println!("{}", cli_analysis_json(&cli_analysis));

    if !cli_analysis.errors.is_empty() {
        std::process::exit(1);
    }
}

/// Run semantic analysis.
///
/// WHY: --permissive mode allows checking files with unresolved imports,
/// useful for partial compilation or library development.
fn cmd_check(command: CheckCommand) {
    let (name, source) = read_source(&command.input);
    let source_file = source_file_from_input(name, source);

    let lex_result = radix::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        for err in &lex_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            eprintln!("{}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let radix::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = radix::cli::analyze(&program, &interner);
    let mut cli_fatal_errors = 0usize;
    for err in &cli_analysis.errors {
        eprintln!("error: {}: {}", format_location(&source_file, err.span.start), err.message);
        cli_fatal_errors += 1;
    }
    if cli_fatal_errors > 0 {
        std::process::exit(1);
    }

    let pass_config = radix::semantic::PassConfig::for_target(radix::codegen::Target::Rust);
    let semantic_result =
        radix::semantic::analyze_with_cli(&program, &pass_config, &interner, cli_analysis.program.as_ref());

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

/// Compile to target language.
///
/// WHY: End-to-end compilation command. Accepts -t flag to select Rust or
/// Faber pretty-print output.
fn cmd_emit(command: EmitCommand) {
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

    println!("{}", output_code(output));
}

fn cmd_build(command: BuildCommand) {
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

    let code = output_code(output);

    fs::write(&output_path, code).unwrap_or_else(|err| {
        eprintln!("error: failed to write '{}': {}", output_path.display(), err);
        std::process::exit(1);
    });

    println!("{}", output_path.display());
}

fn cmd_targets() {
    for target in [
        radix::codegen::Target::Rust,
        radix::codegen::Target::Go,
        radix::codegen::Target::TypeScript,
        radix::codegen::Target::Faber,
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

fn should_treat_as_package(path: &std::path::Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("faber.fab")
}

fn resolve_package_mode(path: &std::path::Path, force_package: bool) -> bool {
    force_package || should_treat_as_package(path)
}

fn compile_cli_input(input: &[String], package: bool, target: radix::codegen::Target) -> radix::CompileResult {
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

fn compile_cli_path(path: &std::path::Path, package: bool, target: radix::codegen::Target) -> radix::CompileResult {
    let config = radix::driver::Config::default().with_target(target);
    let compiler = radix::Compiler::new(config);
    if package {
        compiler.compile_package(path)
    } else {
        compiler.compile(path)
    }
}

fn compile_cli_source(name: &str, source: &str, target: radix::codegen::Target) -> radix::CompileResult {
    let config = radix::driver::Config::default().with_target(target);
    let compiler = radix::Compiler::new(config);
    compiler.compile_str(name, source)
}

fn build_output_path(
    out_dir: &std::path::Path,
    input_path: &std::path::Path,
    target: radix::codegen::Target,
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

fn target_extension(target: radix::codegen::Target) -> &'static str {
    match target {
        radix::codegen::Target::Rust => "rs",
        radix::codegen::Target::Faber => "fab",
        radix::codegen::Target::TypeScript => "ts",
        radix::codegen::Target::Go => "go",
    }
}

fn target_name(target: radix::codegen::Target) -> &'static str {
    match target {
        radix::codegen::Target::Rust => "rust",
        radix::codegen::Target::Faber => "faber",
        radix::codegen::Target::TypeScript => "ts",
        radix::codegen::Target::Go => "go",
    }
}

fn target_capabilities(target: radix::codegen::Target) -> TargetCapabilities {
    match target {
        radix::codegen::Target::Rust => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: true,
            note: "primary backend; package compilation supported",
        },
        radix::codegen::Target::Go => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        radix::codegen::Target::TypeScript => TargetCapabilities {
            check: true,
            build: true,
            run: false,
            package: false,
            note: "file emission supported; package compilation not yet supported",
        },
        radix::codegen::Target::Faber => TargetCapabilities {
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

fn output_code(output: radix::Output) -> String {
    match output {
        radix::Output::Rust(out) => out.code,
        radix::Output::Faber(out) => out.code,
        radix::Output::TypeScript(out) => out.code,
        radix::Output::Go(out) => out.code,
    }
}

fn annotation_json(annotations: &[radix::syntax::Annotation]) -> String {
    annotations
        .iter()
        .map(|annotation| {
            let kind = match &annotation.kind {
                radix::syntax::AnnotationKind::Cli(_) => "Cli",
                radix::syntax::AnnotationKind::Imperium(_) => "Imperium",
                radix::syntax::AnnotationKind::Optio(_) => "Optio",
                radix::syntax::AnnotationKind::Operandus(_) => "Operandus",
                radix::syntax::AnnotationKind::Statement(_) => "Statement",
                radix::syntax::AnnotationKind::Innatum(_) => "Innatum",
                radix::syntax::AnnotationKind::Subsidia(_) => "Subsidia",
                radix::syntax::AnnotationKind::Radix(_) => "Radix",
                radix::syntax::AnnotationKind::Verte(_) => "Verte",
                radix::syntax::AnnotationKind::Externa => "Externa",
                radix::syntax::AnnotationKind::Futura => "Futura",
                radix::syntax::AnnotationKind::Cursor => "Cursor",
                radix::syntax::AnnotationKind::Tag => "Tag",
                radix::syntax::AnnotationKind::Solum => "Solum",
                radix::syntax::AnnotationKind::Omitte => "Omitte",
                radix::syntax::AnnotationKind::Metior => "Metior",
                radix::syntax::AnnotationKind::Publica => "Publica",
                radix::syntax::AnnotationKind::Protecta => "Protecta",
                radix::syntax::AnnotationKind::Privata => "Privata",
            };
            format!(
                "{{ \"kind\": \"{}\", \"span\": [{}, {}] }}",
                kind, annotation.span.start, annotation.span.end
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cli_analysis_json(analysis: &radix::cli::CliAnalysis) -> String {
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

fn cli_program_json(program: &radix::cli::CliProgram, indent: usize) -> String {
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

fn cli_commands_json(commands: &[radix::cli::CliCommand]) -> String {
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

fn cli_options_json(options: &[radix::cli::CliOption]) -> String {
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

fn cli_operands_json(operands: &[radix::cli::CliOperand]) -> String {
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

fn cli_default_json(default: Option<&radix::cli::CliDefault>) -> String {
    match default {
        Some(radix::cli::CliDefault::Text(value)) => {
            format!("{{ \"kind\": \"text\", \"value\": \"{}\" }}", escape_json(value))
        }
        Some(radix::cli::CliDefault::Integer(value)) => format!("{{ \"kind\": \"integer\", \"value\": {} }}", value),
        Some(radix::cli::CliDefault::Float(value)) => format!("{{ \"kind\": \"float\", \"value\": {} }}", value),
        Some(radix::cli::CliDefault::Bool(value)) => format!("{{ \"kind\": \"bool\", \"value\": {} }}", value),
        Some(radix::cli::CliDefault::Nil) => "{ \"kind\": \"nil\" }".to_owned(),
        Some(radix::cli::CliDefault::Expr(value)) => {
            format!("{{ \"kind\": \"expr\", \"value\": \"{}\" }}", escape_json(value))
        }
        None => "null".to_owned(),
    }
}

fn cli_mode_name(mode: &radix::cli::CliMode) -> &'static str {
    match mode {
        radix::cli::CliMode::NotCli => "not-cli",
        radix::cli::CliMode::SingleCommand => "single-command",
        radix::cli::CliMode::Subcommand => "subcommand",
    }
}

fn cli_type_name(ty: &radix::cli::CliType) -> &'static str {
    match ty {
        radix::cli::CliType::Textus => "textus",
        radix::cli::CliType::Numerus => "numerus",
        radix::cli::CliType::Fractus => "fractus",
        radix::cli::CliType::Bivalens => "bivalens",
        radix::cli::CliType::Octeti => "octeti",
        radix::cli::CliType::Ignotum => "ignotum",
        radix::cli::CliType::ListaTextus => "lista<textus>",
        radix::cli::CliType::ListaNumerus => "lista<numerus>",
    }
}

fn json_string_opt(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_owned(),
    }
}

/// Escape special characters for JSON strings.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
#[path = "main_test.rs"]
mod tests;
