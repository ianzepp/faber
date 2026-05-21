use super::super::CodeWriter;
use crate::cli::{CliDefault, CliExit, CliMode, CliOperand, CliOption, CliProgram, CliType};

pub(super) fn generate_cli_support(program: &CliProgram, w: &mut CodeWriter) {
    if program.mode == CliMode::Subcommand {
        generate_subcommand_cli_support(program, w);
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

    generate_args_struct(&options, &operands, w);
    w.newline();
    generate_parse_error(w);
    w.newline();
    generate_help(program, &options, &operands, w);
    w.newline();
    generate_parser(program, &options, &operands, w);
}

pub(super) fn generate_command_dispatch(program: &CliProgram, w: &mut CodeWriter) {
    if program.mode == CliMode::Subcommand {
        w.writeln("dispatch_cli_or_exit();");
    }
}

pub(super) fn generate_cli_exit(exit: &CliExit, w: &mut CodeWriter) {
    match exit {
        CliExit::Fixed(code) => {
            w.write("std::process::exit(");
            w.write(&code.to_string());
            w.writeln(");");
        }
        CliExit::Binding(binding) => {
            w.write("std::process::exit(");
            w.write(binding);
            w.writeln(" as i32);");
        }
        CliExit::Field { object, field } => {
            w.write("std::process::exit(");
            w.write(object);
            w.write(".");
            w.write(field);
            w.writeln(" as i32);");
        }
        CliExit::Unsupported => {
            w.writeln("std::process::exit(0);");
        }
    }
}

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

fn generate_subcommand_cli_support(program: &CliProgram, w: &mut CodeWriter) {
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
        generate_named_args_struct(&command_args_struct_name(command), &options, &operands, w);
        w.newline();
    }
    generate_parse_error(w);
    w.newline();
    generate_root_subcommand_help(program, w);
    w.newline();
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
        generate_command_help(program, command, &options, &operands, w);
        w.newline();
        generate_command_parser(command, &options, &operands, w);
        w.newline();
    }
    generate_dispatcher(program, w);
}

fn generate_args_struct(options: &[&CliOption], operands: &[&CliOperand], w: &mut CodeWriter) {
    generate_named_args_struct("CliArgs", options, operands, w);
}

fn generate_named_args_struct(name: &str, options: &[&CliOption], operands: &[&CliOperand], w: &mut CodeWriter) {
    w.writeln("#[derive(Debug, Clone)]");
    w.write("struct ");
    w.write(name);
    w.writeln(" {");
    w.indented(|w| {
        for option in options {
            w.write("pub ");
            w.write(&option.binding);
            w.write(": ");
            w.write(&rust_cli_type(&option.ty, option.default.is_none() && !option.flag, false));
            w.writeln(",");
        }
        for operand in operands {
            w.write("pub ");
            w.write(&operand.binding);
            w.write(": ");
            w.write(&rust_cli_type(&operand.ty, false, operand.rest));
            w.writeln(",");
        }
    });
    w.writeln("}");
}

fn generate_parse_error(w: &mut CodeWriter) {
    w.writeln("fn cli_parse_error(message: String) -> ! {");
    w.indented(|w| {
        w.writeln("eprintln!(\"error: {}\", message);");
        w.writeln("std::process::exit(2);");
    });
    w.writeln("}");
}

fn generate_help(program: &CliProgram, options: &[&CliOption], operands: &[&CliOperand], w: &mut CodeWriter) {
    w.writeln("fn print_cli_help() {");
    w.indented(|w| {
        write_println(w, &format!("Usage: {}{}", program.name, usage_suffix(options, operands)));
        if let Some(description) = &program.description {
            w.writeln("println!();");
            write_println(w, description);
        }
        if !options.is_empty() {
            w.writeln("println!();");
            write_println(w, "Options:");
            for option in options {
                write_println(
                    w,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        if !operands.is_empty() {
            w.writeln("println!();");
            write_println(w, "Operands:");
            for operand in operands {
                write_println(
                    w,
                    &format!(
                        "  {:<24}{}",
                        operand_label(operand),
                        operand.description.as_deref().unwrap_or("")
                    ),
                );
            }
        }
        w.writeln("println!();");
        write_println(w, "  -h, --help              Print help");
        if program.version.is_some() {
            write_println(w, "      --version           Print version");
        }
    });
    w.writeln("}");
}

fn generate_root_subcommand_help(program: &CliProgram, w: &mut CodeWriter) {
    w.writeln("fn print_cli_help() {");
    w.indented(|w| {
        write_println(w, &format!("Usage: {} [OPTIONS] <COMMAND>", program.name));
        if let Some(description) = &program.description {
            w.writeln("println!();");
            write_println(w, description);
        }
        if !program.global_options.is_empty() {
            w.writeln("println!();");
            write_println(w, "Global Options:");
            for option in &program.global_options {
                write_println(
                    w,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        w.writeln("println!();");
        write_println(w, "Commands:");
        for command in &program.commands {
            let path = command.path.join(" ");
            let aliases = if command.aliases.is_empty() {
                String::new()
            } else {
                format!(" (alias: {})", command.aliases.join(", "))
            };
            write_println(
                w,
                &format!("  {:<24}{}{}", path, command.description.as_deref().unwrap_or(""), aliases),
            );
        }
        w.writeln("println!();");
        write_println(w, "  -h, --help              Print help");
        if program.version.is_some() {
            write_println(w, "      --version           Print version");
        }
    });
    w.writeln("}");
}

fn generate_command_help(
    program: &CliProgram,
    command: &crate::cli::CliCommand,
    options: &[&CliOption],
    operands: &[&CliOperand],
    w: &mut CodeWriter,
) {
    w.write("fn ");
    w.write(&command_help_name(command));
    w.writeln("() {");
    w.indented(|w| {
        write_println(
            w,
            &format!(
                "Usage: {}{}{}",
                program.name,
                format!(" {}", command.path.join(" ")),
                usage_suffix(options, operands)
            ),
        );
        if let Some(description) = &command.description {
            w.writeln("println!();");
            write_println(w, description);
        }
        if !options.is_empty() {
            w.writeln("println!();");
            write_println(w, "Options:");
            for option in options {
                write_println(
                    w,
                    &format!("  {:<24}{}", option_label(option), option.description.as_deref().unwrap_or("")),
                );
            }
        }
        if !operands.is_empty() {
            w.writeln("println!();");
            write_println(w, "Operands:");
            for operand in operands {
                write_println(
                    w,
                    &format!(
                        "  {:<24}{}",
                        operand_label(operand),
                        operand.description.as_deref().unwrap_or("")
                    ),
                );
            }
        }
        w.writeln("println!();");
        write_println(w, "  -h, --help              Print help");
    });
    w.writeln("}");
}

fn generate_parser(program: &CliProgram, options: &[&CliOption], operands: &[&CliOperand], w: &mut CodeWriter) {
    w.writeln("fn parse_cli_args_or_exit() -> CliArgs {");
    w.indented(|w| {
        for option in options {
            w.write("let mut ");
            w.write(&storage_name(&option.binding));
            w.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, w);
            w.writeln(";");
        }
        for operand in operands {
            w.write("let mut ");
            w.write(&storage_name(&operand.binding));
            w.write(": ");
            w.write(&rust_cli_storage_type(&operand.ty, operand.rest));
            w.write(" = ");
            if operand.rest {
                w.writeln("Vec::new();");
            } else {
                w.writeln("None;");
            }
        }
        w.writeln("let mut positional: Vec<String> = Vec::new();");
        w.writeln("let mut iter = std::env::args().skip(1).peekable();");
        w.writeln("while let Some(arg) = iter.next() {");
        w.indented(|w| {
            w.writeln("if arg == \"--\" {");
            w.indented(|w| {
                w.writeln("positional.extend(iter);");
                w.writeln("break;");
            });
            w.writeln("}");
            w.writeln("if arg == \"--help\" || arg == \"-h\" {");
            w.indented(|w| {
                w.writeln("print_cli_help();");
                w.writeln("std::process::exit(0);");
            });
            w.writeln("}");
            if let Some(version) = &program.version {
                w.writeln("if arg == \"--version\" {");
                w.indented(|w| {
                    write_println(w, version);
                    w.writeln("std::process::exit(0);");
                });
                w.writeln("}");
            }
            w.writeln("if arg.starts_with(\"--\") {");
            w.indented(|w| generate_long_option_parser(options, w));
            w.writeln("}");
            w.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            w.indented(|w| generate_short_option_parser(options, w));
            w.writeln("}");
            w.writeln("positional.push(arg);");
        });
        w.writeln("}");
        generate_operand_assignment(operands, w);
        w.writeln("CliArgs {");
        w.indented(|w| {
            for option in options {
                w.write(&option.binding);
                w.write(": ");
                w.write(&storage_name(&option.binding));
                w.writeln(",");
            }
            for operand in operands {
                w.write(&operand.binding);
                w.write(": ");
                if operand.rest {
                    w.write(&storage_name(&operand.binding));
                } else {
                    w.write(&operand.binding);
                }
                w.writeln(",");
            }
        });
        w.writeln("}");
    });
    w.writeln("}");
}

fn generate_command_parser(
    command: &crate::cli::CliCommand,
    options: &[&CliOption],
    operands: &[&CliOperand],
    w: &mut CodeWriter,
) {
    w.write("fn ");
    w.write(&command_parser_name(command));
    w.write("(__radix_cli_input: Vec<String>");
    for option in options.iter().filter(|option| option.global) {
        w.write(", mut ");
        w.write(&storage_name(&option.binding));
        w.write(": ");
        w.write(&rust_cli_type(&option.ty, option.default.is_none() && !option.flag, false));
    }
    w.write(") -> ");
    w.write(&command_args_struct_name(command));
    w.writeln(" {");
    w.indented(|w| {
        for option in options.iter().filter(|option| !option.global) {
            w.write("let mut ");
            w.write(&storage_name(&option.binding));
            w.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, w);
            w.writeln(";");
        }
        for operand in operands {
            w.write("let mut ");
            w.write(&storage_name(&operand.binding));
            w.write(": ");
            w.write(&rust_cli_storage_type(&operand.ty, operand.rest));
            w.write(" = ");
            if operand.rest {
                w.writeln("Vec::new();");
            } else {
                w.writeln("None;");
            }
        }
        w.writeln("let mut positional: Vec<String> = Vec::new();");
        w.writeln("let mut iter = __radix_cli_input.into_iter().peekable();");
        w.writeln("while let Some(arg) = iter.next() {");
        w.indented(|w| {
            w.writeln("if arg == \"--\" {");
            w.indented(|w| {
                w.writeln("positional.extend(iter);");
                w.writeln("break;");
            });
            w.writeln("}");
            w.writeln("if arg == \"--help\" || arg == \"-h\" {");
            w.indented(|w| {
                w.write(&command_help_name(command));
                w.writeln("();");
                w.writeln("std::process::exit(0);");
            });
            w.writeln("}");
            w.writeln("if arg.starts_with(\"--\") {");
            w.indented(|w| generate_long_option_parser(options, w));
            w.writeln("}");
            w.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            w.indented(|w| generate_short_option_parser(options, w));
            w.writeln("}");
            w.writeln("positional.push(arg);");
        });
        w.writeln("}");
        generate_operand_assignment(operands, w);
        w.write(&command_args_struct_name(command));
        w.writeln(" {");
        w.indented(|w| {
            for option in options {
                w.write(&option.binding);
                w.write(": ");
                w.write(&storage_name(&option.binding));
                w.writeln(",");
            }
            for operand in operands {
                w.write(&operand.binding);
                w.write(": ");
                if operand.rest {
                    w.write(&storage_name(&operand.binding));
                } else {
                    w.write(&operand.binding);
                }
                w.writeln(",");
            }
        });
        w.writeln("}");
    });
    w.writeln("}");
}

fn generate_dispatcher(program: &CliProgram, w: &mut CodeWriter) {
    w.writeln("fn dispatch_cli_or_exit() -> ! {");
    w.indented(|w| {
        for option in &program.global_options {
            w.write("let mut ");
            w.write(&storage_name(&option.binding));
            w.write(" = ");
            write_default(option.default.as_ref(), &option.ty, option.flag, w);
            w.writeln(";");
        }
        w.writeln("let mut iter = std::env::args().skip(1).peekable();");
        w.writeln("let mut command_parts: Vec<String> = Vec::new();");
        w.writeln("while let Some(arg) = iter.next() {");
        w.indented(|w| {
            w.writeln("if arg == \"--help\" || arg == \"-h\" {");
            w.indented(|w| {
                w.writeln("print_cli_help();");
                w.writeln("std::process::exit(0);");
            });
            w.writeln("}");
            if let Some(version) = &program.version {
                w.writeln("if arg == \"--version\" {");
                w.indented(|w| {
                    write_println(w, version);
                    w.writeln("std::process::exit(0);");
                });
                w.writeln("}");
            }
            w.writeln("if arg.starts_with(\"--\") {");
            w.indented(|w| generate_long_option_parser(&program.global_options.iter().collect::<Vec<_>>(), w));
            w.writeln("}");
            w.writeln("if arg.starts_with('-') && arg.len() > 1 {");
            w.indented(|w| generate_short_option_parser(&program.global_options.iter().collect::<Vec<_>>(), w));
            w.writeln("}");
            w.writeln("command_parts.push(arg);");
            w.writeln("command_parts.extend(iter);");
            w.writeln("break;");
        });
        w.writeln("}");
        w.writeln("if command_parts.is_empty() {");
        w.indented(|w| {
            w.writeln("print_cli_help();");
            w.writeln("std::process::exit(2);");
        });
        w.writeln("}");
        let mut commands = program.commands.iter().collect::<Vec<_>>();
        commands.sort_by(|left, right| right.path.len().cmp(&left.path.len()));
        for command in commands {
            generate_dispatch_arm(program, command, w);
            for alias in &command.aliases {
                generate_alias_dispatch_arm(program, command, alias, w);
            }
        }
        w.writeln("eprintln!(\"error: unknown command '{}'\", command_parts[0]);");
        w.writeln("print_cli_help();");
        w.writeln("std::process::exit(2);");
    });
    w.writeln("}");
}

fn generate_dispatch_arm(program: &CliProgram, command: &crate::cli::CliCommand, w: &mut CodeWriter) {
    let len = command.path.len();
    w.write("if command_parts.len() >= ");
    w.write(&len.to_string());
    w.write(" && ");
    for (index, part) in command.path.iter().enumerate() {
        if index > 0 {
            w.write(" && ");
        }
        w.write("command_parts[");
        w.write(&index.to_string());
        w.write("] == ");
        write_rust_string_literal(part, w);
    }
    w.writeln(" {");
    w.indented(|w| generate_dispatch_call(program, command, len, w));
    w.writeln("}");
}

fn generate_alias_dispatch_arm(
    program: &CliProgram,
    command: &crate::cli::CliCommand,
    alias: &str,
    w: &mut CodeWriter,
) {
    w.write("if command_parts[0] == ");
    write_rust_string_literal(alias, w);
    w.writeln(" {");
    w.indented(|w| generate_dispatch_call(program, command, 1, w));
    w.writeln("}");
}

fn generate_dispatch_call(program: &CliProgram, command: &crate::cli::CliCommand, consumed: usize, w: &mut CodeWriter) {
    w.write("let args = ");
    w.write(&command_parser_name(command));
    w.write("(command_parts[");
    w.write(&consumed.to_string());
    w.write("..].to_vec()");
    for option in &program.global_options {
        w.write(", ");
        w.write(&storage_name(&option.binding));
    }
    w.writeln(");");
    if command.args_binding.is_some() {
        w.write(&command.function);
        w.writeln("(args);");
    } else {
        w.write(&command.function);
        w.writeln("();");
    }
    w.writeln("std::process::exit(0);");
}

fn generate_long_option_parser(options: &[&CliOption], w: &mut CodeWriter) {
    w.writeln("let (__cli_name, __cli_inline_value) = match arg.split_once('=') {");
    w.indented(|w| {
        w.writeln("Some((name, value)) => (name, Some(value.to_owned())),");
        w.writeln("None => (arg.as_str(), None),");
    });
    w.writeln("};");
    w.writeln("match __cli_name {");
    w.indented(|w| {
        for option in options.iter().filter(|option| option.long.is_some()) {
            let Some(long) = option.long.as_ref() else {
                continue;
            };
            write_string_match(long, "--", w);
            w.writeln(" => {");
            w.indented(|w| generate_option_setter(option, "__cli_name", "__cli_inline_value", w));
            w.writeln("},");
        }
        w.writeln("_ => cli_parse_error(format!(\"unknown option '{}'\", arg)),");
    });
    w.writeln("}");
    w.writeln("continue;");
}

fn generate_short_option_parser(options: &[&CliOption], w: &mut CodeWriter) {
    w.writeln("match arg.as_str() {");
    w.indented(|w| {
        for option in options.iter().filter(|option| option.short.is_some()) {
            let Some(short) = option.short.as_ref() else {
                continue;
            };
            write_string_match(short, "-", w);
            w.writeln(" => {");
            w.indented(|w| generate_option_setter(option, "arg.as_str()", "None", w));
            w.writeln("},");
        }
        w.writeln("_ => cli_parse_error(format!(\"unknown option '{}'\", arg)),");
    });
    w.writeln("}");
    w.writeln("continue;");
}

fn generate_option_setter(option: &CliOption, label_expr: &str, inline_expr: &str, w: &mut CodeWriter) {
    if option.flag {
        w.write(&storage_name(&option.binding));
        w.writeln(" = true;");
        return;
    }

    w.write("let __radix_cli_raw = match ");
    w.write(inline_expr);
    w.writeln(" {");
    w.indented(|w| {
        w.writeln("Some(value) => value,");
        w.write("None => iter.next().unwrap_or_else(|| cli_parse_error(format!(\"missing value for {}\", ");
        w.write(label_expr);
        w.writeln("))),");
    });
    w.writeln("};");
    w.write(&storage_name(&option.binding));
    w.write(" = ");
    if option.default.is_none() {
        w.write("Some(");
        write_parse_value("__radix_cli_raw", &option.ty, w);
        w.writeln(");");
    } else {
        write_parse_value("__radix_cli_raw", &option.ty, w);
        w.writeln(";");
    }
}

fn generate_operand_assignment(operands: &[&CliOperand], w: &mut CodeWriter) {
    w.writeln("let mut positional_iter = positional.into_iter();");
    let has_rest = operands.iter().any(|operand| operand.rest);
    for operand in operands {
        if operand.rest {
            w.write(&storage_name(&operand.binding));
            w.write(" = positional_iter.map(|raw| ");
            write_parse_value_inline("raw", &operand.ty, w);
            w.writeln(").collect();");
            continue;
        }
        w.write(&storage_name(&operand.binding));
        w.write(" = Some(match positional_iter.next() { Some(raw) => ");
        write_parse_value_inline("raw", &operand.ty, w);
        if let Some(default) = &operand.default {
            w.write(", None => ");
            write_default_value(default, &operand.ty, w);
        } else {
            w.write(", None => cli_parse_error(");
            write_rust_string_literal(&format!("missing operand '{}'", operand.binding), w);
            w.write(".to_owned())");
        }
        w.writeln(" });");
    }
    if !has_rest {
        w.writeln("if let Some(extra) = positional_iter.next() {");
        w.indented(|w| {
            w.writeln("cli_parse_error(format!(\"unexpected operand '{}'\", extra));");
        });
        w.writeln("}");
    }
    for operand in operands.iter().filter(|operand| !operand.rest) {
        w.write("let ");
        w.write(&operand.binding);
        w.write(" = ");
        w.write(&storage_name(&operand.binding));
        w.writeln(".expect(\"operand initialized\");");
    }
}

fn write_parse_value(raw_expr: &str, ty: &CliType, w: &mut CodeWriter) {
    write_parse_value_inline(raw_expr, ty, w);
}

fn write_parse_value_inline(raw_expr: &str, ty: &CliType, w: &mut CodeWriter) {
    match ty {
        CliType::Numerus | CliType::ListaNumerus => {
            w.write(raw_expr);
            w.write(".parse::<i64>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid numeric value '{}'\", ");
            w.write(raw_expr);
            w.write(")))");
        }
        CliType::Fractus => {
            w.write(raw_expr);
            w.write(".parse::<f64>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid numeric value '{}'\", ");
            w.write(raw_expr);
            w.write(")))");
        }
        CliType::Bivalens => {
            w.write(raw_expr);
            w.write(".parse::<bool>().unwrap_or_else(|_| cli_parse_error(format!(\"invalid boolean value '{}'\", ");
            w.write(raw_expr);
            w.write(")))");
        }
        _ => {
            w.write(raw_expr);
            w.write(".to_owned()");
        }
    }
}

fn write_default(default: Option<&CliDefault>, ty: &CliType, flag: bool, w: &mut CodeWriter) {
    if flag {
        write_default_value(default.unwrap_or(&CliDefault::Bool(false)), ty, w);
    } else if let Some(default) = default {
        write_default_value(default, ty, w);
    } else {
        w.write("None");
    }
}

fn write_default_value(default: &CliDefault, ty: &CliType, w: &mut CodeWriter) {
    match (default, ty) {
        (CliDefault::Text(value), _) => {
            write_rust_string_literal(value, w);
            w.write(".to_owned()");
        }
        (CliDefault::Integer(value), _) => w.write(&value.to_string()),
        (CliDefault::Float(value), _) => w.write(&value.to_string()),
        (CliDefault::Bool(value), _) => w.write(if *value { "true" } else { "false" }),
        (CliDefault::Nil, _) => w.write("None"),
        (CliDefault::Expr(value), _) => {
            write_rust_string_literal(value, w);
            w.write(".to_owned()");
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

fn write_string_match(value: &str, prefix: &str, w: &mut CodeWriter) {
    write_rust_string_literal(&format!("{prefix}{value}"), w);
}

fn storage_name(binding: &str) -> String {
    format!("__radix_cli_{binding}")
}

fn write_println(w: &mut CodeWriter, text: &str) {
    w.write("println!(");
    write_rust_string_literal(text, w);
    w.writeln(");");
}

fn write_rust_string_literal(text: &str, w: &mut CodeWriter) {
    w.write("\"");
    for ch in text.chars() {
        match ch {
            '\\' => w.write("\\\\"),
            '"' => w.write("\\\""),
            '\n' => w.write("\\n"),
            '\r' => w.write("\\r"),
            '\t' => w.write("\\t"),
            _ => w.write(&ch.to_string()),
        }
    }
    w.write("\"");
}
