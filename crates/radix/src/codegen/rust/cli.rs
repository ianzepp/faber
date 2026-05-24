//! Rust-owned CLI support emission.
//!
//! This module writes the parser, help text, command dispatcher, argument
//! structs, and exit handling that are embedded into generated Rust output for
//! `@ cli` programs. The CLI model is analyzed before codegen, but the runtime
//! mechanics here are target-specific: they depend on `std::env::args`,
//! `std::process::exit`, Rust storage types, and direct calls into generated
//! Rust functions.
//!
//! INVARIANTS
//! ==========
//! - Parse errors print to stderr and exit with code 2.
//! - Help and version output exit with code 0.
//! - Single-command mode emits one `CliArgs` parser consumed by `incipit`.
//! - Subcommand mode emits one args struct/parser per command plus a root
//!   dispatcher that never returns.
//! - Defaults are lowered into Rust literals or strings at generation time;
//!   missing non-flag options remain `Option<_>` until parsing fills them.
//!
//! WHY THIS IS RUST BACKEND OWNED
//! ==============================
//! Faber CLI declarations describe command shape, not a portable runtime API.
//! Keeping this support code in the Rust backend lets other targets choose
//! native process, argument, and exit behavior without carrying Rust-specific
//! parser scaffolding through target-neutral HIR.

use super::super::CodeWriter;
use crate::cli::{CliDefault, CliExit, CliMode, CliOperand, CliOption, CliProgram, CliType};

/// Emit generated parser/help/support functions for the selected CLI mode.
///
/// Single-command mode shares global and command-local options because the
/// entry block receives one argument record. Subcommand mode keeps each command
/// parser separate so dispatch can select a function before parsing
/// command-local operands.
pub(super) fn generate_cli_support(program: &CliProgram, writer: &mut CodeWriter) {
    if program.mode == CliMode::Subcommand {
        generate_subcommand_cli_support(program, writer);
        return;
    }

    let options = program
        .global_options
        .iter()
        .chain(program.options.iter())
        .collect::<Vec<_>>();
    let operands = program
        .global_operands
        .iter()
        .chain(program.operands.iter())
        .collect::<Vec<_>>();

    generate_args_struct(&options, &operands, writer);
    writer.newline();
    generate_parse_error(writer);
    writer.newline();
    generate_help(program, &options, &operands, writer);
    writer.newline();
    generate_parser(program, &options, &operands, writer);
}

pub(super) fn generate_command_dispatch(program: &CliProgram, writer: &mut CodeWriter) {
    if program.mode == CliMode::Subcommand {
        writer.writeln("dispatch_cli_or_exit();");
    }
}

/// Emit the concrete Rust process-exit policy for a Faber CLI declaration.
///
/// Unsupported exit declarations fall back to success here because the driver
/// reports a diagnostic before successful Rust CLI output is exposed; codegen
/// still keeps this arm total for the generated program boundary it was given.
pub(super) fn generate_cli_exit(exit: &CliExit, writer: &mut CodeWriter) {
    match exit {
        CliExit::Fixed(code) => {
            writer.write("std::process::exit(");
            writer.write(&code.to_string());
            writer.writeln(");");
        }
        CliExit::Binding(binding) => {
            writer.write("std::process::exit(");
            writer.write(binding);
            writer.writeln(" as i32);");
        }
        CliExit::Field { object, field } => {
            writer.write("std::process::exit(");
            writer.write(object);
            writer.write(".");
            writer.write(field);
            writer.writeln(" as i32);");
        }
        CliExit::Unsupported => {
            writer.writeln("std::process::exit(0);");
        }
    }
}

/// Stable generated type name for a subcommand's argument record.
///
/// The name is derived from the command path rather than the target function so
/// aliases and mounted functions continue to share one parser contract.
pub(super) fn command_args_struct_name(command: &crate::cli::CliCommand) -> String {
    let suffix = command
        .path
        .iter()
        .flat_map(|part| part.split(|ch: char| !ch.is_ascii_alphanumeric()))
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>();
    format!("CliArgs{suffix}")
}

fn command_parser_name(command: &crate::cli::CliCommand) -> String {
    format!("parse_cli_args_{}", command.path.join("_").replace('-', "_"))
}

fn command_help_name(command: &crate::cli::CliCommand) -> String {
    format!("print_cli_help_{}", command.path.join("_").replace('-', "_"))
}

fn generate_subcommand_cli_support(program: &CliProgram, writer: &mut CodeWriter) {
    // Subcommand output is generated as a small Rust runtime: command-specific
    // records first, shared parse/help support next, dispatcher last so `main`
    // can jump into one non-returning entrypoint.
    for command in &program.commands {
        let options = program
            .global_options
            .iter()
            .chain(command.options.iter())
            .collect::<Vec<_>>();
        let operands = program
            .global_operands
            .iter()
            .chain(command.operands.iter())
            .collect::<Vec<_>>();
        generate_named_args_struct(&command_args_struct_name(command), &options, &operands, writer);
        writer.newline();
    }
    generate_parse_error(writer);
    writer.newline();
    generate_root_subcommand_help(program, writer);
    writer.newline();
    for command in &program.commands {
        let options = program
            .global_options
            .iter()
            .chain(command.options.iter())
            .collect::<Vec<_>>();
        let operands = program
            .global_operands
            .iter()
            .chain(command.operands.iter())
            .collect::<Vec<_>>();
        generate_command_help(program, command, &options, &operands, writer);
        writer.newline();
        generate_command_parser(command, &options, &operands, writer);
        writer.newline();
    }
    generate_dispatcher(program, writer);
}

fn generate_args_struct(options: &[&CliOption], operands: &[&CliOperand], writer: &mut CodeWriter) {
    generate_named_args_struct("CliArgs", options, operands, writer);
}

fn generate_named_args_struct(name: &str, options: &[&CliOption], operands: &[&CliOperand], writer: &mut CodeWriter) {
    writer.writeln("#[derive(Debug, Clone)]");
    writer.write("struct ");
    writer.write(name);
    writer.writeln(" {");
    writer.indented(|writer| {
        for option in options {
            writer.write("pub ");
            writer.write(&option.binding);
            writer.write(": ");
            writer.write(&rust_cli_type(&option.ty, option.default.is_none() && !option.flag, false));
            writer.writeln(",");
        }
        for operand in operands {
            writer.write("pub ");
            writer.write(&operand.binding);
            writer.write(": ");
            writer.write(&rust_cli_type(&operand.ty, false, operand.rest));
            writer.writeln(",");
        }
    });
    writer.writeln("}");
}

fn generate_parse_error(writer: &mut CodeWriter) {
    writer.writeln("fn cli_parse_error(message: String) -> ! {");
    writer.indented(|writer| {
        writer.writeln("eprintln!(\"error: {}\", message);");
        writer.writeln("std::process::exit(2);");
    });
    writer.writeln("}");
}

fn generate_help(program: &CliProgram, options: &[&CliOption], operands: &[&CliOperand], writer: &mut CodeWriter) {
    writer.writeln("fn print_cli_help() {");
    writer.indented(|writer| {
        write_println(writer, &format!("Usage: {}{}", program.name, usage_suffix(options, operands)));
        if let Some(description) = &program.description {
            writer.writeln("println!();");
            write_println(writer, description);
        }
        if !options.is_empty() {
            writer.writeln("println!();");
            write_println(writer, "Options:");
            for option in options {
                write_println(
                    writer,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        if !operands.is_empty() {
            writer.writeln("println!();");
            write_println(writer, "Operands:");
            for operand in operands {
                write_println(
                    writer,
                    &format!(
                        "  {:<24}{}",
                        operand_label(operand),
                        operand.description.as_deref().unwrap_or("")
                    ),
                );
            }
        }
        writer.writeln("println!();");
        write_println(writer, "  -h, --help              Print help");
        if program.version.is_some() {
            write_println(writer, "      --version           Print version");
        }
    });
    writer.writeln("}");
}

fn generate_root_subcommand_help(program: &CliProgram, writer: &mut CodeWriter) {
    writer.writeln("fn print_cli_help() {");
    writer.indented(|writer| {
        write_println(writer, &format!("Usage: {} [OPTIONS] <COMMAND>", program.name));
        if let Some(description) = &program.description {
            writer.writeln("println!();");
            write_println(writer, description);
        }
        if !program.global_options.is_empty() {
            writer.writeln("println!();");
            write_println(writer, "Global Options:");
            for option in &program.global_options {
                write_println(
                    writer,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        writer.writeln("println!();");
        write_println(writer, "Commands:");
        for command in &program.commands {
            let path = command.path.join(" ");
            let aliases = if command.aliases.is_empty() {
                String::new()
            } else {
                format!(" (alias: {})", command.aliases.join(", "))
            };
            write_println(
                writer,
                &format!("  {:<24}{}{}", path, command.description.as_deref().unwrap_or(""), aliases),
            );
        }
        writer.writeln("println!();");
        write_println(writer, "  -h, --help              Print help");
        if program.version.is_some() {
            write_println(writer, "      --version           Print version");
        }
    });
    writer.writeln("}");
}

fn generate_command_help(
    program: &CliProgram,
    command: &crate::cli::CliCommand,
    options: &[&CliOption],
    operands: &[&CliOperand],
    writer: &mut CodeWriter,
) {
    writer.write("fn ");
    writer.write(&command_help_name(command));
    writer.writeln("() {");
    writer.indented(|writer| {
        write_println(
            writer,
            &format!(
                "Usage: {} {}{}",
                program.name,
                command.path.join(" "),
                usage_suffix(options, operands)
            ),
        );
        if let Some(description) = &command.description {
            writer.writeln("println!();");
            write_println(writer, description);
        }
        if !options.is_empty() {
            writer.writeln("println!();");
            write_println(writer, "Options:");
            for option in options {
                write_println(
                    writer,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        if !operands.is_empty() {
            writer.writeln("println!();");
            write_println(writer, "Operands:");
            for operand in operands {
                write_println(
                    writer,
                    &format!(
                        "  {:<24}{}",
                        operand_label(operand),
                        operand.description.as_deref().unwrap_or("")
                    ),
                );
            }
        }
        writer.writeln("println!();");
        write_println(writer, "  -h, --help              Print help");
    });
    writer.writeln("}");
}

fn generate_parser(program: &CliProgram, options: &[&CliOption], operands: &[&CliOperand], writer: &mut CodeWriter) {
    writer.writeln("fn parse_cli_args_or_exit() -> CliArgs {");
    writer.indented(|writer| {
        for option in options {
            writer.write("let mut ");
            writer.write(&storage_name(&option.binding));
            writer.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, writer);
            writer.writeln(";");
        }
        for operand in operands {
            writer.write("let mut ");
            writer.write(&storage_name(&operand.binding));
            writer.write(": ");
            writer.write(&rust_cli_storage_type(&operand.ty, operand.rest));
            writer.write(" = ");
            if operand.rest {
                writer.writeln("Vec::new();");
            } else {
                writer.writeln("None;");
            }
        }
        writer.writeln("let mut positional: Vec<String> = Vec::new();");
        writer.writeln("let mut iter = std::env::args().skip(1).peekable();");
        writer.writeln("while let Some(arg) = iter.next() {");
        writer.indented(|writer| {
            writer.writeln("if arg == \"--\" {");
            writer.indented(|writer| {
                writer.writeln("positional.extend(iter);");
                writer.writeln("break;");
            });
            writer.writeln("}");
            writer.writeln("if arg == \"--help\" || arg == \"-h\" {");
            writer.indented(|writer| {
                writer.writeln("print_cli_help();");
                writer.writeln("std::process::exit(0);");
            });
            writer.writeln("}");
            if let Some(version) = &program.version {
                writer.writeln("if arg == \"--version\" {");
                writer.indented(|writer| {
                    write_println(writer, version);
                    writer.writeln("std::process::exit(0);");
                });
                writer.writeln("}");
            }
            writer.writeln("if arg.starts_with(\"--\") {");
            writer.indented(|writer| generate_long_option_parser(options, writer));
            writer.writeln("}");
            writer.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            writer.indented(|writer| generate_short_option_parser(options, writer));
            writer.writeln("}");
            writer.writeln("positional.push(arg);");
        });
        writer.writeln("}");
        generate_operand_assignment(operands, writer);
        writer.writeln("CliArgs {");
        writer.indented(|writer| {
            for option in options {
                writer.write(&option.binding);
                writer.write(": ");
                writer.write(&storage_name(&option.binding));
                writer.writeln(",");
            }
            for operand in operands {
                writer.write(&operand.binding);
                writer.write(": ");
                if operand.rest {
                    writer.write(&storage_name(&operand.binding));
                } else {
                    writer.write(&operand.binding);
                }
                writer.writeln(",");
            }
        });
        writer.writeln("}");
    });
    writer.writeln("}");
}

fn generate_command_parser(
    command: &crate::cli::CliCommand,
    options: &[&CliOption],
    operands: &[&CliOperand],
    writer: &mut CodeWriter,
) {
    writer.write("fn ");
    writer.write(&command_parser_name(command));
    writer.write("(__radix_cli_input: Vec<String>");
    for option in options.iter().filter(|option| option.global) {
        writer.write(", mut ");
        writer.write(&storage_name(&option.binding));
        writer.write(": ");
        writer.write(&rust_cli_type(&option.ty, option.default.is_none() && !option.flag, false));
    }
    writer.write(") -> ");
    writer.write(&command_args_struct_name(command));
    writer.writeln(" {");
    writer.indented(|writer| {
        for option in options.iter().filter(|option| !option.global) {
            writer.write("let mut ");
            writer.write(&storage_name(&option.binding));
            writer.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, writer);
            writer.writeln(";");
        }
        for operand in operands {
            writer.write("let mut ");
            writer.write(&storage_name(&operand.binding));
            writer.write(": ");
            writer.write(&rust_cli_storage_type(&operand.ty, operand.rest));
            writer.write(" = ");
            if operand.rest {
                writer.writeln("Vec::new();");
            } else {
                writer.writeln("None;");
            }
        }
        writer.writeln("let mut positional: Vec<String> = Vec::new();");
        writer.writeln("let mut iter = __radix_cli_input.into_iter().peekable();");
        writer.writeln("while let Some(arg) = iter.next() {");
        writer.indented(|writer| {
            writer.writeln("if arg == \"--\" {");
            writer.indented(|writer| {
                writer.writeln("positional.extend(iter);");
                writer.writeln("break;");
            });
            writer.writeln("}");
            writer.writeln("if arg == \"--help\" || arg == \"-h\" {");
            writer.indented(|writer| {
                writer.write(&command_help_name(command));
                writer.writeln("();");
                writer.writeln("std::process::exit(0);");
            });
            writer.writeln("}");
            writer.writeln("if arg.starts_with(\"--\") {");
            writer.indented(|writer| generate_long_option_parser(options, writer));
            writer.writeln("}");
            writer.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            writer.indented(|writer| generate_short_option_parser(options, writer));
            writer.writeln("}");
            writer.writeln("positional.push(arg);");
        });
        writer.writeln("}");
        generate_operand_assignment(operands, writer);
        writer.write(&command_args_struct_name(command));
        writer.writeln(" {");
        writer.indented(|writer| {
            for option in options {
                writer.write(&option.binding);
                writer.write(": ");
                writer.write(&storage_name(&option.binding));
                writer.writeln(",");
            }
            for operand in operands {
                writer.write(&operand.binding);
                writer.write(": ");
                if operand.rest {
                    writer.write(&storage_name(&operand.binding));
                } else {
                    writer.write(&operand.binding);
                }
                writer.writeln(",");
            }
        });
        writer.writeln("}");
    });
    writer.writeln("}");
}

fn generate_dispatcher(program: &CliProgram, writer: &mut CodeWriter) {
    // Longest paths win so nested commands are tested before their prefixes.
    // The parser has already accepted the command model; this dispatcher only
    // chooses among concrete generated Rust calls.
    writer.writeln("fn dispatch_cli_or_exit() -> ! {");
    writer.indented(|writer| {
        for option in &program.global_options {
            writer.write("let mut ");
            writer.write(&storage_name(&option.binding));
            writer.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, writer);
            writer.writeln(";");
        }
        writer.writeln("let mut iter = std::env::args().skip(1).peekable();");
        writer.writeln("let mut command_parts: Vec<String> = Vec::new();");
        writer.writeln("while let Some(arg) = iter.next() {");
        writer.indented(|writer| {
            writer.writeln("if arg == \"--help\" || arg == \"-h\" {");
            writer.indented(|writer| {
                writer.writeln("print_cli_help();");
                writer.writeln("std::process::exit(0);");
            });
            writer.writeln("}");
            if let Some(version) = &program.version {
                writer.writeln("if arg == \"--version\" {");
                writer.indented(|writer| {
                    write_println(writer, version);
                    writer.writeln("std::process::exit(0);");
                });
                writer.writeln("}");
            }
            writer.writeln("if arg.starts_with(\"--\") {");
            writer.indented(|writer| {
                generate_long_option_parser(&program.global_options.iter().collect::<Vec<_>>(), writer)
            });
            writer.writeln("}");
            writer.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            writer.indented(|writer| {
                generate_short_option_parser(&program.global_options.iter().collect::<Vec<_>>(), writer)
            });
            writer.writeln("}");
            writer.writeln("command_parts.push(arg);");
            writer.writeln("command_parts.extend(iter);");
            writer.writeln("break;");
        });
        writer.writeln("}");
        writer.writeln("if command_parts.is_empty() {");
        writer.indented(|writer| {
            writer.writeln("print_cli_help();");
            writer.writeln("std::process::exit(2);");
        });
        writer.writeln("}");
        let mut commands = program.commands.iter().collect::<Vec<_>>();
        commands.sort_by_key(|command| std::cmp::Reverse(command.path.len()));
        for command in commands {
            generate_dispatch_arm(program, command, writer);
            for alias in &command.aliases {
                generate_alias_dispatch_arm(program, command, alias, writer);
            }
        }
        writer.writeln("eprintln!(\"error: unknown command '{}'\", command_parts[0]);");
        writer.writeln("print_cli_help();");
        writer.writeln("std::process::exit(2);");
    });
    writer.writeln("}");
}

fn generate_dispatch_arm(program: &CliProgram, command: &crate::cli::CliCommand, writer: &mut CodeWriter) {
    let len = command.path.len();
    writer.write("if command_parts.len() >= ");
    writer.write(&len.to_string());
    writer.write(" && ");
    for (index, part) in command.path.iter().enumerate() {
        if index > 0 {
            writer.write(" && ");
        }
        writer.write("command_parts[");
        writer.write(&index.to_string());
        writer.write("] == ");
        write_rust_string_literal(part, writer);
    }
    writer.writeln(" {");
    writer.indented(|writer| generate_dispatch_call(program, command, len, writer));
    writer.writeln("}");
}

fn generate_alias_dispatch_arm(
    program: &CliProgram,
    command: &crate::cli::CliCommand,
    alias: &str,
    writer: &mut CodeWriter,
) {
    let parts = alias_path(alias);
    writer.write("if command_parts.len() >= ");
    writer.write(&parts.len().to_string());
    writer.write(" && ");
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            writer.write(" && ");
        }
        writer.write("command_parts[");
        writer.write(&index.to_string());
        writer.write("] == ");
        write_rust_string_literal(part, writer);
    }
    writer.writeln(" {");
    writer.indented(|writer| generate_dispatch_call(program, command, parts.len(), writer));
    writer.writeln("}");
}

fn generate_dispatch_call(
    program: &CliProgram,
    command: &crate::cli::CliCommand,
    consumed: usize,
    writer: &mut CodeWriter,
) {
    writer.write("let args = ");
    writer.write(&command_parser_name(command));
    writer.write("(command_parts[");
    writer.write(&consumed.to_string());
    writer.write("..].to_vec()");
    for option in &program.global_options {
        writer.write(", ");
        writer.write(&storage_name(&option.binding));
    }
    writer.writeln(");");
    if command.args_binding.is_some() {
        write_command_function(command, writer);
        writer.writeln("(args);");
    } else {
        write_command_function(command, writer);
        writer.writeln("();");
    }
    writer.writeln("std::process::exit(0);");
}

fn write_command_function(command: &crate::cli::CliCommand, writer: &mut CodeWriter) {
    if let Some(module_path) = &command.module_path {
        for segment in module_path {
            writer.write(segment);
            writer.write("::");
        }
    }
    writer.write(&command.function);
}

fn alias_path(alias: &str) -> Vec<&str> {
    alias.split('/').filter(|part| !part.is_empty()).collect()
}

fn generate_long_option_parser(options: &[&CliOption], writer: &mut CodeWriter) {
    writer.writeln("let (__cli_name, __cli_inline_value) = match arg.split_once('=') {");
    writer.indented(|writer| {
        writer.writeln("Some((name, value)) => (name, Some(value.to_owned())),");
        writer.writeln("None => (arg.as_str(), None),");
    });
    writer.writeln("};");
    writer.writeln("match __cli_name {");
    writer.indented(|writer| {
        for option in options.iter().filter(|option| option.long.is_some()) {
            let Some(long) = option.long.as_ref() else {
                continue;
            };
            write_string_match(long, "--", writer);
            writer.writeln(" => {");
            writer.indented(|writer| generate_option_setter(option, "__cli_name", "__cli_inline_value", writer));
            writer.writeln("},");
        }
        writer.writeln("_ => cli_parse_error(format!(\"unknown option '{}'\", arg)),");
    });
    writer.writeln("}");
    writer.writeln("continue;");
}

fn generate_short_option_parser(options: &[&CliOption], writer: &mut CodeWriter) {
    writer.writeln("match arg.as_str() {");
    writer.indented(|writer| {
        for option in options.iter().filter(|option| option.short.is_some()) {
            let Some(short) = option.short.as_ref() else {
                continue;
            };
            write_string_match(short, "-", writer);
            writer.writeln(" => {");
            writer.indented(|writer| generate_option_setter(option, "arg.as_str()", "None", writer));
            writer.writeln("},");
        }
        writer.writeln("_ => cli_parse_error(format!(\"unknown option '{}'\", arg)),");
    });
    writer.writeln("}");
    writer.writeln("continue;");
}

fn generate_option_setter(option: &CliOption, label_expr: &str, inline_expr: &str, writer: &mut CodeWriter) {
    if option.flag {
        writer.write(&storage_name(&option.binding));
        writer.writeln(" = true;");
        return;
    }

    writer.write("let __radix_cli_raw = match ");
    writer.write(inline_expr);
    writer.writeln(" {");
    writer.indented(|writer| {
        writer.writeln("Some(value) => value,");
        writer.write("None => iter.next().unwrap_or_else(|| cli_parse_error(format!(\"missing value for {}\", ");
        writer.write(label_expr);
        writer.writeln("))),");
    });
    writer.writeln("};");
    writer.write(&storage_name(&option.binding));
    writer.write(" = ");
    if option.default.is_none() {
        writer.write("Some(");
        write_parse_value("__radix_cli_raw", &option.ty, writer);
        writer.writeln(");");
    } else {
        write_parse_value("__radix_cli_raw", &option.ty, writer);
        writer.writeln(";");
    }
}

fn generate_operand_assignment(operands: &[&CliOperand], writer: &mut CodeWriter) {
    // Operand parsing is intentionally fail-fast because generated CLIs are the
    // final user boundary. Optionality/defaults are reflected in storage before
    // this point; after assignment, command functions receive concrete fields.
    writer.writeln("let mut positional_iter = positional.into_iter();");
    let has_rest = operands.iter().any(|operand| operand.rest);
    for operand in operands {
        if operand.rest {
            writer.write(&storage_name(&operand.binding));
            writer.write(" = positional_iter.map(|raw| ");
            write_parse_value_inline("raw", &operand.ty, writer);
            writer.writeln(").collect();");
            continue;
        }
        writer.write(&storage_name(&operand.binding));
        writer.write(" = Some(match positional_iter.next() { Some(raw) => ");
        write_parse_value_inline("raw", &operand.ty, writer);
        if let Some(default) = &operand.default {
            writer.write(", None => ");
            write_default_value(default, &operand.ty, writer);
        } else {
            writer.write(", None => cli_parse_error(");
            write_rust_string_literal(&format!("missing operand '{}'", operand.binding), writer);
            writer.write(".to_owned())");
        }
        writer.writeln(" });");
    }
    if !has_rest {
        writer.writeln("if let Some(extra) = positional_iter.next() {");
        writer.indented(|writer| {
            writer.writeln("cli_parse_error(format!(\"unexpected operand '{}'\", extra));");
        });
        writer.writeln("}");
    }
    for operand in operands.iter().filter(|operand| !operand.rest) {
        writer.write("let ");
        writer.write(&operand.binding);
        writer.write(" = ");
        writer.write(&storage_name(&operand.binding));
        writer.writeln(".expect(\"operand initialized\");");
    }
}

fn write_parse_value(raw_expr: &str, ty: &CliType, writer: &mut CodeWriter) {
    write_parse_value_inline(raw_expr, ty, writer);
}

fn write_parse_value_inline(raw_expr: &str, ty: &CliType, writer: &mut CodeWriter) {
    match ty {
        CliType::Numerus | CliType::ListaNumerus => {
            writer.write(raw_expr);
            writer.write(".parse::<i64>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid numeric value '{}'\", ");
            writer.write(raw_expr);
            writer.write(")))");
        }
        CliType::Fractus => {
            writer.write(raw_expr);
            writer.write(".parse::<f64>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid numeric value '{}'\", ");
            writer.write(raw_expr);
            writer.write(")))");
        }
        CliType::Bivalens => {
            writer.write(raw_expr);
            writer
                .write(".parse::<bool>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid boolean value '{}'\", ");
            writer.write(raw_expr);
            writer.write(")))");
        }
        _ => {
            writer.write(raw_expr);
            writer.write(".to_owned()");
        }
    }
}

fn write_default(default: Option<&CliDefault>, ty: &CliType, flag: bool, writer: &mut CodeWriter) {
    if flag {
        write_default_value(default.unwrap_or(&CliDefault::Bool(false)), ty, writer);
    } else if let Some(default) = default {
        write_default_value(default, ty, writer);
    } else {
        writer.write("None");
    }
}

fn write_default_value(default: &CliDefault, ty: &CliType, writer: &mut CodeWriter) {
    match (default, ty) {
        (CliDefault::Text(value), _) => {
            write_rust_string_literal(value, writer);
            writer.write(".to_owned()");
        }
        (CliDefault::Integer(value), _) => writer.write(&value.to_string()),
        (CliDefault::Float(value), _) => writer.write(&value.to_string()),
        (CliDefault::Bool(value), _) => writer.write(if *value { "true" } else { "false" }),
        (CliDefault::Nil, _) => writer.write("None"),
        (CliDefault::Expr(value), _) => {
            write_rust_string_literal(value, writer);
            writer.write(".to_owned()");
        }
    }
}

fn rust_cli_storage_type(ty: &CliType, rest: bool) -> String {
    if rest {
        return rust_cli_type(ty, false, true);
    }
    format!("Option<{}>", rust_cli_type(ty, false, false))
}

fn rust_cli_type(ty: &CliType, optional: bool, rest: bool) -> String {
    // This is the CLI-specific value policy, not the full Faber type mapper.
    // CLI input enters as strings and lowers only to the scalar/list shapes
    // accepted by the CLI declaration model.
    let base = match ty {
        CliType::Numerus | CliType::ListaNumerus => "i64",
        CliType::Fractus => "f64",
        CliType::Bivalens => "bool",
        CliType::Octeti => "Vec<u8>",
        _ => "String",
    };
    let value = if rest || matches!(ty, CliType::ListaTextus | CliType::ListaNumerus) {
        format!("Vec<{base}>")
    } else {
        base.to_owned()
    };
    if optional {
        format!("Option<{value}>")
    } else {
        value
    }
}

fn usage_suffix(options: &[&CliOption], operands: &[&CliOperand]) -> String {
    let mut parts = Vec::new();
    if !options.is_empty() {
        parts.push("[OPTIONS]".to_owned());
    }
    for operand in operands {
        if operand.rest {
            parts.push(format!("[{}...]", operand.binding));
        } else if operand.default.is_some() {
            parts.push(format!("[{}]", operand.binding));
        } else {
            parts.push(format!("<{}>", operand.binding));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" {}", parts.join(" "))
    }
}

fn option_label(option: &CliOption) -> String {
    let mut names = Vec::new();
    if let Some(short) = &option.short {
        names.push(format!("-{short}"));
    }
    if let Some(long) = &option.long {
        names.push(format!("--{long}"));
    }
    let mut label = names.join(", ");
    if !option.flag {
        label.push(' ');
        label.push_str(value_name(&option.ty));
    }
    label
}

fn operand_label(operand: &CliOperand) -> String {
    if operand.rest {
        format!("{}...", operand.binding)
    } else {
        operand.binding.clone()
    }
}

fn value_name(ty: &CliType) -> &'static str {
    match ty {
        CliType::Numerus | CliType::ListaNumerus => "<NUMERUS>",
        CliType::Fractus => "<FRACTUS>",
        CliType::Bivalens => "<BIVALENS>",
        CliType::Octeti => "<OCTETI>",
        _ => "<TEXTUS>",
    }
}

fn write_string_match(value: &str, prefix: &str, writer: &mut CodeWriter) {
    write_rust_string_literal(&format!("{prefix}{value}"), writer);
}

fn storage_name(binding: &str) -> String {
    format!("__radix_cli_{binding}")
}

fn write_println(writer: &mut CodeWriter, text: &str) {
    writer.write("println!(");
    write_rust_string_literal(text, writer);
    writer.writeln(");");
}

fn write_rust_string_literal(text: &str, writer: &mut CodeWriter) {
    writer.write("\"");
    for ch in text.chars() {
        match ch {
            '\\' => writer.write("\\\\"),
            '"' => writer.write("\\\""),
            '\n' => writer.write("\\n"),
            '\r' => writer.write("\\r"),
            '\t' => writer.write("\\t"),
            _ => writer.write(&ch.to_string()),
        }
    }
    writer.write("\"");
}
