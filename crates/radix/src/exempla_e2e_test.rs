use crate::codegen::Target;
use crate::driver::Session;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WasmTier {
    SourceReadable,
    FrontendAnalyzed,
    MirLowered,
    WasmEmitted,
    CompileValid,
    InstantiateValid,
    Runnable,
    BehaviorChecked,
}

#[derive(Debug)]
struct WasmE2eResult {
    path: PathBuf,
    tier: WasmTier,
    reason: String,
}

#[derive(Debug, Clone, Copy)]
enum WasmValidator {
    WasmTools,
    Wat2Wasm,
}

#[derive(Debug, Clone, Copy)]
struct WasmToolchain {
    validator: Option<WasmValidator>,
    wasmtime: bool,
}

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

        // Exercise the new --format + --linter path in the e2e harness
        let mut output = crate::tool::format_generated_code(crate::codegen::Target::Go, &output).unwrap_or(output);

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

    let salve_ok = results
        .iter()
        .find(|r| r.path.ends_with("salve-munde.fab"))
        .map(|r| r.passed)
        .unwrap_or(false);
    assert!(salve_ok, "salve-munde.fab should pass end-to-end on Go");
}

#[test]
#[ignore = "slow round-trip harness; run explicitly with cargo test exempla_faber_roundtrip_e2e -- --ignored"]
fn exempla_faber_roundtrip_e2e() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();

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

#[test]
#[ignore = "slow baseline harness; run explicitly with cargo test exempla_wasm_e2e -- --ignored --nocapture"]
fn exempla_wasm_e2e() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();
    assert!(!exempla.is_empty(), "Wasm e2e harness found no exempla files");

    let session = Session::new(Config::default().with_target(Target::WasmText));
    let temp_root = make_temp_root();
    let toolchain = detect_wasm_toolchain();
    let mut results = Vec::with_capacity(exempla.len());

    for (idx, file) in exempla.iter().enumerate() {
        results.push(classify_wasm_exemplum(&session, file, idx, &temp_root, toolchain));
    }

    print_wasm_e2e_report(&results, toolchain);
}

fn rustc_available() -> bool {
    Command::new("rustc").arg("--version").output().is_ok()
}

fn go_available() -> bool {
    Command::new("go").arg("version").output().is_ok()
}

fn detect_wasm_toolchain() -> WasmToolchain {
    let validator = if command_available("wasm-tools", &["--version"]) {
        Some(WasmValidator::WasmTools)
    } else if command_available("wat2wasm", &["--version"]) {
        Some(WasmValidator::Wat2Wasm)
    } else {
        None
    };

    WasmToolchain { validator, wasmtime: command_available("wasmtime", &["--version"]) }
}

fn command_available(command: &str, args: &[&str]) -> bool {
    Command::new(command).args(args).output().is_ok()
}

fn classify_wasm_exemplum(
    session: &Session,
    file: &Path,
    idx: usize,
    temp_root: &Path,
    toolchain: WasmToolchain,
) -> WasmE2eResult {
    let source = match fs::read_to_string(file) {
        Ok(source) => source,
        Err(err) => {
            return WasmE2eResult {
                path: file.to_path_buf(),
                tier: WasmTier::SourceReadable,
                reason: format!("cannot read source: {err}"),
            };
        }
    };

    let analysis = match crate::driver::analyze_source(session, &file.display().to_string(), &source) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            return WasmE2eResult {
                path: file.to_path_buf(),
                tier: WasmTier::SourceReadable,
                reason: format!("frontend failed: {}", format_diagnostic_messages(&diagnostics)),
            };
        }
    };

    let mir = match crate::mir::lower_analyzed_unit(&analysis) {
        Ok(mir) => mir,
        Err(errors) => {
            return WasmE2eResult {
                path: file.to_path_buf(),
                tier: WasmTier::FrontendAnalyzed,
                reason: format!(
                    "MIR lowering failed: {}",
                    errors
                        .iter()
                        .map(|error| error.message.clone())
                        .collect::<Vec<_>>()
                        .join(" | ")
                ),
            };
        }
    };

    let wat = match crate::mir::emit_wasm_text_probe(&mir, &analysis.types, &analysis.interner) {
        Ok(wat) => wat,
        Err(error) => {
            return WasmE2eResult {
                path: file.to_path_buf(),
                tier: WasmTier::MirLowered,
                reason: format!("Wasm emission failed: {error}"),
            };
        }
    };

    let stem = file
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("exemplum");
    let wat_file = temp_root.join(format!("{idx:03}-{stem}.wat"));
    if let Err(err) = fs::write(&wat_file, wat) {
        return WasmE2eResult {
            path: file.to_path_buf(),
            tier: WasmTier::WasmEmitted,
            reason: format!("cannot write WAT output: {err}"),
        };
    }

    let Some(validator) = toolchain.validator else {
        return WasmE2eResult {
            path: file.to_path_buf(),
            tier: WasmTier::WasmEmitted,
            reason: "compile validation skipped: no wasm-tools or wat2wasm on PATH".to_owned(),
        };
    };

    let wasm_file = temp_root.join(format!("{idx:03}-{stem}.wasm"));
    let instantiation_file = match validate_wat(validator, &wat_file, &wasm_file) {
        Ok(instantiation_file) => instantiation_file,
        Err(reason) => {
            return WasmE2eResult { path: file.to_path_buf(), tier: WasmTier::WasmEmitted, reason };
        }
    };

    if !toolchain.wasmtime {
        return WasmE2eResult {
            path: file.to_path_buf(),
            tier: WasmTier::CompileValid,
            reason: "instantiate/run skipped: no wasmtime on PATH".to_owned(),
        };
    }

    match instantiate_wasm(&instantiation_file) {
        Ok(()) => WasmE2eResult {
            path: file.to_path_buf(),
            tier: WasmTier::InstantiateValid,
            reason: "run skipped: no Wasm entrypoint policy yet".to_owned(),
        },
        Err(reason) => WasmE2eResult { path: file.to_path_buf(), tier: WasmTier::CompileValid, reason },
    }
}

fn validate_wat(validator: WasmValidator, wat_file: &Path, wasm_file: &Path) -> Result<PathBuf, String> {
    let output = match validator {
        WasmValidator::WasmTools => Command::new("wasm-tools")
            .arg("validate")
            .arg(wat_file)
            .output(),
        WasmValidator::Wat2Wasm => Command::new("wat2wasm")
            .arg(wat_file)
            .arg("-o")
            .arg(wasm_file)
            .output(),
    };

    let output = output.map_err(|err| format!("cannot execute Wasm validator: {err}"))?;
    if output.status.success() {
        Ok(match validator {
            WasmValidator::WasmTools => wat_file.to_path_buf(),
            WasmValidator::Wat2Wasm => wasm_file.to_path_buf(),
        })
    } else {
        Err(format!(
            "Wasm validation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn instantiate_wasm(wasm_file: &Path) -> Result<(), String> {
    let output = Command::new("wasmtime")
        .arg("--invoke")
        .arg("__faber_missing_entry_for_instantiation_probe__")
        .arg(wasm_file)
        .output()
        .map_err(|err| format!("cannot execute wasmtime: {err}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() || stderr.contains("unknown export") || stderr.contains("failed to find export") {
        Ok(())
    } else {
        Err(format!("Wasm instantiation failed: {}", stderr.trim()))
    }
}

fn print_wasm_e2e_report(results: &[WasmE2eResult], toolchain: WasmToolchain) {
    let total = results.len();
    eprintln!("Wasm e2e toolchain:");
    eprintln!(
        "  compile validator: {}",
        match toolchain.validator {
            Some(WasmValidator::WasmTools) => "wasm-tools validate",
            Some(WasmValidator::Wat2Wasm) => "wat2wasm",
            None => "unavailable",
        }
    );
    eprintln!(
        "  instantiator/runtime: {}",
        if toolchain.wasmtime { "wasmtime" } else { "unavailable" }
    );
    eprintln!("Wasm e2e exempla:");
    eprintln!(
        "  frontend analyzed: {}/{}",
        count_wasm_tier(results, WasmTier::FrontendAnalyzed),
        total
    );
    eprintln!("  MIR lowered: {}/{}", count_wasm_tier(results, WasmTier::MirLowered), total);
    eprintln!("  Wasm emitted: {}/{}", count_wasm_tier(results, WasmTier::WasmEmitted), total);
    eprintln!(
        "  compile-valid: {}/{}",
        count_wasm_tier(results, WasmTier::CompileValid),
        total
    );
    eprintln!(
        "  instantiate-valid: {}/{}",
        count_wasm_tier(results, WasmTier::InstantiateValid),
        total
    );
    eprintln!("  runnable: {}/{}", count_wasm_tier(results, WasmTier::Runnable), total);
    eprintln!(
        "  behavior-checked: {}/{}",
        count_wasm_tier(results, WasmTier::BehaviorChecked),
        total
    );

    for result in results
        .iter()
        .filter(|result| result.tier < WasmTier::BehaviorChecked)
    {
        eprintln!("[wasm:{:?}] {} :: {}", result.tier, result.path.display(), result.reason);
    }
}

fn count_wasm_tier(results: &[WasmE2eResult], tier: WasmTier) -> usize {
    results.iter().filter(|result| result.tier >= tier).count()
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

fn is_expected_failure(path: &Path, expected_failures: &[&str]) -> bool {
    expected_failures
        .iter()
        .any(|expected| path.ends_with(expected))
}

fn expected_runtime_failure<'a>(path: &Path, expected_failures: &'a [(&str, &str)]) -> Option<&'a str> {
    expected_failures
        .iter()
        .find_map(|(expected_path, expected_message)| path.ends_with(expected_path).then_some(*expected_message))
}

fn format_result_paths(results: &[&E2eResult]) -> String {
    results
        .iter()
        .map(|result| result.path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalize_newline(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end_matches('\n').to_owned()
}

fn format_diagnostics(result: &crate::CompileResult) -> String {
    if result.diagnostics.is_empty() {
        "no diagnostics".to_owned()
    } else {
        format_diagnostic_messages(&result.diagnostics)
    }
}

fn format_diagnostic_messages(diagnostics: &[crate::Diagnostic]) -> String {
    if diagnostics.is_empty() {
        "no diagnostics".to_owned()
    } else {
        diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
