//! User-facing Faber project tool (`faber` binary).

mod package;

use clap::{Parser, Subcommand};
use faber::explain::{self, Registry};
use radix::tool::{self, BuildCommand, CheckCommand, EmitCommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "faber",
    bin_name = "faber",
    about = "Faber project and package tool",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compile a file or package and write output to disk
    Build(radix::tool::BuildArgs),
    /// Show supported targets and current capability notes
    Targets,
    /// Run semantic analysis on a file or package
    Check(radix::tool::CheckArgs),
    /// Create a new Faber package (planned)
    Init(InitArgs),
    /// Explain a Faber glyph, keyword, or grammar term
    Explain(ExplainArgs),
    /// Build (if needed) and run a compiled package
    Run(RunArgs),
    /// Run package tests (planned)
    Test(TestArgs),
    /// Tokenize source and output JSON (compatibility alias for `radix lex`)
    Lex(radix::tool::InputArgs),
    /// Parse source and output AST as JSON (compatibility alias for `radix parse`)
    Parse(radix::tool::InputArgs),
    /// Lower AST to HIR and output as JSON (compatibility alias for `radix hir`)
    Hir(radix::tool::InputArgs),
    /// Validate and output normalized CLI IR as JSON (compatibility alias for `radix cli-ir`)
    CliIr(radix::tool::InputArgs),
    /// Compile to target for stdout (compatibility alias for `radix emit`)
    Emit(radix::tool::EmitArgs),
}

#[derive(clap::Args, Debug)]
struct InitArgs {
    /// Target directory for the new package
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[derive(clap::Args, Debug)]
struct ExplainArgs {
    /// Emit a machine-readable JSON explanation
    #[arg(long)]
    json: bool,

    /// Search across explain entries and show ranked matches
    #[arg(long)]
    search: Option<String>,

    /// List canonical explain terms
    #[arg(long)]
    list: bool,

    /// List canonical and legacy entries in a category
    #[arg(long)]
    category: Option<String>,

    /// Term, alias, or legacy spelling to explain
    term: Option<String>,
}

#[derive(clap::Args, Debug)]
struct RunArgs {
    /// Package path to run (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Run the release binary
    #[arg(long)]
    release: bool,

    /// Arguments passed to the executed program (after --)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(clap::Args, Debug)]
struct TestArgs {
    /// Package path to test
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Build(args) => package::cmd_build(BuildCommand {
            input: args.input,
            out_dir: args.out_dir,
            package: args.package,
            release: args.release,
            target: args.target.into(),
        }),
        Command::Targets => tool::cmd_targets(),
        Command::Check(args) => {
            if args.package || package::should_treat_as_package_from_args(&args.input) {
                package::cmd_check_package(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
                });
            } else {
                tool::cmd_check(CheckCommand {
                    input: args.input,
                    package: args.package,
                    permissive: args.permissive,
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
                });
            } else {
                tool::cmd_emit(EmitCommand {
                    input: args.input,
                    package: args.package,
                    target: args.target.into(),
                });
            }
        }
    }
}

fn cmd_init(args: InitArgs) {
    let root = args.path;
    let src = root.join("src");
    let manifest = root.join("faber.toml");
    let entry = src.join("main.fab");
    let package_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && *name != ".")
        .unwrap_or("faber-package");

    if manifest.exists() || entry.exists() {
        eprintln!(
            "error: package files already exist in {}; refusing to overwrite",
            root.display()
        );
        std::process::exit(1);
    }

    if let Err(err) = std::fs::create_dir_all(&src) {
        eprintln!("error: failed to create '{}': {}", src.display(), err);
        std::process::exit(1);
    }

    let manifest_source = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[paths]\nsource = \"src\"\nentry = \"main.fab\"\n\n[build]\ntarget = \"rust\"\nkind = \"bin\"\n",
        package_name
    );
    if let Err(err) = std::fs::write(&manifest, manifest_source) {
        eprintln!("error: failed to write '{}': {}", manifest.display(), err);
        std::process::exit(1);
    }

    if let Err(err) = std::fs::write(&entry, "incipit {\n    scribe \"Salve, munde!\"\n}\n") {
        eprintln!("error: failed to write '{}': {}", entry.display(), err);
        std::process::exit(1);
    }

    println!("{}", manifest.display());
}

fn cmd_explain(args: ExplainArgs) {
    let registry = match Registry::load() {
        Ok(registry) => registry,
        Err(err) => {
            eprintln!("error: failed to load explain corpus: {err}");
            std::process::exit(1);
        }
    };

    if args.list {
        print!("{}", explain::render_list(&registry));
        return;
    }

    if let Some(category) = args.category {
        match explain::render_category(&registry, &category) {
            Some(output) => print!("{output}"),
            None => {
                eprintln!("error: no explanations found in category '{category}'");
                let categories = registry
                    .categories()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ");
                eprintln!("hint: available categories: {categories}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(query) = args.search {
        if args.json {
            eprintln!("error: --json cannot be combined with --search");
            std::process::exit(2);
        }
        if args.term.is_some() {
            eprintln!("error: --search cannot be combined with a term");
            std::process::exit(2);
        }

        let hits = registry.search(&query);
        if hits.is_empty() {
            eprintln!("error: no matches found for '{query}'");
            eprintln!("hint: run `faber explain --list`");
            std::process::exit(1);
        }

        print!("{}", explain::render_search(&query, &hits));
        return;
    }

    let Some(term) = args.term else {
        eprintln!("error: no explain query was provided");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("    faber explain <term>");
        eprintln!("    faber explain --list");
        eprintln!("    faber explain --category <category>");
        eprintln!("    faber explain --search <query>");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("    faber explain functio");
        eprintln!("    faber explain ≡");
        eprintln!("    faber explain ==");
        eprintln!("    faber explain --list");
        std::process::exit(2);
    };

    let Some(lookup) = registry.lookup(&term) else {
        eprintln!("error: no explanation found for '{term}'");
        eprintln!("hint: run `faber explain --list`");
        std::process::exit(1);
    };

    if args.json {
        match explain::render_json(&lookup) {
            Ok(json) => println!("{json}"),
            Err(err) => {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
    } else {
        print!("{}", explain::render_plain(&lookup));
    }
}

fn cmd_run(args: RunArgs) {
    use std::path::PathBuf;
    use std::process::Command;

    let input_path = PathBuf::from(&args.path);

    // Always (re)build for the package at the given path.
    // run is intended for packages; treat the input as package input.
    let config = radix::driver::Config::default().with_target(radix::codegen::Target::Rust);
    let result = package::compile_package(&config, &input_path);

    for diag in &result.diagnostics {
        if diag.is_error() {
            eprintln!("error: {}", diag.message);
        } else {
            eprintln!("warning: {}", diag.message);
        }
    }

    let Some(output) = result.output else {
        eprintln!("compilation failed");
        std::process::exit(1);
    };

    // Discover layout (works for package or legacy entry)
    let layout = match package::discover_build_layout(&input_path) {
        Ok(l) => l,
        Err(d) => {
            eprintln!("error: {}", d.message);
            std::process::exit(1);
        }
    };

    let meta = if layout.manifest_path.exists() {
        package::read_manifest(&layout.manifest_path).ok()
    } else {
        None
    };

    let code_string = match output {
        radix::Output::Rust(r) => r.code,
        _ => {
            eprintln!("error: run only supports Rust backend packages");
            std::process::exit(1);
        }
    };

    if let Err(d) = package::emit_generated_crate(&layout, &code_string, meta.as_ref()) {
        eprintln!("error emitting: {}", d.message);
        std::process::exit(1);
    }

    let binary = match package::invoke_cargo_build(&layout, args.release) {
        Ok(b) => b,
        Err(d) => {
            eprintln!("error: {}", d.message);
            std::process::exit(1);
        }
    };

    // Now exec the binary, forwarding args and exit status
    let status = Command::new(&binary)
        .args(&args.args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: failed to execute {}: {}", binary.display(), e);
            std::process::exit(1);
        });

    if let Some(code) = status.code() {
        std::process::exit(code);
    } else {
        std::process::exit(1);
    }
}

fn cmd_test(args: TestArgs) {
    eprintln!(
        "error: `faber test` is not implemented yet; use `faber check` on {}",
        args.path.display()
    );
    std::process::exit(1);
}
