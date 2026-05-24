use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::kernel::HostError;

pub type FrameData = Map<String, Value>;

static NEXT_FRAME_ID: AtomicU64 = AtomicU64::new(1);

/// Lifecycle status for host frames.
///
/// The first kernel slice only emits `Request`, `Done`, and `Error`, but the
/// fuller lifecycle is present now so future streaming, cancellation, and daemon
/// transport work do not need to reshape the core frame envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Request,
    Item,
    Bulk,
    Done,
    Error,
    Cancel,
}

impl Status {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Error | Self::Cancel)
    }
}

/// Universal in-memory host message.
///
/// A `Frame` is intentionally serializable even though the first proof routes it
/// in-process. That keeps the same contract usable for JSON debugging, compact
/// binary streams, local sockets, and eventual provider processes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub id: String,
    pub parent_id: Option<String>,
    pub created_ms: u128,
    pub expires_in: u64,
    pub from: Option<String>,
    pub call: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Value>,
    #[serde(default)]
    pub data: FrameData,
}

impl Frame {
    pub fn request(call: impl Into<String>) -> Self {
        Self::request_with(call, FrameData::new())
    }

    pub fn request_with(call: impl Into<String>, data: FrameData) -> Self {
        Self {
            id: next_frame_id(),
            parent_id: None,
            created_ms: now_millis(),
            expires_in: 0,
            from: None,
            call: call.into(),
            status: Status::Request,
            trace: None,
            data,
        }
    }

    pub fn prefix(&self) -> &str {
        self.call
            .split_once(':')
            .map_or(&self.call, |(prefix, _)| prefix)
    }

    pub fn done(&self) -> Self {
        self.response(Status::Done, FrameData::new())
    }

    pub fn done_with(&self, data: FrameData) -> Self {
        self.response(Status::Done, data)
    }

    pub fn error(&self, error: &HostError) -> Self {
        self.response(Status::Error, error.to_data())
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn with_trace(mut self, trace: Value) -> Self {
        self.trace = Some(trace);
        self
    }

    fn response(&self, status: Status, data: FrameData) -> Self {
        Self {
            id: next_frame_id(),
            parent_id: Some(self.id.clone()),
            created_ms: now_millis(),
            expires_in: 0,
            from: Some("faber-host-macos-arm64".into()),
            call: self.call.clone(),
            status,
            trace: self.trace.clone(),
            data,
        }
    }
}

fn next_frame_id() -> String {
    let seq = NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed);
    format!("frame-{seq}")
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}
