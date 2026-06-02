use super::common::{collect_exempla_files, format_diagnostics};
use super::types::E2eResult;
use crate::codegen::Target;
use crate::{Compiler, Config, Output};
use std::fs;
use std::path::Path;

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
