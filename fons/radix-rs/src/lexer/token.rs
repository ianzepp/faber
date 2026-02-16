//! Token definitions and source span tracking
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Defines the token types produced by the lexer and the span type used
//! throughout the compiler for error reporting and source mapping.
//!
//! COMPILER PHASE: Lexing (data structures)
//! INPUT: N/A (type definitions)
//! OUTPUT: Token stream consumed by parser
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Compact representation: Span uses u32 offsets (not usize) to keep tokens
//!   small on 64-bit systems
//! - Value-carrying tokens: Literals store their parsed values to avoid
//!   re-parsing in later phases
//! - Symbol indirection: Identifiers and strings reference interner symbols
//!   rather than embedding strings

// =============================================================================
// SOURCE SPANS
// =============================================================================

/// Source span tracking byte offsets in the original source.
///
/// WHY: Uses u32 instead of usize to keep spans compact. A 4GB source file
/// limit is acceptable (enforced during file loading). This saves memory and
/// makes tokens cache-friendly.
///
/// INVARIANTS:
/// -----------
/// INV-1: start <= end (enforced by Span::new in debug builds)
/// INV-2: Offsets are UTF-8 character boundaries (guaranteed by cursor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// Create a new span from byte offsets.
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "invalid span: start > end");
        Self { start, end }
    }

    /// Merge two spans into a larger span covering both.
    ///
    /// WHY: Used when building AST nodes that span multiple tokens (e.g., a
    /// binary expression covers from the start of lhs to the end of rhs).
    pub fn merge(self, other: Span) -> Span {
        Span { start: self.start.min(other.start), end: self.end.max(other.end) }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if the span is empty (zero-length).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

// =============================================================================
// TOKENS
// =============================================================================

/// A token with its kind and source location.
///
/// WHY: Pairs the token kind (what it is) with its span (where it came from)
/// for error reporting. This is the core data structure passed from lexer to
/// parser.
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

// =============================================================================
// SYMBOLS
// =============================================================================

/// Interned string handle - cheap to copy, O(1) equality.
///
/// WHY: Identifiers and string literals are duplicated frequently. Storing
/// them as Symbol (just a u32 index) instead of String saves memory and makes
/// comparisons fast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub u32);

// =============================================================================
// TOKEN KINDS
// =============================================================================

/// Token kinds - all possible token types in Faber.
///
/// WHY: Large enum covering all Faber syntax. Literals carry their parsed
/// values (Integer(i64), Float(f64)) to avoid re-parsing. Comments carry
/// symbols so formatters can preserve them.
///
/// ORGANIZATION: Grouped by category with comment headers for readability.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Identifiers and literals ===
    Ident(Symbol),
    Underscore(Symbol), // WHY: Special case for pattern matching wildcard
    Integer(i64),
    Float(f64),
    String(Symbol),
    TemplateString(Symbol),

    // === Keywords: Declarations ===
    Fixum,     // const
    Varia,     // let/var
    Functio,   // function
    Genus,     // class
    Pactum,    // interface
    Typus,     // type alias
    Ordo,      // enum
    Discretio, // tagged union
    Importa,   // import
    Probandum, // test suite
    Proba,     // test case

    // === Keywords: Modifiers ===
    Abstractus, // abstract
    Generis,    // static
    Nexum,      // bound method
    Publica,    // public
    Privata,    // private
    Protecta,   // protected
    Prae,       // override
    Ceteri,     // variadic rest
    Immutata,   // pure function
    Iacit,      // throws
    Curata,     // custom curata type
    Errata,     // custom error type
    Exitus,     // exit code
    Optiones,   // options struct

    // === Keywords: Control flow ===
    Si,       // if
    Sic,      // then (ternary)
    Sin,      // else if
    Secus,    // else
    Dum,      // while
    Itera,    // for
    Elige,    // switch
    Casu,     // case
    Ceterum,  // default
    Discerne, // pattern match
    Custodi,  // guard clauses
    Fac,      // do-while
    Ergo,     // therefore (single-statement then)

    // === Keywords: Transfer ===
    Redde,  // return
    Reddit, // return (inline)
    Rumpe,  // break
    Perge,  // continue
    Tacet,  // silent return (no value)

    // === Keywords: Error handling ===
    Tempta,  // try
    Cape,    // catch
    Demum,   // finally
    Iace,    // throw
    Mori,    // panic
    Moritor, // panic (inline)
    Adfirma, // assert

    // === Keywords: Async ===
    Futura,   // async (annotation)
    Cursor,   // generator (annotation)
    Cede,     // await/yield
    Clausura, // closure

    // === Keywords: Boolean/null ===
    Verum,  // true
    Falsum, // false
    Nihil,  // null/nil

    // === Keywords: Logical operators ===
    Et,  // and (&&)
    Aut, // or (||)
    Non, // not (!)
    Vel, // nullish coalesce (??)
    Est, // identity check (===)

    // === Keywords: Objects ===
    Ego,    // self/this
    Novum,  // new
    Finge,  // construct tagged union variant
    Sub,    // extends
    Implet, // implements

    // === Keywords: Type operations ===
    Qua,        // type cast
    Innatum,    // native FFI type
    Numeratum,  // to integer
    Fractatum,  // to float
    Textatum,   // to string
    Bivalentum, // to boolean

    // === Keywords: Bitwise ===
    Sinistratum, // left shift
    Dextratum,   // right shift

    // === Keywords: Output ===
    Scribe, // print
    Vide,   // debug print
    Mone,   // warn print

    // === Keywords: Entry points ===
    Incipit,   // main (sync)
    Incipiet,  // main (async)
    Argumenta, // command-line args
    Cura,      // resource management
    Arena,     // arena allocator
    Ad,        // HTTP endpoint

    // === Keywords: Misc ===
    Ex,        // extract/move
    De,        // borrow/from
    In,        // mutable borrow/in
    Ut,        // as/alias
    Pro,       // for (iteration)
    Omnia,     // all (exhaustive match)
    Sparge,    // spread operator
    Praefixum, // comptime
    Scriptum,  // interpolated script
    Lege,      // read input
    Lineam,    // read line
    Sed,       // regex literal

    // === Keywords: Ranges ===
    Ante,  // exclusive end range
    Usque, // inclusive end range
    Per,   // step
    Intra, // within
    Inter, // between

    // === Keywords: Collection DSL ===
    Ab,     // from (collection query)
    Ubi,    // where
    Prima,  // first
    Ultima, // last
    Summa,  // sum

    // === Keywords: Testing ===
    Praepara,    // before each
    Praeparabit, // before all
    Postpara,    // after each
    Postparabit, // after all
    Omitte,      // skip test
    Futurum,     // TODO test
    Solum,       // only this test
    Tag,         // test tag
    Temporis,    // test timeout
    Metior,      // benchmark
    Repete,      // repeat test
    Fragilis,    // flaky test retries
    Requirit,    // test requires
    SolumIn,     // only in environment

    // === Keywords: Nullability ===
    Nulla,     // is null check
    Nonnulla,  // is not null check
    Nonnihil,  // is not nil check
    Negativum, // is negative check
    Positivum, // is positive check

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
    At,        // @ (annotation marker)
    Section,   // § (section marker)

    // === Operators ===
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    Amp,      // & (bitwise AND)
    Pipe,     // | (bitwise OR)
    Caret,    // ^ (bitwise XOR)
    Tilde,    // ~ (bitwise NOT)
    Bang,     // !
    Question, // ?

    // === Comparison ===
    Eq,       // =
    EqEq,     // ==
    EqEqEq,   // === (strict equality)
    BangEq,   // !=
    BangEqEq, // !== (strict inequality)
    Lt,       // <
    Gt,       // >
    LtEq,     // <=
    GtEq,     // >=

    // === Compound assignment ===
    PlusEq,  // +=
    MinusEq, // -=
    StarEq,  // *=
    SlashEq, // /=
    AmpEq,   // &=
    PipeEq,  // |=

    // === Logical ===
    AmpAmp,   // && (logical AND)
    PipePipe, // || (logical OR)

    // === Optional chaining ===
    QuestionDot,     // ?. (optional member access)
    QuestionBracket, // ?[ (optional index)
    QuestionParen,   // ?( (optional call)
    BangDot,         // !. (non-null assertion member)
    BangBracket,     // ![ (non-null assertion index)
    BangParen,       // !( (non-null assertion call)

    // === Range ===
    DotDot,   // ‥ (exclusive range)
    Ellipsis, // … (inclusive range)

    // === Comments ===
    LineComment(Symbol),  // // or #
    BlockComment(Symbol), // /* */
    DocComment(Symbol),   // ///

    // === Special ===
    Eof,   // End of file marker
    Error, // Lexer error placeholder
}

impl TokenKind {
    /// Check if this token is a keyword.
    ///
    /// WHY: Used by parser for better error messages ("expected identifier,
    /// found keyword 'si'") and by formatters to apply keyword-specific styling.
    pub fn is_keyword(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Fixum
                | Varia
                | Functio
                | Genus
                | Pactum
                | Typus
                | Ordo
                | Discretio
                | Importa
                | Probandum
                | Proba
                | Abstractus
                | Generis
                | Nexum
                | Publica
                | Privata
                | Protecta
                | Prae
                | Ceteri
                | Immutata
                | Iacit
                | Curata
                | Errata
                | Exitus
                | Optiones
                | Si
                | Sic
                | Sin
                | Secus
                | Dum
                | Itera
                | Elige
                | Casu
                | Ceterum
                | Discerne
                | Custodi
                | Fac
                | Ergo
                | Redde
                | Reddit
                | Rumpe
                | Perge
                | Tacet
                | Tempta
                | Cape
                | Demum
                | Iace
                | Mori
                | Moritor
                | Adfirma
                | Futura
                | Cursor
                | Cede
                | Clausura
                | Verum
                | Falsum
                | Nihil
                | Et
                | Aut
                | Non
                | Vel
                | Est
                | Ego
                | Novum
                | Finge
                | Sub
                | Implet
                | Qua
                | Innatum
                | Numeratum
                | Fractatum
                | Textatum
                | Bivalentum
                | Sinistratum
                | Dextratum
                | Scribe
                | Vide
                | Mone
                | Incipit
                | Incipiet
                | Argumenta
                | Cura
                | Arena
                | Ad
                | Ex
                | De
                | In
                | Ut
                | Pro
                | Omnia
                | Sparge
                | Praefixum
                | Scriptum
                | Lege
                | Lineam
                | Sed
                | Ante
                | Usque
                | Per
                | Intra
                | Inter
                | Ab
                | Ubi
                | Prima
                | Ultima
                | Summa
                | Nulla
                | Nonnulla
                | Nonnihil
                | Negativum
                | Positivum
        )
    }

    /// Check if this token is a comment.
    ///
    /// WHY: Parsers typically skip comments, but formatters need to preserve
    /// them. This helper makes the distinction explicit.
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            TokenKind::LineComment(_) | TokenKind::BlockComment(_) | TokenKind::DocComment(_)
        )
    }
}
