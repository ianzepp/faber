use std::process::Command;

use faber_host_macos_arm64::kernel::FrameData;
use faber_host_macos_arm64::{Frame, HostKernel, Status};
use serde_json::Value;

#[test]
fn routes_host_echo_as_done_frame() {
    let kernel = HostKernel::new();
    let mut data = FrameData::new();
    data.insert("value".into(), Value::String("salve".into()));
    let request = Frame::request_with("host:echo", data);

    let response = kernel.route(&request);

    assert_eq!(response.status, Status::Done);
    assert_eq!(response.parent_id.as_deref(), Some(request.id.as_str()));
    assert_eq!(response.call, "host:echo");
    assert_eq!(
        response.data["echo"]["value"],
        Value::String("salve".into())
    );
}

#[test]
fn reports_unresolved_call_as_no_route_error_frame() {
    let kernel = HostKernel::new();
    let request = Frame::request("pg:query");

    let response = kernel.route(&request);

    assert_eq!(response.status, Status::Error);
    assert_eq!(response.parent_id.as_deref(), Some(request.id.as_str()));
    assert_eq!(response.data["code"], Value::String("E_NO_ROUTE".into()));
    assert_eq!(response.data["retryable"], Value::Bool(false));
}

#[test]
fn manifest_lists_builtin_host_echo_and_no_default_providers() {
    let kernel = HostKernel::new();

    let manifest = kernel.manifest();

    assert_eq!(manifest.host, "macos-arm64");
    assert_eq!(manifest.manifest_version, 1);
    assert!(manifest
        .builtins
        .iter()
        .any(|item| item.name == "host:echo"));
    assert!(manifest.providers.is_empty());
}

#[test]
fn cli_manifest_prints_host_echo() {
    let output = Command::new(env!("CARGO_BIN_EXE_faber-host-macos-arm64"))
        .arg("manifest")
        .output()
        .expect("failed to run host manifest command");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("manifest should be JSON");
    assert_eq!(json["host"], Value::String("macos-arm64".into()));
    assert!(json["builtins"]
        .as_array()
        .expect("builtins should be an array")
        .iter()
        .any(|item| item["name"] == Value::String("host:echo".into())));
}

#[test]
fn cli_unresolved_call_prints_no_route_frame() {
    let output = Command::new(env!("CARGO_BIN_EXE_faber-host-macos-arm64"))
        .args(["call", "pg:query", "{}"])
        .output()
        .expect("failed to run host call command");

    assert_eq!(output.status.code(), Some(2));
    let json: Value = serde_json::from_slice(&output.stdout).expect("response should be JSON");
    assert_eq!(json["status"], Value::String("error".into()));
    assert_eq!(json["data"]["code"], Value::String("E_NO_ROUTE".into()));
}
