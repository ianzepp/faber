//! Phase-inspection commands that emit JSON or MIR text to stdout.

use super::json::{annotation_json, cli_analysis_json, escape_json};
use super::source::{format_location, format_optional_location, read_source, source_file_from_input};

pub fn cmd_lex(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let result = crate::lexer::lex(source_file.content.as_str());

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

/// Parse source and emit a compact AST inspection payload.
///
/// This command stops after parsing and reports lexer/parser diagnostics
/// directly, making it useful for grammar work where semantic phases would add
/// distracting failures.
pub fn cmd_parse(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);
    let lex_result = crate::lexer::lex(source_file.content.as_str());

    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);

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

/// Lower parsed source to HIR and emit a compact inspection payload.
///
/// HIR inspection runs the prerequisite collection and resolution passes before
/// lowering so IDs and spans reflect the same semantic inputs used by codegen.
pub fn cmd_hir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    // WHY: HIR lowering requires lexing, parsing, and name resolution
    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let mut resolver = crate::semantic::Resolver::new();
    let mut types = crate::semantic::TypeTable::new();

    if let Err(e) = crate::semantic::passes::collect::collect(&program, &mut resolver, &mut types) {
        eprintln!("collection errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    if let Err(e) = crate::semantic::passes::resolve::resolve(&program, &mut resolver, &interner, &mut types) {
        eprintln!("resolution errors:");
        for err in e {
            eprintln!("  {:?}: {}", err.kind, err.message);
        }
        std::process::exit(1);
    }

    let (hir, errors) = crate::hir::lower(&program, &resolver, &mut types, &interner);

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

/// Lower checked source to MIR and print the deterministic MIR dump.
pub fn cmd_mir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    match mir_output_for_source(&source_file.name, &source_file.content) {
        Ok(output) => print!("{output}"),
        Err(messages) => {
            for message in messages {
                eprintln!("{message}");
            }
            std::process::exit(1);
        }
    }
}

/// Produce MIR inspection text without exiting the process.
///
/// Errors are returned already formatted for terminal display because MIR
/// inspection is primarily a developer-tool surface, not a library data model.
pub fn mir_output_for_source(name: &str, source: &str) -> Result<String, Vec<String>> {
    let source_file = source_file_from_input(name.to_owned(), source.to_owned());
    let session =
        crate::driver::Session::new(crate::driver::Config::default().with_target(crate::codegen::Target::Faber));

    let analysis = match crate::driver::analyze_source(&session, &source_file.name, &source_file.content) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            return Err(diagnostics
                .into_iter()
                .map(|diagnostic| {
                    let prefix = if diagnostic.is_error() { "error" } else { "warning" };
                    format!(
                        "{}: {}: {}",
                        prefix,
                        format_optional_location(&source_file, diagnostic.span),
                        diagnostic.message
                    )
                })
                .collect())
        }
    };

    match crate::mir::dump_analyzed_unit(&analysis) {
        Ok(output) => Ok(output),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| format!("error: {}: {}", format_location(&source_file, err.span.start), err.message))
            .collect()),
    }
}

/// Analyze CLI annotations and print the normalized CLI IR.
pub fn cmd_cli_ir(args: &[String]) {
    let (name, source) = read_source(args);
    let source_file = source_file_from_input(name, source);

    let lex_result = crate::lexer::lex(source_file.content.as_str());
    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let parse_result = crate::parser::parse(lex_result);
    if !parse_result.success() {
        eprintln!("parser errors:");
        for err in &parse_result.errors {
            eprintln!("  {}: {}", format_location(&source_file, err.span.start), err.message);
        }
        std::process::exit(1);
    }

    let crate::parser::ParseResult { program, interner, .. } = parse_result;
    let Some(program) = program else {
        eprintln!("internal error: successful parse result missing program");
        std::process::exit(1);
    };

    let cli_analysis = crate::cli::analyze(&program, &interner);
    println!("{}", cli_analysis_json(&cli_analysis));

    if !cli_analysis.errors.is_empty() {
        std::process::exit(1);
    }
}
