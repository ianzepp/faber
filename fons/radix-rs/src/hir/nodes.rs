//! HIR node definitions

use crate::lexer::{Span, Symbol};
use crate::semantic::TypeId;

/// Definition ID - uniquely identifies a named item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

/// HIR node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(pub u32);

/// HIR program
#[derive(Debug)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub entry: Option<HirBlock>,
}

/// Top-level item
#[derive(Debug)]
pub struct HirItem {
    pub id: HirId,
    pub def_id: DefId,
    pub kind: HirItemKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum HirItemKind {
    Function(HirFunction),
    Struct(HirStruct),
    Enum(HirEnum),
    Interface(HirInterface),
    TypeAlias(HirTypeAlias),
    Const(HirConst),
    Import(HirImport),
}

/// Function definition
#[derive(Debug)]
pub struct HirFunction {
    pub name: Symbol,
    pub type_params: Vec<HirTypeParam>,
    pub params: Vec<HirParam>,
    pub ret_ty: Option<TypeId>,
    pub body: Option<HirBlock>,
    pub is_async: bool,
    pub is_generator: bool,
}

#[derive(Debug)]
pub struct HirTypeParam {
    pub def_id: DefId,
    pub name: Symbol,
    pub span: Span,
}

#[derive(Debug)]
pub struct HirParam {
    pub def_id: DefId,
    pub name: Symbol,
    pub ty: TypeId,
    pub mode: HirParamMode,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum HirParamMode {
    Owned,
    Ref,
    MutRef,
    Move,
}

/// Struct definition
#[derive(Debug)]
pub struct HirStruct {
    pub name: Symbol,
    pub type_params: Vec<HirTypeParam>,
    pub fields: Vec<HirField>,
    pub methods: Vec<HirMethod>,
    pub extends: Option<DefId>,
    pub implements: Vec<DefId>,
}

#[derive(Debug)]
pub struct HirField {
    pub def_id: DefId,
    pub name: Symbol,
    pub ty: TypeId,
    pub is_static: bool,
    pub init: Option<HirExpr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct HirMethod {
    pub def_id: DefId,
    pub func: HirFunction,
    pub receiver: HirReceiver,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum HirReceiver {
    None,   // static
    Ref,    // &self
    MutRef, // &mut self
    Owned,  // self
}

/// Enum definition
#[derive(Debug)]
pub struct HirEnum {
    pub name: Symbol,
    pub type_params: Vec<HirTypeParam>,
    pub variants: Vec<HirVariant>,
}

#[derive(Debug)]
pub struct HirVariant {
    pub def_id: DefId,
    pub name: Symbol,
    pub fields: Vec<HirVariantField>,
    pub span: Span,
}

#[derive(Debug)]
pub struct HirVariantField {
    pub name: Symbol,
    pub ty: TypeId,
    pub span: Span,
}

/// Interface definition
#[derive(Debug)]
pub struct HirInterface {
    pub name: Symbol,
    pub type_params: Vec<HirTypeParam>,
    pub methods: Vec<HirInterfaceMethod>,
}

#[derive(Debug)]
pub struct HirInterfaceMethod {
    pub name: Symbol,
    pub params: Vec<HirParam>,
    pub ret_ty: Option<TypeId>,
    pub span: Span,
}

/// Type alias
#[derive(Debug)]
pub struct HirTypeAlias {
    pub name: Symbol,
    pub ty: TypeId,
}

/// Constant
#[derive(Debug)]
pub struct HirConst {
    pub name: Symbol,
    pub ty: Option<TypeId>,
    pub value: HirExpr,
}

/// Import
#[derive(Debug)]
pub struct HirImport {
    pub path: Symbol,
    pub items: Vec<HirImportItem>,
}

#[derive(Debug)]
pub struct HirImportItem {
    pub def_id: DefId,
    pub name: Symbol,
    pub alias: Option<Symbol>,
}

// =============================================================================
// Statements and Expressions
// =============================================================================

/// Block of statements
#[derive(Debug)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub expr: Option<Box<HirExpr>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct HirStmt {
    pub id: HirId,
    pub kind: HirStmtKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum HirStmtKind {
    Local(HirLocal),
    Expr(HirExpr),
    Redde(Option<HirExpr>),
    Rumpe,
    Perge,
}

#[derive(Debug)]
pub struct HirLocal {
    pub def_id: DefId,
    pub name: Symbol,
    pub ty: Option<TypeId>,
    pub init: Option<HirExpr>,
    pub mutable: bool,
}

#[derive(Debug)]
pub struct HirExpr {
    pub id: HirId,
    pub kind: HirExprKind,
    pub ty: Option<TypeId>,
    pub span: Span,
}

#[derive(Debug)]
pub enum HirExprKind {
    /// Resolved path to a definition
    Path(DefId),
    /// Literal value
    Literal(HirLiteral),
    /// Binary operation
    Binary(HirBinOp, Box<HirExpr>, Box<HirExpr>),
    /// Unary operation
    Unary(HirUnOp, Box<HirExpr>),
    /// Function call
    Call(Box<HirExpr>, Vec<HirExpr>),
    /// Method call (desugared from x.method(args))
    MethodCall(Box<HirExpr>, Symbol, Vec<HirExpr>),
    /// Field access
    Field(Box<HirExpr>, Symbol),
    /// Index access
    Index(Box<HirExpr>, Box<HirExpr>),
    /// Block expression
    Block(HirBlock),
    /// If expression
    Si(Box<HirExpr>, HirBlock, Option<HirBlock>),
    /// Match expression
    Discerne(Box<HirExpr>, Vec<HirCasuArm>),
    /// Loop (while true)
    Loop(HirBlock),
    /// While loop
    Dum(Box<HirExpr>, HirBlock),
    /// For loop
    Itera(DefId, Box<HirExpr>, HirBlock),
    /// Assignment
    Assign(Box<HirExpr>, Box<HirExpr>),
    /// Compound assignment
    AssignOp(HirBinOp, Box<HirExpr>, Box<HirExpr>),
    /// Array literal
    Array(Vec<HirExpr>),
    /// Struct literal
    Struct(DefId, Vec<(Symbol, HirExpr)>),
    /// Tuple (for multiple return, etc)
    Tuple(Vec<HirExpr>),
    /// Closure
    Clausura(Vec<HirParam>, Option<TypeId>, Box<HirExpr>),
    /// Await
    Cede(Box<HirExpr>),
    /// Type cast
    Qua(Box<HirExpr>, TypeId),
    /// Reference
    Ref(HirRefKind, Box<HirExpr>),
    /// Dereference
    Deref(Box<HirExpr>),
    /// Error placeholder
    Error,
}

#[derive(Debug)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone, Copy)]
pub enum HirBinOp {
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
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy)]
pub enum HirUnOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy)]
pub enum HirRefKind {
    Shared,
    Mutable,
}

#[derive(Debug)]
pub struct HirCasuArm {
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug)]
pub enum HirPattern {
    Wildcard,
    Binding(DefId, Symbol),
    Variant(DefId, Vec<HirPattern>),
    Literal(HirLiteral),
}
