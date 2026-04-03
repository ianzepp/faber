use serde::{Deserialize, Serialize};

/// Source location for error reporting.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Locus {
    pub linea: i32,
    pub columna: i32,
    pub index: i32,
}

/// Token produced by lexer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub tag: String,
    pub valor: String,
    pub locus: Locus,
}

/// Token tag constants.
pub const TOKEN_EOF: &str = "EOF";
pub const TOKEN_NEWLINE: &str = "Newline";
pub const TOKEN_IDENTIFIER: &str = "Identifier";
pub const TOKEN_NUMERUS: &str = "Numerus";
pub const TOKEN_TEXTUS: &str = "Textus";
pub const TOKEN_OPERATOR: &str = "Operator";
pub const TOKEN_PUNCTUATOR: &str = "Punctuator";
pub const TOKEN_KEYWORD: &str = "Keyword";
pub const TOKEN_COMMENT: &str = "Comment";

/// Literal species for ExprLittera.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LitteraSpecies {
    Numerus,
    Fractus,
    Textus,
    Verum,
    Falsum,
    Nihil,
}

impl Default for LitteraSpecies {
    fn default() -> Self {
        LitteraSpecies::Nihil
    }
}

/// Variable declaration species.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VariaSpecies {
    Varia,
    Fixum,
}

impl Default for VariaSpecies {
    fn default() -> Self {
        VariaSpecies::Varia
    }
}

/// Type annotations in AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum Typus {
    #[serde(rename = "Nomen")]
    Nomen { nomen: String },

    #[serde(rename = "Nullabilis")]
    Nullabilis { inner: Box<Typus> },

    #[serde(rename = "Genericus")]
    Genericus { nomen: String, args: Vec<Typus> },

    #[serde(rename = "Functio")]
    Functio {
        params: Vec<Typus>,
        returns: Option<Box<Typus>>,
    },

    #[serde(rename = "Unio")]
    Unio { members: Vec<Typus> },

    #[serde(rename = "Litteralis")]
    Litteralis { valor: String },
}

/// Expressions in AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum Expr {
    #[serde(rename = "Nomen")]
    Nomen { locus: Locus, valor: String },

    #[serde(rename = "Ego")]
    Ego { locus: Locus },

    #[serde(rename = "Littera")]
    Littera {
        locus: Locus,
        species: LitteraSpecies,
        valor: String,
    },

    #[serde(rename = "Binaria")]
    Binaria {
        locus: Locus,
        signum: String,
        sin: Box<Expr>,
        dex: Box<Expr>,
    },

    #[serde(rename = "Unaria")]
    Unaria {
        locus: Locus,
        signum: String,
        arg: Box<Expr>,
    },

    #[serde(rename = "Assignatio")]
    Assignatio {
        locus: Locus,
        signum: String,
        sin: Box<Expr>,
        dex: Box<Expr>,
    },

    #[serde(rename = "Condicio")]
    Condicio {
        locus: Locus,
        cond: Box<Expr>,
        cons: Box<Expr>,
        alt: Box<Expr>,
    },

    #[serde(rename = "Vocatio")]
    Vocatio {
        locus: Locus,
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    #[serde(rename = "Membrum")]
    Membrum {
        locus: Locus,
        obj: Box<Expr>,
        prop: Box<Expr>,
        computed: bool,
        #[serde(rename = "nonNull")]
        non_null: bool,
    },

    #[serde(rename = "Series")]
    Series { locus: Locus, elementa: Vec<Expr> },

    #[serde(rename = "Obiectum")]
    Obiectum {
        locus: Locus,
        props: Vec<ObiectumProp>,
    },

    #[serde(rename = "Clausura")]
    Clausura {
        locus: Locus,
        params: Vec<Param>,
        corpus: ClausuraCorpus,
    },

    #[serde(rename = "Novum")]
    Novum {
        locus: Locus,
        callee: Box<Expr>,
        args: Vec<Expr>,
        init: Option<Box<Expr>>,
    },

    #[serde(rename = "Cede")]
    Cede { locus: Locus, arg: Box<Expr> },

    #[serde(rename = "Qua")]
    Qua {
        locus: Locus,
        expr: Box<Expr>,
        typus: Typus,
    },

    #[serde(rename = "Innatum")]
    Innatum {
        locus: Locus,
        expr: Box<Expr>,
        typus: Typus,
    },

    #[serde(rename = "Conversio")]
    Conversio {
        locus: Locus,
        expr: Box<Expr>,
        species: String,
        fallback: Option<Box<Expr>>,
    },

    #[serde(rename = "PostfixNovum")]
    PostfixNovum {
        locus: Locus,
        expr: Box<Expr>,
        typus: Typus,
    },

    #[serde(rename = "Finge")]
    Finge {
        locus: Locus,
        variant: String,
        campi: Vec<ObiectumProp>,
        typus: Option<Typus>,
    },

    #[serde(rename = "Scriptum")]
    Scriptum {
        locus: Locus,
        template: String,
        args: Vec<Expr>,
    },

    #[serde(rename = "Ambitus")]
    Ambitus {
        locus: Locus,
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
}

impl Expr {
    pub fn locus(&self) -> Locus {
        match self {
            Expr::Nomen { locus, .. } => *locus,
            Expr::Ego { locus } => *locus,
            Expr::Littera { locus, .. } => *locus,
            Expr::Binaria { locus, .. } => *locus,
            Expr::Unaria { locus, .. } => *locus,
            Expr::Assignatio { locus, .. } => *locus,
            Expr::Condicio { locus, .. } => *locus,
            Expr::Vocatio { locus, .. } => *locus,
            Expr::Membrum { locus, .. } => *locus,
            Expr::Series { locus, .. } => *locus,
            Expr::Obiectum { locus, .. } => *locus,
            Expr::Clausura { locus, .. } => *locus,
            Expr::Novum { locus, .. } => *locus,
            Expr::Cede { locus, .. } => *locus,
            Expr::Qua { locus, .. } => *locus,
            Expr::Innatum { locus, .. } => *locus,
            Expr::Conversio { locus, .. } => *locus,
            Expr::PostfixNovum { locus, .. } => *locus,
            Expr::Finge { locus, .. } => *locus,
            Expr::Scriptum { locus, .. } => *locus,
            Expr::Ambitus { locus, .. } => *locus,
        }
    }
}

/// Clausura body can be either a statement or an expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClausuraCorpus {
    Stmt(Box<Stmt>),
    Expr(Box<Expr>),
}

/// Object literal property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObiectumProp {
    pub locus: Locus,
    pub key: Expr,
    pub valor: Expr,
    pub shorthand: bool,
    pub computed: bool,
}

/// Statements in AST.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag")]
pub enum Stmt {
    #[serde(rename = "Massa")]
    Massa { locus: Locus, corpus: Vec<Stmt> },

    #[serde(rename = "Expressia")]
    Expressia { locus: Locus, expr: Expr },

    #[serde(rename = "Varia")]
    Varia {
        locus: Locus,
        species: VariaSpecies,
        nomen: String,
        typus: Option<Typus>,
        valor: Option<Expr>,
        publica: bool,
        externa: bool,
    },

    #[serde(rename = "Functio")]
    Functio {
        locus: Locus,
        nomen: String,
        params: Vec<Param>,
        #[serde(rename = "typusReditus")]
        typus_reditus: Option<Typus>,
        corpus: Option<Box<Stmt>>,
        asynca: bool,
        publica: bool,
        generics: Vec<String>,
        externa: bool,
    },

    #[serde(rename = "Genus")]
    Genus {
        locus: Locus,
        nomen: String,
        campi: Vec<CampusDecl>,
        methodi: Vec<Stmt>,
        implet: Vec<String>,
        generics: Vec<String>,
        publica: bool,
        abstractus: bool,
    },

    #[serde(rename = "Pactum")]
    Pactum {
        locus: Locus,
        nomen: String,
        methodi: Vec<PactumMethodus>,
        generics: Vec<String>,
        publica: bool,
    },

    #[serde(rename = "Ordo")]
    Ordo {
        locus: Locus,
        nomen: String,
        membra: Vec<OrdoMembrum>,
        publica: bool,
    },

    #[serde(rename = "Discretio")]
    Discretio {
        locus: Locus,
        nomen: String,
        variantes: Vec<VariansDecl>,
        generics: Vec<String>,
        publica: bool,
    },

    #[serde(rename = "TypusAlias")]
    TypusAlias {
        locus: Locus,
        nomen: String,
        typus: Typus,
        publica: bool,
    },

    #[serde(rename = "In")]
    In {
        locus: Locus,
        expr: Expr,
        corpus: Box<Stmt>,
    },

    #[serde(rename = "Importa")]
    Importa {
        locus: Locus,
        fons: String,
        imported: Option<String>,
        local: String,
        totum: bool,
        publica: bool,
    },

    #[serde(rename = "Si")]
    Si {
        locus: Locus,
        cond: Expr,
        cons: Box<Stmt>,
        alt: Option<Box<Stmt>>,
    },

    #[serde(rename = "Dum")]
    Dum {
        locus: Locus,
        cond: Expr,
        corpus: Box<Stmt>,
    },

    #[serde(rename = "FacDum")]
    FacDum {
        locus: Locus,
        corpus: Box<Stmt>,
        cond: Expr,
    },

    #[serde(rename = "Iteratio")]
    Iteratio {
        locus: Locus,
        species: String,
        binding: String,
        iter: Expr,
        corpus: Box<Stmt>,
        asynca: bool,
    },

    #[serde(rename = "Elige")]
    Elige {
        locus: Locus,
        discrim: Expr,
        casus: Vec<EligeCasus>,
        #[serde(rename = "default_")]
        default: Option<Box<Stmt>>,
    },

    #[serde(rename = "Discerne")]
    Discerne {
        locus: Locus,
        discrim: Vec<Expr>,
        casus: Vec<DiscerneCasus>,
    },

    #[serde(rename = "Custodi")]
    Custodi {
        locus: Locus,
        clausulae: Vec<CustodiClausula>,
    },

    #[serde(rename = "Tempta")]
    Tempta {
        locus: Locus,
        corpus: Box<Stmt>,
        cape: Option<CapeClausula>,
        demum: Option<Box<Stmt>>,
    },

    #[serde(rename = "Redde")]
    Redde {
        locus: Locus,
        valor: Option<Expr>,
    },

    #[serde(rename = "Iace")]
    Iace {
        locus: Locus,
        arg: Expr,
        fatale: bool,
    },

    #[serde(rename = "Scribe")]
    Scribe {
        locus: Locus,
        gradus: String,
        args: Vec<Expr>,
    },

    #[serde(rename = "Adfirma")]
    Adfirma {
        locus: Locus,
        cond: Expr,
        msg: Option<Expr>,
    },

    #[serde(rename = "Rumpe")]
    Rumpe { locus: Locus },

    #[serde(rename = "Perge")]
    Perge { locus: Locus },

    #[serde(rename = "Incipit")]
    Incipit {
        locus: Locus,
        corpus: Box<Stmt>,
        asynca: bool,
    },

    #[serde(rename = "Probandum")]
    Probandum {
        locus: Locus,
        nomen: String,
        corpus: Vec<Stmt>,
    },

    #[serde(rename = "Proba")]
    Proba {
        locus: Locus,
        nomen: String,
        corpus: Box<Stmt>,
    },
}

impl Stmt {
    pub fn locus(&self) -> Locus {
        match self {
            Stmt::Massa { locus, .. } => *locus,
            Stmt::Expressia { locus, .. } => *locus,
            Stmt::Varia { locus, .. } => *locus,
            Stmt::Functio { locus, .. } => *locus,
            Stmt::Genus { locus, .. } => *locus,
            Stmt::Pactum { locus, .. } => *locus,
            Stmt::Ordo { locus, .. } => *locus,
            Stmt::Discretio { locus, .. } => *locus,
            Stmt::TypusAlias { locus, .. } => *locus,
            Stmt::In { locus, .. } => *locus,
            Stmt::Importa { locus, .. } => *locus,
            Stmt::Si { locus, .. } => *locus,
            Stmt::Dum { locus, .. } => *locus,
            Stmt::FacDum { locus, .. } => *locus,
            Stmt::Iteratio { locus, .. } => *locus,
            Stmt::Elige { locus, .. } => *locus,
            Stmt::Discerne { locus, .. } => *locus,
            Stmt::Custodi { locus, .. } => *locus,
            Stmt::Tempta { locus, .. } => *locus,
            Stmt::Redde { locus, .. } => *locus,
            Stmt::Iace { locus, .. } => *locus,
            Stmt::Scribe { locus, .. } => *locus,
            Stmt::Adfirma { locus, .. } => *locus,
            Stmt::Rumpe { locus } => *locus,
            Stmt::Perge { locus } => *locus,
            Stmt::Incipit { locus, .. } => *locus,
            Stmt::Probandum { locus, .. } => *locus,
            Stmt::Proba { locus, .. } => *locus,
        }
    }
}

/// Function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub locus: Locus,
    pub nomen: String,
    pub typus: Option<Typus>,
    #[serde(rename = "default_")]
    pub default: Option<Expr>,
    pub rest: bool,
    /// Ownership annotation: "ex" (consume), "de" (borrow), "in" (mutate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<String>,
}

/// Field declaration in genus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampusDecl {
    pub locus: Locus,
    pub nomen: String,
    pub typus: Option<Typus>,
    pub valor: Option<Expr>,
    pub visibilitas: String,
}

/// Protocol method declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactumMethodus {
    pub locus: Locus,
    pub nomen: String,
    pub params: Vec<Param>,
    #[serde(rename = "typusReditus")]
    pub typus_reditus: Option<Typus>,
    pub asynca: bool,
}

/// Enum member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdoMembrum {
    pub locus: Locus,
    pub nomen: String,
    pub valor: Option<String>,
}

/// Discriminated union variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariansDecl {
    pub locus: Locus,
    pub nomen: String,
    pub campi: Vec<VariansCampus>,
}

/// Field in a variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariansCampus {
    pub nomen: String,
    pub typus: Typus,
}

/// Import specifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSpec {
    pub locus: Locus,
    pub imported: String,
    pub local: String,
}

/// Case in elige (switch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligeCasus {
    pub locus: Locus,
    pub cond: Expr,
    pub corpus: Box<Stmt>,
}

/// Case in discerne (pattern match).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscerneCasus {
    pub locus: Locus,
    pub patterns: Vec<VariansPattern>,
    pub corpus: Box<Stmt>,
}

/// Pattern in discerne case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariansPattern {
    pub locus: Locus,
    pub variant: String,
    pub bindings: Vec<String>,
    pub alias: Option<String>,
    pub wildcard: bool,
}

/// Guard clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodiClausula {
    pub locus: Locus,
    pub cond: Expr,
    pub corpus: Box<Stmt>,
}

/// Catch clause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapeClausula {
    pub locus: Locus,
    pub param: String,
    pub corpus: Box<Stmt>,
}

/// Top-level compilation unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modulus {
    pub locus: Locus,
    pub corpus: Vec<Stmt>,
}
