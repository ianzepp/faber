//! Middle IR model.
//!
//! MIR is the execution-shaped layer below typed HIR and above target emitters.
//! The current layer includes a data model, deterministic dump rendering,
//! HIR-to-MIR lowering for the supported compiler-developer subset, validation,
//! the `radix mir` inspection command, and a temporary Rust probe. Normal target
//! codegen still uses the existing HIR-backed backend path.

mod dump;
mod lower;
mod nodes;
mod rust_probe;
mod validate;
pub mod visit;

pub use dump::dump_program;
pub use lower::{dump_analyzed_unit, lower_analyzed_unit, MirError};
pub use nodes::*;
pub use rust_probe::{emit_rust_probe, MirRustProbeError};
pub use validate::{validate_program, MirFunctionSignature, MirValidationContext, MirValidationError};
