use crate::codegen::{self, Target};
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

#[derive(Debug)]
struct E2eFinding {
    path: PathBuf,
    reason: String,
}

const RUST_EXPECTED_FAILURES: &[&str] = &[];
const RUST_EXPECTED_RUNTIME_FAILURES: &[(&str, &str)] = &[("ad/ad.fab", "E_NO_ROUTE: unresolved capability")];
const GO_EXPECTED_FAILURES: &[&str] = &[
    "ad/ad.fab",
    "inter/inter.fab",
    "itera/cursor-iteratio.fab",
    "itera/nidificatus.fab",
    "syntaxis/arena-mixta.fab",
    "syntaxis/destructura-sparsa.fab",
    "syntaxis/fluxus-cede.fab",
];

#[derive(Debug)]
struct TsE2eResult {
    path: PathBuf,
    frontend_analyzed: bool,
    typescript_emitted: bool,
    formatted: TierState,
    linted: TierState,
    typecheck_valid: TierState,
    runnable: TierState,
    behavior_checked: TierState,
    reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TierState {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug)]
struct TsToolchain {
    formatter: TsFormatter,
    linter: TsLinter,
    typechecker: TsTypechecker,
    runtime: TsRuntime,
}

#[derive(Debug, Clone, Copy)]
enum TsFormatter {
    Prettier,
    Deno,
    Missing,
}

#[derive(Debug, Clone, Copy)]
enum TsLinter {
    Biome,
    Eslint,
    Missing,
}

#[derive(Debug, Clone, Copy)]
enum TsTypechecker {
    Tsc,
    Deno,
    Missing,
}

#[derive(Debug, Clone, Copy)]
enum TsRuntime {
    NodeViaTsc,
    Deno,
    Missing,
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

const WASM_EXPECTED_TIER_FLOORS: &[(&str, WasmTier)] = &[
    ("abstractus/abstractus.fab", WasmTier::CompileValid),
    ("adfirma/adfirma.fab", WasmTier::CompileValid),
    ("adfirma/in-functione.fab", WasmTier::CompileValid),
    ("ante/ante.fab", WasmTier::CompileValid),
    ("assignatio/assignatio.fab", WasmTier::CompileValid),
    ("aut/aut.fab", WasmTier::CompileValid),
    ("binarius/binarius.fab", WasmTier::CompileValid),
    ("ceteri/ceteri.fab", WasmTier::CompileValid),
    ("conversio/conversio.fab", WasmTier::CompileValid),
    ("cura/cura.fab", WasmTier::CompileValid),
    ("cura/nidificatus.fab", WasmTier::CompileValid),
    ("custodi/custodi.fab", WasmTier::CompileValid),
    ("destructura/destructura.fab", WasmTier::CompileValid),
    ("discretio/discretio.fab", WasmTier::CompileValid),
    ("dum/conditio-complexa.fab", WasmTier::CompileValid),
    ("dum/dum.fab", WasmTier::CompileValid),
    ("dum/in-functione.fab", WasmTier::CompileValid),
    ("ego/ego.fab", WasmTier::CompileValid),
    ("elige/ceterum.fab", WasmTier::CompileValid),
    ("elige/elige.fab", WasmTier::CompileValid),
    ("elige/ergo-redde.fab", WasmTier::CompileValid),
    ("elige/in-functione.fab", WasmTier::CompileValid),
    ("est/est.fab", WasmTier::CompileValid),
    ("et/et.fab", WasmTier::CompileValid),
    ("finge/finge.fab", WasmTier::CompileValid),
    ("fixum/fixum.fab", WasmTier::CompileValid),
    ("functio/functio.fab", WasmTier::CompileValid),
    ("functio/recursio.fab", WasmTier::CompileValid),
    ("functio/typicus.fab", WasmTier::CompileValid),
    ("genus/creo.fab", WasmTier::CompileValid),
    ("genus/genus.fab", WasmTier::CompileValid),
    ("genus/methodi.fab", WasmTier::CompileValid),
    ("implet/implet.fab", WasmTier::CompileValid),
    ("incipit/functionibus.fab", WasmTier::CompileValid),
    ("incipit/incipit.fab", WasmTier::CompileValid),
    ("itera/ex.fab", WasmTier::CompileValid),
    ("itera/in-functione.fab", WasmTier::CompileValid),
    ("itera/intervallum-gradus.fab", WasmTier::CompileValid),
    ("itera/intervallum.fab", WasmTier::CompileValid),
    ("itera/nidificatus.fab", WasmTier::CompileValid),
    ("lista/lista.fab", WasmTier::CompileValid),
    ("mone/mone.fab", WasmTier::CompileValid),
    ("mori/mori.fab", WasmTier::CompileValid),
    ("nexum/nexum.fab", WasmTier::CompileValid),
    ("nota/gradus.fab", WasmTier::CompileValid),
    ("nota/nota.fab", WasmTier::CompileValid),
    ("novum/novum.fab", WasmTier::CompileValid),
    ("pactum/pactum.fab", WasmTier::CompileValid),
    ("per/per.fab", WasmTier::CompileValid),
    ("perge/perge.fab", WasmTier::CompileValid),
    ("privata/privata.fab", WasmTier::CompileValid),
    ("protecta/protecta.fab", WasmTier::CompileValid),
    ("publica/publica.fab", WasmTier::CompileValid),
    ("redde/redde.fab", WasmTier::CompileValid),
    ("rumpe/rumpe.fab", WasmTier::CompileValid),
    ("salve-munde.fab", WasmTier::CompileValid),
    ("scriptum/scriptum.fab", WasmTier::CompileValid),
    ("si/ergo.fab", WasmTier::CompileValid),
    ("si/est.fab", WasmTier::MirLowered),
    ("si/nidificatus.fab", WasmTier::CompileValid),
    ("si/secus.fab", WasmTier::CompileValid),
    ("si/si.fab", WasmTier::CompileValid),
    ("si/sin.fab", WasmTier::CompileValid),
    ("sparge/sparge.fab", WasmTier::CompileValid),
    ("sub/sub.fab", WasmTier::CompileValid),
    ("typus/typus.fab", WasmTier::CompileValid),
    ("unarius/unarius.fab", WasmTier::CompileValid),
    ("usque/usque.fab", WasmTier::CompileValid),
    ("varia/destructura.fab", WasmTier::CompileValid),
    ("varia/typicus.fab", WasmTier::CompileValid),
    ("varia/varia.fab", WasmTier::CompileValid),
    ("vide/vide.fab", WasmTier::CompileValid),
];

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
#[ignore = "slow TypeScript end-to-end baseline; run explicitly with cargo test exempla_ts_e2e -- --ignored --nocapture"]
fn exempla_ts_e2e() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();

    let toolchain = detect_ts_toolchain();
    let session = crate::driver::Session::new(Config::default().with_target(Target::TypeScript));
    let temp_root = make_temp_root();
    let mut results = Vec::with_capacity(exempla.len());
    let mut expected_count = 0usize;

    for (idx, file) in exempla.iter().enumerate() {
        if read_expected_stdout(file).is_some() {
            expected_count += 1;
        }
        let source = match fs::read_to_string(file) {
            Ok(source) => source,
            Err(err) => {
                results.push(TsE2eResult::failed_frontend(file.clone(), format!("cannot read source: {err}")));
                continue;
            }
        };

        let analysis = match crate::driver::analyze_source(&session, &file.display().to_string(), &source) {
            Ok(analysis) => analysis,
            Err(diagnostics) => {
                results.push(TsE2eResult::failed_frontend(
                    file.clone(),
                    format!("frontend failed: {}", format_diagnostic_messages(&diagnostics)),
                ));
                continue;
            }
        };

        let ts = match codegen::generate(Target::TypeScript, &analysis.hir, &analysis.types, &analysis.interner) {
            Ok(Output::TypeScript(output)) => output.code,
            Ok(_) => {
                results.push(TsE2eResult::failed_codegen(
                    file.clone(),
                    "compiler did not produce TypeScript output".to_owned(),
                ));
                continue;
            }
            Err(err) => {
                results.push(TsE2eResult::failed_codegen(
                    file.clone(),
                    format!("TypeScript codegen failed: {err}"),
                ));
                continue;
            }
        };

        let (formatted, code, format_reason) = run_ts_format_tier(&toolchain, &ts);
        let (linted, code, lint_reason) = run_ts_lint_tier(&toolchain, &code);

        let stem = file
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("exemplum");
        let case_dir = temp_root.join(format!("{idx:03}-{stem}"));
        if let Err(err) = fs::create_dir_all(&case_dir) {
            results.push(TsE2eResult::after_codegen(
                file.clone(),
                formatted,
                linted,
                TierState::Failed,
                TierState::Skipped,
                TierState::Skipped,
                format!("cannot create TypeScript temp dir: {err}"),
            ));
            continue;
        }

        let ts_file = case_dir.join("main.ts");
        if let Err(err) = fs::write(&ts_file, &code) {
            results.push(TsE2eResult::after_codegen(
                file.clone(),
                formatted,
                linted,
                TierState::Failed,
                TierState::Skipped,
                TierState::Skipped,
                format!("cannot write TypeScript output: {err}"),
            ));
            continue;
        }

        let (typecheck_valid, typecheck_reason) = run_ts_typecheck_tier(&toolchain, &case_dir);
        if typecheck_valid != TierState::Passed {
            results.push(TsE2eResult::after_codegen(
                file.clone(),
                formatted,
                linted,
                typecheck_valid,
                TierState::Skipped,
                TierState::Skipped,
                join_reasons([format_reason, lint_reason, typecheck_reason]),
            ));
            continue;
        }

        let (runnable, stdout, run_reason) = run_ts_runtime_tier(&toolchain, &case_dir);
        if runnable != TierState::Passed {
            results.push(TsE2eResult::after_codegen(
                file.clone(),
                formatted,
                linted,
                typecheck_valid,
                runnable,
                TierState::Skipped,
                join_reasons([format_reason, lint_reason, run_reason]),
            ));
            continue;
        }

        let (behavior_checked, behavior_reason) = match read_expected_stdout(file) {
            Some(expected) if normalize_newline(&stdout) == expected => (TierState::Passed, String::new()),
            Some(expected) => (
                TierState::Failed,
                format!("stdout mismatch: expected `{expected}`, got `{}`", normalize_newline(&stdout)),
            ),
            None => (TierState::Skipped, "no sibling .expected file".to_owned()),
        };

        results.push(TsE2eResult::after_codegen(
            file.clone(),
            formatted,
            linted,
            typecheck_valid,
            runnable,
            behavior_checked,
            join_reasons([format_reason, lint_reason, behavior_reason]),
        ));
    }

    print_ts_e2e_report(&results, &toolchain, expected_count);
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
    assert_wasm_expected_tiers(&results);
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

    let mir = match crate::mir::lower_analyzed_unit_with_context(&analysis) {
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

    let wat = match crate::mir::emit_wasm_text_probe_with_context(&mir.program, &mir.validation, &analysis.interner) {
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

fn assert_wasm_expected_tiers(results: &[WasmE2eResult]) {
    let regressions = results
        .iter()
        .filter_map(|result| {
            let expected = expected_wasm_tier(&result.path);
            (result.tier < expected).then_some(format!(
                "{} expected at least {:?}, reached {:?}: {}",
                wasm_exemplum_key(&result.path),
                expected,
                result.tier,
                result.reason
            ))
        })
        .collect::<Vec<_>>();

    assert!(
        regressions.is_empty(),
        "unexpected Wasm e2e tier regressions:\n{}",
        regressions.join("\n")
    );
}

fn expected_wasm_tier(path: &Path) -> WasmTier {
    let key = wasm_exemplum_key(path);
    WASM_EXPECTED_TIER_FLOORS
        .iter()
        .find_map(|(expected, tier)| (*expected == key).then_some(*tier))
        .unwrap_or(WasmTier::FrontendAnalyzed)
}

fn wasm_exemplum_key(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized
        .split("/examples/exempla/")
        .nth(1)
        .unwrap_or(normalized.as_str())
        .to_owned()
}

fn detect_ts_toolchain() -> TsToolchain {
    let formatter = if command_available("prettier", &["--version"]) {
        TsFormatter::Prettier
    } else if command_available("deno", &["--version"]) {
        TsFormatter::Deno
    } else {
        TsFormatter::Missing
    };
    let linter = if command_available("biome", &["--version"]) {
        TsLinter::Biome
    } else if command_available("eslint", &["--version"]) {
        TsLinter::Eslint
    } else {
        TsLinter::Missing
    };
    let typechecker = if command_available("tsc", &["--version"]) {
        TsTypechecker::Tsc
    } else if command_available("deno", &["--version"]) {
        TsTypechecker::Deno
    } else {
        TsTypechecker::Missing
    };
    let runtime = if matches!(typechecker, TsTypechecker::Tsc) && command_available("node", &["--version"]) {
        TsRuntime::NodeViaTsc
    } else if command_available("deno", &["--version"]) {
        TsRuntime::Deno
    } else {
        TsRuntime::Missing
    };
    TsToolchain { formatter, linter, typechecker, runtime }
}

fn run_ts_format_tier(toolchain: &TsToolchain, code: &str) -> (TierState, String, String) {
    match toolchain.formatter {
        TsFormatter::Missing => (
            TierState::Skipped,
            code.to_owned(),
            "formatted skipped: no prettier or deno".to_owned(),
        ),
        TsFormatter::Prettier | TsFormatter::Deno => match crate::tool::format_generated_code(Target::TypeScript, code)
        {
            Ok(formatted) => (TierState::Passed, formatted, String::new()),
            Err(err) => (TierState::Failed, code.to_owned(), format!("format failed: {err}")),
        },
    }
}

fn run_ts_lint_tier(toolchain: &TsToolchain, code: &str) -> (TierState, String, String) {
    match toolchain.linter {
        TsLinter::Missing => (
            TierState::Skipped,
            code.to_owned(),
            "linted skipped: no biome or eslint".to_owned(),
        ),
        TsLinter::Biome | TsLinter::Eslint => match crate::tool::lint_generated_code(Target::TypeScript, code) {
            Ok(fixed) => (TierState::Passed, fixed, String::new()),
            Err(err) => (TierState::Failed, code.to_owned(), format!("lint failed: {err}")),
        },
    }
}

fn run_ts_typecheck_tier(toolchain: &TsToolchain, case_dir: &Path) -> (TierState, String) {
    match toolchain.typechecker {
        TsTypechecker::Missing => (TierState::Skipped, "typecheck skipped: no tsc or deno".to_owned()),
        TsTypechecker::Tsc => {
            let output = Command::new("tsc")
                .args([
                    "--strict", "--noEmit", "--target", "ES2022", "--module", "commonjs", "main.ts",
                ])
                .current_dir(case_dir)
                .output();
            command_tier(output, "tsc typecheck failed")
        }
        TsTypechecker::Deno => {
            let output = Command::new("deno")
                .args(["check", "main.ts"])
                .current_dir(case_dir)
                .output();
            command_tier(output, "deno check failed")
        }
    }
}

fn run_ts_runtime_tier(toolchain: &TsToolchain, case_dir: &Path) -> (TierState, String, String) {
    match toolchain.runtime {
        TsRuntime::Missing => (
            TierState::Skipped,
            String::new(),
            "runtime skipped: no node+tsc or deno".to_owned(),
        ),
        TsRuntime::NodeViaTsc => {
            let transpile = Command::new("tsc")
                .args(["--target", "ES2022", "--module", "commonjs", "main.ts"])
                .current_dir(case_dir)
                .output();
            let (state, reason) = command_tier(transpile, "tsc transpile failed");
            if state != TierState::Passed {
                return (state, String::new(), reason);
            }
            let run = Command::new("node")
                .arg("main.js")
                .current_dir(case_dir)
                .output();
            match run {
                Ok(run) if run.status.success() => (
                    TierState::Passed,
                    String::from_utf8_lossy(&run.stdout).to_string(),
                    String::new(),
                ),
                Ok(run) => (
                    TierState::Failed,
                    String::from_utf8_lossy(&run.stdout).to_string(),
                    format!("node run failed: {}", command_stderr(&run)),
                ),
                Err(err) => (TierState::Failed, String::new(), format!("cannot execute node: {err}")),
            }
        }
        TsRuntime::Deno => {
            let run = Command::new("deno")
                .args(["run", "main.ts"])
                .current_dir(case_dir)
                .output();
            match run {
                Ok(run) if run.status.success() => (
                    TierState::Passed,
                    String::from_utf8_lossy(&run.stdout).to_string(),
                    String::new(),
                ),
                Ok(run) => (
                    TierState::Failed,
                    String::from_utf8_lossy(&run.stdout).to_string(),
                    format!("deno run failed: {}", command_stderr(&run)),
                ),
                Err(err) => (TierState::Failed, String::new(), format!("cannot execute deno: {err}")),
            }
        }
    }
}

fn command_tier(output: Result<std::process::Output, std::io::Error>, failure_prefix: &str) -> (TierState, String) {
    match output {
        Ok(output) if output.status.success() => (TierState::Passed, String::new()),
        Ok(output) => (TierState::Failed, format!("{failure_prefix}: {}", command_stderr(&output))),
        Err(err) => (TierState::Failed, format!("cannot execute command: {err}")),
    }
}

fn command_stderr(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr
    }
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

impl TsE2eResult {
    fn failed_frontend(path: PathBuf, reason: String) -> Self {
        Self {
            path,
            frontend_analyzed: false,
            typescript_emitted: false,
            formatted: TierState::Skipped,
            linted: TierState::Skipped,
            typecheck_valid: TierState::Skipped,
            runnable: TierState::Skipped,
            behavior_checked: TierState::Skipped,
            reason,
        }
    }

    fn failed_codegen(path: PathBuf, reason: String) -> Self {
        Self {
            path,
            frontend_analyzed: true,
            typescript_emitted: false,
            formatted: TierState::Skipped,
            linted: TierState::Skipped,
            typecheck_valid: TierState::Skipped,
            runnable: TierState::Skipped,
            behavior_checked: TierState::Skipped,
            reason,
        }
    }

    fn after_codegen(
        path: PathBuf,
        formatted: TierState,
        linted: TierState,
        typecheck_valid: TierState,
        runnable: TierState,
        behavior_checked: TierState,
        reason: String,
    ) -> Self {
        Self {
            path,
            frontend_analyzed: true,
            typescript_emitted: true,
            formatted,
            linted,
            typecheck_valid,
            runnable,
            behavior_checked,
            reason,
        }
    }
}

fn print_ts_e2e_report(results: &[TsE2eResult], toolchain: &TsToolchain, expected_count: usize) {
    let total = results.len();
    let frontend = results
        .iter()
        .filter(|result| result.frontend_analyzed)
        .count();
    let emitted = results
        .iter()
        .filter(|result| result.typescript_emitted)
        .count();
    let formatted = results
        .iter()
        .filter(|result| result.formatted == TierState::Passed)
        .count();
    let linted = results
        .iter()
        .filter(|result| result.linted == TierState::Passed)
        .count();
    let typecheck_valid = results
        .iter()
        .filter(|result| result.typecheck_valid == TierState::Passed)
        .count();
    let runnable = results
        .iter()
        .filter(|result| result.runnable == TierState::Passed)
        .count();
    let behavior_checked = results
        .iter()
        .filter(|result| result.behavior_checked == TierState::Passed)
        .count();

    eprintln!("TypeScript toolchain:");
    eprintln!("  formatter: {}", formatter_label(toolchain.formatter));
    eprintln!("  linter: {}", linter_label(toolchain.linter));
    eprintln!("  typechecker: {}", typechecker_label(toolchain.typechecker));
    eprintln!("  runtime: {}", runtime_label(toolchain.runtime));
    eprintln!("TypeScript e2e exempla:");
    eprintln!("  frontend analyzed: {frontend}/{total}");
    eprintln!("  TypeScript emitted: {emitted}/{total}");
    eprintln!("  formatted: {formatted}/{total} ({})", formatter_label(toolchain.formatter));
    eprintln!("  linted: {linted}/{total} ({})", linter_label(toolchain.linter));
    eprintln!("  typecheck-valid: {typecheck_valid}/{total}");
    eprintln!("  runnable: {runnable}/{total}");
    eprintln!("  behavior-checked: {behavior_checked}/{total}");
    eprintln!("Expected-output checks available for {expected_count} exempla files");

    for fail in results.iter().filter(|result| !result.is_fully_clean()) {
        eprintln!("[ts] {} :: {}", fail.path.display(), fail.reason);
    }
}

impl TsE2eResult {
    fn is_fully_clean(&self) -> bool {
        self.frontend_analyzed
            && self.typescript_emitted
            && !matches!(self.formatted, TierState::Failed)
            && !matches!(self.linted, TierState::Failed)
            && matches!(self.typecheck_valid, TierState::Passed)
            && matches!(self.runnable, TierState::Passed)
            && !matches!(self.behavior_checked, TierState::Failed)
    }
}

fn formatter_label(formatter: TsFormatter) -> &'static str {
    match formatter {
        TsFormatter::Prettier => "prettier --parser typescript",
        TsFormatter::Deno => "deno fmt --ext ts -",
        TsFormatter::Missing => "skipped: no prettier or deno",
    }
}

fn linter_label(linter: TsLinter) -> &'static str {
    match linter {
        TsLinter::Biome => "biome check",
        TsLinter::Eslint => "eslint",
        TsLinter::Missing => "skipped: no biome or eslint",
    }
}

fn typechecker_label(typechecker: TsTypechecker) -> &'static str {
    match typechecker {
        TsTypechecker::Tsc => "tsc --noEmit main.ts",
        TsTypechecker::Deno => "deno check main.ts",
        TsTypechecker::Missing => "skipped: no tsc or deno",
    }
}

fn runtime_label(runtime: TsRuntime) -> &'static str {
    match runtime {
        TsRuntime::NodeViaTsc => "tsc main.ts; node main.js",
        TsRuntime::Deno => "deno run main.ts",
        TsRuntime::Missing => "skipped: no node+tsc or deno",
    }
}

fn join_reasons<const N: usize>(reasons: [String; N]) -> String {
    reasons
        .into_iter()
        .filter(|reason| !reason.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
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
        result
            .diagnostics
            .iter()
            .map(|diag| diag.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
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
