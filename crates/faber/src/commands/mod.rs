//! Command handlers for the `faber` binary.
//!
//! Package and radix-delegated commands are dispatched here; Faber-specific
//! handlers live in [`init`], [`explain`], [`run`], and [`test`].

mod explain;
mod init;
mod run;
mod test;

use crate::cli::Command;
use crate::package;
use clap::Parser;
use radix::tool::{self, BuildCommand, CheckCommand, DiagnosticMode, EmitCommand};

use explain::cmd_explain;
use init::cmd_init;
use run::cmd_run;
use test::cmd_test;

/// Parse argv and dispatch to the selected command handler.
pub fn run() {
    let cli = crate::cli::Cli::parse();
    dispatch(cli.command);
}

fn dispatch(command: Command) {
    match command {
        Command::Build(args) => package::cmd_build(BuildCommand {
            input: args.input,
            out_dir: args.out_dir,
            package: args.package,
            release: args.release,
            target: args.target.into(),
            format: args.format,
            linter: args.linter,
        }),
        Command::Targets => tool::cmd_targets(),
        Command::Check(args) => {
            if args.package || package::should_treat_as_package_from_args(&args.input) {
                package::cmd_check_package(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
                    diagnostic_mode: if args.diagnostics {
                        DiagnosticMode::Diagnostics
                    } else {
                        DiagnosticMode::Normal
                    },
                });
            } else {
                tool::cmd_check(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
                    diagnostic_mode: if args.diagnostics {
                        DiagnosticMode::Diagnostics
                    } else {
                        DiagnosticMode::Normal
                    },
                });
            }
        }
        Command::Init(args) => cmd_init(args),
        Command::Explain(args) => cmd_explain(args),
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
                    format: args.format,
                    linter: args.linter,
                    diagnostic_mode: if args.diagnostics {
                        DiagnosticMode::Diagnostics
                    } else {
                        DiagnosticMode::Normal
                    },
                });
            } else {
                tool::cmd_emit(EmitCommand {
                    input: args.input,
                    package: args.package,
                    target: args.target.into(),
                    format: args.format,
                    linter: args.linter,
                    diagnostic_mode: if args.diagnostics {
                        DiagnosticMode::Diagnostics
                    } else {
                        DiagnosticMode::Normal
                    },
                });
            }
        }
    }
}
