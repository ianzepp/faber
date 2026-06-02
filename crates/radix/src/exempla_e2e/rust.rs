use super::common::{
    collect_exempla_files, expected_runtime_failure, format_result_paths, is_expected_failure,
    make_temp_root, normalize_newline, read_expected_stdout,
};
use super::types::E2eResult;
use crate::{Compiler, Config, Output};
use std::fs;
use std::path::Path;
use std::process::Command;

const RUST_EXPECTED_FAILURES: &[&str] = &[];
const RUST_EXPECTED_RUNTIME_FAILURES: &[(&str, &str)] = &[("ad/ad.fab", "E_NO_ROUTE: unresolved capability")];

#[test]
#[ignore = "slow end-to-end harness; run explicitly with cargo test exempla_rust_e2e -- --ignored"]
fn exempla_rust_e2e() {
    if !rustc_available() {
        eprintln!("rustc not found on PATH; skipping exempla end-to-end harness");
        return;
    }

    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();

    let compiler = Compiler::new(Config::default());
    let temp_root = make_temp_root();
    let mut results = Vec::with_capacity(exempla.len());
    let mut expected_count = 0usize;

    for (idx, file) in exempla.iter().enumerate() {
        let expected = read_expected_stdout(file);
        if expected.is_some() {
            expected_count += 1;
        }

        let result = compiler.compile(file);
        let output = match result.output {
            Some(Output::Rust(output)) => output.code,
            Some(_) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: "compiler did not produce Rust output".to_owned(),
                });
                continue;
            }
            None => {
                let diagnostics = result
                    .diagnostics
                    .iter()
                    .map(|diag| diag.message.clone())
                    .collect::<Vec<_>>()
                    .join(" | ");
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("compile failed: {diagnostics}"),
                });
                continue;
            }
        };

        // Exercise the new --format + --linter path in the e2e harness
        let mut output = crate::tool::format_generated_code(crate::codegen::Target::Rust, &output).unwrap_or(output);

        if let Ok(fixed) = crate::tool::lint_generated_code(crate::codegen::Target::Rust, &output) {
            output = fixed;
        }

        let stem = file
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("exemplum");
        let rust_file = temp_root.join(format!("{idx:03}-{stem}.rs"));
        let bin_file = temp_root.join(format!("{idx:03}-{stem}.bin"));

        if let Err(err) = fs::write(&rust_file, output) {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: format!("cannot write rust output: {err}"),
            });
            continue;
        }

        let rustc = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&rust_file)
            .arg("-o")
            .arg(&bin_file)
            .output();

        let rustc = match rustc {
            Ok(rustc) => rustc,
            Err(err) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("cannot execute rustc: {err}"),
                });
                continue;
            }
        };

        if !rustc.status.success() {
            let stderr = String::from_utf8_lossy(&rustc.stderr).trim().to_owned();
            results.push(E2eResult { path: file.clone(), passed: false, reason: format!("rustc failed: {stderr}") });
            continue;
        }

        let run = Command::new(&bin_file).output();
        let run = match run {
            Ok(run) => run,
            Err(err) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("cannot run binary: {err}"),
                });
                continue;
            }
        };

        if !run.status.success() {
            let stderr = String::from_utf8_lossy(&run.stderr).trim().to_owned();
            if let Some(expected) = expected_runtime_failure(file, RUST_EXPECTED_RUNTIME_FAILURES) {
                if stderr.contains(expected) {
                    results.push(E2eResult {
                        path: file.clone(),
                        passed: true,
                        reason: format!("expected runtime failure: {expected}"),
                    });
                    continue;
                }
            }
            results.push(E2eResult { path: file.clone(), passed: false, reason: format!("binary failed: {stderr}") });
            continue;
        }
        if let Some(expected) = expected_runtime_failure(file, RUST_EXPECTED_RUNTIME_FAILURES) {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: format!("expected runtime failure containing `{expected}`, but binary succeeded"),
            });
            continue;
        }

        let stdout = normalize_newline(&String::from_utf8_lossy(&run.stdout));
        if let Some(expected) = expected {
            if stdout != expected {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("stdout mismatch: expected `{expected}`, got `{stdout}`"),
                });
                continue;
            }
        }

        results.push(E2eResult { path: file.clone(), passed: true, reason: String::new() });
    }

    let pass_count = results.iter().filter(|r| r.passed).count();
    eprintln!("Rust e2e exempla: {pass_count}/{} exempla files pass end-to-end", results.len());
    eprintln!("Expected-output checks enabled for {expected_count} exempla files");

    for fail in results.iter().filter(|r| !r.passed) {
        eprintln!("[fail] {} :: {}", fail.path.display(), fail.reason);
    }

    let unexpected_failures = results
        .iter()
        .filter(|r| !r.passed && !is_expected_failure(&r.path, RUST_EXPECTED_FAILURES))
        .collect::<Vec<_>>();
    let unexpected_passes = results
        .iter()
        .filter(|r| r.passed && is_expected_failure(&r.path, RUST_EXPECTED_FAILURES))
        .collect::<Vec<_>>();

    assert!(
        unexpected_failures.is_empty(),
        "unexpected Rust e2e failures: {}",
        format_result_paths(&unexpected_failures)
    );
    assert!(
        unexpected_passes.is_empty(),
        "Rust e2e expected failures now pass and should be removed from metadata: {}",
        format_result_paths(&unexpected_passes)
    );
}

fn rustc_available() -> bool {
    Command::new("rustc").arg("--version").output().is_ok()
}
