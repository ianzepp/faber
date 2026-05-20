//! HIR node definitions
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Defines the data structures for the High-Level Intermediate Representation.
//! Each HIR node represents a resolved, desugared program construct with all
//! names resolved to DefIds and types (when available) resolved to TypeIds.
//!
//! COMPILER PHASE: HIR Lowering
//! INPUT: N/A (these are the output structures)
//! OUTPUT: Used by semantic analysis, type checking, and code generation
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Two-Level Identification: DefId identifies declarations; HirId identifies
//!   all nodes for type checking results and annotations
//! - Optional Types: Type information (TypeId) is None during lowering and
//!   filled in by the type checker
//! - Span Preservation: Every node includes a Span for error reporting in
//!   later passes (type checking, borrow analysis, codegen)

use crate::lexer::{Span, Symbol};
use crate::semantic::TypeId;

// =============================================================================
// IDENTIFIERS
// =============================================================================
//
// DefId and HirId serve different purposes:
// - DefId: Identifies definitions (function, struct, variable) for name resolution
// - HirId: Identifies every HIR node for type checker to attach type information

/// Definition ID uniquely identifies a named declaration.
///
/// WHY: Separates the concept of "what is this definition" from "what is this
/// node". A single function definition has one DefId but many HirIds (one per
/// expression, statement, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

/// HIR node ID uniquely identifies any HIR node.
///
/// WHY: Type checker needs to attach type information to every expression,
/// not just definitions. HirId provides a unique identifier for this mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(pub u32);

// =============================================================================
// PROGRAM STRUCTURE
// =============================================================================
//
// The HIR program separates top-level items from entry point code.
// WHY: Matches target language structure (Rust, Go) where items are declared
// separately from the main entry point.

/// HIR program containing all top-level items and optional entry point.
///
/// WHY: Separates module-level declarations from entry point execution,
/// matching the structure of target languages like Rust (items vs main fn).
#[derive(Debug)]
pub struct HirProgram {
    /// Top-level declarations (functions, structs, enums, etc.)
    pub items: Vec<HirItem>,
    /// Entry point block (from incipit or implicit top-level statements)
    pub entry: Option<HirBlock>,
}

/// Top-level declaration item.
///
/// WHY: All top-level constructs share common metadata (id, def_id, span)
/// but have different kinds. This structure avoids duplication.
#[derive(Debug)]
pub struct HirItem {
    /// Node identifier for type checker annotations
    pub id: HirId,
    /// Definition identifier for name resolution
    pub def_id: DefId,
    /// The specific kind of item (function, struct, etc.)
    pub kind: HirItemKind,
    /// Source location for error reporting
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

/// Function definition with signature and optional body.
///
/// WHY: Functions may be interface methods (no body) or concrete implementations.
/// Optional ret_ty allows type inference for return types.
#[derive(Debug)]
pub struct HirFunction {
    pub name: Symbol,
    pub type_params: Vec<HirTypeParam>,
    pub params: Vec<HirParam>,
    /// Return type; None until type checker infers it
    pub ret_ty: Option<TypeId>,
    /// Function body; None for interface methods
    pub body: Option<HirBlock>,
    /// Whether function is async (futura)
    pub is_async: bool,
    /// Whether function is a generator (not yet supported)
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
    pub optional: bool,
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
    pub visibility: crate::syntax::Visibility,
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
    Ad(HirAd),
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
pub struct HirAd {
    pub path: Symbol,
    pub args: Vec<HirExpr>,
    pub binding: Option<HirAdBinding>,
    pub body: Option<HirBlock>,
    pub catch: Option<HirBlock>,
}

#[derive(Debug)]
pub struct HirAdBinding {
    pub verb: HirEndpointVerb,
    pub ty: Option<TypeId>,
    pub name: Symbol,
    pub alias: Option<Symbol>,
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
    /// Optional chaining (null-safe member/index/call)
    OptionalChain(Box<HirExpr>, HirOptionalChainKind),
    /// Non-null assertion member/index/call
    NonNull(Box<HirExpr>, HirNonNullKind),
    /// Collection pipeline DSL (ab)
    Ab {
        source: Box<HirExpr>,
        filter: Option<HirCollectionFilter>,
        transforms: Vec<HirCollectionTransform>,
    },
    /// Block expression
    Block(HirBlock),
    /// If expression
    Si(Box<HirExpr>, HirBlock, Option<HirBlock>),
    /// Match expression
    Discerne(Vec<HirExpr>, Vec<HirCasuArm>),
    /// Loop (while true)
    Loop(HirBlock),
    /// While loop
    Dum(Box<HirExpr>, HirBlock),
    /// For loop
    Itera(HirIteraMode, DefId, Symbol, Box<HirExpr>, HirBlock),
    /// Range expression
    Intervallum {
        start: Box<HirExpr>,
        end: Box<HirExpr>,
        step: Option<Box<HirExpr>>,
        kind: HirRangeKind,
    },
    /// Assignment
    Assign(Box<HirExpr>, Box<HirExpr>),
    /// Compound assignment
    AssignOp(HirBinOp, Box<HirExpr>, Box<HirExpr>),
    /// Array literal
    Array(Vec<HirArrayElement>),
    /// Struct literal
    Struct(DefId, Vec<(Symbol, HirExpr)>),
    /// Tuple (for multiple return, etc)
    Tuple(Vec<HirExpr>),
    /// Print/log statement expression
    Scribe(Vec<HirExpr>),
    /// String interpolation expression (scriptum)
    Scriptum(Symbol, Vec<HirExpr>),
    /// Assert statement expression
    Adfirma(Box<HirExpr>, Option<Box<HirExpr>>),
    /// Panic/throw statement expression
    Panic(Box<HirExpr>),
    /// Recoverable throw expression (`iace`)
    Throw(Box<HirExpr>),
    /// try/catch/finally expression
    Tempta {
        body: HirBlock,
        catch: Option<HirBlock>,
        finally: Option<HirBlock>,
    },
    /// Closure
    Clausura(Vec<HirParam>, Option<TypeId>, Box<HirExpr>),
    /// Await
    Cede(Box<HirExpr>),
    /// Unified type conversion / construction expression.
    /// Subsumes qua (cast), innatum (native construction), and novum (struct instantiation).
    /// The type checker and codegen dispatch on the resolved target TypeId to determine semantics.
    Verte {
        source: Box<HirExpr>,
        target: TypeId,
        /// Extracted object fields for map/struct construction.
        /// Present when source is an object literal being constructed into a Map or Struct.
        entries: Option<Vec<HirObjectField>>,
    },
    /// Runtime value conversion (numeratum/fractatum/textatum/bivalentum/⇒).
    /// Unlike Verte (compile-time cast), this performs actual parsing/conversion
    /// and supports fallback values via `vel`.
    Conversio {
        source: Box<HirExpr>,
        target: TypeId,
        /// Codegen hint parameters (e.g., `i32`, `Hex` in `numeratum<i32, Hex>`).
        /// Stored as raw symbols because these are target-specific hints, not Faber types.
        params: Vec<Symbol>,
        fallback: Option<Box<HirExpr>>,
    },
    /// Reference
    Ref(HirRefKind, Box<HirExpr>),
    /// Dereference
    Deref(Box<HirExpr>),
    /// Error placeholder
    Error,
}

#[derive(Debug)]
pub enum HirArrayElement {
    Expr(HirExpr),
    Spread(HirExpr),
}

#[derive(Debug)]
pub struct HirObjectField {
    pub key: HirObjectKey,
    pub value: Option<HirExpr>,
}

#[derive(Debug)]
pub enum HirObjectKey {
    Ident(Symbol),
    String(Symbol),
    Computed(HirExpr),
    Spread(HirExpr),
}

#[derive(Debug)]
pub enum HirOptionalChainKind {
    Member(Symbol),
    Index(Box<HirExpr>),
    Call(Vec<HirExpr>),
}

#[derive(Debug)]
pub enum HirNonNullKind {
    Member(Symbol),
    Index(Box<HirExpr>),
    Call(Vec<HirExpr>),
}

#[derive(Debug)]
pub struct HirCollectionFilter {
    pub negated: bool,
    pub kind: HirCollectionFilterKind,
}

#[derive(Debug)]
pub enum HirCollectionFilterKind {
    Condition(Box<HirExpr>),
    Property(Symbol),
}

#[derive(Debug)]
pub struct HirCollectionTransform {
    pub kind: HirTransformKind,
    pub arg: Option<Box<HirExpr>>,
}

#[derive(Debug, Clone, Copy)]
pub enum HirTransformKind {
    First,
    Last,
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirIteraMode {
    Ex,
    De,
    Pro,
}

#[derive(Debug, Clone, Copy)]
pub enum HirRangeKind {
    Exclusive,
    Inclusive,
}

#[derive(Debug, Clone, Copy)]
pub enum HirEndpointVerb {
    Fit,
    Fiet,
    Fiunt,
    Fient,
}

#[derive(Debug)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    String(Symbol),
    Regex(Symbol, Option<Symbol>),
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
    StrictEq,
    StrictNotEq,
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
    Is,
    IsNot,
    InRange,
    Between,
}

#[derive(Debug, Clone, Copy)]
pub enum HirUnOp {
    Neg,
    Not,
    BitNot,
    IsNull,
    IsNotNull,
    IsNil,
    IsNotNil,
    IsNeg,
    IsPos,
    IsTrue,
    IsFalse,
}

#[derive(Debug, Clone, Copy)]
pub enum HirRefKind {
    Shared,
    Mutable,
}

#[derive(Debug)]
pub struct HirCasuArm {
    pub patterns: Vec<HirPattern>,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug)]
pub enum HirPattern {
    Wildcard,
    Binding(DefId, Symbol),
    Alias(DefId, Symbol, Box<HirPattern>),
    Variant(DefId, Vec<HirPattern>),
    Literal(HirLiteral),
}
