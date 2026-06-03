//! Hand-built JSON serializers for CLI inspection commands.

pub(crate) fn annotation_json(annotations: &[crate::syntax::Annotation]) -> String {
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

pub(crate) fn cli_analysis_json(analysis: &crate::cli::CliAnalysis) -> String {
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
