use super::common::{
    collect_exempla_files, format_diagnostics, format_result_paths, is_expected_failure, make_temp_root,
    normalize_newline, read_expected_stdout,
};
use super::types::{E2eFinding, E2eResult};
use crate::codegen::Target;
use crate::{Compiler, Config, Output};
use std::fs;
use std::path::Path;
use std::process::Command;

const GO_EXPECTED_FAILURES: &[&str] = &[
    "ad/ad.fab",
    "inter/inter.fab",
    "itera/cursor-iteratio.fab",
    "itera/nidificatus.fab",
    "syntaxis/arena-mixta.fab",
    "syntaxis/destructura-sparsa.fab",
    "syntaxis/fluxus-cede.fab",
];

#[test]
#[ignore = "slow end-to-end harness; run explicitly with cargo test exempla_go_e2e -- --ignored"]
fn exempla_go_e2e() {
    if !go_available() {
        eprintln!("go not found on PATH; skipping Go exempla end-to-end harness");
        return;
    }

    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();

    let compiler = Compiler::new(Config::default().with_target(Target::Go));
    let temp_root = make_temp_root();
    let mut results = Vec::with_capacity(exempla.len());
    let mut vet_findings = Vec::new();
    let mut expected_count = 0usize;

    for file in &exempla {
        let expected = read_expected_stdout(file);
        if expected.is_some() {
            expected_count += 1;
        }

        let result = compiler.compile(file);
        let output = match result.output {
            Some(Output::Go(output)) => output.code,
            Some(_) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: "compiler did not produce Go output".to_owned(),
                });
                continue;
            }
            None => {
                let diagnostics = format_diagnostics(&result);
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("compile failed: {diagnostics}"),
                });
                continue;
            }
        };

        let mut output = match crate::tool::format_generated_code(crate::codegen::Target::Go, &output) {
            Ok(output) => output,
            Err(err) => {
                results.push(E2eResult { path: file.clone(), passed: false, reason: format!("gofmt failed: {err}") });
                continue;
            }
        };

        // Go lint is currently best-effort in the shared hook. Keep calling it
        // so the harness exercises the tool path, but report real vet findings
        // separately below until the backend is stable enough to make vet hard.
        if let Ok(fixed) = crate::tool::lint_generated_code(crate::codegen::Target::Go, &output) {
            output = fixed;
        }

        let go_file = temp_root.join("main.go");

        if let Err(err) = fs::write(&go_file, output) {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: format!("cannot write go output: {err}"),
            });
            continue;
        }

        let go_vet = Command::new("go")
            .arg("vet")
            .arg("main.go")
            .current_dir(&temp_root)
            .output();

        match go_vet {
            Ok(go_vet) if !go_vet.status.success() => {
                let stderr = String::from_utf8_lossy(&go_vet.stderr).trim().to_owned();
                let stdout = String::from_utf8_lossy(&go_vet.stdout).trim().to_owned();
                let reason = if stderr.is_empty() {
                    stdout
                } else if stdout.is_empty() {
                    stderr
                } else {
                    format!("{stderr}\n{stdout}")
                };
                vet_findings.push(E2eFinding { path: file.clone(), reason: format!("go vet failed: {reason}") });
            }
            Ok(_) => {}
            Err(err) => {
                vet_findings.push(E2eFinding { path: file.clone(), reason: format!("cannot execute go vet: {err}") });
            }
        }

        let go_run = Command::new("go")
            .arg("run")
            .arg("main.go")
            .current_dir(&temp_root)
            .output();

        let go_run = match go_run {
            Ok(go_run) => go_run,
            Err(err) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("cannot execute go: {err}"),
                });
                continue;
            }
        };

        if !go_run.status.success() {
            let stderr = String::from_utf8_lossy(&go_run.stderr).trim().to_owned();
            results.push(E2eResult { path: file.clone(), passed: false, reason: format!("go run failed: {stderr}") });
            continue;
        }

        let stdout = normalize_newline(&String::from_utf8_lossy(&go_run.stdout));
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
    eprintln!("Go e2e exempla: {pass_count}/{} exempla files pass end-to-end", results.len());
    eprintln!("Expected-output checks enabled for {expected_count} exempla files");

    for fail in results.iter().filter(|r| !r.passed) {
        eprintln!("[fail] {} :: {}", fail.path.display(), fail.reason);
    }
    for finding in &vet_findings {
        eprintln!("[vet] {} :: {}", finding.path.display(), finding.reason);
    }

    let unexpected_failures = results
        .iter()
        .filter(|r| !r.passed && !is_expected_failure(&r.path, GO_EXPECTED_FAILURES))
        .collect::<Vec<_>>();
    let unexpected_passes = results
        .iter()
        .filter(|r| r.passed && is_expected_failure(&r.path, GO_EXPECTED_FAILURES))
        .collect::<Vec<_>>();

    assert!(
        unexpected_failures.is_empty(),
        "unexpected Go e2e failures: {}",
        format_result_paths(&unexpected_failures)
    );
    assert!(
        unexpected_passes.is_empty(),
        "Go e2e expected failures now pass and should be removed from metadata: {}",
        format_result_paths(&unexpected_passes)
    );
}

fn go_available() -> bool {
    Command::new("go").arg("version").output().is_ok()
}
