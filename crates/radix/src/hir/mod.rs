//! High-level intermediate representation for semantically resolved Faber.
//!
//! HIR is the compiler boundary where parsed syntax becomes a stable semantic
//! model. Lowering consumes the AST after name resolution, carries `DefId`
//! references for definitions and bindings, preserves source spans for later
//! diagnostics, and normalizes constructs that later phases should not need to
//! rediscover from grammar shape.
//!
//! This layer is still intentionally high-level. It records declarations,
//! statements, expressions, handler structure, collection forms, and target
//! type references, but it does not prove type correctness, choose MIR-level
//! control-flow shape, or decide backend emission details. Typecheck, MIR, and
//! codegen should be able to trust HIR identity and span invariants without
//! treating HIR as already fully typed.
//!
//! INVARIANTS
//! ==========
//! - `DefId` identifies declarations and bindings after resolution.
//! - `HirId` identifies nodes that later analyses may annotate.
//! - Spans remain attached to HIR nodes even when syntax is normalized.
//! - Optional `TypeId` fields mean "not established yet" unless the field-level
//!   contract says the type was already required by the source form.
//! - Top-level executable source is represented as `HirProgram::entry`, separate
//!   from item declarations, so package and backend code see one program shape.
//!
//! PHASE BOUNDARY
//! ==============
//! HIR lowering owns syntax-to-semantics normalization. Typecheck owns inferred
//! types and semantic validity. MIR/codegen own backend-shaped control flow,
//! storage layout, and target-specific lowering.

mod lower;
mod nodes;
pub mod visit;

pub use lower::{lower, lower_with_cli, LowerError};
pub use nodes::{
    DefId, HirAd, HirAdBinding, HirArrayElement, HirBinOp, HirBlock, HirCape, HirCasuArm, HirCollectionFilter,
    HirCollectionFilterKind, HirCollectionTransform, HirConst, HirEndpointVerb, HirEnum, HirExpr, HirExprKind,
    HirField, HirFunction, HirId, HirImport, HirImportItem, HirInterface, HirInterfaceMethod, HirItem, HirItemKind,
    HirIteraMode, HirLiteral, HirLocal, HirMethod, HirNonNullKind, HirObjectField, HirObjectKey, HirOptionalChainKind,
    HirParam, HirParamMode, HirPattern, HirProgram, HirRangeKind, HirReceiver, HirRefKind, HirScribeKind, HirStmt,
    HirStmtKind, HirStruct, HirTestMetadata, HirTestModifier, HirTransformKind, HirTypeAlias, HirTypeParam, HirUnOp,
    HirVariant, HirVariantField,
};
