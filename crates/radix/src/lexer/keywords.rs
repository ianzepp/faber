//! Keyword spelling registry for lexer and language-tool policy.
//!
//! The scanner's `match` table in `scan.rs` is the live tokenization surface.
//! This registry is the richer contract around that surface: it records which
//! spellings are current keywords, which are annotation or section names, which
//! are intentionally contextual, and which spellings are aliases for migration
//! or teaching purposes.
//!
//! WHY THIS EXISTS
//! ===============
//! Faber's front end is still moving some words from globally reserved tokens
//! toward parser-owned or metadata-owned contexts. Keeping that intent in a
//! structured registry lets diagnostics and explain output work from one
//! taxonomy instead of rediscovering policy from a large scanner `match`.
//!
//! INVARIANTS
//! ==========
//! - `token_kind: Some(_)` means the current lexer can emit that token in at
//!   least one source mode; `None` means the spelling is tracked metadata and
//!   should lex as an identifier today.
//! - `Alias` entries name their canonical spelling but may still lex as their
//!   own token while migration diagnostics are developed.
//! - Annotation and section spellings are registry entries, not normal-mode
//!   reserved words, unless `scan.rs` explicitly promotes them.
//! - This file describes policy; `scan.rs` remains authoritative for behavior.

use super::token::TokenKind;

/// Where a keyword spelling is valid or intended to be owned.
///
/// Scope is policy metadata, not automatic lexer behavior. It tells tools and
/// parser work whether a spelling is globally reserved, context-owned,
/// annotation-only, section-only, test-owned, or transitional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordScope {
    /// Reserved anywhere normal-mode keyword recognition runs.
    Global,

    /// Intended for one or more parser contexts, even if currently lexed as a keyword.
    Contextual(&'static [KeywordOwner]),

    /// Annotation name or annotation-owned metadata after `@`.
    Annotation,

    /// Section marker payload after `§`.
    Section,

    /// Test syntax surface owned by the test language layer.
    TestOwned,

    /// Accepted spelling whose canonical replacement is named here.
    Alias { canonical: &'static str },
}

/// Parser or tooling surface that owns a contextual keyword spelling.
///
/// These owners let diagnostics describe why a word is special without turning
/// every contextual use into a global language reservation forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordOwner {
    /// Entrypoint-level modifier or command metadata.
    EntryModifier,

    /// Function declaration modifier.
    FunctionModifier,

    /// Class-like `genus` header syntax.
    GenusHeader,

    /// Class-like `genus` member syntax.
    GenusMember,

    /// Rest binding in patterns or parameters.
    RestPattern,

    /// Spread expression syntax.
    SpreadExpression,

    /// Annotation name position.
    AnnotationName,

    /// Annotation argument or modifier position.
    AnnotationModifier,

    /// Member name position where keyword-looking words may be accepted.
    MemberIdentifier,

    /// Import visibility surface.
    ImportVisibility,

    /// Type-parameter modifier surface.
    TypeParameter,

    /// Parameter passing mode.
    ParameterMode,

    /// Import or binding alias marker.
    AliasMarker,

    /// Iteration mode keyword.
    IterationMode,

    /// Endpoint binding syntax.
    EndpointBinding,

    /// Collection query operator.
    CollectionQuery,

    /// Predicate-style operator.
    PredicateOperator,
}

/// Keyword grouping used by diagnostics, explain data, and registry consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCategory {
    /// Declaration-introducing syntax.
    Declaration,

    /// Declaration or behavior modifier.
    Modifier,

    /// Branching, looping, or pattern-control syntax.
    ControlFlow,

    /// Function or loop transfer syntax.
    Transfer,

    /// Error and assertion syntax.
    ErrorHandling,

    /// Async, generator, or closure-adjacent syntax.
    Async,

    /// Built-in literal spelling.
    Literal,

    /// Word operator.
    Operator,

    /// Object, self, inheritance, or construction syntax.
    Object,

    /// Type conversion or construction syntax.
    TypeOperation,

    /// User-visible output helper.
    Output,

    /// Program entrypoint surface.
    EntryPoint,

    /// Resource-management syntax.
    Resource,

    /// Endpoint binding syntax.
    Endpoint,

    /// Miscellaneous language surface not yet split into a narrower category.
    Misc,

    /// Range operator word.
    Range,

    /// Collection query DSL.
    CollectionDsl,

    /// Test declaration or test modifier syntax.
    Testing,

    /// Nullability or predicate helper.
    Nullability,

    /// Annotation metadata.
    Annotation,

    /// Section metadata.
    Section,
}

/// Registry row for one known keyword or keyword-like spelling.
///
/// This is a public data contract for tools that need to explain or audit the
/// language surface. It is not a generated lexer table: consumers must still
/// check `token_kind` and scope before assuming a spelling is reserved.
#[derive(Debug, Clone)]
pub struct KeywordSpec {
    /// Source spelling as users write it.
    pub text: &'static str,

    /// Token emitted by the current lexer, or `None` for metadata-only entries.
    pub token_kind: Option<TokenKind>,

    /// Ownership and reservation policy for this spelling.
    pub scope: KeywordScope,

    /// Taxonomy bucket used by diagnostics and reference tooling.
    pub category: KeywordCategory,
}

impl KeywordSpec {
    /// Return whether the live lexer currently emits a keyword token here.
    pub fn currently_lexes_as_keyword(&self) -> bool {
        self.token_kind.is_some()
    }
}

const GENUS_HEADER: &[KeywordOwner] = &[KeywordOwner::GenusHeader];
const GENUS_MEMBER: &[KeywordOwner] = &[KeywordOwner::GenusMember];
const IMPORT_VISIBILITY_OR_ANNOTATION: &[KeywordOwner] =
    &[KeywordOwner::ImportVisibility, KeywordOwner::AnnotationName];
const ANNOTATION_NAME: &[KeywordOwner] = &[KeywordOwner::AnnotationName];
const TYPE_PARAMETER: &[KeywordOwner] = &[KeywordOwner::TypeParameter];
const REST_PATTERN: &[KeywordOwner] = &[KeywordOwner::RestPattern, KeywordOwner::AnnotationModifier];
const FUNCTION_MODIFIER: &[KeywordOwner] = &[KeywordOwner::FunctionModifier];
const ENTRY_OR_FUNCTION_MODIFIER: &[KeywordOwner] = &[KeywordOwner::EntryModifier, KeywordOwner::FunctionModifier];
const PARAMETER_MODE: &[KeywordOwner] = &[KeywordOwner::ParameterMode, KeywordOwner::IterationMode];
const ALIAS_MARKER: &[KeywordOwner] = &[KeywordOwner::AliasMarker];
const ITERATION_MODE: &[KeywordOwner] = &[KeywordOwner::IterationMode, KeywordOwner::EndpointBinding];
const SPREAD_EXPRESSION: &[KeywordOwner] = &[KeywordOwner::SpreadExpression];
const COLLECTION_QUERY: &[KeywordOwner] = &[KeywordOwner::CollectionQuery];
const PREDICATE_OPERATOR: &[KeywordOwner] = &[KeywordOwner::PredicateOperator];
const ANNOTATION_MODIFIER: &[KeywordOwner] = &[KeywordOwner::AnnotationModifier];

/// Complete registry of current, contextual, alias, annotation, and section spellings.
pub static KEYWORD_SPECS: &[KeywordSpec] = &[
    KeywordSpec {
        text: "fixum",
        token_kind: Some(TokenKind::Fixum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "varia",
        token_kind: Some(TokenKind::Varia),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "functio",
        token_kind: Some(TokenKind::Functio),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "genus",
        token_kind: Some(TokenKind::Genus),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "pactum",
        token_kind: Some(TokenKind::Pactum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "typus",
        token_kind: Some(TokenKind::Typus),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "ordo",
        token_kind: Some(TokenKind::Ordo),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "discretio",
        token_kind: Some(TokenKind::Discretio),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "importa",
        token_kind: Some(TokenKind::Importa),
        scope: KeywordScope::Global,
        category: KeywordCategory::Declaration,
    },
    KeywordSpec {
        text: "probandum",
        token_kind: Some(TokenKind::Probandum),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "proba",
        token_kind: Some(TokenKind::Proba),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "abstractus",
        token_kind: Some(TokenKind::Abstractus),
        scope: KeywordScope::Contextual(GENUS_HEADER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "generis",
        token_kind: Some(TokenKind::Generis),
        scope: KeywordScope::Contextual(GENUS_MEMBER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "nexum",
        token_kind: Some(TokenKind::Nexum),
        scope: KeywordScope::Contextual(GENUS_MEMBER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "publica",
        token_kind: Some(TokenKind::Publica),
        scope: KeywordScope::Contextual(IMPORT_VISIBILITY_OR_ANNOTATION),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "privata",
        token_kind: Some(TokenKind::Privata),
        scope: KeywordScope::Contextual(IMPORT_VISIBILITY_OR_ANNOTATION),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "protecta",
        token_kind: Some(TokenKind::Protecta),
        scope: KeywordScope::Contextual(ANNOTATION_NAME),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "prae",
        token_kind: Some(TokenKind::Prae),
        scope: KeywordScope::Contextual(TYPE_PARAMETER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "ceteri",
        token_kind: Some(TokenKind::Ceteri),
        scope: KeywordScope::Contextual(REST_PATTERN),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "immutata",
        token_kind: Some(TokenKind::Immutata),
        scope: KeywordScope::Contextual(FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "iacit",
        token_kind: Some(TokenKind::Iacit),
        scope: KeywordScope::Contextual(FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "curata",
        token_kind: Some(TokenKind::Curata),
        scope: KeywordScope::Contextual(FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "errata",
        token_kind: Some(TokenKind::Errata),
        scope: KeywordScope::Contextual(FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "exitus",
        token_kind: Some(TokenKind::Exitus),
        scope: KeywordScope::Contextual(ENTRY_OR_FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "optiones",
        token_kind: Some(TokenKind::Optiones),
        scope: KeywordScope::Contextual(FUNCTION_MODIFIER),
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "sponte",
        token_kind: Some(TokenKind::Sponte),
        scope: KeywordScope::Global,
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "fixus",
        token_kind: Some(TokenKind::Fixus),
        scope: KeywordScope::Global,
        category: KeywordCategory::Modifier,
    },
    KeywordSpec {
        text: "si",
        token_kind: Some(TokenKind::Si),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "sic",
        token_kind: Some(TokenKind::Sic),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "sin",
        token_kind: Some(TokenKind::Sin),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "secus",
        token_kind: Some(TokenKind::Secus),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "dum",
        token_kind: Some(TokenKind::Dum),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "itera",
        token_kind: Some(TokenKind::Itera),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "elige",
        token_kind: Some(TokenKind::Elige),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "casu",
        token_kind: Some(TokenKind::Casu),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "ceterum",
        token_kind: Some(TokenKind::Ceterum),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "discerne",
        token_kind: Some(TokenKind::Discerne),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "custodi",
        token_kind: Some(TokenKind::Custodi),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "fac",
        token_kind: Some(TokenKind::Fac),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "ergo",
        token_kind: Some(TokenKind::Ergo),
        scope: KeywordScope::Global,
        category: KeywordCategory::ControlFlow,
    },
    KeywordSpec {
        text: "redde",
        token_kind: Some(TokenKind::Redde),
        scope: KeywordScope::Global,
        category: KeywordCategory::Transfer,
    },
    KeywordSpec {
        text: "rumpe",
        token_kind: Some(TokenKind::Rumpe),
        scope: KeywordScope::Global,
        category: KeywordCategory::Transfer,
    },
    KeywordSpec {
        text: "perge",
        token_kind: Some(TokenKind::Perge),
        scope: KeywordScope::Global,
        category: KeywordCategory::Transfer,
    },
    KeywordSpec {
        text: "tacet",
        token_kind: Some(TokenKind::Tacet),
        scope: KeywordScope::Global,
        category: KeywordCategory::Transfer,
    },
    KeywordSpec {
        text: "tempta",
        token_kind: Some(TokenKind::Tempta),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "cape",
        token_kind: Some(TokenKind::Cape),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "demum",
        token_kind: Some(TokenKind::Demum),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "iace",
        token_kind: Some(TokenKind::Iace),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "mori",
        token_kind: Some(TokenKind::Mori),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "adfirma",
        token_kind: Some(TokenKind::Adfirma),
        scope: KeywordScope::Global,
        category: KeywordCategory::ErrorHandling,
    },
    KeywordSpec {
        text: "clausura",
        token_kind: Some(TokenKind::Clausura),
        scope: KeywordScope::Global,
        category: KeywordCategory::Async,
    },
    KeywordSpec {
        text: "cede",
        token_kind: Some(TokenKind::Cede),
        scope: KeywordScope::Global,
        category: KeywordCategory::Async,
    },
    KeywordSpec {
        text: "verum",
        token_kind: Some(TokenKind::Verum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Literal,
    },
    KeywordSpec {
        text: "falsum",
        token_kind: Some(TokenKind::Falsum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Literal,
    },
    KeywordSpec {
        text: "nihil",
        token_kind: Some(TokenKind::Nihil),
        scope: KeywordScope::Global,
        category: KeywordCategory::Literal,
    },
    KeywordSpec {
        text: "et",
        token_kind: Some(TokenKind::Et),
        scope: KeywordScope::Global,
        category: KeywordCategory::Operator,
    },
    KeywordSpec {
        text: "aut",
        token_kind: Some(TokenKind::Aut),
        scope: KeywordScope::Global,
        category: KeywordCategory::Operator,
    },
    KeywordSpec {
        text: "non",
        token_kind: Some(TokenKind::Non),
        scope: KeywordScope::Global,
        category: KeywordCategory::Operator,
    },
    KeywordSpec {
        text: "vel",
        token_kind: Some(TokenKind::Vel),
        scope: KeywordScope::Global,
        category: KeywordCategory::Operator,
    },
    KeywordSpec {
        text: "est",
        token_kind: Some(TokenKind::Est),
        scope: KeywordScope::Global,
        category: KeywordCategory::Operator,
    },
    KeywordSpec {
        text: "ego",
        token_kind: Some(TokenKind::Ego),
        scope: KeywordScope::Global,
        category: KeywordCategory::Object,
    },
    KeywordSpec {
        text: "finge",
        token_kind: Some(TokenKind::Finge),
        scope: KeywordScope::Global,
        category: KeywordCategory::Object,
    },
    KeywordSpec {
        text: "sub",
        token_kind: Some(TokenKind::Sub),
        scope: KeywordScope::Contextual(GENUS_HEADER),
        category: KeywordCategory::Object,
    },
    KeywordSpec {
        text: "implet",
        token_kind: Some(TokenKind::Implet),
        scope: KeywordScope::Contextual(GENUS_HEADER),
        category: KeywordCategory::Object,
    },
    KeywordSpec {
        text: "scribe",
        token_kind: Some(TokenKind::Scribe),
        scope: KeywordScope::Alias { canonical: "nota" },
        category: KeywordCategory::Output,
    },
    KeywordSpec {
        text: "vide",
        token_kind: Some(TokenKind::Vide),
        scope: KeywordScope::Global,
        category: KeywordCategory::Output,
    },
    KeywordSpec {
        text: "mone",
        token_kind: Some(TokenKind::Mone),
        scope: KeywordScope::Global,
        category: KeywordCategory::Output,
    },
    KeywordSpec {
        text: "nota",
        token_kind: Some(TokenKind::Nota),
        scope: KeywordScope::Global,
        category: KeywordCategory::Output,
    },
    KeywordSpec {
        text: "incipit",
        token_kind: Some(TokenKind::Incipit),
        scope: KeywordScope::Global,
        category: KeywordCategory::EntryPoint,
    },
    KeywordSpec {
        text: "incipiet",
        token_kind: Some(TokenKind::Incipiet),
        scope: KeywordScope::Global,
        category: KeywordCategory::EntryPoint,
    },
    KeywordSpec {
        text: "argumenta",
        token_kind: Some(TokenKind::Argumenta),
        scope: KeywordScope::Contextual(ENTRY_OR_FUNCTION_MODIFIER),
        category: KeywordCategory::EntryPoint,
    },
    KeywordSpec {
        text: "cura",
        token_kind: Some(TokenKind::Cura),
        scope: KeywordScope::Global,
        category: KeywordCategory::Resource,
    },
    KeywordSpec {
        text: "ad",
        token_kind: Some(TokenKind::Ad),
        scope: KeywordScope::Global,
        category: KeywordCategory::Endpoint,
    },
    KeywordSpec {
        text: "ex",
        token_kind: Some(TokenKind::Ex),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "de",
        token_kind: Some(TokenKind::De),
        scope: KeywordScope::Contextual(PARAMETER_MODE),
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "in",
        token_kind: Some(TokenKind::In),
        scope: KeywordScope::Contextual(PARAMETER_MODE),
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "ut",
        token_kind: Some(TokenKind::Ut),
        scope: KeywordScope::Contextual(ALIAS_MARKER),
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "pro",
        token_kind: Some(TokenKind::Pro),
        scope: KeywordScope::Contextual(ITERATION_MODE),
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "omnia",
        token_kind: Some(TokenKind::Omnia),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "sparge",
        token_kind: Some(TokenKind::Sparge),
        scope: KeywordScope::Contextual(SPREAD_EXPRESSION),
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "praefixum",
        token_kind: Some(TokenKind::Praefixum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "scriptum",
        token_kind: Some(TokenKind::Scriptum),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "lege",
        token_kind: Some(TokenKind::Lege),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "lineam",
        token_kind: Some(TokenKind::Lineam),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "sed",
        token_kind: Some(TokenKind::Sed),
        scope: KeywordScope::Global,
        category: KeywordCategory::Misc,
    },
    KeywordSpec {
        text: "ante",
        token_kind: Some(TokenKind::Ante),
        scope: KeywordScope::Global,
        category: KeywordCategory::Range,
    },
    KeywordSpec {
        text: "usque",
        token_kind: Some(TokenKind::Usque),
        scope: KeywordScope::Global,
        category: KeywordCategory::Range,
    },
    KeywordSpec {
        text: "per",
        token_kind: Some(TokenKind::Per),
        scope: KeywordScope::Global,
        category: KeywordCategory::Range,
    },
    KeywordSpec {
        text: "intra",
        token_kind: Some(TokenKind::Intra),
        scope: KeywordScope::Global,
        category: KeywordCategory::Range,
    },
    KeywordSpec {
        text: "inter",
        token_kind: Some(TokenKind::Inter),
        scope: KeywordScope::Global,
        category: KeywordCategory::Range,
    },
    KeywordSpec {
        text: "ab",
        token_kind: Some(TokenKind::Ab),
        scope: KeywordScope::Global,
        category: KeywordCategory::CollectionDsl,
    },
    KeywordSpec {
        text: "ubi",
        token_kind: Some(TokenKind::Ubi),
        scope: KeywordScope::Contextual(COLLECTION_QUERY),
        category: KeywordCategory::CollectionDsl,
    },
    KeywordSpec {
        text: "prima",
        token_kind: Some(TokenKind::Prima),
        scope: KeywordScope::Contextual(COLLECTION_QUERY),
        category: KeywordCategory::CollectionDsl,
    },
    KeywordSpec {
        text: "ultima",
        token_kind: Some(TokenKind::Ultima),
        scope: KeywordScope::Contextual(COLLECTION_QUERY),
        category: KeywordCategory::CollectionDsl,
    },
    KeywordSpec {
        text: "summa",
        token_kind: Some(TokenKind::Summa),
        scope: KeywordScope::Contextual(COLLECTION_QUERY),
        category: KeywordCategory::CollectionDsl,
    },
    KeywordSpec {
        text: "praepara",
        token_kind: Some(TokenKind::Praepara),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "praeparabit",
        token_kind: Some(TokenKind::Praeparabit),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "postpara",
        token_kind: Some(TokenKind::Postpara),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "postparabit",
        token_kind: Some(TokenKind::Postparabit),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "omitte",
        token_kind: Some(TokenKind::Omitte),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "futurum",
        token_kind: Some(TokenKind::Futurum),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "solum",
        token_kind: Some(TokenKind::Solum),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "tag",
        token_kind: Some(TokenKind::Tag),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "temporis",
        token_kind: Some(TokenKind::Temporis),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "metior",
        token_kind: Some(TokenKind::Metior),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "repete",
        token_kind: Some(TokenKind::Repete),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "fragilis",
        token_kind: Some(TokenKind::Fragilis),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "requirit",
        token_kind: Some(TokenKind::Requirit),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "solum_in",
        token_kind: Some(TokenKind::SolumIn),
        scope: KeywordScope::TestOwned,
        category: KeywordCategory::Testing,
    },
    KeywordSpec {
        text: "nulla",
        token_kind: Some(TokenKind::Nulla),
        scope: KeywordScope::Contextual(PREDICATE_OPERATOR),
        category: KeywordCategory::Nullability,
    },
    KeywordSpec {
        text: "nonnulla",
        token_kind: Some(TokenKind::Nonnulla),
        scope: KeywordScope::Contextual(PREDICATE_OPERATOR),
        category: KeywordCategory::Nullability,
    },
    KeywordSpec {
        text: "nonnihil",
        token_kind: Some(TokenKind::Nonnihil),
        scope: KeywordScope::Contextual(PREDICATE_OPERATOR),
        category: KeywordCategory::Nullability,
    },
    KeywordSpec {
        text: "negativum",
        token_kind: Some(TokenKind::Negativum),
        scope: KeywordScope::Contextual(PREDICATE_OPERATOR),
        category: KeywordCategory::Nullability,
    },
    KeywordSpec {
        text: "positivum",
        token_kind: Some(TokenKind::Positivum),
        scope: KeywordScope::Contextual(PREDICATE_OPERATOR),
        category: KeywordCategory::Nullability,
    },
    KeywordSpec {
        text: "futura",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "cursor",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "cli",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "imperium",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "optio",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "operandus",
        token_kind: None,
        scope: KeywordScope::Annotation,
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "brevis",
        token_kind: None,
        scope: KeywordScope::Contextual(ANNOTATION_MODIFIER),
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "longum",
        token_kind: None,
        scope: KeywordScope::Contextual(ANNOTATION_MODIFIER),
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "descriptio",
        token_kind: None,
        scope: KeywordScope::Contextual(ANNOTATION_MODIFIER),
        category: KeywordCategory::Annotation,
    },
    KeywordSpec {
        text: "ubique",
        token_kind: None,
        scope: KeywordScope::Contextual(ANNOTATION_MODIFIER),
        category: KeywordCategory::Annotation,
    },
    KeywordSpec { text: "sectio", token_kind: None, scope: KeywordScope::Section, category: KeywordCategory::Section },
];

/// Return the keyword spelling registry in deterministic declaration order.
pub fn keyword_specs() -> &'static [KeywordSpec] {
    KEYWORD_SPECS
}

/// Look up registry metadata for an exact source spelling.
pub fn lookup_keyword_spec(text: &str) -> Option<&'static KeywordSpec> {
    KEYWORD_SPECS.iter().find(|spec| spec.text == text)
}
