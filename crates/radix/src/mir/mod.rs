//! Middle IR model.
//!
//! MIR is the execution-shaped layer below typed HIR and above target emitters.
//! Phase 1 defines the data model and deterministic rendering only; HIR lowering,
//! validation, CLI inspection, and backend consumption are intentionally deferred.

mod dump;
mod nodes;

pub use dump::dump_program;
pub use nodes::*;
