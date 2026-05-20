use crate::{CompileResult, Compiler, Config, Diagnostic};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_package_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("radix-lib-test-package-{}-{}-{}", label, std::process::id(), nanos));
    std::fs::create_dir_all(&path).expect("create temp package");
    path
}

#[test]
fn compile_result_success_requires_output_and_no_errors() {
    let ok = CompileResult {
        output: Some(crate::Output::Rust(crate::RustOutput { code: String::new() })),
        diagnostics: vec![Diagnostic::warning("non-fatal")],
    };
    assert!(ok.success());

    let no_output = CompileResult { output: None, diagnostics: Vec::new() };
    assert!(!no_output.success());

    let with_error = CompileResult {
        output: Some(crate::Output::Faber(crate::FaberOutput { code: String::new() })),
        diagnostics: vec![Diagnostic::error("fatal")],
    };
    assert!(!with_error.success());
}

#[test]
fn compiler_compile_str_produces_output_for_valid_source() {
    let compiler = Compiler::new(Config::default());
    let result = compiler.compile_str("test.fab", "incipit {}");

    assert!(result.success());
    assert!(result.output.is_some());
}

#[test]
fn compiler_compile_reports_io_error_for_missing_file() {
    let compiler = Compiler::new(Config::default());
    let result = compiler.compile(Path::new("/definitely/missing/faber_test_input.fab"));

    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|d| d.is_error()));
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("cannot read")));
}

#[test]
fn compiler_compile_package_supports_manifest_example() {
    let package = temp_package_dir("basic");
    std::fs::write(package.join("main.fab"), "incipit {}").expect("write package entry");

    let compiler = Compiler::new(Config::default());
    let result = compiler.compile_package(&package);

    assert!(result.success(), "expected package compile success");
}

#[test]
fn compiler_emits_rust_runnable_cli_codegen_after_phase_03() {
    let compiler = Compiler::new(Config::default());
    let result = compiler.compile_str(
        "cli.fab",
        r#"
@ cli "tool"
incipit argumenta args {}
"#,
    );

    assert!(result.success());
    let Some(crate::Output::Rust(output)) = result.output else {
        panic!("expected Rust CLI output");
    };
    assert!(output.code.contains("parse_cli_args_or_exit"));
}
