//! Token and span data contracts shared after lexing.
//!
//! This module is the stable data boundary between the lexer and parser. It
//! defines compact byte spans, interned symbol handles, and the complete set of
//! token kinds the parser can observe. Scanner policy lives in `scan.rs`;
//! keyword taxonomy and alias metadata live in `keywords.rs`; this file keeps
//! the transport representation explicit and cheap to clone.
//!
//! INVARIANTS
//! ==========
//! - Spans are byte offsets into the original UTF-8 source, not character
//!   counts and not offsets into normalized strings.
//! - `Symbol` values are indexes into the [`Interner`](super::Interner) that
//!   came from the same lexer run.
//! - Literal tokens carry parsed numeric values; source spelling is recovered
//!   through spans when diagnostics or formatters need it.
//!
//! COMPATIBILITY
//! =============
//! Some variants are reserved for parser or internal syntax surfaces even when
//! no current source spelling produces them. Removing such variants is a
//! language representation change, not just lexer cleanup.

// =============================================================================
// SOURCE SPANS
// =============================================================================

/// Half-open byte range in the original source buffer.
///
/// Spans are the compiler's source-location currency. They intentionally point
/// at original bytes rather than normalized interner strings so diagnostics can
/// quote exactly what the user wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Inclusive byte offset where the range begins.
    pub start: u32,

    /// Exclusive byte offset where the range ends.
    pub end: u32,
}

impl Span {
    /// Create a span from cursor-derived byte offsets.
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "invalid span: start > end");
        Self { start, end }
    }

    /// Return the smallest span covering both inputs.
    ///
    /// AST construction uses this to preserve source coverage for composite
    /// syntax without retaining every child token at diagnostic time.
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

/// One lexical item paired with its original source location.
///
/// Tokens never borrow source text directly. Payload text moves through
/// [`Symbol`] handles, and exact spelling remains available through the span
/// and original source buffer.
#[derive(Debug, Clone)]
pub struct Token {
    /// Classified token payload emitted by the scanner.
    pub kind: TokenKind,

    /// Source bytes consumed to produce this token.
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

/// Interned string handle local to one lexer result.
///
/// Symbols are cheap to copy and compare, but they are not globally meaningful.
/// Resolve them only with the [`Interner`](super::Interner) that produced the
/// token stream containing the symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub u32);

// =============================================================================
// TOKEN KINDS
// =============================================================================

/// Complete token vocabulary visible to the parser.
///
/// This enum is deliberately broad: it includes current source tokens, metadata
/// tokens, error/recovery sentinels, and a small number of reserved internal
/// variants. Keep spelling policy in scanner/keyword tables; keep this type as
/// the parser-facing vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === Identifiers and literals ===
    Ident(Symbol),
    Underscore(Symbol), // WHY: Special case for pattern matching wildcard
    Integer(i64),
    Float(f64),
    String(Symbol),

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
    Curata,     // Zig allocator requirement
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
    Redde, // return
    Rumpe, // break
    Perge, // continue
    Tacet, // explicit noop

    // === Keywords: Error handling ===
    Tempta,  // try
    Cape,    // catch
    Demum,   // finally
    Iace,    // throw
    Mori,    // panic
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
    Et,  // and
    Aut, // or
    Non, // not (!)
    Vel, // nullish coalesce (??)
    Est, // identity check (===)

    // === Keywords: Objects ===
    Ego,    // self/this
    Finge,  // construct tagged union variant
    Sub,    // extends
    Implet, // implements

    // === Keywords: Type operations ===
    Verte,     // ⇢ — unified postfix type conversion / construction (qua/innatum/novum removed)
    Conversio, // ⇒ — runtime value conversion

    // === Shift operators ===
    Sinistratum, // ≪
    Dextratum,   // ≫

    // === Keywords: Output ===
    Scribe, // print
    Vide,   // debug print
    Mone,   // warn print
    Nota,   // neutral diagnostic note

    // === Keywords: Entry points ===
    Incipit,   // main (sync)
    Incipiet,  // main (async)
    Argumenta, // command-line args
    Cura,      // Zig allocator scope
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
    Futurum,     // todo test modifier
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

    // === Declaration markers (post-name, sponte/fixus) ===
    Sponte, // voluntary / optional declaration slot
    Fixus,  // fixed after first initialization

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
    Arrow,     // →
    ExitArrow, // ⇥
    Cup,       // ∪ (inline union type T ∪ U, T ∪ nihil)
    At,        // @ (annotation marker)
    Section,   // § (section marker)

    // === Operators ===
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    Amp,      // ∧ (bitwise AND)
    Pipe,     // ∨ (bitwise OR)
    Caret,    // ⊻ (bitwise XOR)
    Tilde,    // ¬ (bitwise NOT)
    Bang,     // !
    Question, // ?

    // === Comparison ===
    Eq,       // =
    Assign,   // ←
    EqEq,     // ≡
    EqEqEq,   // (reserved for future / internal; no source form)
    BangEq,   // ≠
    BangEqEq, // (reserved for future / internal; no source form)
    Lt,       // <
    Gt,       // >
    LtEq,     // ≤
    GtEq,     // ≥

    // === Compound assignment ===
    PlusEq,  // ⊕
    MinusEq, // ⊖
    StarEq,  // ⊛
    SlashEq, // ⊘
    AmpEq,   // ⊜
    PipeEq,  // ⊚

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
    LineComment(Symbol), // #

    // === Special ===
    Eof,   // End of file marker
    Error, // Lexer error placeholder
}

impl TokenKind {
    /// Return whether this token kind is reserved-word syntax.
    ///
    /// The parser and diagnostics use this to distinguish "identifier-like"
    /// payloads from tokens that need keyword-specific messages or formatting.
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
                | Rumpe
                | Perge
                | Tacet
                | Tempta
                | Cape
                | Demum
                | Iace
                | Mori
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
                | Finge
                | Sub
                | Implet
                | Verte
                | Conversio
                | Scribe
                | Vide
                | Mone
                | Nota
                | Incipit
                | Incipiet
                | Argumenta
                | Cura
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
                | Sponte
                | Fixus
        )
    }

    /// Return whether this token carries source comment text.
    ///
    /// Parsers may skip comments while tools such as formatters preserve them.
    pub fn is_comment(&self) -> bool {
        matches!(self, TokenKind::LineComment(_))
    }
}
