//! Frame-shaped syscall kernel for the macOS host.
//!
//! TARGET: This module proves the host-internal contract that future Wasm
//! imports, local sockets, and provider processes should all converge on.
//! WHY: Faber `ad` calls need a stable runtime shape even while the outer host
//! topology is still evolving. A call may start in-process today and cross a
//! socket later, but it should still be routed as a frame with structured
//! terminal success or failure.

mod error;
mod frame;
mod host;
mod router;
mod syscall;

pub use error::{HostError, HostResult};
pub use frame::{Frame, FrameData, Status};
pub use host::HostKernel;
pub use router::Router;
pub use syscall::{HostEcho, Syscall, SyscallInfo};
