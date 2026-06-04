use super::common::{collect_exempla_files, command_available, format_diagnostic_messages, make_temp_root};
use crate::codegen::Target;
use crate::driver::Session;
use crate::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LlvmTier {
    SourceReadable,
    FrontendAnalyzed,
    MirLowered,
    LlvmEmitted,
    LlvmVerifierValid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlvmEmissionBucket {
    FrontendFailed,
    MirLoweringFailed,
    Emitted,
    Unsupported,
    EmissionFailed,
    OutputWriteFailed,
    VerifierValid,
    VerifierFailed,
}

#[derive(Debug, Clone, Copy)]
enum LlvmVerifier {
    LlvmAs,
    Opt,
}

#[derive(Debug, Clone)]
struct LlvmToolchain {
    verifier: Option<LlvmVerifier>,
    verifier_version: Option<String>,
}

const EXPECTED_FRONTEND_ANALYZED_FLOOR: usize = 102;
const EXPECTED_MIR_LOWERED_FLOOR: usize = 74;
const EXPECTED_LLVM_EMITTED_FLOOR: usize = 35;
const EXPECTED_LLVM_VERIFIER_VALID_FLOOR: usize = 0;
const EXPECTED_UNSUPPORTED_DIAGNOSTIC_FLOOR: usize = 39;

#[derive(Debug)]
struct LlvmE2eResult {
    path: PathBuf,
    tier: LlvmTier,
    bucket: LlvmEmissionBucket,
    reason: String,
}

#[test]
#[ignore = "slow baseline harness; run explicitly with cargo test exempla_llvm_e2e -- --ignored --nocapture"]
fn exempla_llvm_e2e() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let mut exempla = collect_exempla_files(&exempla_dir);
    exempla.sort();
    assert!(!exempla.is_empty(), "LLVM e2e harness found no exempla files");

    let session = Session::new(Config::default().with_target(Target::LlvmText));
    let temp_root = make_temp_root();
    let toolchain = detect_llvm_toolchain();
    let mut results = Vec::with_capacity(exempla.len());

    for (idx, file) in exempla.iter().enumerate() {
        results.push(classify_llvm_exemplum(&session, file, idx, &temp_root, &toolchain));
    }

    print_llvm_e2e_report(&results, &toolchain);
    assert_llvm_expected_floors(&results);
}

fn detect_llvm_toolchain() -> LlvmToolchain {
    let verifier = if command_available("llvm-as", &["--version"]) {
        Some(LlvmVerifier::LlvmAs)
    } else if command_available("opt", &["--version"]) {
        Some(LlvmVerifier::Opt)
    } else {
        None
    };
    let verifier_version = verifier.map(llvm_verifier_version);

    LlvmToolchain { verifier, verifier_version }
}

fn llvm_verifier_version(verifier: LlvmVerifier) -> String {
    let output = match verifier {
        LlvmVerifier::LlvmAs => Command::new("llvm-as").arg("--version").output(),
        LlvmVerifier::Opt => Command::new("opt").arg("--version").output(),
    };
    let Ok(output) = output else {
        return "version unavailable".to_owned();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .unwrap_or("version unavailable")
        .to_owned()
}

fn classify_llvm_exemplum(
    session: &Session,
    file: &Path,
    idx: usize,
    temp_root: &Path,
    toolchain: &LlvmToolchain,
) -> LlvmE2eResult {
    let source = match fs::read_to_string(file) {
        Ok(source) => source,
        Err(err) => {
            return llvm_result(
                file,
                LlvmTier::SourceReadable,
                LlvmEmissionBucket::OutputWriteFailed,
                format!("cannot read source: {err}"),
            );
        }
    };

    let analysis = match crate::driver::analyze_source(session, &file.display().to_string(), &source) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            return llvm_result(
                file,
                LlvmTier::SourceReadable,
                LlvmEmissionBucket::FrontendFailed,
                format!("frontend failed: {}", format_diagnostic_messages(&diagnostics)),
            );
        }
    };

    let mir = match crate::mir::lower_analyzed_unit(&analysis) {
        Ok(mir) => mir,
        Err(errors) => {
            return llvm_result(
                file,
                LlvmTier::FrontendAnalyzed,
                LlvmEmissionBucket::MirLoweringFailed,
                format!(
                    "MIR lowering failed: {}",
                    errors
                        .iter()
                        .map(|error| error.message.clone())
                        .collect::<Vec<_>>()
                        .join(" | ")
                ),
            );
        }
    };

    let llvm = match crate::mir::emit_llvm_text_probe(&mir, &analysis.types, &analysis.interner) {
        Ok(llvm) => llvm,
        Err(error) if error.message.starts_with("MIR-to-LLVM unsupported:") => {
            return llvm_result(
                file,
                LlvmTier::MirLowered,
                LlvmEmissionBucket::Unsupported,
                format!("LLVM emission unsupported: {error}"),
            );
        }
        Err(error) => {
            return llvm_result(
                file,
                LlvmTier::MirLowered,
                LlvmEmissionBucket::EmissionFailed,
                format!("LLVM emission failed: {error}"),
            );
        }
    };

    let stem = file
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("exemplum");
    let llvm_file = temp_root.join(format!("{idx:03}-{stem}.ll"));
    if let Err(err) = fs::write(&llvm_file, &llvm) {
        return llvm_result(
            file,
            LlvmTier::LlvmEmitted,
            LlvmEmissionBucket::OutputWriteFailed,
            format!("cannot write LLVM output: {err}"),
        );
    }

    if let Some(verifier) = toolchain.verifier {
        return match verify_llvm(verifier, &llvm_file) {
            Ok(()) => llvm_result(
                file,
                LlvmTier::LlvmVerifierValid,
                LlvmEmissionBucket::VerifierValid,
                format!(
                    "LLVM text emitted and verified with {} at {}",
                    verifier.command(),
                    llvm_file.display()
                ),
            ),
            Err(reason) => llvm_result(
                file,
                LlvmTier::LlvmEmitted,
                LlvmEmissionBucket::VerifierFailed,
                format!("LLVM text emitted to {}; verifier failed: {reason}", llvm_file.display()),
            ),
        };
    }

    llvm_result(
        file,
        LlvmTier::LlvmEmitted,
        LlvmEmissionBucket::Emitted,
        format!("LLVM text emitted to {}; verifier unavailable", llvm_file.display()),
    )
}

fn llvm_result(file: &Path, tier: LlvmTier, bucket: LlvmEmissionBucket, reason: String) -> LlvmE2eResult {
    LlvmE2eResult { path: file.to_path_buf(), tier, bucket, reason }
}

fn verify_llvm(verifier: LlvmVerifier, llvm_file: &Path) -> Result<(), String> {
    let output = match verifier {
        LlvmVerifier::LlvmAs => Command::new("llvm-as")
            .arg("-o")
            .arg(if cfg!(windows) { "NUL" } else { "/dev/null" })
            .arg(llvm_file)
            .output(),
        LlvmVerifier::Opt => Command::new("opt")
            .arg("-disable-output")
            .arg(llvm_file)
            .output(),
    };

    let output = output.map_err(|err| format!("cannot execute LLVM verifier: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn print_llvm_e2e_report(results: &[LlvmE2eResult], toolchain: &LlvmToolchain) {
    let total = results.len();
    eprintln!("LLVM e2e toolchain:");
    eprintln!(
        "  verifier: {}",
        match toolchain.verifier {
            Some(verifier) => match &toolchain.verifier_version {
                Some(version) => format!("{} ({version})", verifier.command()),
                None => verifier.command().to_owned(),
            },
            None => "unavailable (llvm-as/opt not found)".to_owned(),
        }
    );
    eprintln!("  execution/runtime: unavailable");
    eprintln!("LLVM e2e exempla:");
    eprintln!(
        "  frontend analyzed: {}/{}",
        count_llvm_tier(results, LlvmTier::FrontendAnalyzed),
        total
    );
    eprintln!("  MIR lowered: {}/{}", count_llvm_tier(results, LlvmTier::MirLowered), total);
    eprintln!("  LLVM emitted: {}/{}", count_llvm_tier(results, LlvmTier::LlvmEmitted), total);
    eprintln!(
        "  verifier-valid: {}/{}",
        count_llvm_tier(results, LlvmTier::LlvmVerifierValid),
        total
    );
    eprintln!(
        "  frontend failed: {}",
        count_emission_bucket(results, LlvmEmissionBucket::FrontendFailed)
    );
    eprintln!(
        "  MIR lowering failed: {}",
        count_emission_bucket(results, LlvmEmissionBucket::MirLoweringFailed)
    );
    eprintln!(
        "  unsupported diagnostic: {}",
        count_emission_bucket(results, LlvmEmissionBucket::Unsupported)
    );
    eprintln!(
        "  emission failed: {}",
        count_emission_bucket(results, LlvmEmissionBucket::EmissionFailed)
    );
    eprintln!(
        "  output write failed: {}",
        count_emission_bucket(results, LlvmEmissionBucket::OutputWriteFailed)
    );
    eprintln!(
        "  verifier failed: {}",
        count_emission_bucket(results, LlvmEmissionBucket::VerifierFailed)
    );

    for result in results
        .iter()
        .filter(|result| result.tier < LlvmTier::LlvmEmitted)
    {
        eprintln!("[llvm:{:?}] {} :: {}", result.tier, result.path.display(), result.reason);
    }
}

fn count_llvm_tier(results: &[LlvmE2eResult], tier: LlvmTier) -> usize {
    results.iter().filter(|result| result.tier >= tier).count()
}

fn count_emission_bucket(results: &[LlvmE2eResult], bucket: LlvmEmissionBucket) -> usize {
    results
        .iter()
        .filter(|result| result.bucket == bucket)
        .count()
}

fn assert_llvm_expected_floors(results: &[LlvmE2eResult]) {
    let frontend = count_llvm_tier(results, LlvmTier::FrontendAnalyzed);
    let mir = count_llvm_tier(results, LlvmTier::MirLowered);
    let llvm = count_llvm_tier(results, LlvmTier::LlvmEmitted);
    let verifier = count_llvm_tier(results, LlvmTier::LlvmVerifierValid);
    let unsupported = count_emission_bucket(results, LlvmEmissionBucket::Unsupported);

    let regressions = [
        ("frontend analyzed", frontend, EXPECTED_FRONTEND_ANALYZED_FLOOR),
        ("MIR lowered", mir, EXPECTED_MIR_LOWERED_FLOOR),
        ("LLVM emitted", llvm, EXPECTED_LLVM_EMITTED_FLOOR),
        ("LLVM verifier-valid", verifier, EXPECTED_LLVM_VERIFIER_VALID_FLOOR),
        ("unsupported diagnostic", unsupported, EXPECTED_UNSUPPORTED_DIAGNOSTIC_FLOOR),
    ]
    .into_iter()
    .filter_map(|(label, actual, expected)| {
        (actual < expected).then_some(format!("{label} expected at least {expected}, got {actual}"))
    })
    .collect::<Vec<_>>();

    assert!(
        regressions.is_empty(),
        "unexpected LLVM e2e tier regressions:\n{}",
        regressions.join("\n")
    );
}

impl LlvmVerifier {
    fn command(self) -> &'static str {
        match self {
            LlvmVerifier::LlvmAs => "llvm-as",
            LlvmVerifier::Opt => "opt -disable-output",
        }
    }
}
