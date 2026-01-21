mod emitter_faber;
mod emitter_ts;
mod lexer;

use std::env;
use std::io::{self, Read};
use std::process;

use subsidia_rs::{format_error, parse, prepare};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_usage();
        process::exit(0);
    }

    let command = &args[1];
    let valid_commands = ["emit", "parse", "lex"];
    if !valid_commands.contains(&command.as_str()) {
        eprintln!("Unknown command: {}", command);
        process::exit(1);
    }

    // Parse flags for emit command
    let mut target = "fab".to_string();
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-t" && i + 1 < args.len() {
            target = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    // Validate target
    if command == "emit" && target != "fab" && target != "ts" {
        eprintln!("Unknown target: {}. Valid: fab, ts", target);
        process::exit(1);
    }

    // Read source from stdin
    let mut source = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut source) {
        eprintln!("{}", e);
        process::exit(1);
    }

    match run(command, &source, &target) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("{}", format_error(&e, &source));
            process::exit(1);
        }
    }
}

fn run(command: &str, source: &str, target: &str) -> Result<String, subsidia_rs::CompileError> {
    match command {
        "lex" => {
            let tokens = lexer::lex(source, "<stdin>")?;
            let json = serde_json::to_string_pretty(&tokens).unwrap();
            Ok(json)
        }
        "parse" => {
            let tokens = lexer::lex(source, "<stdin>")?;
            let tokens = prepare(tokens);
            let ast = parse(tokens, "<stdin>")?;
            let json = serde_json::to_string_pretty(&ast).unwrap();
            Ok(json)
        }
        "emit" => {
            let tokens = lexer::lex(source, "<stdin>")?;
            let tokens = prepare(tokens);
            let ast = parse(tokens, "<stdin>")?;
            match target {
                "fab" => Ok(emitter_faber::emit_faber(&ast)),
                "ts" => Ok(emitter_ts::emit_ts(&ast)),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

fn print_usage() {
    println!("nanus-rs: Faber microcompiler (stdin/stdout)");
    println!();
    println!("Usage: <source> | nanus-rs <command> [options]");
    println!();
    println!("Commands:");
    println!("  emit     Compile Faber to target language");
    println!("  parse    Output AST as JSON");
    println!("  lex      Output tokens as JSON");
    println!();
    println!("Options (emit only):");
    println!("  -t <target>   Output target: fab, ts (default: fab)");
}
