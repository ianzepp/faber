//! Thin binary entry point for the compiler-developer `radix` tool.
//!
//! The command grammar and execution policy live in `radix::tool`; this file is
//! intentionally only the clap parse boundary plus subcommand dispatch. Keeping
//! it thin lets the same command implementation stay callable from tests and
//! from any future wrapper without depending on process argument parsing.
//!
//! INVARIANT
//! =========
//! Every arm should delegate immediately to `tool` and avoid embedding compiler
//! phase behavior in the binary crate.

use clap::Parser;
use radix::tool::{self, CheckCommand, DiagnosticMode, EmitCommand, RadixCli, RadixCommand};

fn main() {
    let cli = RadixCli::parse();

    match cli.command {
        RadixCommand::Targets => tool::cmd_targets(),
        RadixCommand::Lex(args) => tool::cmd_lex(&args.input),
        RadixCommand::Parse(args) => tool::cmd_parse(&args.input),
        RadixCommand::Hir(args) => tool::cmd_hir(&args.input),
        RadixCommand::Mir(args) => tool::cmd_mir(&args.input),
        RadixCommand::CliIr(args) => tool::cmd_cli_ir(&args.input),
        RadixCommand::Check(args) => tool::cmd_check(CheckCommand {
            input: args.input,
            package: args.package,
            permissive: args.permissive,
            diagnostic_mode: if args.diagnostics {
                DiagnosticMode::Diagnostics
            } else {
                DiagnosticMode::Normal
            },
        }),
        RadixCommand::Emit(args) => tool::cmd_emit(EmitCommand {
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
        }),
    }
}
