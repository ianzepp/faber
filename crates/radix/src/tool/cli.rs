//! Clap command shapes for `radix` and shared `faber` flag types.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

const RADIX_AFTER_HELP: &str = include_str!("../../../../docs/help/radix-after-help.md");

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
    /// Compile to target (rust, faber, ts, go, wasm-text, llvm-text)
    Emit(EmitArgs),
}

/// Clap parser for the developer-facing `radix` binary.
#[derive(Parser, Debug)]
#[command(
    name = "radix",
    bin_name = "radix",
    about = "Faber compiler developer tool",
    after_long_help = RADIX_AFTER_HELP,
    version
)]
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
    /// Compile to target (rust, faber, ts, go, wasm-text, llvm-text)
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
    #[value(name = "wasm-text", alias = "wasm", alias = "wat")]
    WasmText,

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
            CliTarget::WasmText => crate::codegen::Target::WasmText,
            CliTarget::LlvmText => crate::codegen::Target::LlvmText,
        }
    }
}
