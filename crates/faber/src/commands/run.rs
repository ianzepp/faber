//! `faber run` — compile a package and execute its Rust binary.

use crate::cli::RunArgs;
use crate::package;

/// Builds a package as Rust and forwards process exit semantics from the result binary.
pub(super) fn cmd_run(args: RunArgs) {
    use std::path::PathBuf;
    use std::process::Command;

    let input_path = PathBuf::from(&args.path);

    // POLICY: `run` is package-scoped, so stale generated crates are never
    // trusted over the current Faber sources.
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

    // EDGE: legacy entry paths still need a build layout so existing examples
    // remain runnable while package manifests become the preferred surface.
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

    // CONTRACT: `faber run` behaves like the compiled program for callers that
    // depend on argv forwarding and process status.
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
