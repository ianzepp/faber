//! Token definitions

/// Source span (byte offsets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A token with its kind and source location
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Interned string handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub u32);

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Identifiers and literals ===
    Ident(Symbol),
    Integer(i64),
    Float(f64),
    String(Symbol),
    TemplateString(Symbol),

    // === Keywords: Declarations ===
    Fixum,
    Varia,
    Functio,
    Genus,
    Pactum,
    Typus,
    Ordo,
    Discretio,
    Importa,
    Probandum,
    Proba,

    // === Keywords: Modifiers ===
    Abstractus,
    Generis,
    Nexum,
    Publica,
    Privata,
    Protecta,
    Prae,
    Ceteri,
    Immutata,
    Iacit,
    Curata,
    Errata,
    Exitus,
    Optiones,

    // === Keywords: Control flow ===
    Si,
    Sic,
    Sin,
    Secus,
    Dum,
    Itera,
    Elige,
    Casu,
    Ceterum,
    Discerne,
    Custodi,
    Fac,
    Ergo,

    // === Keywords: Transfer ===
    Redde,
    Reddit,
    Rumpe,
    Perge,
    Tacet,

    // === Keywords: Error handling ===
    Tempta,
    Cape,
    Demum,
    Iace,
    Mori,
    Moritor,
    Adfirma,

    // === Keywords: Async ===
    Futura,
    Cursor,
    Cede,
    Clausura,

    // === Keywords: Boolean/null ===
    Verum,
    Falsum,
    Nihil,

    // === Keywords: Logical operators ===
    Et,
    Aut,
    Non,
    Vel,
    Est,

    // === Keywords: Objects ===
    Ego,
    Novum,
    Finge,
    Sub,
    Implet,

    // === Keywords: Type operations ===
    Qua,
    Innatum,
    Numeratum,
    Fractatum,
    Textatum,
    Bivalentum,

    // === Keywords: Bitwise ===
    Sinistratum,
    Dextratum,

    // === Keywords: Output ===
    Scribe,
    Vide,
    Mone,

    // === Keywords: Entry points ===
    Incipit,
    Incipiet,
    Cura,
    Arena,
    Ad,

    // === Keywords: Misc ===
    Ex,
    De,
    In,
    Ut,
    Pro,
    Omnia,
    Sparge,
    Praefixum,
    Scriptum,
    Lege,
    Lineam,
    Sed,

    // === Keywords: Ranges ===
    Ante,
    Usque,
    Per,
    Intra,
    Inter,

    // === Keywords: Collection DSL ===
    Ab,
    Ubi,
    Prima,
    Ultima,
    Summa,

    // === Keywords: Testing ===
    Praepara,
    Praeparabit,
    Postpara,
    Postparabit,
    Omitte,
    Futurum,
    Solum,
    Tag,
    Temporis,
    Metior,
    Repete,
    Fragilis,
    Requirit,
    SolumIn,

    // === Keywords: Nullability ===
    Nulla,
    Nonnulla,
    Nonnihil,
    Negativum,
    Positivum,

    // === Punctuation ===
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Colon,     // :
    Semicolon, // ;
    Dot,       // .
    Arrow,     // ->
    At,        // @
    Section,   // §

    // === Operators ===
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    Amp,       // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    Bang,      // !
    Question,  // ?

    // === Comparison ===
    Eq,        // =
    EqEq,      // ==
    EqEqEq,    // ===
    BangEq,    // !=
    BangEqEq,  // !==
    Lt,        // <
    Gt,        // >
    LtEq,      // <=
    GtEq,      // >=

    // === Compound assignment ===
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    AmpEq,     // &=
    PipeEq,    // |=

    // === Logical ===
    AmpAmp,    // &&
    PipePipe,  // ||

    // === Optional chaining ===
    QuestionDot,     // ?.
    QuestionBracket, // ?[
    QuestionParen,   // ?(
    BangDot,         // !.
    BangBracket,     // ![
    BangParen,       // !(

    // === Range ===
    DotDot,    // ..

    // === Comments ===
    LineComment(Symbol),
    BlockComment(Symbol),
    DocComment(Symbol),

    // === Special ===
    Eof,
    Error,
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Fixum | Varia | Functio | Genus | Pactum
                | Typus | Ordo | Discretio | Importa | Probandum | Proba
                | Abstractus | Generis | Nexum | Publica | Privata | Protecta
                | Prae | Ceteri | Immutata | Iacit | Curata | Errata | Exitus | Optiones
                | Si | Sic | Sin | Secus | Dum | Itera | Elige | Casu | Ceterum
                | Discerne | Custodi | Fac | Ergo | Redde | Reddit | Rumpe | Perge | Tacet
                | Tempta | Cape | Demum | Iace | Mori | Moritor | Adfirma
                | Futura | Cursor | Cede | Clausura | Verum | Falsum | Nihil
                | Et | Aut | Non | Vel | Est | Ego | Novum | Finge | Sub | Implet
                | Qua | Innatum | Numeratum | Fractatum | Textatum | Bivalentum
                | Sinistratum | Dextratum | Scribe | Vide | Mone
                | Incipit | Incipiet | Cura | Arena | Ad
                | Ex | De | In | Ut | Pro | Omnia | Sparge
                | Praefixum | Scriptum | Lege | Lineam | Sed
                | Ante | Usque | Per | Intra | Inter
                | Ab | Ubi | Prima | Ultima | Summa
                | Nulla | Nonnulla | Nonnihil | Negativum | Positivum
        )
    }

    /// Check if this token is a comment
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            TokenKind::LineComment(_) | TokenKind::BlockComment(_) | TokenKind::DocComment(_)
        )
    }
}
