//! Clap command shapes for the `faber` binary.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

const FABER_AFTER_HELP: &str = include_str!("../../../../docs/help/faber-after-help.md");

/// Root parser for the `faber` binary.
#[derive(Parser, Debug)]
#[command(
    name = "faber",
    bin_name = "faber",
    about = "Faber project and package tool",
    after_long_help = FABER_AFTER_HELP,
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// User-facing `faber` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Compile a file or package and write output to disk
    Build(radix::tool::BuildArgs),

    /// Show supported targets and current capability notes
    Targets,

    /// Run semantic analysis on a file or package
    Check(radix::tool::CheckArgs),

    /// Create a new Faber package
    Init(InitArgs),

    /// Explain a Faber glyph, keyword, or grammar term
    Explain(ExplainArgs),

    /// Build (if needed) and run a compiled package
    Run(RunArgs),

    /// Run package tests via the generated Rust test harness (Cargo-backed)
    Test(TestArgs),

    /// Tokenize source and output JSON (compatibility alias for `radix lex`)
    Lex(radix::tool::InputArgs),

    /// Parse source and output AST as JSON (compatibility alias for `radix parse`)
    Parse(radix::tool::InputArgs),

    /// Lower AST to HIR and output as JSON (compatibility alias for `radix hir`)
    Hir(radix::tool::InputArgs),

    /// Validate and output normalized CLI IR as JSON (compatibility alias for `radix cli-ir`)
    CliIr(radix::tool::InputArgs),

    /// Compile to target for stdout (compatibility alias for `radix emit`)
    Emit(radix::tool::EmitArgs),
}

/// Arguments for `faber init`.
#[derive(clap::Args, Debug)]
pub struct InitArgs {
    /// Target directory for the new package
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

/// Arguments for `faber explain`.
#[derive(clap::Args, Debug)]
pub struct ExplainArgs {
    /// Emit a machine-readable JSON explanation
    #[arg(long)]
    pub json: bool,

    /// Search across explain entries and show ranked matches
    #[arg(long)]
    pub search: Option<String>,

    /// List canonical explain terms
    #[arg(long)]
    pub list: bool,

    /// List canonical and legacy entries in a category
    #[arg(long)]
    pub category: Option<String>,

    /// Term, alias, or legacy spelling to explain
    pub term: Option<String>,
}

/// Arguments for `faber run`.
#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Package path to run (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Run the release binary
    #[arg(long)]
    pub release: bool,

    /// Arguments passed to the executed program (after --)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

/// Arguments for `faber test`.
#[derive(clap::Args, Debug)]
pub struct TestArgs {
    /// Package path to test
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Test name filter passed to the Rust test harness (matches on generated proba_* names)
    #[arg(value_name = "FILTER")]
    pub filter: Option<String>,

    /// Select tests by source-level proba name
    #[arg(long)]
    pub name: Option<String>,

    /// Select tests by source-level probandum suite path, joined with `/`
    #[arg(long)]
    pub suite: Option<String>,

    /// Select tests by source-level tag modifier
    #[arg(long)]
    pub tag: Option<String>,

    /// Run only tests whose name exactly matches the filter
    #[arg(long)]
    pub exact: bool,

    /// Show test output (do not capture stdout/stderr from test bodies)
    #[arg(long)]
    pub nocapture: bool,

    /// Limit the number of test threads used by the harness
    #[arg(long, value_name = "N")]
    pub test_threads: Option<usize>,

    /// Only run Rust-ignored tests, including `omitte` / `futurum` and selection-ignored cases
    #[arg(long, conflicts_with = "include_ignored")]
    pub ignored: bool,

    /// Run normal tests and Rust-ignored tests
    #[arg(long, conflicts_with = "ignored")]
    pub include_ignored: bool,
}
