//! High-level Intermediate Representation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! HIR provides a simplified, resolved representation of Faber programs after
//! name resolution and before type checking. This intermediate form eliminates
//! syntactic sugar and resolves all identifiers to definition IDs, making
//! subsequent analysis passes (type checking, borrow checking) simpler.
//!
//! COMPILER PHASE: HIR Lowering
//! INPUT: AST (syntax::Program) with resolved names from the Resolver
//! OUTPUT: HirProgram with resolved DefIds and desugared constructs
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Name Resolution: All identifiers become DefId references, eliminating
//!   the need for repeated symbol table lookups in later passes
//! - Explicit Control Flow: Implicit returns (ergo/reddit) become explicit
//!   return statements for simpler control-flow analysis
//! - Normalized Structure: Entry point code is separated from item declarations,
//!   mirroring target language semantics (e.g., Rust's main function vs items)
//! - Preserved Span Information: All nodes retain source location data for
//!   error reporting in subsequent passes
//!
//! KEY TRANSFORMATIONS
//! ===================
//! 1. Names → DefIds: `functio salve()` → `HirItem { def_id: DefId(42), ... }`
//! 2. Implicit returns: `si x ergo y` → `si x { redde y }`
//! 3. Method normalization: `obj.method()` → `HirExprKind::MethodCall(obj, method, [])`
//! 4. Entry point extraction: Top-level statements → `HirProgram::entry`

mod lower;
mod nodes;

pub use lower::{lower, LowerError};
pub use nodes::{
    DefId, HirBinOp, HirBlock, HirCasuArm, HirConst, HirEnum, HirExpr, HirExprKind, HirField, HirFunction, HirId,
    HirImport, HirImportItem, HirInterface, HirInterfaceMethod, HirItem, HirItemKind, HirIteraMode, HirLiteral,
    HirLocal, HirMethod, HirParam, HirParamMode, HirPattern, HirProgram, HirReceiver, HirRefKind, HirStmt, HirStmtKind,
    HirStruct, HirTypeAlias, HirTypeParam, HirUnOp, HirVariant, HirVariantField,
};
