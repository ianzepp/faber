use super::common::{collect_exempla_files, format_diagnostic_messages, make_temp_root};
use super::wasm_expectations::WASM_EXPECTED_TIER_FLOORS;
use super::wasm_host::{
    probe_wat_instantiation, probe_wat_instantiation_with_stub_host, WasmInstantiationBucket,
};
use crate::codegen::Target;
use crate::driver::Session;
use crate::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum WasmTier {
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
    stubless_bucket: Option<WasmInstantiationBucket>,
    stub_bucket: Option<WasmInstantiationBucket>,
}

#[derive(Debug, Clone, Copy)]
enum WasmValidator {
    WasmTools,
    Wat2Wasm,
}

#[derive(Debug, Clone, Copy)]
struct WasmToolchain {
    validator: Option<WasmValidator>,
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

fn detect_wasm_toolchain() -> WasmToolchain {
    let validator = if command_available("wasm-tools", &["--version"]) {
        Some(WasmValidator::WasmTools)
    } else if command_available("wat2wasm", &["--version"]) {
        Some(WasmValidator::Wat2Wasm)
    } else {
        None
    };

    WasmToolchain { validator }
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
            return wasm_result(file, WasmTier::SourceReadable, format!("cannot read source: {err}"), None, None);
        }
    };

    let analysis = match crate::driver::analyze_source(session, &file.display().to_string(), &source) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            return wasm_result(
                file,
                WasmTier::SourceReadable,
                format!("frontend failed: {}", format_diagnostic_messages(&diagnostics)),
                None,
                None,
            );
        }
    };

    let mir = match crate::mir::lower_analyzed_unit_with_context(&analysis) {
        Ok(mir) => mir,
        Err(errors) => {
            return wasm_result(
                file,
                WasmTier::FrontendAnalyzed,
                format!(
                    "MIR lowering failed: {}",
                    errors
                        .iter()
                        .map(|error| error.message.clone())
                        .collect::<Vec<_>>()
                        .join(" | ")
                ),
                None,
                None,
            );
        }
    };

    let wat = match crate::mir::emit_wasm_text_probe_with_context(&mir.program, &mir.validation, &analysis.interner)
    {
        Ok(wat) => wat,
        Err(error) => {
            return wasm_result(file, WasmTier::MirLowered, format!("Wasm emission failed: {error}"), None, None);
        }
    };

    let stem = file
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("exemplum");
    let wat_file = temp_root.join(format!("{idx:03}-{stem}.wat"));
    if let Err(err) = fs::write(&wat_file, &wat) {
        return wasm_result(
            file,
            WasmTier::WasmEmitted,
            format!("cannot write WAT output: {err}"),
            None,
            None,
        );
    }

    let Some(validator) = toolchain.validator else {
        return wasm_result(
            file,
            WasmTier::WasmEmitted,
            "compile validation skipped: no wasm-tools or wat2wasm on PATH".to_owned(),
            None,
            None,
        );
    };

    let wasm_file = temp_root.join(format!("{idx:03}-{stem}.wasm"));
    if let Err(reason) = validate_wat(validator, &wat_file, &wasm_file) {
        return wasm_result(file, WasmTier::WasmEmitted, reason, None, None);
    }

    let stubless_probe = probe_wat_instantiation(&wat);
    let stub_probe = probe_wat_instantiation_with_stub_host(&wat);
    let (tier, reason) = match stub_probe.bucket {
        WasmInstantiationBucket::InstantiateValid => (
            WasmTier::InstantiateValid,
            format!("{}; run skipped: no Wasm entrypoint policy yet", stub_probe.reason),
        ),
        WasmInstantiationBucket::MissingImport => (
            WasmTier::CompileValid,
            format!(
                "stub host blocked ({}): {}; stubless ({})",
                stub_probe.bucket, stub_probe.reason, stubless_probe.bucket
            ),
        ),
        WasmInstantiationBucket::InstantiationTrap => (
            WasmTier::CompileValid,
            format!(
                "stub host failed ({}): {}; stubless ({})",
                stub_probe.bucket, stub_probe.reason, stubless_probe.bucket
            ),
        ),
        WasmInstantiationBucket::NoRuntime => (
            WasmTier::CompileValid,
            format!(
                "stub host skipped ({}): {}; stubless ({})",
                stub_probe.bucket, stub_probe.reason, stubless_probe.bucket
            ),
        ),
    };

    wasm_result(
        file,
        tier,
        reason,
        Some(stubless_probe.bucket),
        Some(stub_probe.bucket),
    )
}

fn wasm_result(
    file: &Path,
    tier: WasmTier,
    reason: String,
    stubless_bucket: Option<WasmInstantiationBucket>,
    stub_bucket: Option<WasmInstantiationBucket>,
) -> WasmE2eResult {
    WasmE2eResult {
        path: file.to_path_buf(),
        tier,
        reason,
        stubless_bucket,
        stub_bucket,
    }
}

fn validate_wat(validator: WasmValidator, wat_file: &Path, wasm_file: &Path) -> Result<(), String> {
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
        Ok(())
    } else {
        Err(format!(
            "Wasm validation failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
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
    eprintln!("  instantiator/runtime: wasmtime dev-dependency (in-process linker probe)");
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

    let compile_valid = results
        .iter()
        .filter(|result| result.tier >= WasmTier::CompileValid)
        .collect::<Vec<_>>();
    eprintln!("Wasm instantiation buckets (compile-valid subset, stubless linker):");
    eprintln!(
        "  missing-import: {}",
        count_stubless_bucket(&compile_valid, WasmInstantiationBucket::MissingImport)
    );
    eprintln!(
        "  instantiation-trap: {}",
        count_stubless_bucket(&compile_valid, WasmInstantiationBucket::InstantiationTrap)
    );
    eprintln!(
        "  instantiate-valid: {}",
        count_stubless_bucket(&compile_valid, WasmInstantiationBucket::InstantiateValid)
    );
    eprintln!(
        "  no-runtime: {}",
        count_stubless_bucket(&compile_valid, WasmInstantiationBucket::NoRuntime)
    );
    eprintln!("Wasm instantiation buckets (compile-valid subset, stub host):");
    eprintln!(
        "  instantiate-valid: {}",
        count_stub_bucket(&compile_valid, WasmInstantiationBucket::InstantiateValid)
    );
    eprintln!(
        "  missing-import: {}",
        count_stub_bucket(&compile_valid, WasmInstantiationBucket::MissingImport)
    );
    eprintln!(
        "  instantiation-trap: {}",
        count_stub_bucket(&compile_valid, WasmInstantiationBucket::InstantiationTrap)
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

fn count_stubless_bucket(results: &[&WasmE2eResult], bucket: WasmInstantiationBucket) -> usize {
    results
        .iter()
        .filter(|result| result.stubless_bucket == Some(bucket))
        .count()
}

fn count_stub_bucket(results: &[&WasmE2eResult], bucket: WasmInstantiationBucket) -> usize {
    results
        .iter()
        .filter(|result| result.stub_bucket == Some(bucket))
        .count()
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