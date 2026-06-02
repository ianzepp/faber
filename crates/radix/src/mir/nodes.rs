//! Core MIR node model.
//!
//! This file defines the representation shared by MIR lowering, validation,
//! dumps, visitors, and backend probes. The model is intentionally plain data:
//! MIR producers assign stable IDs and preserve storage order, while downstream
//! passes derive maps or control-flow views from those vectors when needed.
//!
//! INVARIANTS
//! ==========
//! - IDs are local to their MIR namespace and are not positional aliases, even
//!   when current lowerers allocate them monotonically.
//! - `MirType` wraps semantic type-table IDs; optional layout IDs are a later
//!   lowering hook and may be absent.
//! - Places describe assignable storage, operands describe readable inputs, and
//!   values describe typed computations whose result may be referenced by ID
//!   after the defining statement in the same block.
//! - Terminators own CFG edges. Statements may call or construct values, but
//!   only terminators decide where control moves next.

use crate::hir::DefId;
use crate::lexer::{Span, Symbol};
use crate::semantic::TypeId;

/// Stable identifier for a MIR function within one `MirProgram`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirFunctionId(pub u32);

/// Stable identifier for a basic block within one MIR function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirBlockId(pub u32);

/// Stable identifier for a named or synthetic local storage slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirLocalId(pub u32);

/// Stable identifier for compiler-created temporary storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirTempId(pub u32);

/// Stable identifier for a typed value computed by a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirValueId(pub u32);

/// Backend-layout handle reserved for later physical representation lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirLayoutId(pub u32);

/// MIR type reference: semantic identity plus optional backend layout identity.
///
/// MIR validation and current probes reason through `semantic`; `layout` exists
/// so later ABI or storage lowering can attach target-specific representation
/// without rewriting the semantic type graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirType {
    semantic: TypeId,
    layout: Option<MirLayoutId>,
}

impl MirType {
    pub fn semantic(semantic: TypeId) -> Self {
        Self { semantic, layout: None }
    }

    pub fn with_layout(semantic: TypeId, layout: MirLayoutId) -> Self {
        Self { semantic, layout: Some(layout) }
    }

    pub fn semantic_id(self) -> TypeId {
        self.semantic
    }

    pub fn layout_id(self) -> Option<MirLayoutId> {
        self.layout
    }
}

/// Complete MIR unit emitted for one analyzed source/package entry.
///
/// Function order is stable output order for dumps and probes. Do not treat the
/// vector index as the function ID; callers should build explicit maps when
/// they need ID lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

impl MirProgram {
    pub fn new() -> Self {
        Self { functions: Vec::new() }
    }
}

impl Default for MirProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution-shaped body for one lowered function.
///
/// Parameters are modeled as locals so calls, assignments, and projections can
/// use one place model. `blocks` are ordered for deterministic rendering; CFG
/// reachability is expressed by terminator target IDs rather than by adjacency.
#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub id: MirFunctionId,
    pub source: Option<DefId>,
    pub name: Option<Symbol>,
    pub params: Vec<MirParam>,
    pub locals: Vec<MirLocal>,
    pub temps: Vec<MirTemp>,
    pub blocks: Vec<MirBlock>,
    pub return_ty: MirType,
    pub error_ty: Option<MirType>,
    pub span: Span,
}

/// Function parameter bound to the local slot that carries its runtime value.
#[derive(Debug, Clone, PartialEq)]
pub struct MirParam {
    pub local: MirLocalId,
    pub name: Option<Symbol>,
    pub ty: MirType,
    pub span: Span,
}

/// Addressable function-local storage.
///
/// Locals include source bindings and synthetic slots that need stable storage
/// identity across statements or blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct MirLocal {
    pub id: MirLocalId,
    pub name: Option<Symbol>,
    pub ty: MirType,
    pub mutable: bool,
    pub span: Span,
}

/// Compiler-created temporary storage with a known semantic type.
#[derive(Debug, Clone, PartialEq)]
pub struct MirTemp {
    pub id: MirTempId,
    pub ty: MirType,
    pub span: Span,
}

/// Basic block: straight-line statements followed by exactly one terminator.
#[derive(Debug, Clone, PartialEq)]
pub struct MirBlock {
    pub id: MirBlockId,
    pub statements: Vec<MirStmt>,
    pub terminator: MirTerminator,
    pub span: Span,
}

/// Side-effecting or value-defining operation that does not choose CFG edges.
#[derive(Debug, Clone, PartialEq)]
pub struct MirStmt {
    pub kind: MirStmtKind,
    pub span: Span,
}

/// Statement forms that execute within the current block.
///
/// Calls and runtime calls may write to a destination, while failable calls are
/// terminators because success and alternate-exit paths split control flow.
#[derive(Debug, Clone, PartialEq)]
pub enum MirStmtKind {
    Assign {
        place: MirPlace,
        value: MirValue,
    },
    Call {
        destination: Option<MirPlace>,
        callee: MirCallee,
        args: Vec<MirOperand>,
    },
    RuntimeCall {
        destination: Option<MirPlace>,
        call: MirRuntimeCall,
    },
    Construct {
        destination: MirPlace,
        aggregate: MirAggregate,
    },
}

/// Block-final control-flow operation.
#[derive(Debug, Clone, PartialEq)]
pub struct MirTerminator {
    pub kind: MirTerminatorKind,
    pub span: Span,
}

/// Terminator taxonomy for all CFG exits from a block.
///
/// `TryCall` is a terminator, not a statement, because the callee's success and
/// error continuations must be explicit in the CFG. `ReturnError` is separate
/// from `Return` so alternate-exit typing remains visible after HIR lowering.
#[derive(Debug, Clone, PartialEq)]
pub enum MirTerminatorKind {
    Return(Option<MirOperand>),
    ReturnError(MirOperand),
    TryCall {
        destination: Option<MirPlace>,
        callee: MirCallee,
        args: Vec<MirOperand>,
        ok_block: MirBlockId,
        error_place: MirPlace,
        error_block: MirBlockId,
    },
    Goto(MirBlockId),
    Branch {
        condition: MirOperand,
        then_block: MirBlockId,
        else_block: MirBlockId,
    },
    Switch {
        value: MirOperand,
        cases: Vec<MirSwitchCase>,
        default: MirBlockId,
    },
    Unreachable,
}

/// One constant-dispatch edge for `Switch`.
#[derive(Debug, Clone, PartialEq)]
pub struct MirSwitchCase {
    pub value: MirConstant,
    pub target: MirBlockId,
}

/// Typed computation result defined by a statement.
///
/// A `MirValueId` is valid only after its defining statement has been processed
/// in the same block. Validation intentionally enforces that local ordering so
/// MIR cannot smuggle unsequenced data dependencies into later phases.
#[derive(Debug, Clone, PartialEq)]
pub struct MirValue {
    pub id: MirValueId,
    pub kind: MirValueKind,
    pub ty: MirType,
    pub span: Span,
}

/// Computation shapes that produce a `MirValue`.
#[derive(Debug, Clone, PartialEq)]
pub enum MirValueKind {
    Operand(MirOperand),
    Unary {
        op: MirUnOp,
        operand: MirOperand,
    },
    Binary {
        op: MirBinOp,
        lhs: MirOperand,
        rhs: MirOperand,
    },
    Option(MirOptionOp),
}

/// Readable input to a computation, call, projection, or terminator.
///
/// Operands are intentionally smaller than expressions. Complex computation is
/// made explicit as statement-defined `MirValue`s so later passes can inspect
/// evaluation order without recovering it from nested syntax.
#[derive(Debug, Clone, PartialEq)]
pub enum MirOperand {
    Place(MirPlace),
    Temp(MirTempId),
    Value(MirValueId),
    Constant(MirConstant),
}

/// Assignable or projectable storage location.
///
/// A place starts from a local or temp and then applies field, variant-field, or
/// index projections. Validation owns the type walk through those projections.
#[derive(Debug, Clone, PartialEq)]
pub struct MirPlace {
    pub base: MirPlaceBase,
    pub projections: Vec<MirProjection>,
}

impl MirPlace {
    pub fn local(id: MirLocalId) -> Self {
        Self { base: MirPlaceBase::Local(id), projections: Vec::new() }
    }

    pub fn temp(id: MirTempId) -> Self {
        Self { base: MirPlaceBase::Temp(id), projections: Vec::new() }
    }
}

/// Root storage namespace for a `MirPlace`.
#[derive(Debug, Clone, PartialEq)]
pub enum MirPlaceBase {
    Local(MirLocalId),
    Temp(MirTempId),
}

/// Projection from a place base into a narrower storage location.
///
/// Variant-field projections carry the variant `DefId` because field symbols
/// alone are not globally unique across enum constructors.
#[derive(Debug, Clone, PartialEq)]
pub enum MirProjection {
    Field(Symbol),
    VariantField { variant: DefId, field: Symbol },
    Index(MirOperand),
}

/// Literal values embedded directly in MIR.
#[derive(Debug, Clone, PartialEq)]
pub enum MirConstant {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Nil,
    Unit,
}

/// Unary operation lowered into MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnOp {
    Neg,
    Not,
    BitNot,
    IsNil,
    IsNonNil,
}

/// Binary operation lowered into MIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Coalesce,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Callable target reference.
///
/// Direct MIR functions are used for functions already lowered into the same
/// program. Definition IDs let validators and probes bridge back to semantic
/// signatures when the callee is known but not represented by a MIR ID.
#[derive(Debug, Clone, PartialEq)]
pub enum MirCallee {
    Function(MirFunctionId),
    Definition(DefId),
    Value(MirOperand),
}

/// Runtime intrinsic call with explicit arguments and result type.
///
/// These nodes keep compiler-known operations distinct from ordinary function
/// calls so validation and backend probes can apply operation-specific arity and
/// type policy.
#[derive(Debug, Clone, PartialEq)]
pub struct MirRuntimeCall {
    pub intrinsic: MirIntrinsic,
    pub args: Vec<MirOperand>,
    pub return_ty: MirType,
}

/// Compiler-runtime operations that are not ordinary user calls.
#[derive(Debug, Clone, PartialEq)]
pub enum MirIntrinsic {
    Diagnostic(MirDiagnosticKind),
    FormatString { template: Symbol },
    Convert(MirConversion),
    Collection(MirCollectionOp),
    Panic,
    Provider(MirProvider),
}

/// Diagnostic runtime channels surfaced by Faber source constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirDiagnosticKind {
    Nota,
    Vide,
    Mone,
    Scribe,
}

/// Conversion intrinsic payload.
///
/// `params` preserve source hints for runtime conversion policy; `fallback`
/// captures explicit fallback behavior as a normal MIR operand.
#[derive(Debug, Clone, PartialEq)]
pub struct MirConversion {
    pub flavor: MirConversionFlavor,
    pub target_ty: MirType,
    pub params: Vec<Symbol>,
    pub fallback: Option<MirOperand>,
}

/// Conversion implementation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirConversionFlavor {
    Cast,
    Runtime,
}

/// Built-in collection operation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirCollectionOp {
    Append,
    AppendImmutable,
    Index,
    Length,
    Contains,
}

/// Provider runtime identity retained from module/name resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct MirProvider {
    pub module: Vec<Symbol>,
    pub name: Symbol,
}

/// Aggregate construction payload.
///
/// Kind and field shape are intentionally separate so validation can reject
/// mismatches such as map construction with ordered fields or struct
/// construction without named operands.
#[derive(Debug, Clone, PartialEq)]
pub struct MirAggregate {
    pub kind: MirAggregateKind,
    pub ty: MirType,
    pub fields: MirAggregateFields,
}

/// Domain kind for a constructed aggregate.
#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateKind {
    Tuple,
    Array,
    Map,
    Set,
    Struct(DefId),
    EnumVariant(DefId),
}

/// Field payload shape for aggregate construction.
#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateFields {
    Ordered(Vec<MirAggregateItem>),
    Named(Vec<MirNamedOperand>),
    Keyed(Vec<MirKeyValueOperand>),
}

/// Ordered aggregate element, either a single operand or a spread.
#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateItem {
    Operand(MirOperand),
    Spread(MirOperand),
}

/// Named aggregate field value.
#[derive(Debug, Clone, PartialEq)]
pub struct MirNamedOperand {
    pub name: Symbol,
    pub value: MirOperand,
}

/// Key/value aggregate entry used by map construction.
#[derive(Debug, Clone, PartialEq)]
pub struct MirKeyValueOperand {
    pub key: MirOperand,
    pub value: MirOperand,
}

/// Explicit nullable-value operation.
///
/// Nullable behavior is represented as MIR operations instead of implicit codegen
/// convention so validation can enforce option payload/result contracts before
/// target-specific lowering.
#[derive(Debug, Clone, PartialEq)]
pub enum MirOptionOp {
    None,
    Some(MirOperand),
    IsNil(MirOperand),
    IsNonNil(MirOperand),
    Unwrap {
        value: MirOperand,
        mode: MirOptionUnwrapMode,
    },
    Coalesce {
        value: MirOperand,
        fallback: MirOperand,
    },
    Chain {
        base: MirOperand,
        link: MirOptionChainLink,
    },
}

/// Policy for unwrap operations after semantic nullability analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirOptionUnwrapMode {
    Assert,
    Assume,
}

/// One optional-chain step from a nullable base.
#[derive(Debug, Clone, PartialEq)]
pub enum MirOptionChainLink {
    Field(Symbol),
    VariantField { variant: DefId, field: Symbol },
    Index(MirOperand),
    Call { callee: MirCallee, args: Vec<MirOperand> },
}

#[cfg(test)]
#[path = "nodes_test.rs"]
mod tests;
