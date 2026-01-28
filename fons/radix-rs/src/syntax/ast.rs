//! Abstract Syntax Tree node definitions

use crate::lexer::{Span, Symbol};

/// Unique node identifier
pub type NodeId = u32;

/// Root of the AST
#[derive(Debug)]
pub struct Program {
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
}

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
    /// Directive: § ...
    Directive(DirectiveDecl),
    /// Test suite: probandum
    Test(TestDecl),
    /// Block: { ... }
    Block(BlockStmt),
    /// Expression statement
    Expr(ExprStmt),
    /// If statement
    If(IfStmt),
    /// While loop: dum
    While(WhileStmt),
    /// For loop: itera
    Iter(IterStmt),
    /// Switch: elige
    Switch(SwitchStmt),
    /// Pattern match: discerne
    Match(MatchStmt),
    /// Guard: custodi
    Guard(GuardStmt),
    /// Scoped block: fac
    Fac(FacStmt),
    /// Return: redde
    Return(ReturnStmt),
    /// Break: rumpe
    Break(BreakStmt),
    /// Continue: perge
    Continue(ContinueStmt),
    /// Throw: iace
    Throw(ThrowStmt),
    /// Panic: mori
    Panic(PanicStmt),
    /// Try/catch: tempta
    Try(TryStmt),
    /// Assert: adfirma
    Assert(AssertStmt),
    /// Output: scribe/vide/mone
    Output(OutputStmt),
    /// Entry point: incipit/incipiet
    Entry(EntryStmt),
    /// Resource management: cura
    Resource(ResourceStmt),
    /// Endpoint: ad
    Endpoint(EndpointStmt),
}

// =============================================================================
// Declarations
// =============================================================================

#[derive(Debug)]
pub struct VarDecl {
    pub mutability: Mutability,
    pub is_await: bool,
    pub ty: Option<TypeExpr>,
    pub name: Ident,
    pub init: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable, // fixum, figendum
    Mutable,   // varia, variandum
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
    pub optional: bool,       // si
    pub mode: ParamMode,      // de/in/ex
    pub rest: bool,           // ceteri
    pub ty: TypeExpr,
    pub name: Ident,
    pub alias: Option<Ident>, // ut NAME
    pub default: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParamMode {
    #[default]
    Owned,    // (none) - take ownership
    Ref,      // de - borrow
    MutRef,   // in - mutable borrow
    Move,     // ex - explicit move
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
    pub is_static: bool,   // generis
    pub is_bound: bool,    // nexum
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
pub struct TestDecl {
    pub name: Symbol,
    pub body: TestBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct TestBody {
    pub setup: Vec<SetupBlock>,
    pub tests: Vec<TestCase>,
    pub nested: Vec<TestDecl>,
}

#[derive(Debug)]
pub struct SetupBlock {
    pub kind: SetupKind,
    pub all: bool,
    pub body: BlockStmt,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum SetupKind {
    Before,      // praepara
    BeforeAll,   // praeparabit
    After,       // postpara
    AfterAll,    // postparabit
}

#[derive(Debug)]
pub struct TestCase {
    pub modifiers: Vec<TestModifier>,
    pub name: Symbol,
    pub body: BlockStmt,
    pub span: Span,
}

#[derive(Debug)]
pub enum TestModifier {
    Skip(Symbol),
    Future(Symbol),
    Only,
    Tag(Symbol),
    Timeout(i64),
    Bench,
    Repeat(i64),
    Flaky(i64),
    Requires(Symbol),
    OnlyIn(Symbol),
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
pub struct IfStmt {
    pub cond: Box<Expr>,
    pub then: IfBody,
    pub catch: Option<CatchClause>,
    pub else_: Option<ElseClause>,
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
pub enum ElseClause {
    If(Box<IfStmt>),
    Block(BlockStmt),
    Stmt(Box<Stmt>),
    InlineReturn(InlineReturn),
}

#[derive(Debug)]
pub struct WhileStmt {
    pub cond: Box<Expr>,
    pub body: IfBody,
    pub catch: Option<CatchClause>,
}

#[derive(Debug)]
pub struct IterStmt {
    pub mode: IterMode,
    pub iterable: Box<Expr>,
    pub mutability: Mutability,
    pub binding: Ident,
    pub body: IfBody,
    pub catch: Option<CatchClause>,
}

#[derive(Debug, Clone, Copy)]
pub enum IterMode {
    Values, // ex
    Keys,   // de
    Range,  // pro
}

#[derive(Debug)]
pub struct SwitchStmt {
    pub expr: Box<Expr>,
    pub cases: Vec<SwitchCase>,
    pub default: Option<SwitchDefault>,
    pub catch: Option<CatchClause>,
}

#[derive(Debug)]
pub struct SwitchCase {
    pub value: Box<Expr>,
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct SwitchDefault {
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct MatchStmt {
    pub exhaustive: bool, // omnia
    pub subjects: Vec<Expr>,
    pub arms: Vec<MatchArm>,
    pub default: Option<SwitchDefault>,
}

#[derive(Debug)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub enum Pattern {
    Wildcard(Span),
    Ident(Ident, Option<PatternBind>),
}

#[derive(Debug)]
pub enum PatternBind {
    Alias(Ident),             // ut NAME
    Destructure(Vec<Ident>),  // pro NAME, NAME (deprecated)
}

#[derive(Debug)]
pub struct GuardStmt {
    pub clauses: Vec<GuardClause>,
}

#[derive(Debug)]
pub struct GuardClause {
    pub cond: Box<Expr>,
    pub body: IfBody,
    pub span: Span,
}

#[derive(Debug)]
pub struct FacStmt {
    pub body: BlockStmt,
    pub catch: Option<CatchClause>,
    pub while_: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Debug)]
pub struct ContinueStmt {
    pub span: Span,
}

#[derive(Debug)]
pub struct ThrowStmt {
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct PanicStmt {
    pub value: Box<Expr>,
}

#[derive(Debug)]
pub struct TryStmt {
    pub body: BlockStmt,
    pub catch: Option<CatchClause>,
    pub finally: Option<BlockStmt>,
}

#[derive(Debug)]
pub struct CatchClause {
    pub binding: Ident,
    pub body: BlockStmt,
    pub span: Span,
}

#[derive(Debug)]
pub struct AssertStmt {
    pub cond: Box<Expr>,
    pub message: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct OutputStmt {
    pub kind: OutputKind,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    Log,   // scribe
    Debug, // vide
    Warn,  // mone
}

#[derive(Debug)]
pub struct EntryStmt {
    pub is_async: bool, // incipiet vs incipit
    pub body: IfBody,
}

#[derive(Debug)]
pub struct ResourceStmt {
    pub kind: Option<ResourceKind>,
    pub init: Option<Box<Expr>>,
    pub mutability: Mutability,
    pub ty: Option<TypeExpr>,
    pub binding: Ident,
    pub body: BlockStmt,
    pub catch: Option<CatchClause>,
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceKind {
    Arena,
    Page,
}

#[derive(Debug)]
pub struct EndpointStmt {
    pub path: Symbol,
    pub args: Vec<Argument>,
    pub binding: Option<EndpointBinding>,
    pub body: Option<BlockStmt>,
    pub catch: Option<CatchClause>,
}

#[derive(Debug)]
pub struct EndpointBinding {
    pub verb: EndpointVerb,
    pub ty: Option<TypeExpr>,
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug, Clone, Copy)]
pub enum EndpointVerb {
    Fit,    // sync singular
    Fiet,   // async singular
    Fiunt,  // sync plural
    Fient,  // async plural
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
    Cast(CastExpr),
    /// Native construction: innatum
    Construct(ConstructExpr),
    /// New instance: novum
    New(NewExpr),
    /// Variant construction: finge
    Variant(VariantExpr),
    /// Closure: clausura
    Closure(ClosureExpr),
    /// Await: cede
    Await(AwaitExpr),
    /// Array literal
    Array(ArrayExpr),
    /// Object literal
    Object(ObjectExpr),
    /// Range: x..y, x ante y, x usque y
    Range(RangeExpr),
    /// Collection DSL: ab
    Collection(CollectionExpr),
    /// Type conversion: numeratum, fractatum, textatum, bivalentum
    Conversion(ConversionExpr),
    /// Interpolated script: scriptum
    Script(ScriptExpr),
    /// Read input: lege
    Read(ReadExpr),
    /// Regex literal: sed
    Regex(RegexExpr),
    /// Prefix expression: praefixum
    Prefix(PrefixExpr),
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
pub struct CastExpr {
    pub expr: Box<Expr>,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct ConstructExpr {
    pub expr: Box<Expr>,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub struct NewExpr {
    pub ty: Ident,
    pub args: Option<Vec<Argument>>,
    pub init: Option<NewInit>,
}

#[derive(Debug)]
pub enum NewInit {
    Object(Vec<ObjectField>),
    From(Box<Expr>), // de
}

#[derive(Debug)]
pub struct VariantExpr {
    pub variant: Ident,
    pub fields: Vec<VariantFieldInit>,
    pub cast: Option<Ident>, // qua
}

#[derive(Debug)]
pub struct VariantFieldInit {
    pub name: Ident,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ClosureExpr {
    pub params: Vec<ClosureParam>,
    pub ret: Option<TypeExpr>,
    pub body: ClosureBody,
}

#[derive(Debug)]
pub struct ClosureParam {
    pub ty: TypeExpr,
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub enum ClosureBody {
    Expr(Box<Expr>),
    Block(BlockStmt),
}

#[derive(Debug)]
pub struct AwaitExpr {
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
pub struct RangeExpr {
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
pub struct CollectionExpr {
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
    First,  // prima
    Last,   // ultima
    Sum,    // summa
}

#[derive(Debug)]
pub struct ConversionExpr {
    pub expr: Box<Expr>,
    pub kind: ConversionKind,
    pub type_params: Vec<TypeExpr>,
    pub fallback: Option<Box<Expr>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConversionKind {
    ToInt,    // numeratum
    ToFloat,  // fractatum
    ToString, // textatum
    ToBool,   // bivalentum
}

#[derive(Debug)]
pub struct ScriptExpr {
    pub template: Symbol,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct ReadExpr {
    pub line: bool, // lineam
    pub span: Span,
}

#[derive(Debug)]
pub struct RegexExpr {
    pub pattern: Symbol,
    pub flags: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug)]
pub struct PrefixExpr {
    pub body: PrefixBody,
}

#[derive(Debug)]
pub enum PrefixBody {
    Block(BlockStmt),
    Expr(Box<Expr>),
}

// =============================================================================
// Types
// =============================================================================

#[derive(Debug)]
pub struct TypeExpr {
    pub nullable: bool,    // si
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
    /// Simple annotation: @ NAME+
    Simple(Vec<Ident>),
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
