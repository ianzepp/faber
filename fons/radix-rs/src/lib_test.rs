use crate::{CompileResult, Compiler, Config, Diagnostic};
use std::path::Path;

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
