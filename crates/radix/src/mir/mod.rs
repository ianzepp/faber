//! Middle IR model.
//!
//! MIR is the execution-shaped layer below typed HIR and above target emitters.
//! Phase 1 defines the data model and deterministic rendering only; HIR lowering,
//! validation, CLI inspection, and backend consumption are intentionally deferred.

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
