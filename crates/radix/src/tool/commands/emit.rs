//! `emit` and `build` commands plus emitted-artifact path helpers.

use super::super::cli::{BuildCommand, DiagnosticMode, EmitCommand};
use super::compile::{compile_cli_input, compile_cli_path, resolve_package_mode};
use super::postprocess::{format_generated_code, lint_generated_code};
use super::targets::target_extension;
use std::fs;
use std::path::PathBuf;

/// Compile single-file input to a target language and print generated source.
///
/// Package inputs are rejected before compilation so users do not accidentally
/// get a single-file interpretation of a package directory or manifest.
pub fn cmd_emit(command: EmitCommand) {
    let result = compile_cli_input(&command.input, command.package, command.target);

    if command.diagnostic_mode == DiagnosticMode::Diagnostics {
        if !result.diagnostics.is_empty() {
            eprintln!("{}", crate::diagnostics::render_expanded_diagnostics(&result.diagnostics));
        }
    } else {
        for diag in &result.diagnostics {
            if diag.is_error() {
                eprintln!("error: {}", diag.message);
            } else {
                eprintln!("warning: {}", diag.message);
            }
        }
    }

    let Some(output) = result.output else {
        eprintln!("compilation failed");
        std::process::exit(1);
    };

    let mut code = output_code(output);

    if command.format {
        match format_generated_code(command.target, &code) {
            Ok(formatted) => code = formatted,
            Err(err) => {
                eprintln!("warning: formatting failed: {err}");
            }
        }
    }

    if command.linter {
        match lint_generated_code(command.target, &code) {
            Ok(fixed) => code = fixed,
            Err(err) => {
                eprintln!("warning: linter failed: {err}");
            }
        }
    }

    println!("{}", code);
}

/// Compile single-file input and write generated source to disk.
///
/// This is intentionally source-emission only inside `radix`; executable
/// package builds are routed through `faber`, where generated Cargo layout
/// policy is available.
pub fn cmd_build(command: BuildCommand) {
    let input_path = PathBuf::from(&command.input);
    let is_package = resolve_package_mode(&input_path, command.package);
    let result = compile_cli_path(&input_path, is_package, command.target);

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

    let output_path = build_output_path(&command.out_dir, &input_path, command.target, is_package);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            eprintln!("error: failed to create '{}': {}", parent.display(), err);
            std::process::exit(1);
        });
    }

    let mut code = output_code(output);

    if command.format {
        match format_generated_code(command.target, &code) {
            Ok(formatted) => code = formatted,
            Err(err) => {
                eprintln!("warning: formatting failed: {err}");
            }
        }
    }

    if command.linter {
        match lint_generated_code(command.target, &code) {
            Ok(fixed) => code = fixed,
            Err(err) => {
                eprintln!("warning: linter failed: {err}");
            }
        }
    }

    fs::write(&output_path, code).unwrap_or_else(|err| {
        eprintln!("error: failed to write '{}': {}", output_path.display(), err);
        std::process::exit(1);
    });

    println!("{}", output_path.display());
}

/// Derive the output path for source-emission builds.
///
/// Package mode currently uses a stable `main.<ext>` placeholder here because
/// real package build layout is outside `radix` and owned by `faber`.
pub fn build_output_path(
    out_dir: &std::path::Path,
    input_path: &std::path::Path,
    target: crate::codegen::Target,
    is_package: bool,
) -> PathBuf {
    let base_name = if is_package {
        "main".to_owned()
    } else {
        input_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("out")
            .to_owned()
    };
    out_dir.join(format!("{}.{}", base_name, target_extension(target)))
}

/// Extract generated source from a target-specific compiler output.
pub fn output_code(output: crate::Output) -> String {
    match output {
        crate::Output::Rust(out) => out.code,
        crate::Output::Faber(out) => out.code,
        crate::Output::TypeScript(out) => out.code,
        crate::Output::Go(out) => out.code,
        crate::Output::WasmText(out) => out.code,
        crate::Output::LlvmText(out) => out.code,
    }
}
