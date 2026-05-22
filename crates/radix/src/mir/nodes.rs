use crate::hir::DefId;
use crate::lexer::{Span, Symbol};
use crate::semantic::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirFunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirBlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirLocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirTempId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirValueId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MirLayoutId(pub u32);

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

#[derive(Debug, Clone, PartialEq)]
pub struct MirParam {
    pub local: MirLocalId,
    pub name: Option<Symbol>,
    pub ty: MirType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirLocal {
    pub id: MirLocalId,
    pub name: Option<Symbol>,
    pub ty: MirType,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirTemp {
    pub id: MirTempId,
    pub ty: MirType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirBlock {
    pub id: MirBlockId,
    pub statements: Vec<MirStmt>,
    pub terminator: MirTerminator,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirStmt {
    pub kind: MirStmtKind,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct MirTerminator {
    pub kind: MirTerminatorKind,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct MirSwitchCase {
    pub value: MirConstant,
    pub target: MirBlockId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirValue {
    pub id: MirValueId,
    pub kind: MirValueKind,
    pub ty: MirType,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum MirOperand {
    Place(MirPlace),
    Temp(MirTempId),
    Value(MirValueId),
    Constant(MirConstant),
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum MirPlaceBase {
    Local(MirLocalId),
    Temp(MirTempId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirProjection {
    Field(Symbol),
    VariantField { variant: DefId, field: Symbol },
    Index(MirOperand),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirConstant {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Nil,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnOp {
    Neg,
    Not,
    BitNot,
    IsNil,
    IsNonNil,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum MirCallee {
    Function(MirFunctionId),
    Definition(DefId),
    Value(MirOperand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirRuntimeCall {
    pub intrinsic: MirIntrinsic,
    pub args: Vec<MirOperand>,
    pub return_ty: MirType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirIntrinsic {
    Diagnostic(MirDiagnosticKind),
    FormatString { template: Symbol },
    Convert(MirConversion),
    Collection(MirCollectionOp),
    Panic,
    Provider(MirProvider),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirDiagnosticKind {
    Nota,
    Vide,
    Mone,
    Scribe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirConversion {
    pub flavor: MirConversionFlavor,
    pub target_ty: MirType,
    pub fallback: Option<MirOperand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirConversionFlavor {
    Cast,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirCollectionOp {
    Append,
    AppendImmutable,
    Index,
    Length,
    Contains,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirProvider {
    pub module: Vec<Symbol>,
    pub name: Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirAggregate {
    pub kind: MirAggregateKind,
    pub ty: MirType,
    pub fields: MirAggregateFields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateKind {
    Tuple,
    Array,
    Map,
    Set,
    Struct(DefId),
    EnumVariant(DefId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateFields {
    Ordered(Vec<MirAggregateItem>),
    Named(Vec<MirNamedOperand>),
    Keyed(Vec<MirKeyValueOperand>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MirAggregateItem {
    Operand(MirOperand),
    Spread(MirOperand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirNamedOperand {
    pub name: Symbol,
    pub value: MirOperand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirKeyValueOperand {
    pub key: MirOperand,
    pub value: MirOperand,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirOptionUnwrapMode {
    Assert,
    Assume,
}

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
