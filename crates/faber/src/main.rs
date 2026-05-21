//! User-facing Faber project tool (`faber` binary).

mod package;

use clap::{Parser, Subcommand};
use radix::tool::{self, BuildCommand, CheckCommand, EmitCommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "faber",
    bin_name = "faber",
    about = "Faber project and package tool",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compile a file or package and write output to disk
    Build(radix::tool::BuildArgs),
    /// Show supported targets and current capability notes
    Targets,
    /// Run semantic analysis on a file or package
    Check(radix::tool::CheckArgs),
    /// Create a new Faber package (planned)
    Init(InitArgs),
    /// Run a compiled package (planned)
    Run(RunArgs),
    /// Run package tests (planned)
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

#[derive(clap::Args, Debug)]
struct InitArgs {
    /// Target directory for the new package
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(clap::Args, Debug)]
struct RunArgs {
    /// Package path to run
    path: PathBuf,
}

#[derive(clap::Args, Debug)]
struct TestArgs {
    /// Package path to test
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Build(args) => package::cmd_build(BuildCommand {
            input: args.input,
            out_dir: args.out_dir,
            package: args.package,
            target: args.target.into(),
        }),
        Command::Targets => tool::cmd_targets(),
        Command::Check(args) => {
            if args.package || package::should_treat_as_package_from_args(&args.input) {
                package::cmd_check_package(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
                });
            } else {
                tool::cmd_check(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
                });
            }
        }
        Command::Init(args) => cmd_init(args),
        Command::Run(args) => cmd_run(args),
        Command::Test(args) => cmd_test(args),
        Command::Lex(args) => tool::cmd_lex(&args.input),
        Command::Parse(args) => tool::cmd_parse(&args.input),
        Command::Hir(args) => tool::cmd_hir(&args.input),
        Command::CliIr(args) => tool::cmd_cli_ir(&args.input),
        Command::Emit(args) => {
            if args.package || package::should_treat_as_package_from_args(&args.input) {
                package::cmd_emit_package(EmitCommand {
                    input: args.input,
                    package: args.package,
                    target: args.target.into(),
                });
            } else {
                tool::cmd_emit(EmitCommand {
                    input: args.input,
                    package: args.package,
                    target: args.target.into(),
                });
            }
        }
    }
}

fn cmd_init(args: InitArgs) {
    eprintln!(
        "error: `faber init` is not implemented yet; create a directory with `main.fab` or a `faber.fab` manifest manually in {}",
        args.path.display()
    );
    std::process::exit(1);
}

fn cmd_run(args: RunArgs) {
    eprintln!(
        "error: `faber run` is not implemented yet; build with `faber build` and run the generated Rust artifact from {}",
        args.path.display()
    );
    std::process::exit(1);
}

fn cmd_test(args: TestArgs) {
    eprintln!(
        "error: `faber test` is not implemented yet; use `faber check` on {}",
        args.path.display()
    );
    std::process::exit(1);
}
