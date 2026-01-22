mod bundler;
mod emitter_faber;
mod emitter_rs;
mod lexer;

use std::env;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use subsidia_rs::{format_error, parse, prepare};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_usage();
        process::exit(0);
    }

    let command = &args[1];
    let valid_commands = ["emit", "parse", "lex", "bundle"];
    if !valid_commands.contains(&command.as_str()) {
        eprintln!("Unknown command: {}", command);
        process::exit(1);
    }

    if command == "bundle" {
        if args.len() < 3 {
            eprintln!("Usage: nanus-rs bundle <dir> [--entry <file>]");
            process::exit(1);
        }
        let dir_path = &args[2];
        let mut entry_name = "main.rs".to_string();

        let mut i = 3;
        while i < args.len() {
            if args[i] == "--entry" && i + 1 < args.len() {
                entry_name = args[i + 1].clone();
                i += 2;
            } else {
                i += 1;
            }
        }

        let dir = Path::new(dir_path);
        let entry_file = dir.join(entry_name);

        if !dir.exists() {
            eprintln!("Directory not found: {}", dir_path);
            process::exit(1);
        }
        if !entry_file.exists() {
            eprintln!("Entry file not found: {}", entry_file.display());
            process::exit(1);
        }

        if let Err(e) = bundler::bundle(dir, &entry_file) {
            eprintln!("Bundle failed: {}", e);
            process::exit(1);
        }
        println!("Bundle complete for {}", dir_path);
        return;
    }

    // Parse flags
    let mut target = "rs".to_string();
    let mut filename = "<stdin>".to_string();
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-t" && i + 1 < args.len() {
            target = args[i + 1].clone();
            i += 2;
        } else if args[i] == "--stdin-filename" && i + 1 < args.len() {
            filename = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    // Validate target
    if command == "emit" && target != "rs" && target != "fab" {
        eprintln!("Unknown target: {}. Valid: rs, fab", target);
        process::exit(1);
    }

    // Read source from stdin
    let mut source = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut source) {
        eprintln!("{}", e);
        process::exit(1);
    }

    match run(command, &source, &target, &filename) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("{}", format_error(&e, &source));
            process::exit(1);
        }
    }
}

fn run(
    command: &str,
    source: &str,
    target: &str,
    filename: &str,
) -> Result<String, subsidia_rs::CompileError> {
    match command {
        "lex" => {
            let tokens = lexer::lex(source, filename)?;
            let json = serde_json::to_string_pretty(&tokens).unwrap();
            Ok(json)
        }
        "parse" => {
            let tokens = lexer::lex(source, filename)?;
            let tokens = prepare(tokens);
            let ast = parse(tokens, filename)?;
            let json = serde_json::to_string_pretty(&ast).unwrap();
            Ok(json)
        }
        "emit" => {
            let tokens = lexer::lex(source, filename)?;
            let tokens = prepare(tokens);
            let ast = parse(tokens, filename)?;
            match target {
                "fab" => Ok(emitter_faber::emit_faber(&ast)),
                "rs" => Ok(emitter_rs::emit_rs(&ast)),
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
    println!("  bundle   Wire up generated Rust files into a module tree");
    println!();
    println!("Options:");
    println!("  -t <target>            Output target: rs, fab (default: rs)");
    println!("  --stdin-filename <f>   Filename for error messages (default: <stdin>)");
    println!("  --entry <file>         Entry point for bundle (default: main.rs)");
}
