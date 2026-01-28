//! radix CLI

use std::io::{self, Read};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "lex" => cmd_lex(&args[2..]),
        "parse" => cmd_parse(&args[2..]),
        "check" => cmd_check(&args[2..]),
        "emit" => cmd_emit(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("radix - Faber compiler");
    eprintln!();
    eprintln!("Usage: radix <command> [options] [file]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  lex <file>              Tokenize and output JSON");
    eprintln!("  parse <file>            Parse and output AST as JSON");
    eprintln!("  check <file>            Run semantic analysis");
    eprintln!("  emit [-t target] <file> Compile to target (rust, faber)");
    eprintln!();
    eprintln!("If no file is given, reads from stdin.");
}

fn read_source(args: &[String]) -> (String, String) {
    if args.is_empty() || args[0] == "-" {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source).expect("failed to read stdin");
        ("<stdin>".to_owned(), source)
    } else {
        let path = PathBuf::from(&args[0]);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| {
                eprintln!("error: cannot read '{}': {}", path.display(), e);
                std::process::exit(1);
            });
        (args[0].clone(), source)
    }
}

fn cmd_lex(args: &[String]) {
    let (name, source) = read_source(args);
    let result = radix::lexer::lex(&source);

    // Output as JSON
    println!("{{");
    println!("  \"file\": \"{}\",", escape_json(&name));
    println!("  \"success\": {},", result.success());
    println!("  \"tokens\": [");

    for (i, token) in result.tokens.iter().enumerate() {
        let comma = if i + 1 < result.tokens.len() { "," } else { "" };
        let kind = format!("{:?}", token.kind);
        // Truncate long token kinds
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

fn cmd_parse(args: &[String]) {
    let (name, source) = read_source(args);
    let lex_result = radix::lexer::lex(&source);

    if !lex_result.success() {
        eprintln!("lexer errors:");
        for err in &lex_result.errors {
            eprintln!("  {}: {}", err.span.start, err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);

    println!("{{");
    println!("  \"file\": \"{}\",", escape_json(&name));
    println!("  \"success\": {},", parse_result.success());

    if let Some(program) = &parse_result.program {
        println!("  \"statements\": {},", program.stmts.len());
        println!("  \"ast\": [");

        for (i, stmt) in program.stmts.iter().enumerate() {
            let comma = if i + 1 < program.stmts.len() { "," } else { "" };
            let kind = format!("{:?}", stmt.kind);
            // Just show the variant name
            let kind_name = kind.split('(').next().unwrap_or(&kind);
            println!(
                "    {{ \"id\": {}, \"kind\": \"{}\", \"span\": [{}, {}] }}{}",
                stmt.id,
                kind_name,
                stmt.span.start,
                stmt.span.end,
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

fn cmd_check(args: &[String]) {
    let (name, source) = read_source(args);

    let lex_result = radix::lexer::lex(&source);
    if !lex_result.success() {
        for err in &lex_result.errors {
            eprintln!("{}:{}: {}", name, err.span.start, err.message);
        }
        std::process::exit(1);
    }

    let parse_result = radix::parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            eprintln!("{}:{}: {}", name, err.span.start, err.message);
        }
        std::process::exit(1);
    }

    let program = parse_result.program.unwrap();
    let pass_config = radix::semantic::PassConfig::for_target(radix::codegen::Target::Rust);
    let semantic_result = radix::semantic::analyze(&program, &pass_config);

    for err in &semantic_result.errors {
        let prefix = if err.is_error() { "error" } else { "warning" };
        eprintln!("{}:{}:{}: {}", prefix, name, err.span.start, err.message);
    }

    if semantic_result.success() {
        eprintln!("ok: {}", name);
    } else {
        std::process::exit(1);
    }
}

fn cmd_emit(args: &[String]) {
    let mut target = radix::codegen::Target::Rust;
    let mut file_args = args;

    // Parse -t/--target flag
    if args.len() >= 2 && (args[0] == "-t" || args[0] == "--target") {
        target = match args[1].as_str() {
            "rust" | "rs" => radix::codegen::Target::Rust,
            "faber" | "fab" => radix::codegen::Target::Faber,
            other => {
                eprintln!("unknown target: {}", other);
                std::process::exit(1);
            }
        };
        file_args = &args[2..];
    }

    let (name, source) = read_source(file_args);

    let config = radix::driver::Config::default()
        .with_target(target)
        .with_crate_name("output");

    let compiler = radix::Compiler::new(config);
    let result = compiler.compile_str(&name, &source);

    for diag in &result.diagnostics {
        if diag.is_error() {
            eprintln!("error: {}", diag.message);
        } else {
            eprintln!("warning: {}", diag.message);
        }
    }

    match result.output {
        Some(radix::Output::Rust(out)) => {
            println!("{}", out.code);
        }
        Some(radix::Output::Faber(out)) => {
            println!("{}", out.code);
        }
        None => {
            eprintln!("compilation failed");
            std::process::exit(1);
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
