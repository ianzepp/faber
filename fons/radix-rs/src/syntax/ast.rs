//! Abstract Syntax Tree node definitions

use crate::lexer::{Span, Symbol, Token};

/// Unique node identifier
pub type NodeId = u32;

/// Root of the AST
#[derive(Debug)]
pub struct Program {
    pub directives: Vec<DirectiveDecl>,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

// =============================================================================
// Statements
// =============================================================================

#[derive(Debug)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
    pub annotations: Vec<Annotation>,
}

pub type AnnotatedStmt = Stmt;

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
    /// Try/catch: tempta
    Tempta(TemptaStmt),
    /// Assert: adfirma
    Adfirma(AdfirmaStmt),
    /// Output: scribe/vide/mone
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
    pub mutability: Mutability,
    pub is_await: bool,
    pub ty: Option<TypeExpr>,
    pub binding: BindingPattern,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug)]
pub enum BindingPattern {
    Ident(Ident),
    Wildcard(Span),
    Array {
        elements: Vec<BindingPattern>,
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
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub modifiers: Vec<FuncModifier>,
    pub ret: Option<TypeExpr>,
    pub body: Option<BlockStmt>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
pub struct TypeParam {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct Param {
    pub optional: bool,  // si
    pub mode: ParamMode, // de/in/ex
    pub rest: bool,      // ceteri
    pub ty: TypeExpr,
    pub name: Ident,
    pub alias: Option<Ident>, // ut NAME
    pub default: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParamMode {
    #[default]
    Owned, // (none) - take ownership
    Ref,    // de - borrow
    MutRef, // in - mutable borrow
    Move,   // ex - explicit move
}

#[derive(Debug)]
pub enum FuncModifier {
    Curata(Ident),
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
    pub path: Symbol,
    pub visibility: Visibility,
    pub kind: ImportKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug)]
pub enum ImportKind {
    Named { name: Ident, alias: Option<Ident> },
    Wildcard { alias: Ident },
}

#[derive(Debug)]
pub struct DirectiveDecl {
    pub name: Ident,
    pub args: Vec<DirectiveArg>,
    pub span: Span,
}

#[derive(Debug)]
pub enum DirectiveArg {
    String(Symbol),
    Ident(Ident),
}

#[derive(Debug)]
pub struct ProbandumDecl {
    pub name: Symbol,
    pub body: ProbandumBody,
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

#[derive(Debug)]
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
    InlineReturn(InlineReturn),
}

#[derive(Debug)]
pub enum InlineReturn {
    Reddit(Box<Expr>),
    Iacit(Box<Expr>),
    Moritor(Box<Expr>),
    Tacet,
}

#[derive(Debug)]
pub enum SecusClause {
    Sin(Box<SiStmt>),
    Block(BlockStmt),
    Stmt(Box<Stmt>),
    InlineReturn(InlineReturn),
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
    pub exhaustive: bool, // omnia
    pub subjects: Vec<Expr>,
    pub arms: Vec<CasuArm>,
    pub default: Option<CeterumDefault>,
}

#[derive(Debug)]
pub struct CasuArm {
    pub patterns: Vec<Pattern>,
    pub body: IfBody,
    pub span: Span,
}

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
    Alias(Ident), // ut NAME
    Bindings {
        mutability: Mutability,
        names: Vec<Ident>,
    },
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
}

#[derive(Debug)]
pub struct IncipitStmt {
    pub is_async: bool, // incipiet vs incipit
    pub body: IfBody,
    pub args: Option<Ident>,
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
    pub kind: Option<CuraKind>,
    pub init: Option<Box<Expr>>,
    pub mutability: Mutability,
    pub ty: Option<TypeExpr>,
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
    Fit,   // sync singular
    Fiet,  // async singular
    Fiunt, // sync plural
    Fient, // async plural
}

// =============================================================================
// Expressions
// =============================================================================

#[derive(Debug)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExprKind {
    /// Identifier
    Ident(Ident),
    /// Literal value
    Literal(Literal),
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
    /// Type cast: qua
    Qua(QuaExpr),
    /// Native construction: innatum
    Innatum(InnatumExpr),
    /// New instance: novum
    Novum(NovumExpr),
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
    /// Range: x..y, x ante y, x usque y
    Intervallum(IntervallumExpr),
    /// Collection DSL: ab
    Ab(AbExpr),
    /// Type conversion: numeratum, fractatum, textatum, bivalentum
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
    pub name: Symbol,
    pub span: Span,
}

#[derive(Debug)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(Symbol),
    TemplateString(Symbol),
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
    BitNot,    // ~
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
pub struct QuaExpr {
    pub expr: Box<Expr>,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct InnatumExpr {
    pub expr: Box<Expr>,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct NovumExpr {
    pub ty: Ident,
    pub args: Option<Vec<Argument>>,
    pub init: Option<NovumInit>,
}

#[derive(Debug)]
pub enum NovumInit {
    Object(Vec<ObjectField>),
    From(Box<Expr>), // de
}

#[derive(Debug)]
pub struct FingeExpr {
    pub variant: Ident,
    pub fields: Vec<FingeFieldInit>,
    pub cast: Option<Ident>, // qua
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
    Exclusive, // ..
    Ante,      // ante (exclusive end)
    Usque,     // usque (inclusive end)
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
    pub kind: ConversioKind,
    pub type_params: Vec<TypeExpr>,
    pub fallback: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConversioKind {
    Numeratum,
    Fractatum,
    Textatum,
    Bivalentum,
}

#[derive(Debug)]
pub struct ScriptumExpr {
    pub template: Symbol,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct LegeExpr {
    pub line: bool, // lineam
    pub span: Span,
}

#[derive(Debug)]
pub struct SedExpr {
    pub pattern: Symbol,
    pub flags: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug)]
/// Comptime expression: praefixum(expr) - forces compile-time evaluation
/// Maps to Zig's `comptime`
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
    pub nullable: bool,         // si
    pub mode: Option<TypeMode>, // de/in
    pub kind: TypeExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeMode {
    Ref,    // de
    MutRef, // in
}

#[derive(Debug)]
pub enum TypeExprKind {
    /// Named type with optional type parameters
    Named(Ident, Vec<TypeExpr>),
    /// Array type: T[]
    Array(Box<TypeExpr>),
    /// Function type: (A, B) -> C
    Func(FuncTypeExpr),
}

#[derive(Debug)]
pub struct FuncTypeExpr {
    pub params: Vec<TypeExpr>,
    pub ret: Box<TypeExpr>,
}

// =============================================================================
// Annotations
// =============================================================================

#[derive(Debug)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub span: Span,
}

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
    /// @ imperium / @ cli
    Cli,
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
    pub name: Ident,
    pub args: Vec<Token>,
}

#[derive(Debug)]
pub struct TargetMapping {
    pub target: Ident,
    pub value: Symbol,
    pub span: Span,
}

#[derive(Debug)]
pub struct VerteMapping {
    pub target: Ident,
    pub kind: VerteMappingKind,
}

#[derive(Debug)]
pub enum VerteMappingKind {
    Simple(Symbol),
    Template(Vec<Ident>, Symbol),
}

#[derive(Debug)]
pub struct OptioAnnotation {
    pub name: Ident,
    pub short: Option<Symbol>,
    pub long: Option<Symbol>,
    pub flag: bool,
    pub description: Option<Symbol>,
}

#[derive(Debug)]
pub struct OperandusAnnotation {
    pub rest: bool,
    pub ty: Ident,
    pub name: Ident,
    pub description: Option<Symbol>,
}
