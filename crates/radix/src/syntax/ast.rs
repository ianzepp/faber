//! Concrete syntax model for parsed Faber source.
//!
//! This file defines the AST that the parser hands to the rest of `radix`.
//! It is intentionally a source-shaped model: declarations, expressions, type
//! syntax, annotations, and directives are recorded as the user wrote them, with
//! spans and node IDs for later phases to attach diagnostics and side tables.
//!
//! The AST does not decide semantic meaning. Name resolution, inferred types,
//! target compatibility, stdlib translation behavior, exhaustiveness, and data
//! flow live in later compiler phases keyed by these nodes. Keeping the tree
//! syntactic lets the parser preserve edge cases and lets semantic analysis
//! reject bad programs with source-accurate context instead of losing grammar
//! detail during parse.
//!
//! INVARIANTS
//! ==========
//! - Every statement and expression parsed from source carries a [`NodeId`].
//! - `span` fields are parse provenance for diagnostics and source mapping.
//! - [`TypeExpr`] represents written type syntax, not a checked type.
//! - Annotation nodes preserve metadata syntax; interpretation is owned by
//!   lowering, package tooling, and backend-specific codegen.
//! - Legacy compatibility fields remain explicit so callers do not guess when
//!   grammar migrations are incomplete.

use crate::lexer::{Span, Symbol, Token};

// =============================================================================
// CORE TYPES
// =============================================================================

/// Parser-assigned identifier for statement and expression nodes.
///
/// `NodeId` is the rendezvous point between the syntax tree and later phase
/// side tables such as inferred types, flow facts, and resolved definitions. It
/// is not globally meaningful outside one parsed program/session, and it should
/// not be serialized as a stable language artifact.
pub type NodeId = u32;

/// Parsed contents of one Faber source unit.
///
/// Directives are kept separate from ordinary statements because they configure
/// source-unit behavior before semantic analysis walks the body. Package mode
/// may assemble many `Program`s, but each value remains the parse result for one
/// file-like source.
#[derive(Debug)]
pub struct Program {
    /// File-scoped directives parsed before the statement body.
    pub directives: Vec<DirectiveDecl>,

    /// Top-level declarations and executable statements in source order.
    pub stmts: Vec<Stmt>,

    /// Source span covering the parsed unit.
    pub span: Span,
}

// =============================================================================
// Statements
// =============================================================================

#[derive(Debug)]
pub struct Stmt {
    /// Stable handle used by later phases for statement-scoped facts.
    pub id: NodeId,

    /// Grammar form of this statement.
    pub kind: StmtKind,

    /// Source location for diagnostics.
    pub span: Span,

    /// Metadata attached directly to this statement by the parser.
    pub annotations: Vec<Annotation>,
}

pub type AnnotatedStmt = Stmt;

/// Top-level and block-level statement forms recognized by the parser.
///
/// Variants name grammar categories, not semantic classes. For example,
/// `Import`, `Ad`, and `Incipit` still require package or lowering policy
/// before they become module dependencies, endpoints, or entrypoints.
#[derive(Debug)]
pub enum StmtKind {
    /// Variable declaration: fixum/varia
    Var(VarDecl),
    /// Function declaration
    Func(FuncDecl),
    /// Class declaration: genus
    Class(ClassDecl),
    /// Interface declaration: pactum
    Interface(InterfaceDecl),
    /// Type alias: typus
    TypeAlias(TypeAliasDecl),
    /// Enum: ordo
    Enum(EnumDecl),
    /// Tagged union: discretio
    Union(UnionDecl),
    /// Import statement
    Import(ImportDecl),
    /// Test suite: probandum
    Probandum(ProbandumDecl),
    /// Test case: proba
    Proba(ProbaCase),
    /// Extract: ex <expr> fixum|varia <fields>
    Ex(ExStmt),
    /// Block: { ... }
    Block(BlockStmt),
    /// Expression statement
    Expr(ExprStmt),
    /// If statement
    Si(SiStmt),
    /// While loop: dum
    Dum(DumStmt),
    /// For loop: itera
    Itera(IteraStmt),
    /// Switch: elige
    Elige(EligeStmt),
    /// Pattern match: discerne
    Discerne(DiscerneStmt),
    /// Guard: custodi
    Custodi(CustodiStmt),
    /// Scoped block: fac
    Fac(FacStmt),
    /// Return: redde
    Redde(ReddeStmt),
    /// Break: rumpe
    Rumpe(RumpeStmt),
    /// Continue: perge
    Perge(PergeStmt),
    /// Throw: iace
    Iace(IaceStmt),
    /// Panic: mori
    Mori(MoriStmt),
    /// Explicit noop: tacet
    Tacet(TacetStmt),
    /// Try/catch: tempta
    Tempta(TemptaStmt),
    /// Assert: adfirma
    Adfirma(AdfirmaStmt),
    /// Diagnostics: nota/vide/mone/scribe
    Scribe(ScribeStmt),
    /// Entry point: incipit/incipiet
    Incipit(IncipitStmt),
    /// Resource management: cura
    Cura(CuraStmt),
    /// Endpoint: ad
    Ad(AdStmt),
}

// =============================================================================
// Declarations
// =============================================================================

#[derive(Debug)]
pub struct VarDecl {
    /// Declared binding policy from `fixum` or `varia`.
    pub mutability: Mutability,

    /// Whether the initializer was marked for await-like binding behavior.
    pub is_await: bool,

    /// Written type annotation, if present; inference is resolved later.
    pub ty: Option<TypeExpr>,

    /// Identifier or destructuring pattern introduced by the declaration.
    pub binding: BindingPattern,

    /// Optional initializer expression.
    pub init: Option<Box<Expr>>,
}

/// Binding target syntax used by declarations and destructuring forms.
///
/// Patterns only describe where names are introduced. They do not encode
/// resolved field types, tuple/array lengths, or whether a source expression can
/// actually be destructured; those are semantic checks.
#[derive(Debug)]
pub enum BindingPattern {
    Ident(Ident),
    Wildcard(Span),
    Array {
        elements: Vec<BindingPattern>,
        rest: Option<Ident>,
        span: Span,
    },
    Object {
        fields: Vec<ExField>,
        rest: Option<Ident>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable, // fixum
    Mutable,   // varia
}

#[derive(Debug)]
pub struct FuncDecl {
    /// Declared function name.
    pub name: Ident,

    /// Generic parameters as written in source order.
    pub type_params: Vec<TypeParam>,

    /// Parameter list, including ownership mode and optional defaults.
    pub params: Vec<Param>,

    /// Function-level modifiers that affect CLI/runtime/lowering policy.
    pub modifiers: Vec<FuncModifier>,

    /// Return type annotation. Absence does not imply `vacuum` until checked.
    pub ret: Option<TypeExpr>,

    /// Declared error type for throwing functions, if present.
    pub err: Option<TypeExpr>,

    /// Function body, absent for interface or external declarations.
    pub body: Option<BlockStmt>,

    /// Annotations parsed as part of the declaration payload.
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
pub struct TypeParam {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct Param {
    /// `sponte` marks a voluntary parameter slot; it is syntax, not nullability.
    pub sponte: bool,

    /// `fixus` marks a parameter fixed after initialization.
    pub fixus: bool,

    /// Ownership/borrowing mode from `de`, `in`, or `ex`.
    pub mode: ParamMode,

    /// `ceteri` rest parameter marker.
    pub rest: bool,

    /// Written parameter type.
    pub ty: TypeExpr,

    /// Binding name introduced by the parameter.
    pub name: Ident,

    /// Optional external alias introduced by `ut`.
    pub alias: Option<Ident>,

    /// Default expression, if the grammar supplied one.
    pub default: Option<Box<Expr>>,

    /// Full parameter span for diagnostics.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParamMode {
    /// Default value-passing mode when no ownership marker is written.
    #[default]
    Owned,

    /// `de`: shared borrow-like parameter mode.
    Ref,

    /// `in`: mutable borrow-like parameter mode.
    MutRef,

    /// `ex`: explicit move mode.
    Move,
}

/// Function modifiers parsed from declaration syntax.
///
/// These values are still syntax-level metadata. Package command generation,
/// CLI binding, runtime integration, and codegen decide which combinations are
/// meaningful for a target.
#[derive(Debug)]
pub enum FuncModifier {
    Argumenta(Ident),
    Curata { required: Ident, alias: Option<Ident> },
    Errata(Ident),
    Exitus(ExitusValue),
    Immutata,
    Iacit,
    Optiones(Ident),
}

#[derive(Debug)]
pub enum ExitusValue {
    Name(Ident),
    Number(i64),
}

#[derive(Debug)]
pub struct ClassDecl {
    pub is_abstract: bool,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub extends: Option<Ident>,
    pub implements: Vec<Ident>,
    pub members: Vec<ClassMember>,
}

#[derive(Debug)]
pub struct ClassMember {
    pub annotations: Vec<Annotation>,
    pub kind: ClassMemberKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ClassMemberKind {
    Field(FieldDecl),
    Method(FuncDecl),
}

#[derive(Debug)]
pub struct FieldDecl {
    pub is_static: bool, // generis
    pub is_bound: bool,  // nexum
    pub sponte: bool,    // sponte — voluntary field
    pub fixus: bool,     // fixus — fixed after init
    pub ty: TypeExpr,
    pub name: Ident,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct InterfaceDecl {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub methods: Vec<InterfaceMethod>,
}

#[derive(Debug)]
pub struct InterfaceMethod {
    pub name: Ident,
    pub params: Vec<Param>,
    pub modifiers: Vec<FuncModifier>,
    pub ret: Option<TypeExpr>,
    pub err: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct TypeAliasDecl {
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct EnumDecl {
    pub name: Ident,
    pub members: Vec<EnumMember>,
}

#[derive(Debug)]
pub struct EnumMember {
    pub name: Ident,
    pub value: Option<EnumValue>,
    pub span: Span,
}

#[derive(Debug)]
pub enum EnumValue {
    Integer(i64),
    String(Symbol),
}

#[derive(Debug)]
pub struct UnionDecl {
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Variant {
    pub name: Ident,
    pub fields: Vec<VariantField>,
    pub span: Span,
}

#[derive(Debug)]
pub struct VariantField {
    pub ty: TypeExpr,
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct ImportDecl {
    /// Import specifier exactly as parsed from source.
    pub path: Symbol,

    /// Source visibility marker; package resolution interprets its reach.
    pub visibility: Visibility,

    /// Import binding shape.
    pub kind: ImportKind,

    /// Span of the full import declaration.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

/// Parser-level import binding form.
///
/// The path may later resolve to a local module or a built-in library interface;
/// that boundary is owned outside syntax so parsing does not special-case
/// providers such as `norma`.
#[derive(Debug)]
pub enum ImportKind {
    Named { name: Ident, alias: Option<Ident> },
    Wildcard { alias: Ident },
}

#[derive(Debug)]
pub struct DirectiveDecl {
    /// Directive name following the source directive marker.
    pub name: Ident,

    /// Literal or identifier arguments preserved for later interpretation.
    pub args: Vec<DirectiveArg>,

    /// Span of the complete directive.
    pub span: Span,
}

#[derive(Debug)]
pub enum DirectiveArg {
    String(Symbol),
    Ident(Ident),
}

#[derive(Debug)]
pub struct ProbandumDecl {
    /// Suite name as source text.
    pub name: Symbol,

    /// Test-suite modifiers that later test lowering interprets.
    pub modifiers: Vec<ProbaModifier>,

    /// Setup blocks, cases, and nested suites.
    pub body: ProbandumBody,

    /// Span of the full suite declaration.
    pub span: Span,
}

#[derive(Debug)]
pub struct ProbandumBody {
    pub setup: Vec<PraeparaBlock>,
    pub tests: Vec<ProbaCase>,
    pub nested: Vec<ProbandumDecl>,
}

#[derive(Debug)]
pub struct PraeparaBlock {
    pub kind: PraeparaKind,
    pub all: bool,
    pub body: BlockStmt,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum PraeparaKind {
    Praepara,
    Praeparabit,
    Postpara,
    Postparabit,
}

#[derive(Debug)]
pub struct ProbaCase {
    pub modifiers: Vec<ProbaModifier>,
    pub name: Symbol,
    pub body: BlockStmt,
    pub span: Span,
}

/// Parsed test modifier taxonomy.
///
/// The AST preserves these modifiers as declared; scheduling behavior, target
/// filtering, retries, and timing policy are owned by the test backend.
#[derive(Debug, Clone)]
pub enum ProbaModifier {
    Omitte(Symbol),
    Futurum(Symbol),
    Solum,
    Tag(Symbol),
    Temporis(i64),
    Metior,
    Repete(i64),
    Fragilis(i64),
    Requirit(Symbol),
    SolumIn(Symbol),
}

// =============================================================================
// Statements (control flow)
// =============================================================================

#[derive(Debug)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ExprStmt {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct SiStmt {
    pub cond: Box<Expr>,
    pub then: IfBody,
    pub catch: Option<CapeClause>,
    pub else_: Option<SecusClause>,
}

#[derive(Debug)]
pub enum IfBody {
    Block(BlockStmt),
    Ergo(Box<Stmt>),
}

#[derive(Debug)]
pub enum SecusClause {
    Sin(Box<SiStmt>),
    Block { body: BlockStmt, catch: Option<CapeClause> },
    Stmt { stmt: Box<Stmt>, catch: Option<CapeClause> },
}

#[derive(Debug)]
pub struct DumStmt {
    pub cond: Box<Expr>,
    pub body: IfBody,
    pub catch: Option<CapeClause>,
}

#[derive(Debug)]
pub struct IteraStmt {
    pub mode: IteraMode,
    pub iterable: Box<Expr>,
    pub mutability: Mutability,
    pub binding: Ident,
    pub body: IfBody,
    pub catch: Option<CapeClause>,
}

#[derive(Debug, Clone, Copy)]
pub enum IteraMode {
    Ex,
    De,
    Pro,
}

#[derive(Debug)]
pub struct EligeStmt {
    pub expr: Box<Expr>,
    pub cases: Vec<CasuCase>,
    pub default: Option<CeterumDefault>,
    pub catch: Option<CapeClause>,
}

#[derive(Debug)]
pub struct CasuCase {
    pub value: Box<Expr>,
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct CeterumDefault {
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct DiscerneStmt {
    /// `omnia` marker requiring exhaustive semantic coverage.
    pub exhaustive: bool,

    /// Expressions being matched.
    pub subjects: Vec<Expr>,

    /// Pattern arms in source order.
    pub arms: Vec<CasuArm>,

    /// Optional fallback arm.
    pub default: Option<CeterumDefault>,
}

#[derive(Debug)]
pub struct CasuArm {
    pub patterns: Vec<Pattern>,
    pub body: IfBody,
    pub span: Span,
}

/// Pattern syntax accepted inside `discerne`.
///
/// Patterns here identify grammar shape and bound names only. Variant
/// resolution, literal compatibility, and exhaustiveness are checked after name
/// and type information exist.
#[derive(Debug)]
pub enum Pattern {
    Wildcard(Span),
    Ident(Ident, Option<PatternBind>),
    Literal(Literal, Span),
    Path(PathPattern),
}

#[derive(Debug)]
pub struct PathPattern {
    pub segments: Vec<Ident>,
    pub bind: Option<PatternBind>,
    pub span: Span,
}

#[derive(Debug)]
pub enum PatternBind {
    /// `ut NAME`: bind the matched value under an alias.
    Alias(Ident),

    /// Destructure and bind the listed names with the declared mutability.
    Bindings { mutability: Mutability, names: Vec<Ident> },
}

#[derive(Debug)]
pub struct CustodiStmt {
    pub clauses: Vec<CustodiClause>,
}

#[derive(Debug)]
pub struct CustodiClause {
    pub cond: Box<Expr>,
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct FacStmt {
    pub body: BlockStmt,
    pub catch: Option<CapeClause>,
    pub while_: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct ReddeStmt {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct RumpeStmt {
    pub span: Span,
}

#[derive(Debug)]
pub struct PergeStmt {
    pub span: Span,
}

#[derive(Debug)]
pub struct IaceStmt {
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct MoriStmt {
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct TacetStmt {
    pub span: Span,
}

#[derive(Debug)]
pub struct TemptaStmt {
    pub body: BlockStmt,
    pub catch: Option<CapeClause>,
    pub finally: Option<BlockStmt>,
}

#[derive(Debug)]
pub struct CapeClause {
    pub binding: Ident,
    pub body: BlockStmt,
    pub span: Span,
}

#[derive(Debug)]
pub struct AdfirmaStmt {
    pub cond: Box<Expr>,
    pub message: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct ScribeStmt {
    pub kind: ScribeKind,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum ScribeKind {
    Scribe,
    Vide,
    Mone,
    Nota,
}

#[derive(Debug)]
pub struct IncipitStmt {
    /// Distinguishes `incipiet` from synchronous `incipit`.
    pub is_async: bool,

    /// Entrypoint body in either block or single-statement form.
    pub body: IfBody,

    /// Optional argument binding exposed to the entrypoint body.
    pub args: Option<Ident>,

    /// Optional exit expression supplied by the entrypoint declaration.
    pub exitus: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct ExStmt {
    pub source: Box<Expr>,
    pub mutability: Mutability,
    pub fields: Vec<ExField>,
    pub rest: Option<Ident>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ExField {
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug)]
pub struct CuraStmt {
    pub kind: CuraKind,
    pub mutability: Mutability,
    pub ty: TypeExpr,
    pub binding: Ident,
    pub body: BlockStmt,
    pub catch: Option<CapeClause>,
}

#[derive(Debug, Clone, Copy)]
pub enum CuraKind {
    Arena,
    Page,
}

#[derive(Debug)]
pub struct AdStmt {
    pub path: Symbol,
    pub args: Vec<Argument>,
    pub binding: Option<AdBinding>,
    pub body: Option<BlockStmt>,
    pub catch: Option<CapeClause>,
}

#[derive(Debug)]
pub struct AdBinding {
    pub verb: EndpointVerb,
    pub ty: Option<TypeExpr>,
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone, Copy)]
pub enum EndpointVerb {
    /// `fit`: synchronous singular endpoint binding.
    Fit,

    /// `fiet`: asynchronous singular endpoint binding.
    Fiet,

    /// `fiunt`: synchronous plural endpoint binding.
    Fiunt,

    /// `fient`: asynchronous plural endpoint binding.
    Fient,
}

// =============================================================================
// Expressions
// =============================================================================

#[derive(Debug)]
pub struct Expr {
    /// Stable handle used by later phases for expression-scoped facts.
    pub id: NodeId,

    /// Grammar form of this expression.
    pub kind: ExprKind,

    /// Source location for diagnostics.
    pub span: Span,
}

/// Value-producing source forms.
///
/// These variants encode parse shape, including sugared forms such as optional
/// chains and collection DSL fragments. Lowering decides which forms normalize
/// to shared HIR operations and which remain target-specific.
#[derive(Debug)]
pub enum ExprKind {
    /// Identifier
    Ident(Ident),
    /// Literal value
    Literal(Literal),
    /// Contextual empty value expression: vacua
    Vacua(Span),
    /// Binary operation
    Binary(BinaryExpr),
    /// Unary operation
    Unary(UnaryExpr),
    /// Ternary conditional
    Ternary(TernaryExpr),
    /// Function call
    Call(CallExpr),
    /// Member access: x.y
    Member(MemberExpr),
    /// Index access: x[y]
    Index(IndexExpr),
    /// Optional chain: x?.y, x?[y], x?(args)
    OptionalChain(OptionalChainExpr),
    /// Non-null assertion: x!.y, x![y], x!(args)
    NonNull(NonNullExpr),
    /// Assignment
    Assign(AssignExpr),
    /// Unified type conversion: ⇢ (postfix only; Latin aliases removed)
    Verte(VerteExpr),
    /// Variant construction: finge
    Finge(FingeExpr),
    /// Closure: clausura
    Clausura(ClausuraExpr),
    /// Await: cede
    Cede(CedeExpr),
    /// Array literal
    Array(ArrayExpr),
    /// Object literal
    Object(ObjectExpr),
    /// Range: x‥y, x…y, x ante y, x usque y
    Intervallum(IntervallumExpr),
    /// Collection DSL: ab
    Ab(AbExpr),
    /// Runtime value conversion: ⇒ target
    Conversio(ConversioExpr),
    /// Interpolated script: scriptum
    Scriptum(ScriptumExpr),
    /// Read input: lege
    Lege(LegeExpr),
    /// Regex literal: sed
    Sed(SedExpr),
    /// Comptime expression: praefixum(expr)
    Praefixum(PraefixumExpr),
    /// Self reference
    Ego(Span),
    /// Parenthesized expression
    Paren(Box<Expr>),
}

#[derive(Debug)]
pub struct Ident {
    /// Interned spelling of the identifier.
    pub name: Symbol,

    /// Span of the identifier token.
    pub span: Span,
}

#[derive(Debug)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(Symbol),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    NotEq,
    StrictEq,
    StrictNotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // Logical
    And,
    Or,
    // Nullish
    Coalesce,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Identity
    Is,
    IsNot,
    // Range
    InRange,
    Between,
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub operand: Box<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,       // -
    Not,       // non
    BitNot,    // ¬
    IsNull,    // nulla
    IsNotNull, // nonnulla
    IsNil,     // nihil (the check, not the literal)
    IsNotNil,  // nonnihil
    IsNeg,     // negativum
    IsPos,     // positivum
    IsTrue,    // verum
    IsFalse,   // falsum
}

#[derive(Debug)]
pub struct TernaryExpr {
    pub cond: Box<Expr>,
    pub then: Box<Expr>,
    pub else_: Box<Expr>,
    pub style: TernaryStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum TernaryStyle {
    QuestionColon, // ? :
    SicSecus,      // sic secus
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Argument>,
}

#[derive(Debug)]
pub struct Argument {
    pub spread: bool, // sparge
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct MemberExpr {
    pub object: Box<Expr>,
    pub member: Ident,
}

#[derive(Debug)]
pub struct IndexExpr {
    pub object: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug)]
pub struct OptionalChainExpr {
    pub object: Box<Expr>,
    pub chain: OptionalChainKind,
}

#[derive(Debug)]
pub enum OptionalChainKind {
    Member(Ident),
    Index(Box<Expr>),
    Call(Vec<Argument>),
}

#[derive(Debug)]
pub struct NonNullExpr {
    pub object: Box<Expr>,
    pub chain: NonNullKind,
}

#[derive(Debug)]
pub enum NonNullKind {
    Member(Ident),
    Index(Box<Expr>),
    Call(Vec<Argument>),
}

#[derive(Debug)]
pub struct AssignExpr {
    pub op: AssignOp,
    pub target: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    BitAndAssign,
    BitOrAssign,
}

#[derive(Debug)]
pub struct VerteExpr {
    pub expr: Box<Expr>,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct FingeExpr {
    pub variant: Ident,
    pub fields: Vec<FingeFieldInit>,
    pub cast: Option<Ident>, // ⇢ (finge verte form)
}

#[derive(Debug)]
pub struct FingeFieldInit {
    pub name: Ident,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ClausuraExpr {
    pub params: Vec<ClausuraParam>,
    pub ret: Option<TypeExpr>,
    pub err: Option<TypeExpr>,
    pub body: ClausuraBody,
}

#[derive(Debug)]
pub struct ClausuraParam {
    pub ty: TypeExpr,
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub enum ClausuraBody {
    Expr(Box<Expr>),
    Block(BlockStmt),
    Fac(FacStmt),
}

#[derive(Debug)]
pub struct CedeExpr {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct ArrayExpr {
    pub elements: Vec<ArrayElement>,
}

#[derive(Debug)]
pub enum ArrayElement {
    Expr(Box<Expr>),
    Spread(Box<Expr>),
}

#[derive(Debug)]
pub struct ObjectExpr {
    pub fields: Vec<ObjectField>,
}

#[derive(Debug)]
pub struct ObjectField {
    pub key: ObjectKey,
    pub value: Option<Box<Expr>>, // None for shorthand
    pub span: Span,
}

#[derive(Debug)]
pub enum ObjectKey {
    Ident(Ident),
    String(Symbol),
    Computed(Box<Expr>),
    Spread(Box<Expr>),
}

#[derive(Debug)]
pub struct IntervallumExpr {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub step: Option<Box<Expr>>,
    pub kind: RangeKind,
}

#[derive(Debug, Clone, Copy)]
pub enum RangeKind {
    Exclusive, // ‥ or ante
    Inclusive, // … or usque
}

#[derive(Debug)]
pub struct AbExpr {
    pub source: Box<Expr>,
    pub filter: Option<CollectionFilter>,
    pub transforms: Vec<CollectionTransform>,
}

#[derive(Debug)]
pub struct CollectionFilter {
    pub negated: bool,
    pub kind: CollectionFilterKind,
}

#[derive(Debug)]
pub enum CollectionFilterKind {
    Condition(Box<Expr>), // ubi EXPR
    Property(Ident),      // IDENT (boolean property)
}

#[derive(Debug)]
pub struct CollectionTransform {
    pub kind: TransformKind,
    pub arg: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum TransformKind {
    First, // prima
    Last,  // ultima
    Sum,   // summa
}

#[derive(Debug)]
pub struct ConversioExpr {
    pub expr: Box<Expr>,
    pub target: ConversioTarget,
    pub type_params: Vec<TypeExpr>,
    pub fallback: Option<Box<Expr>>,
}

/// Target of a conversio expression.
#[derive(Debug)]
pub enum ConversioTarget {
    Explicit(TypeExpr),
}

#[derive(Debug)]
pub struct ScriptumExpr {
    pub template: Symbol,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct LegeExpr {
    /// `lineam` marker for line-oriented input.
    pub line: bool,

    pub span: Span,
}

#[derive(Debug)]
pub struct SedExpr {
    pub pattern: Symbol,
    pub flags: Option<Symbol>,
    pub span: Span,
}

/// Comptime expression: praefixum(expr) - forces compile-time evaluation
/// Maps to Zig's `comptime`
#[derive(Debug)]
pub struct PraefixumExpr {
    pub body: PraefixumBody,
}

#[derive(Debug)]
pub enum PraefixumBody {
    Block(BlockStmt),
    Expr(Box<Expr>),
}

// =============================================================================
// Types
// =============================================================================

#[derive(Debug)]
pub struct TypeExpr {
    /// Legacy `si` nullability marker.
    ///
    /// This field remains explicit for migration safety, but new nullable value
    /// types are represented as [`TypeExprKind::Union`] with `nihil` as a
    /// member.
    pub nullable: bool,

    /// Borrowing mode marker from `de` or `in`, if written in type position.
    pub mode: Option<TypeMode>,

    /// Syntactic form of the type expression.
    pub kind: TypeExprKind,

    /// Span of the complete type expression.
    pub span: Span,
}

/// Borrowing mode written in type position.
///
/// This is syntax-level intent. Later phases decide whether the selected target
/// or semantic context supports the mode.
#[derive(Debug, Clone, Copy)]
pub enum TypeMode {
    /// `de`: shared reference-like mode.
    Ref,

    /// `in`: mutable reference-like mode.
    MutRef,
}

/// Source-level type syntax.
///
/// This enum is deliberately not the compiler's checked type model. It preserves
/// source spelling such as `_`, generic application, array syntax, function
/// types, and inline unions so type checking can report errors against the
/// syntax the user wrote.
#[derive(Debug)]
pub enum TypeExprKind {
    /// Inferred type marker: _
    Infer,
    /// Named type with optional type parameters
    Named(Ident, Vec<TypeExpr>),
    /// Array type: T[]
    Array(Box<TypeExpr>),
    /// Function type: (A, B) -> C
    Func(FuncTypeExpr),
    /// Inline union type: T ∪ U (including T ∪ nihil for nullables)
    Union(Vec<TypeExpr>),
}

#[derive(Debug)]
pub struct FuncTypeExpr {
    pub params: Vec<TypeExpr>,
    pub ret: Box<TypeExpr>,
    pub err: Option<Box<TypeExpr>>,
}

// =============================================================================
// Annotations
// =============================================================================

#[derive(Debug)]
pub struct Annotation {
    /// Parsed annotation family and payload.
    pub kind: AnnotationKind,

    /// Span of the full annotation.
    pub span: Span,
}

/// Annotation taxonomy preserved by the parser.
///
/// An annotation may describe target translation, CLI/package surfaces, access
/// control, async/cursor behavior, test selection, or raw metadata. Syntax keeps
/// those families distinct without deciding whether a specific declaration may
/// legally use them; that policy belongs to the consumer phase.
#[derive(Debug)]
pub enum AnnotationKind {
    /// Annotation as a statement with args
    Statement(AnnotationStmt),
    /// @ innatum TARGET STRING, ...
    Innatum(Vec<TargetMapping>),
    /// @ subsidia TARGET STRING, ...
    Subsidia(Vec<TargetMapping>),
    /// @ radix NAME, ...
    Radix(Vec<Ident>),
    /// @ verte NAME (mapping)
    Verte(VerteMapping),
    /// @ externa
    Externa,
    /// @ cli STRING
    Cli(CliAnnotation),
    /// @ imperium STRING
    Imperium(ImperiumAnnotation),
    /// @ optio ...
    Optio(OptioAnnotation),
    /// @ operandus ...
    Operandus(OperandusAnnotation),
    /// @ futura
    Futura,
    /// @ cursor
    Cursor,
    /// @ tag
    Tag,
    /// @ solum
    Solum,
    /// @ omitte
    Omitte,
    /// @ metior
    Metior,
    /// @ publica
    Publica,
    /// @ protecta
    Protecta,
    /// @ privata
    Privata,
}

#[derive(Debug)]
pub struct AnnotationStmt {
    /// Raw annotation name for forms not modeled as structured annotations.
    pub name: Ident,

    /// Token payload preserved for the eventual consumer.
    pub args: Vec<Token>,
}

#[derive(Debug)]
pub struct CliAnnotation {
    pub name: Symbol,
}

#[derive(Debug)]
pub struct ImperiumAnnotation {
    pub name: Symbol,
}

#[derive(Debug)]
pub struct TargetMapping {
    /// Backend target named by the annotation, such as `rs` or `ts`.
    pub target: Ident,

    /// Target-specific replacement text or symbol payload.
    pub value: Symbol,

    /// Span of this mapping entry.
    pub span: Span,
}

/// Parsed `@ verte` mapping for target-specific lowering.
///
/// The syntax layer records either a direct replacement or a template. It does
/// not expand placeholders or validate backend availability.
#[derive(Debug)]
pub struct VerteMapping {
    /// Target language or backend selector.
    pub target: Ident,

    /// Mapping payload.
    pub kind: VerteMappingKind,
}

#[derive(Debug)]
pub enum VerteMappingKind {
    Simple(Symbol),
    Template(Vec<Ident>, Symbol),
}

#[derive(Debug)]
pub struct OptioAnnotation {
    /// Program binding populated from the option.
    pub binding: Ident,

    /// Optional declared value type. Flags may omit this.
    pub ty: Option<TypeExpr>,

    /// Short option spelling.
    pub short: Option<Symbol>,

    /// Long option spelling.
    pub long: Option<Symbol>,

    /// Whether the option is a boolean flag.
    pub flag: bool,

    /// User-facing CLI help text, if supplied.
    pub description: Option<Symbol>,

    /// Whether the option applies outside one command scope.
    pub global: bool,

    /// Default value expression parsed from the annotation.
    pub default: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct OperandusAnnotation {
    /// Whether this positional operand captures the remaining arguments.
    pub rest: bool,

    /// Declared operand type.
    pub ty: TypeExpr,

    /// Program binding populated from the operand.
    pub binding: Ident,

    /// User-facing CLI help text, if supplied.
    pub description: Option<Symbol>,

    /// Whether the operand applies outside one command scope.
    pub global: bool,

    /// Default value expression parsed from the annotation.
    pub default: Option<Box<Expr>>,
}
