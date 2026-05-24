//! Shared Wasm import routing for Faber capability calls.
//!
//! TARGET: Generated Rust currently lowers `ad` calls to a tiny route-code ABI.
//! This module keeps that temporary ABI aligned for core Wasm modules and
//! Component Model wrappers while both paths route through the same frame
//! kernel internally.

use serde_json::Value;

use crate::kernel::FrameData;
use crate::{Frame, HostError, HostKernel};

pub const CAPABILITY_CALL_IMPORT: &str = "capability-call";
pub const COMPONENT_CODE_HOST_ECHO: u32 = 1;
pub const COMPONENT_CODE_PG_QUERY: u32 = 2;

pub fn route_capability_code(kernel: &HostKernel, route_code: i32) -> Frame {
    let (call, data) = match route_code {
        code if code == COMPONENT_CODE_HOST_ECHO as i32 => {
            let mut data = FrameData::new();
            data.insert("value".into(), Value::String("salve".into()));
            ("host:echo", data)
        }
        code if code == COMPONENT_CODE_PG_QUERY as i32 => ("pg:query", FrameData::new()),
        other => {
            let request = Frame::request("host:unknown").with_from("wasm");
            return request.error(&HostError::invalid_args(format!(
                "unknown capability route code: {other}"
            )));
        }
    };
    let request = Frame::request_with(call, data).with_from("wasm");
    kernel.route(&request)
}
