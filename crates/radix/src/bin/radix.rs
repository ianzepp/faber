//! Compiler-developer CLI for Faber (`radix` binary).

use clap::Parser;
use radix::tool::{self, CheckCommand, EmitCommand, RadixCli, RadixCommand};

fn main() {
    let cli = RadixCli::parse();

    match cli.command {
        RadixCommand::Targets => tool::cmd_targets(),
        RadixCommand::Lex(args) => tool::cmd_lex(&args.input),
        RadixCommand::Parse(args) => tool::cmd_parse(&args.input),
        RadixCommand::Hir(args) => tool::cmd_hir(&args.input),
        RadixCommand::CliIr(args) => tool::cmd_cli_ir(&args.input),
        RadixCommand::Check(args) => {
            tool::cmd_check(CheckCommand { input: args.input, package: args.package, permissive: args.permissive })
        }
        RadixCommand::Emit(args) => {
            tool::cmd_emit(EmitCommand { input: args.input, package: args.package, target: args.target.into() })
        }
    }
}
