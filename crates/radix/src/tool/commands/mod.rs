//! Command handlers for the developer `radix` tool and shared helpers.
//!
//! Submodules group inspection JSON, semantic check, compile/emit, targets,
//! and post-processing. The parent [`crate::tool`] module re-exports the public
//! surface for `radix::tool::*` callers.

mod check;
mod compile;
mod emit;
mod inspect;
mod json;
mod package;
mod postprocess;
mod source;
mod targets;

pub use check::*;
pub use compile::*;
pub use emit::*;
pub use inspect::*;
pub use json::escape_json;
pub use postprocess::*;
pub use source::*;
pub use targets::*;

#[cfg(test)]
#[path = "../../tool_test.rs"]
mod tests;