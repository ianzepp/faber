use crate::codegen::Target;
use crate::driver::Session;
use crate::exempla_e2e::wasm_behavior_fixtures::{behavior_matches, expected_wasm_behavior, WASM_BEHAVIOR_FIXTURES};
use crate::exempla_e2e::wasm_host::{run_wat_entry_with_stub_host, WasmRunBucket};
use crate::Config;
use std::fs;
use std::path::Path;

#[test]
fn wasm_behavior_fixtures_match_stub_host_diag_traces() {
    let exempla_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/exempla");
    let session = Session::new(Config::default().with_target(Target::WasmText));

    for fixture in WASM_BEHAVIOR_FIXTURES {
        let path = exempla_dir.join(fixture.exemplum);
        let source = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("cannot read {}: {err}", path.display());
        });
        let analysis = crate::driver::analyze_source(&session, &path.display().to_string(), &source).unwrap();
        let mir = crate::mir::lower_analyzed_unit_with_context(&analysis).unwrap();
        let wat =
            crate::mir::emit_wasm_text_probe_with_context(&mir.program, &mir.validation, &analysis.interner).unwrap();
        let run_probe = run_wat_entry_with_stub_host(&wat);
        assert_eq!(
            run_probe.bucket,
            WasmRunBucket::Runnable,
            "{}: {}",
            fixture.exemplum,
            run_probe.reason
        );
        assert!(
            behavior_matches(fixture.expected_diag, &run_probe.diag_events),
            "{}: expected {:?}, got {:?}",
            fixture.exemplum,
            fixture.expected_diag,
            run_probe.diag_events
        );
        assert_eq!(expected_wasm_behavior(fixture.exemplum), Some(fixture.expected_diag));
    }
}
