use super::common::{collect_exempla_files, format_diagnostic_messages, make_temp_root};
use crate::codegen::Target;
use crate::driver::Session;
use crate::Config;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LlvmTier {
    SourceReadable,
    FrontendAnalyzed,
    MirLowered,
    LlvmEmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LlvmEmissionBucket {
    FrontendFailed,
    MirLoweringFailed,
    Emitted,
    Unsupported,
    EmissionFailed,
    OutputWriteFailed,
}

const EXPECTED_FRONTEND_ANALYZED_FLOOR: usize = 101;
const EXPECTED_MIR_LOWERED_FLOOR: usize = 73;
const EXPECTED_LLVM_EMITTED_FLOOR: usize = 0;
const EXPECTED_UNSUPPORTED_DIAGNOSTIC_FLOOR: usize = 73;

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
    let mut results = Vec::with_capacity(exempla.len());

    for (idx, file) in exempla.iter().enumerate() {
        results.push(classify_llvm_exemplum(&session, file, idx, &temp_root));
    }

    print_llvm_e2e_report(&results);
    assert_llvm_expected_floors(&results);
}

fn classify_llvm_exemplum(session: &Session, file: &Path, idx: usize, temp_root: &Path) -> LlvmE2eResult {
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

    llvm_result(
        file,
        LlvmTier::LlvmEmitted,
        LlvmEmissionBucket::Emitted,
        format!("LLVM text emitted to {}", llvm_file.display()),
    )
}

fn llvm_result(file: &Path, tier: LlvmTier, bucket: LlvmEmissionBucket, reason: String) -> LlvmE2eResult {
    LlvmE2eResult { path: file.to_path_buf(), tier, bucket, reason }
}

fn print_llvm_e2e_report(results: &[LlvmE2eResult]) {
    let total = results.len();
    eprintln!("LLVM e2e toolchain:");
    eprintln!("  verifier: unavailable");
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
    let unsupported = count_emission_bucket(results, LlvmEmissionBucket::Unsupported);

    let regressions = [
        ("frontend analyzed", frontend, EXPECTED_FRONTEND_ANALYZED_FLOOR),
        ("MIR lowered", mir, EXPECTED_MIR_LOWERED_FLOOR),
        ("LLVM emitted", llvm, EXPECTED_LLVM_EMITTED_FLOOR),
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
