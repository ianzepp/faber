use faber_host_macos_arm64::syscall_import::{COMPONENT_CODE_HOST_ECHO, COMPONENT_CODE_PG_QUERY};
use faber_host_macos_arm64::wasm::WasmHost;
use faber_host_macos_arm64::Status;
use serde_json::Value;

const ROUTE_MODULE: &[u8] = include_bytes!("fixtures/core-route-proof.wat");
const ROUTE_MODULE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/core-route-proof.wat"
);

#[test]
fn core_wasm_import_routes_host_echo_through_frame_kernel() {
    let host = WasmHost::new();

    let output = host
        .call_export(ROUTE_MODULE, "route", COMPONENT_CODE_HOST_ECHO)
        .expect("wasm call should succeed");

    assert_eq!(output.module_status, 0);
    assert_eq!(output.response.status, Status::Done);
    assert_eq!(
        output.response.from.as_deref(),
        Some("faber-host-macos-arm64")
    );
    assert_eq!(
        output.response.data["echo"]["value"],
        Value::String("salve".into())
    );
}

#[test]
fn core_wasm_import_routes_unresolved_call_as_no_route_frame() {
    let host = WasmHost::new();

    let output = host
        .call_export(ROUTE_MODULE, "route", COMPONENT_CODE_PG_QUERY)
        .expect("wasm call should succeed");

    assert_eq!(output.module_status, 1);
    assert_eq!(output.response.status, Status::Error);
    assert_eq!(
        output.response.data["code"],
        Value::String("E_NO_ROUTE".into())
    );
}

#[test]
fn cli_wasm_call_loads_module_and_prints_frame_json() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_faber-host-macos-arm64"))
        .args(["wasm-call", ROUTE_MODULE_PATH, "route", "1"])
        .output()
        .expect("failed to run wasm-call command");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("response should be JSON");
    assert_eq!(json["status"], Value::String("done".into()));
    assert_eq!(json["data"]["echo"]["value"], Value::String("salve".into()));
}
