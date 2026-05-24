use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::kernel::FrameData;

pub type HostResult<T> = Result<T, HostError>;

/// Machine-readable host error carried by error frames.
///
/// The `E_` code convention follows the host syscall model and mirrors the
/// Muninn kernel semantics Faber is adapting. These errors are part of the host
/// contract: strict-mode checks, runtime diagnostics, and future provider
/// transports all need stable codes rather than prose-only failures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl HostError {
    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self::new("E_INVALID_ARGS", message, false)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("E_FORBIDDEN", message, false)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new("E_TIMEOUT", message, true)
    }

    pub fn cancelled() -> Self {
        Self::new("E_CANCELLED", "operation cancelled", false)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("E_INTERNAL", message, false)
    }

    pub fn no_route(message: impl Into<String>) -> Self {
        Self::new("E_NO_ROUTE", message, false)
    }

    pub fn to_data(&self) -> FrameData {
        let mut data = Map::new();
        data.insert("code".into(), Value::String(self.code.clone()));
        data.insert("message".into(), Value::String(self.message.clone()));
        data.insert("retryable".into(), Value::Bool(self.retryable));
        data
    }

    fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for HostError {}
