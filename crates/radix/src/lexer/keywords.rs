//! Keyword scope metadata.
//!
//! WHY: The lexer currently reserves every normal-mode keyword globally. This
//! registry records the intended ownership of each spelling before parser
//! helpers and contextual migrations start consuming the metadata.

use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordScope {
    Global,
    Contextual(&'static [KeywordOwner]),
    Annotation,
    Section,
    TestOwned,
    Alias { canonical: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordOwner {
    EntryModifier,
    FunctionModifier,
    GenusHeader,
    GenusMember,
    RestPattern,
    SpreadExpression,
    AnnotationName,
    AnnotationModifier,
    MemberIdentifier,
    ImportVisibility,
    TypeParameter,
    ParameterMode,
    AliasMarker,
    IterationMode,
    EndpointBinding,
    CollectionQuery,
    PredicateOperator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordCategory {
    Declaration,
    Modifier,
    ControlFlow,
    Transfer,
    ErrorHandling,
    Async,
    Literal,
    Operator,
    Object,
    TypeOperation,
    Output,
    EntryPoint,
    Resource,
    Endpoint,
    Misc,
    Range,
    CollectionDsl,
    Testing,
    Nullability,
    Annotation,
    Section,
}

#[derive(Debug, Clone)]
pub struct KeywordSpec {
    pub text: &'static str,
    pub token_kind: Option<TokenKind>,
    pub scope: KeywordScope,
    pub category: KeywordCategory,
}

impl KeywordSpec {
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

pub fn keyword_specs() -> &'static [KeywordSpec] {
    KEYWORD_SPECS
}

pub fn lookup_keyword_spec(text: &str) -> Option<&'static KeywordSpec> {
    KEYWORD_SPECS.iter().find(|spec| spec.text == text)
}
