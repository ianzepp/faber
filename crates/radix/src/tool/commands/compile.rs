//! Single-file compilation helpers and package-mode policy.

use super::source::read_source;
use std::path::PathBuf;

/// Return whether an input path syntactically names package mode.
pub fn should_treat_as_package(path: &std::path::Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("faber.toml")
}

/// Combine explicit package mode with path-based package detection.
pub fn resolve_package_mode(path: &std::path::Path, force_package: bool) -> bool {
    force_package || should_treat_as_package(path)
}

/// Compile command input after applying stdin and package-mode policy.
pub fn compile_cli_input(input: &[String], package: bool, target: crate::codegen::Target) -> crate::CompileResult {
    if input.is_empty() || input[0] == "-" {
        if package {
            eprintln!("error: package compilation requires a path input");
            std::process::exit(1);
        }

        let (name, source) = read_source(input);
        return compile_cli_source(&name, &source, target);
    }

    let path = PathBuf::from(&input[0]);
    compile_cli_path(&path, resolve_package_mode(&path, package), target)
}

/// Compile a single file path through the public compiler API.
pub fn compile_cli_path(path: &std::path::Path, package: bool, target: crate::codegen::Target) -> crate::CompileResult {
    if package || should_treat_as_package(path) {
        eprintln!("error: package compilation is owned by the `faber` tool; rerun with `faber build` or `faber emit`");
        std::process::exit(1);
    }

    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile(path)
}

/// Compile in-memory command source through the public compiler API.
pub fn compile_cli_source(name: &str, source: &str, target: crate::codegen::Target) -> crate::CompileResult {
    let config = crate::driver::Config::default().with_target(target);
    let compiler = crate::Compiler::new(config);
    compiler.compile_str(name, source)
}
