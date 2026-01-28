//! High-level Intermediate Representation
//!
//! HIR is a desugared form of the AST where:
//! - All names are resolved to DefIds
//! - Implicit returns are made explicit
//! - ergo/reddit syntax is desugared to blocks
//! - Method calls are normalized
//! - Entry blocks are stored directly on the program

mod lower;
mod nodes;

pub use lower::{lower, LowerError};
pub use nodes::{
    DefId, HirBinOp, HirBlock, HirConst, HirEnum, HirExpr, HirExprKind, HirField, HirFunction,
    HirId, HirImport, HirImportItem, HirInterface, HirInterfaceMethod, HirItem, HirItemKind,
    HirLiteral, HirLocal, HirMatchArm, HirMethod, HirParam, HirParamMode, HirPattern, HirProgram,
    HirReceiver, HirRefKind, HirStmt, HirStmtKind, HirStruct, HirTypeAlias, HirTypeParam, HirUnOp,
    HirVariant, HirVariantField,
};
