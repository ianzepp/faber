use crate::codegen::Target;
use crate::{Compiler, Config, Output};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct E2eResult {
    path: PathBuf,
    passed: bool,
    reason: String,
}

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
    assert_eq!(exempla.len(), 131, "expected 131 exempla files");

    let compiler = Compiler::new(Config::default());
    let temp_root = make_temp_root();
    let mut results = Vec::with_capacity(exempla.len());
    let mut expected_count = 0usize;

    for (idx, file) in exempla.iter().enumerate() {
        let expected = read_expected_stdout(file);
        if expected.is_some() {
            expected_count += 1;
        }

        let result = compiler.compile_package(file);
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
            results.push(E2eResult { path: file.clone(), passed: false, reason: format!("binary failed: {stderr}") });
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

    let salve_ok = results
        .iter()
        .find(|r| r.path.ends_with("salve-munde.fab"))
        .map(|r| r.passed)
        .unwrap_or(false);
    assert!(salve_ok, "salve-munde.fab should pass end-to-end");
}

#[test]
#[ignore = "slow round-trip harness; run explicitly with cargo test exempla_faber_roundtrip_e2e -- --ignored"]
fn exempla_faber_roundtrip_e2e() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();
    assert_eq!(exempla.len(), 131, "expected 131 exempla files");

    let compiler = Compiler::new(Config::default().with_target(Target::Faber));
    let mut results = Vec::with_capacity(exempla.len());

    for file in &exempla {
        let source = match fs::read_to_string(file) {
            Ok(source) => source,
            Err(err) => {
                results.push(E2eResult {
                    path: file.clone(),
                    passed: false,
                    reason: format!("cannot read source: {err}"),
                });
                continue;
            }
        };

        let first = compiler.compile(file);
        let Some(Output::Faber(first_output)) = first.output else {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: format!("first faber compile failed: {}", format_diagnostics(&first)),
            });
            continue;
        };

        let second = compiler.compile_str(&file.display().to_string(), &first_output.code);
        let Some(Output::Faber(second_output)) = second.output else {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: format!("second faber compile failed: {}", format_diagnostics(&second)),
            });
            continue;
        };

        if first_output.code != second_output.code {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: "faber emit did not stabilize after one round-trip".to_owned(),
            });
            continue;
        }

        if source.trim().is_empty() {
            results.push(E2eResult {
                path: file.clone(),
                passed: false,
                reason: "source file was unexpectedly empty".to_owned(),
            });
            continue;
        }

        results.push(E2eResult { path: file.clone(), passed: true, reason: String::new() });
    }

    let pass_count = results.iter().filter(|r| r.passed).count();
    eprintln!(
        "Faber roundtrip exempla: {pass_count}/{} exempla files stabilize",
        results.len()
    );

    for fail in results.iter().filter(|r| !r.passed) {
        eprintln!("[fail] {} :: {}", fail.path.display(), fail.reason);
    }

    let salve_ok = results
        .iter()
        .find(|r| r.path.ends_with("salve-munde.fab"))
        .map(|r| r.passed)
        .unwrap_or(false);
    assert!(salve_ok, "salve-munde.fab should stabilize through Faber round-trip");
}

fn rustc_available() -> bool {
    Command::new("rustc").arg("--version").output().is_ok()
}

fn make_temp_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("radix-rs-e2e-{nanos}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn collect_exempla_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_exempla_files_recursive(dir, &mut files);
    files
}

fn collect_exempla_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_exempla_files_recursive(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("fab") {
            out.push(path);
        }
    }
}

fn read_expected_stdout(fab_path: &Path) -> Option<String> {
    let expected_path = fab_path.with_extension("expected");
    let content = fs::read_to_string(expected_path).ok()?;
    Some(normalize_newline(&content))
}

fn normalize_newline(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end_matches('\n').to_owned()
}

fn format_diagnostics(result: &crate::CompileResult) -> String {
    if result.diagnostics.is_empty() {
        "no diagnostics".to_owned()
    } else {
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
