//! Middle IR model for execution-shaped compiler analysis.
//!
//! MIR sits below typed HIR and above eventual target emitters. HIR preserves
//! much of the source-level syntax; MIR makes control flow, storage locations,
//! temporary values, calls, runtime intrinsics, and aggregate construction
//! explicit enough for validation, inspection, and backend experiments. Normal
//! production code generation still uses the existing HIR-backed backend path;
//! this module is the newer compiler-internal representation being proven out.
//!
//! INVARIANTS
//! ==========
//! - MIR nodes carry semantic `TypeId`s from the type checker, not independent
//!   type definitions.
//! - Function, block, local, temporary, and value IDs are stable within a MIR
//!   program and are rendered directly by dumps and probes.
//! - Vector order is meaningful: functions, locals, temps, blocks, statements,
//!   switch cases, and aggregate fields render and traverse in storage order.
//! - Validation is structural and semantic enough to catch malformed MIR before
//!   experiments consume it, but it does not replace HIR type checking.
//!
//! CURRENT SCOPE
//! =============
//! The module includes the data model, deterministic dump rendering, read-only
//! visitors, HIR-to-MIR lowering for the supported developer subset, validation,
//! the `radix mir` inspection command, and a deliberately temporary Rust probe.

mod dump;
mod llvm_text;
mod lower;
mod nodes;
mod rust_probe;
mod validate;
pub mod visit;
mod wasm_text;

pub use dump::dump_program;
pub use llvm_text::{emit_llvm_text_probe, MirLlvmTextProbeError};
pub use lower::{dump_analyzed_unit, lower_analyzed_unit, lower_analyzed_unit_with_context, LoweredMirUnit, MirError};
pub use nodes::*;
pub use rust_probe::{emit_rust_probe, MirRustProbeError};
pub use validate::{validate_program, MirFunctionSignature, MirValidationContext, MirValidationError};
pub use wasm_text::{emit_wasm_text_probe, emit_wasm_text_probe_with_context, MirWasmTextProbeError};
