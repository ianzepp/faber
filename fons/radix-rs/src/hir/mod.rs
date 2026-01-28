//! High-level Intermediate Representation
//!
//! HIR is a desugared form of the AST where:
//! - All names are resolved to DefIds
//! - Implicit returns are made explicit
//! - ergo/reddit syntax is desugared to blocks
//! - Method calls are normalized

mod nodes;
mod lower;

pub use nodes::{
    DefId, HirId, HirProgram, HirItem, HirItemKind,
    HirFunction, HirTypeParam, HirParam, HirParamMode,
    HirStruct, HirField, HirMethod, HirReceiver,
    HirEnum, HirVariant, HirVariantField,
    HirInterface, HirInterfaceMethod,
    HirTypeAlias, HirConst, HirImport, HirImportItem,
    HirBlock, HirStmt, HirStmtKind, HirLocal,
    HirExpr, HirExprKind, HirLiteral, HirBinOp, HirUnOp, HirRefKind,
    HirMatchArm, HirPattern,
};
pub use lower::lower;
