//! macOS arm64 host runtime primitives for Faber.
//!
//! This crate is the first Faber-owned proof of the host syscall model. It keeps
//! the frame/router/kernel shape local to the macOS host until a second host or
//! concrete duplication justifies extraction. The model is adapted from Muninn's
//! frame and kernel semantics, but this crate intentionally has no Muninn
//! runtime dependency.

pub mod component;
pub mod hal;
pub mod kernel;
pub mod manifest;
pub mod syscall_import;
pub mod wasm;

pub use kernel::{Frame, HostError, HostKernel, Status};
pub use manifest::{CapabilityManifest, RegisteredProvider, SyscallManifest};
