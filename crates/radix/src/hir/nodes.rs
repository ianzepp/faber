//! Data model for the high-level intermediate representation.
//!
//! These structs are the compiler's shared contract after AST lowering. They
//! keep source-level structure recognizable enough for diagnostics and
//! language-aware analysis, while removing grammar trivia that later passes
//! should not depend on. Declarations and bindings are addressed by `DefId`;
//! expression and statement nodes carry `HirId` so analysis results can attach
//! to stable nodes without mutating the tree shape.
//!
//! HIR does not mean "fully typed." A `TypeId` in a syntactic type position is
//! the semantic reference produced from source. Optional `TypeId` fields on
//! expressions, locals, and returns are deliberately left open for typecheck to
//! fill or validate. Codegen must treat missing type information as an upstream
//! analysis failure, not as an invitation to guess.
//!
//! INVARIANTS
//! ==========
//! - `DefId` values refer to declarations, parameters, locals, pattern bindings,
//!   imports, fields, methods, and other named semantic definitions.
//! - `HirId` values identify annotatable HIR nodes, especially statements and
//!   expressions.
//! - `Span` values point back to the source construct that should receive a
//!   diagnostic, even when lowering normalized the syntax around it.
//! - `TypeId` is a reference into the semantic type arena; it is not backend
//!   layout information.
//! - Variants should preserve source-level intent. Lower MIR/control-flow and
//!   target-specific lowering belong in later phases.

use crate::lexer::{Span, Symbol};
use crate::semantic::TypeId;

/// Stable identity for a named semantic definition.
///
/// Most `DefId` values come from name resolution; lowering also allocates
/// synthetic ids for locals and bindings introduced below the resolver's scope.
/// Later passes should carry these ids instead of re-resolving textual names.
/// One definition may own many HIR nodes: a function has one `DefId`, while its
/// body contains many `HirId` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);

/// Stable identity for an annotatable HIR node.
///
/// Analysis tables use this ID to attach type facts, flow facts, and diagnostics
/// to statements and expressions without making those facts fields on every HIR
/// variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId(pub u32);

/// Lowered program split into declarations and executable entry code.
///
/// HIR makes the source/package distinction explicit: declarations are always
/// top-level items, while implicit top-level execution and explicit entry forms
/// become one optional entry block.
#[derive(Debug)]
pub struct HirProgram {
    /// Top-level declarations available to later semantic and backend phases.
    pub items: Vec<HirItem>,

    /// Executable entry block from `incipit` or package-level statements.
    pub entry: Option<HirBlock>,
}

/// Top-level declaration with shared identity and diagnostic context.
///
/// The item-level `DefId` is the definition reached by paths that name this
/// declaration. The item-level `HirId` exists for analyses that attach facts to
/// the declaration node itself rather than to its body expressions.
#[derive(Debug)]
pub struct HirItem {
    /// Node identifier for item-level analysis annotations.
    pub id: HirId,

    /// Definition identifier produced by name resolution.
    pub def_id: DefId,

    /// Declaration payload.
    pub kind: HirItemKind,

    /// Source location for diagnostics about the whole declaration.
    pub span: Span,
}

/// Source-level declaration taxonomy preserved after lowering.
#[derive(Debug)]
pub enum HirItemKind {
    /// Concrete or declared function.
    Function(HirFunction),

    /// Genus-like product type with fields and methods.
    Struct(HirStruct),

    /// Sum type with named variants.
    Enum(HirEnum),

    /// Behavioral contract with method signatures but no bodies.
    Interface(HirInterface),

    /// Alias from a source name to a resolved semantic type.
    TypeAlias(HirTypeAlias),

    /// Top-level constant whose value remains an expression for typecheck.
    Const(HirConst),

    /// Import declaration retained for diagnostics and public surface tracking.
    Import(HirImport),
}

/// Function signature and optional lowered body.
///
/// Function HIR is shared by top-level functions and methods. A missing body
/// means the source declared a signature-only surface, such as an interface
/// method. A missing return type means the source did not force one and
/// typecheck must infer or validate the effective return.
#[derive(Debug)]
pub struct HirFunction {
    /// Interned source name of the callable.
    pub name: Symbol,

    /// Generic type parameters introduced by the signature.
    pub type_params: Vec<HirTypeParam>,

    /// Ordinary call parameters, in declaration order.
    pub params: Vec<HirParam>,

    /// CLI argument bundle synthesized for mounted command entrypoints.
    pub cli_args: Option<HirParam>,

    /// Return type; None until type checker infers it
    pub ret_ty: Option<TypeId>,

    /// Recoverable alternate-exit type declared with `⇥`.
    pub err_ty: Option<TypeId>,

    /// Function body; None for interface methods
    pub body: Option<HirBlock>,

    /// Whether function is async (futura)
    pub is_async: bool,

    /// Whether function is a generator (not yet supported)
    pub is_generator: bool,

    /// Structured metadata for lowered Faber test cases.
    pub test: Option<HirTestMetadata>,
}

/// Lowered test metadata attached to the generated test function item.
///
/// The function still behaves like a normal HIR function for traversal and
/// typecheck. This side channel preserves source-level test taxonomy for
/// harness/codegen decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTestMetadata {
    /// Display/test name after lowering.
    pub name: Symbol,

    /// Suite nesting path used by generated test output.
    pub suite_path: Vec<Symbol>,

    /// Source modifiers that affect scheduling or selection.
    pub modifiers: Vec<HirTestModifier>,

    /// Source span for diagnostics about the test declaration.
    pub span: Span,
}

/// Test modifier taxonomy preserved for harness selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirTestModifier {
    /// Skip marker with the source reason/label.
    Omitte(Symbol),

    /// Async/future marker with source payload.
    Futurum(Symbol),

    /// Run only this test.
    Solum,

    /// Arbitrary tag used by test selection.
    Tag(Symbol),

    /// Timeout or duration-like modifier, stored as the parsed numeric value.
    Temporis(i64),

    /// Measurement/benchmark marker.
    Metior,

    /// Repeat-count modifier.
    Repete(i64),

    /// Allowed flake/failure tolerance marker.
    Fragilis(i64),

    /// Required capability or environment marker.
    Requirit(Symbol),

    /// Suite-scoped only marker.
    SolumIn(Symbol),
}

/// Generic type parameter introduced by an item or callable signature.
#[derive(Debug)]
pub struct HirTypeParam {
    /// Definition identity for references to this type parameter.
    pub def_id: DefId,

    /// Interned source name.
    pub name: Symbol,

    /// Declaration span for diagnostics.
    pub span: Span,
}

/// Function, method, closure, or CLI parameter binding.
///
/// Parameter types are resolved during lowering because Faber parameters are
/// type-first source syntax. Call-site optionality and provider-slot markers are
/// distinct source contracts and are kept as separate flags.
#[derive(Debug)]
pub struct HirParam {
    /// Binding definition introduced by this parameter.
    pub def_id: DefId,

    /// Interned parameter name.
    pub name: Symbol,

    /// Declared parameter type.
    pub ty: TypeId,

    /// Ownership/reference mode requested by the signature.
    pub mode: HirParamMode,

    /// Voluntary / optional at call site (sourced from `sponte` in source).
    /// Kept for compatibility with existing call-checking and test mocks.
    pub optional: bool,

    /// Post-name declaration marker: the slot is voluntary (not required from provider).
    pub sponte: bool,

    /// Post-name declaration marker: the slot becomes immutable after its first value
    /// (from argument, default, or initializer).
    pub fixus: bool,

    /// Optional source-level default supplied with `vel`.
    pub default: Option<HirExpr>,

    /// Source span for diagnostics about this binding.
    pub span: Span,
}

/// Parameter passing mode lowered from source markers.
#[derive(Debug, Clone, Copy)]
pub enum HirParamMode {
    /// Value is passed by ownership/value semantics.
    Owned,

    /// Shared reference parameter.
    Ref,

    /// Mutable reference parameter.
    MutRef,

    /// Explicit move parameter.
    Move,
}

/// Product type declaration with fields, methods, and declared relationships.
///
/// HIR stores inheritance/interface references as `DefId` values so later
/// checks can reason about relationships without reopening lexical scopes.
#[derive(Debug)]
pub struct HirStruct {
    /// Interned source name.
    pub name: Symbol,

    /// Generic type parameters declared on the type.
    pub type_params: Vec<HirTypeParam>,

    /// Instance/static fields in source order.
    pub fields: Vec<HirField>,

    /// Methods declared inside the type body.
    pub methods: Vec<HirMethod>,

    /// Resolved base type, if the source declares extension.
    pub extends: Option<DefId>,

    /// Resolved interfaces this type claims to implement.
    pub implements: Vec<DefId>,
}

/// Field declaration inside a product type.
///
/// Field type is already a resolved semantic type reference. Initializer
/// expressions remain HIR because typecheck still owns compatibility and
/// default-value validation.
#[derive(Debug)]
pub struct HirField {
    /// Field definition identity used by field lookup and diagnostics.
    pub def_id: DefId,

    /// Interned field name.
    pub name: Symbol,

    /// Declared field type.
    pub ty: TypeId,

    /// Whether this field belongs to the type rather than instances.
    pub is_static: bool,

    /// Post-name declaration marker: the field is voluntary in object literals / construction.
    pub sponte: bool,

    /// Post-name declaration marker: the field is fixed after its initial value
    /// (from literal, default via `:`, or later assignment).
    pub fixus: bool,

    /// Optional source initializer/default expression.
    pub init: Option<HirExpr>,

    /// Declaration span for diagnostics.
    pub span: Span,
}

/// Method declaration paired with receiver metadata.
#[derive(Debug)]
pub struct HirMethod {
    /// Method definition identity reached by method lookup.
    pub def_id: DefId,

    /// Shared function payload for the method signature and body.
    pub func: HirFunction,

    /// Receiver mode lowered from method syntax.
    pub receiver: HirReceiver,

    /// Whole-method source span.
    pub span: Span,
}

/// Receiver mode for methods after source lowering.
#[derive(Debug, Clone, Copy)]
pub enum HirReceiver {
    /// Static method with no instance receiver.
    None,

    /// Shared receiver.
    Ref,

    /// Mutable receiver.
    MutRef,

    /// Owned receiver.
    Owned,
}

/// Sum type declaration with resolved variant definitions.
#[derive(Debug)]
pub struct HirEnum {
    /// Interned source name.
    pub name: Symbol,

    /// Generic type parameters declared on the enum.
    pub type_params: Vec<HirTypeParam>,

    /// Variants in source order.
    pub variants: Vec<HirVariant>,
}

/// Enum variant declaration.
#[derive(Debug)]
pub struct HirVariant {
    /// Variant definition identity.
    pub def_id: DefId,

    /// Interned variant name.
    pub name: Symbol,

    /// Positional or named fields carried by the variant.
    pub fields: Vec<HirVariantField>,

    /// Variant declaration span.
    pub span: Span,
}

/// Field carried by an enum variant.
#[derive(Debug)]
pub struct HirVariantField {
    /// Field name when the variant uses named payloads.
    pub name: Symbol,

    /// Declared payload type.
    pub ty: TypeId,

    /// Source span for diagnostics.
    pub span: Span,
}

/// Interface declaration whose methods define a callable contract.
///
/// Interface methods intentionally carry signatures only; implementation
/// matching is a later semantic check against these resolved parameter and
/// return type references.
#[derive(Debug)]
pub struct HirInterface {
    /// Interned interface name.
    pub name: Symbol,

    /// Generic type parameters declared on the interface.
    pub type_params: Vec<HirTypeParam>,

    /// Required method signatures.
    pub methods: Vec<HirInterfaceMethod>,
}

/// Signature required by an interface.
#[derive(Debug)]
pub struct HirInterfaceMethod {
    /// Interned method name.
    pub name: Symbol,

    /// Required parameter contract.
    pub params: Vec<HirParam>,

    /// Declared return type, or `None` when typecheck must infer/validate absence.
    pub ret_ty: Option<TypeId>,

    /// Declared recoverable alternate-exit type.
    pub err_ty: Option<TypeId>,

    /// Signature span for diagnostics.
    pub span: Span,
}

/// Source alias from a name to a resolved semantic type.
#[derive(Debug)]
pub struct HirTypeAlias {
    /// Alias name.
    pub name: Symbol,

    /// Target type reference.
    pub ty: TypeId,
}

/// Top-level constant declaration.
///
/// The value remains an expression because constant evaluation and type
/// compatibility are later-phase responsibilities.
#[derive(Debug)]
pub struct HirConst {
    /// Constant name.
    pub name: Symbol,

    /// Explicit constant type, when declared.
    pub ty: Option<TypeId>,

    /// Lowered initializer expression.
    pub value: HirExpr,
}

/// Import declaration retained after package/library resolution.
///
/// Import items introduce definitions into the local semantic surface. Keeping
/// the declaration in HIR lets later diagnostics and generated metadata refer
/// back to the source import instead of losing that boundary.
#[derive(Debug)]
pub struct HirImport {
    /// Interned import path/specifier.
    pub path: Symbol,

    /// Visibility declared for the import.
    pub visibility: crate::syntax::Visibility,

    /// Named items introduced by the import.
    pub items: Vec<HirImportItem>,
}

/// One binding introduced by an import declaration.
#[derive(Debug)]
pub struct HirImportItem {
    /// Definition identity of the imported binding in this program.
    pub def_id: DefId,

    /// Exported/source name.
    pub name: Symbol,

    /// Local alias, when the import renames the item.
    pub alias: Option<Symbol>,
}

/// Ordered statement block with an optional trailing expression.
///
/// HIR keeps expression-position blocks and statement-position blocks in one
/// shape. Typecheck decides whether the trailing expression contributes a value
/// in its context.
#[derive(Debug)]
pub struct HirBlock {
    /// Statements executed in order.
    pub stmts: Vec<HirStmt>,

    /// Optional tail expression.
    pub expr: Option<Box<HirExpr>>,

    /// Span covering the block construct.
    pub span: Span,
}

/// Statement node with stable analysis identity.
#[derive(Debug)]
pub struct HirStmt {
    /// Statement identity for analysis tables.
    pub id: HirId,

    /// Statement payload.
    pub kind: HirStmtKind,

    /// Source span for diagnostics.
    pub span: Span,
}

/// Statement taxonomy after syntax normalization.
#[derive(Debug)]
pub enum HirStmtKind {
    /// Local binding declaration.
    Local(HirLocal),

    /// Expression used for effects.
    Expr(HirExpr),

    /// Host/provider capability-call form.
    Ad(HirAd),

    /// Function return with optional value.
    Redde(Option<HirExpr>),

    /// Loop break.
    Rumpe,

    /// Loop continue.
    Perge,

    /// Explicit no-op.
    Tacet,
}

/// Local binding introduced inside a block.
///
/// The optional type distinguishes explicit annotations from inference. The
/// initializer remains an expression so typecheck can establish the final local
/// type from annotation, initializer, or both.
#[derive(Debug)]
pub struct HirLocal {
    /// Local definition identity.
    pub def_id: DefId,

    /// Interned local name.
    pub name: Symbol,

    /// Explicit local type annotation, if present.
    pub ty: Option<TypeId>,

    /// Initializer expression, if present.
    pub init: Option<HirExpr>,

    /// Whether the source declaration permits mutation.
    pub mutable: bool,
}

/// Lowered `ad` capability-call form.
///
/// HIR preserves the routing/binding shape because endpoint semantics cross
/// parser, package CLI mounting, typecheck, and backend generation. The body and
/// catch blocks remain ordinary HIR blocks once the endpoint boundary is known.
#[derive(Debug)]
pub struct HirAd {
    /// Capability path or route specifier.
    pub path: Symbol,

    /// Arguments supplied to the capability call.
    pub args: Vec<HirExpr>,

    /// Optional success binding introduced by the form.
    pub binding: Option<HirAdBinding>,

    /// Declared recoverable error-channel type.
    pub err_ty: Option<TypeId>,

    /// Success body block.
    pub body: Option<HirBlock>,

    /// Recoverable handler block.
    pub catch: Option<HirBlock>,
}

/// Binding introduced by a capability-call form.
#[derive(Debug)]
pub struct HirAdBinding {
    /// Capability-call verb taxonomy after lowering.
    pub verb: HirEndpointVerb,

    /// Declared success binding type.
    pub ty: TypeId,

    /// Binding name introduced into the endpoint body.
    pub name: Symbol,

    /// Alias used for generated or mounted surfaces.
    pub alias: Option<Symbol>,
}

/// Expression node with stable identity, optional type facts, and source span.
///
/// The `ty` slot is analysis-owned. Lowering may seed it when the expression
/// syntax carries an unavoidable type reference, but downstream consumers should
/// expect `None` until typecheck has run.
#[derive(Debug)]
pub struct HirExpr {
    /// Expression identity for analysis tables.
    pub id: HirId,

    /// Expression payload.
    pub kind: HirExprKind,

    /// Type established by typecheck, or absent before analysis.
    pub ty: Option<TypeId>,

    /// Source span for diagnostics about this expression.
    pub span: Span,
}

/// Function or method call argument.
#[derive(Debug)]
pub struct HirCallArg {
    /// Source field name for named construction forms such as `finge`.
    pub name: Option<Symbol>,

    /// Whether the source argument used `sparge`.
    pub spread: bool,

    /// Lowered argument expression.
    pub expr: HirExpr,

    /// Whole argument span.
    pub span: Span,
}

/// Recoverable error handler binding and body.
///
/// `cape` introduces a binding for the recovered value. HIR records that binding
/// as a normal definition so visitors and typecheck see the handler scope
/// without special-case name resolution.
#[derive(Debug)]
pub struct HirCape {
    /// Definition introduced for the caught value.
    pub binding_def_id: DefId,

    /// Interned binding name.
    pub binding_name: Symbol,

    /// Declared or inferred binding type.
    pub binding_ty: Option<TypeId>,

    /// Handler body.
    pub body: HirBlock,

    /// Handler span for diagnostics.
    pub span: Span,
}

/// Expression taxonomy after AST-to-HIR normalization.
///
/// Variants preserve language intent rather than backend strategy. For example,
/// optional chaining, non-null assertions, conversion, and recoverable handling
/// remain explicit here so typecheck can enforce Faber semantics before MIR or
/// codegen chooses a target representation.
#[derive(Debug)]
pub enum HirExprKind {
    /// Resolved path to a definition.
    Path(DefId),

    /// Literal value.
    Literal(HirLiteral),

    /// Contextual empty value expression: `vacua`.
    Vacua,

    /// Binary operation.
    Binary(HirBinOp, Box<HirExpr>, Box<HirExpr>),

    /// Unary operation.
    Unary(HirUnOp, Box<HirExpr>),

    /// Function call.
    Call(Box<HirExpr>, Vec<HirCallArg>),

    /// Method call normalized from receiver syntax.
    MethodCall(Box<HirExpr>, Symbol, Vec<HirCallArg>),

    /// Field access.
    Field(Box<HirExpr>, Symbol),

    /// Index access.
    Index(Box<HirExpr>, Box<HirExpr>),

    /// Optional chaining for null-safe member, index, or call access.
    OptionalChain(Box<HirExpr>, HirOptionalChainKind),

    /// Non-null assertion for member, index, or call access.
    NonNull(Box<HirExpr>, HirNonNullKind),

    /// Block expression.
    Block(HirBlock),

    /// Conditional expression or statement form.
    Si {
        cond: Box<HirExpr>,
        then_block: HirBlock,
        then_catch: Option<Box<HirCape>>,
        else_block: Option<HirBlock>,
    },

    /// Pattern matching expression.
    Discerne(Vec<HirExpr>, Vec<HirCasuArm>),

    /// Infinite loop.
    Loop(HirBlock),

    /// While loop.
    Dum(Box<HirExpr>, HirBlock),

    /// Iteration loop with a binding introduced by the loop head.
    Itera(HirIteraMode, DefId, Symbol, Box<HirExpr>, HirBlock),

    /// Range expression.
    Intervallum {
        start: Box<HirExpr>,
        end: Box<HirExpr>,
        step: Option<Box<HirExpr>>,
        kind: HirRangeKind,
    },

    /// Assignment.
    Assign(Box<HirExpr>, Box<HirExpr>),

    /// Compound assignment.
    AssignOp(HirBinOp, Box<HirExpr>, Box<HirExpr>),

    /// Array literal.
    Array(Vec<HirArrayElement>),

    /// Struct literal whose type name is already resolved.
    Struct(DefId, Vec<(Symbol, HirExpr)>),

    /// Tuple expression, including multiple-return carriers.
    Tuple(Vec<HirExpr>),

    /// Diagnostic output expression.
    Scribe(HirScribeKind, Vec<HirExpr>),

    /// String interpolation expression (`scriptum`).
    Scriptum(Symbol, Vec<HirExpr>),

    /// Assertion expression.
    Adfirma(Box<HirExpr>, Option<Box<HirExpr>>),

    /// Unrecoverable panic expression.
    Panic(Box<HirExpr>),

    /// Recoverable throw expression (`iace`)
    Throw(Box<HirExpr>),

    /// Structured local recoverable-error handler.
    Handled { body: HirBlock, catch: Box<HirCape> },

    /// try/catch/finally expression.
    Tempta {
        body: HirBlock,
        catch: Option<HirBlock>,
        finally: Option<HirBlock>,
    },

    /// Closure with parameters, optional return/error types, and expression body.
    Clausura(Vec<HirParam>, Option<TypeId>, Option<TypeId>, Box<HirExpr>),

    /// Await expression.
    Cede(Box<HirExpr>),

    /// Static type-ascription expression (via ∷).
    ///
    /// Typed constructors and object literals may still lower through this
    /// internal carrier so backends can reuse target-shaped aggregate logic.
    Verte {
        source: Box<HirExpr>,
        target: TypeId,

        /// Extracted object fields for map/struct construction.
        /// Present when source is an object literal being constructed into a Map or Struct.
        entries: Option<Vec<HirObjectField>>,
    },

    /// Runtime value conversion (`⇒ target`).
    ///
    /// Unlike Verte (compile-time cast), this performs actual parsing/conversion
    /// and supports fallback values via `vel`.
    Conversio {
        source: Box<HirExpr>,
        target: TypeId,

        /// Codegen hint parameters (e.g., `i32`, `Hex` in `⇒ numerus<i32, Hex>`).
        /// Stored as raw symbols because these are target-specific hints, not Faber types.
        params: Vec<Symbol>,
        fallback: Option<Box<HirExpr>>,
    },

    /// Reference expression.
    Ref(HirRefKind, Box<HirExpr>),

    /// Dereference expression.
    Deref(Box<HirExpr>),

    /// Error placeholder that keeps the tree traversable after a diagnostic.
    Error,
}

/// Diagnostic output channel selected by source syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirScribeKind {
    Nota,
    Vide,
    Mone,
    Scribe,
}

/// Array literal element, including spread syntax.
#[derive(Debug)]
pub enum HirArrayElement {
    /// Ordinary element expression.
    Expr(HirExpr),

    /// Spread element expression.
    Spread(HirExpr),
}

/// Object/map construction field used by literals and conversion forms.
///
/// `value: None` represents shorthand forms where the key also names the value
/// binding; typecheck is responsible for validating that lookup.
#[derive(Debug)]
pub struct HirObjectField {
    /// Key syntax preserved for diagnostics and codegen.
    pub key: HirObjectKey,

    /// Explicit value expression, or absent for shorthand.
    pub value: Option<HirExpr>,
}

/// Object key taxonomy after lowering.
#[derive(Debug)]
pub enum HirObjectKey {
    /// Identifier key.
    Ident(Symbol),

    /// String literal key.
    String(Symbol),

    /// Computed key expression.
    Computed(HirExpr),

    /// Spread object expression.
    Spread(HirExpr),
}

/// One optional-chain operation after the base expression.
#[derive(Debug)]
pub enum HirOptionalChainKind {
    /// Null-safe member access.
    Member(Symbol),

    /// Null-safe index access.
    Index(Box<HirExpr>),

    /// Null-safe call.
    Call(Vec<HirCallArg>),
}

/// One non-null assertion operation after the base expression.
#[derive(Debug)]
pub enum HirNonNullKind {
    /// Assert then access a member.
    Member(Symbol),

    /// Assert then index.
    Index(Box<HirExpr>),

    /// Assert then call.
    Call(Vec<HirCallArg>),
}

/// Iteration source mode lowered from `itera` syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirIteraMode {
    /// Iterate values from a collection.
    Ex,

    /// Iterate entries/keys from an object-like source.
    De,

    /// Iterate numeric/progressive ranges.
    Pro,
}

/// Range endpoint inclusion policy.
#[derive(Debug, Clone, Copy)]
pub enum HirRangeKind {
    /// End is excluded.
    Exclusive,

    /// End is included.
    Inclusive,
}

/// Capability-call verb taxonomy for `ad` forms.
#[derive(Debug, Clone, Copy)]
pub enum HirEndpointVerb {
    Fit,
    Fiet,
    Fiunt,
    Fient,
}

/// Literal values preserved directly in HIR.
#[derive(Debug)]
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    String(Symbol),
    Regex(Symbol, Option<Symbol>),
    Bool(bool),
    Nil,
}

/// Binary operator taxonomy after lexical/operator resolution.
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

/// Unary operator taxonomy after lexical/operator resolution.
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

/// Reference expression mode.
#[derive(Debug, Clone, Copy)]
pub enum HirRefKind {
    /// Shared borrow/reference.
    Shared,

    /// Mutable borrow/reference.
    Mutable,
}

/// One `casu` arm in a `discerne` expression.
///
/// Patterns introduce definitions and are therefore part of the HIR semantic
/// surface, not parser-only syntax.
#[derive(Debug)]
pub struct HirCasuArm {
    /// Accepted patterns for this arm.
    pub patterns: Vec<HirPattern>,

    /// Optional guard expression.
    pub guard: Option<HirExpr>,

    /// Arm body expression.
    pub body: HirExpr,

    /// Source span for diagnostics about this arm.
    pub span: Span,
}

/// Pattern taxonomy for `discerne` and related binding positions.
#[derive(Debug)]
pub enum HirPattern {
    /// Wildcard pattern that binds nothing.
    Wildcard,

    /// New binding introduced by the pattern.
    Binding(DefId, Symbol),

    /// Alias binding layered over another pattern.
    Alias(DefId, Symbol, Box<HirPattern>),

    /// Resolved enum/variant pattern with nested subpatterns.
    Variant(DefId, Vec<HirPattern>),

    /// Literal pattern.
    Literal(HirLiteral),
}
