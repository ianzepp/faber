use serde_json::Value;

use crate::kernel::{Frame, FrameData, HostError, HostResult};

/// Describes a syscall exposed by the host manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyscallInfo {
    pub name: &'static str,
    pub prefix: &'static str,
    pub summary: &'static str,
}

/// Built-in host syscall handler.
///
/// Handlers own a prefix such as `host` or `fs`. The router dispatches by
/// prefix first, while each handler remains responsible for validating exact
/// call names under that prefix.
pub trait Syscall: Send + Sync {
    fn prefix(&self) -> &'static str;

    fn syscalls(&self) -> &'static [SyscallInfo];

    fn dispatch(&self, request: &Frame) -> HostResult<Frame>;
}

/// Minimal built-in host namespace used to prove routing.
pub struct HostEcho;

const HOST_ECHO_SYSCALLS: &[SyscallInfo] = &[SyscallInfo {
    name: "host:echo",
    prefix: "host",
    summary: "Return the request payload unchanged.",
}];

impl Syscall for HostEcho {
    fn prefix(&self) -> &'static str {
        "host"
    }

    fn syscalls(&self) -> &'static [SyscallInfo] {
        HOST_ECHO_SYSCALLS
    }

    fn dispatch(&self, request: &Frame) -> HostResult<Frame> {
        match request.call.as_str() {
            "host:echo" => Ok(request.done_with(echo_data(&request.data))),
            other => Err(HostError::no_route(format!(
                "no built-in host syscall registered for {other}"
            ))),
        }
    }
}

fn echo_data(data: &FrameData) -> FrameData {
    let mut response = FrameData::new();
    response.insert("echo".into(), Value::Object(data.clone()));
    response
}
