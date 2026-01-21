"""AST node definitions for Faber."""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any
from enum import Enum

from errors import Locus


# Token types
class TokenTag(str, Enum):
    EOF = "EOF"
    NEWLINE = "Newline"
    IDENTIFIER = "Identifier"
    NUMERUS = "Numerus"
    TEXTUS = "Textus"
    OPERATOR = "Operator"
    PUNCTUATOR = "Punctuator"
    KEYWORD = "Keyword"
    COMMENT = "Comment"


@dataclass
class Token:
    """A lexical token."""
    tag: TokenTag
    valor: str
    locus: Locus


# Literal species
class LitteraSpecies(str, Enum):
    NUMERUS = "Numerus"
    FRACTUS = "Fractus"
    TEXTUS = "Textus"
    VERUM = "Verum"
    FALSUM = "Falsum"
    NIHIL = "Nihil"


# Variable species
class VariaSpecies(str, Enum):
    VARIA = "Varia"
    FIXUM = "Fixum"
    FIGENDUM = "Figendum"
    VARIANDUM = "Variandum"


# Type nodes
@dataclass
class TypusNomen:
    """Simple type name."""
    nomen: str
    tag: str = "TypusNomen"


@dataclass
class TypusNullabilis:
    """Nullable type T?"""
    inner: Any  # Typus
    tag: str = "TypusNullabilis"


@dataclass
class TypusGenericus:
    """Generic type like lista<T>"""
    nomen: str
    args: list[Any] = field(default_factory=list)  # list[Typus]
    tag: str = "TypusGenericus"


@dataclass
class TypusFunctio:
    """Function type (params) -> returns"""
    params: list[Any] = field(default_factory=list)  # list[Typus]
    returns: Any = None  # Typus
    tag: str = "TypusFunctio"


@dataclass
class TypusUnio:
    """Union type A | B"""
    members: list[Any] = field(default_factory=list)  # list[Typus]
    tag: str = "TypusUnio"


@dataclass
class TypusLitteralis:
    """Literal type like "hello" or 42"""
    valor: str
    tag: str = "TypusLitteralis"


Typus = TypusNomen | TypusNullabilis | TypusGenericus | TypusFunctio | TypusUnio | TypusLitteralis | None


# Expression nodes
@dataclass
class ExprNomen:
    """Identifier reference."""
    valor: str
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprNomen"


@dataclass
class ExprEgo:
    """Self reference (ego)."""
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprEgo"


@dataclass
class ExprLittera:
    """Literal value."""
    species: LitteraSpecies
    valor: str
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprLittera"


@dataclass
class ExprBinaria:
    """Binary expression."""
    signum: str
    sin: Any  # Expr
    dex: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprBinaria"


@dataclass
class ExprUnaria:
    """Unary expression."""
    signum: str
    arg: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprUnaria"


@dataclass
class ExprAssignatio:
    """Assignment expression."""
    signum: str
    sin: Any  # Expr
    dex: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprAssignatio"


@dataclass
class ExprCondicio:
    """Ternary conditional."""
    cond: Any  # Expr
    cons: Any  # Expr
    alt: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprCondicio"


@dataclass
class ExprVocatio:
    """Function call."""
    callee: Any  # Expr
    args: list[Any] = field(default_factory=list)  # list[Expr]
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprVocatio"


@dataclass
class ExprMembrum:
    """Member access."""
    obj: Any  # Expr
    prop: Any  # Expr
    computed: bool = False
    non_null: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprMembrum"


@dataclass
class ExprSeries:
    """Array literal."""
    elementa: list[Any] = field(default_factory=list)  # list[Expr]
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprSeries"


@dataclass
class ObiectumProp:
    """Object property."""
    key: Any  # Expr
    valor: Any = None  # Expr
    shorthand: bool = False
    computed: bool = False
    locus: Locus = field(default_factory=Locus)


@dataclass
class ExprObiectum:
    """Object literal."""
    props: list[ObiectumProp] = field(default_factory=list)
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprObiectum"


@dataclass
class ExprClausura:
    """Lambda/closure."""
    params: list[Any] = field(default_factory=list)  # list[Param]
    corpus: Any = None  # Stmt | Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprClausura"


@dataclass
class ExprNovum:
    """Constructor call (novum Type(...))."""
    callee: Any  # Expr
    args: list[Any] = field(default_factory=list)  # list[Expr]
    init: Any = None  # Expr (optional initializer)
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprNovum"


@dataclass
class ExprCede:
    """Await expression (cede)."""
    arg: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprCede"


@dataclass
class ExprQua:
    """Type assertion (expr qua Type)."""
    expr: Any  # Expr
    typus: Any = None  # Typus
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprQua"


@dataclass
class ExprInnatum:
    """Inline type assertion."""
    expr: Any  # Expr
    typus: Any = None  # Typus
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprInnatum"


@dataclass
class ExprPostfixNovum:
    """Postfix constructor ({ ... } novum Type)."""
    expr: Any  # Expr
    typus: Any = None  # Typus
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprPostfixNovum"


@dataclass
class ExprFinge:
    """Variant constructor (finge Variant { ... })."""
    variant: str
    campi: list[ObiectumProp] = field(default_factory=list)
    typus: Any = None  # Typus
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprFinge"


@dataclass
class ExprScriptum:
    """Template string."""
    template: str
    args: list[Any] = field(default_factory=list)  # list[Expr]
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprScriptum"


@dataclass
class ExprAmbitus:
    """Range expression (start usque/ante end)."""
    start: Any  # Expr
    end: Any  # Expr
    inclusive: bool = True
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprAmbitus"


@dataclass
class ExprConversio:
    """Type conversion (numeratum, textatum, etc.)."""
    expr: Any  # Expr
    species: str
    fallback: Any = None  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "ExprConversio"


Expr = (ExprNomen | ExprEgo | ExprLittera | ExprBinaria | ExprUnaria |
        ExprAssignatio | ExprCondicio | ExprVocatio | ExprMembrum |
        ExprSeries | ExprObiectum | ExprClausura | ExprNovum | ExprCede |
        ExprQua | ExprInnatum | ExprPostfixNovum | ExprFinge | ExprScriptum |
        ExprAmbitus | ExprConversio | None)


# Supporting types
@dataclass
class Param:
    """Function parameter."""
    nomen: str
    typus: Any = None  # Typus
    default: Any = None  # Expr
    rest: bool = False
    optional: bool = False
    ownership: str = ""
    locus: Locus = field(default_factory=Locus)


@dataclass
class CampusDecl:
    """Class field declaration."""
    nomen: str
    typus: Any = None  # Typus
    valor: Any = None  # Expr
    visibilitas: str = ""
    locus: Locus = field(default_factory=Locus)


@dataclass
class PactumMethodus:
    """Interface method declaration."""
    nomen: str
    params: list[Param] = field(default_factory=list)
    typus_reditus: Any = None  # Typus
    asynca: bool = False
    locus: Locus = field(default_factory=Locus)


@dataclass
class OrdoMembrum:
    """Enum member."""
    nomen: str
    valor: str | None = None
    locus: Locus = field(default_factory=Locus)


@dataclass
class VariansCampus:
    """Variant field."""
    nomen: str
    typus: Any = None  # Typus


@dataclass
class VariansDecl:
    """Variant declaration in discretio."""
    nomen: str
    campi: list[VariansCampus] = field(default_factory=list)
    locus: Locus = field(default_factory=Locus)


@dataclass
class ImportSpec:
    """Import specifier."""
    imported: str
    local: str
    locus: Locus = field(default_factory=Locus)


@dataclass
class EligeCasus:
    """Switch case."""
    cond: Any  # Expr
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)


@dataclass
class VariansPattern:
    """Pattern for match/discerne."""
    variant: str = ""
    bindings: list[str] = field(default_factory=list)
    alias: str | None = None
    wildcard: bool = False
    locus: Locus = field(default_factory=Locus)


@dataclass
class DiscerneCasus:
    """Match case."""
    patterns: list[VariansPattern] = field(default_factory=list)
    corpus: Any = None  # Stmt
    locus: Locus = field(default_factory=Locus)


@dataclass
class CustodiClausula:
    """Guard clause."""
    cond: Any  # Expr
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)


@dataclass
class CapeClausula:
    """Catch clause."""
    param: str
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)


# Statement nodes
@dataclass
class StmtMassa:
    """Block statement."""
    corpus: list[Any] = field(default_factory=list)  # list[Stmt]
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtMassa"


@dataclass
class StmtExpressia:
    """Expression statement."""
    expr: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtExpressia"


@dataclass
class StmtVaria:
    """Variable declaration."""
    nomen: str
    species: VariaSpecies = VariaSpecies.VARIA
    typus: Any = None  # Typus
    valor: Any = None  # Expr
    publica: bool = False
    externa: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtVaria"


@dataclass
class StmtFunctio:
    """Function declaration."""
    nomen: str
    params: list[Param] = field(default_factory=list)
    typus_reditus: Any = None  # Typus
    corpus: Any = None  # Stmt
    asynca: bool = False
    publica: bool = False
    generics: list[str] = field(default_factory=list)
    externa: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtFunctio"


@dataclass
class StmtGenus:
    """Class declaration."""
    nomen: str
    campi: list[CampusDecl] = field(default_factory=list)
    methodi: list[Any] = field(default_factory=list)  # list[Stmt]
    implet: list[str] = field(default_factory=list)
    generics: list[str] = field(default_factory=list)
    publica: bool = False
    abstractus: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtGenus"


@dataclass
class StmtPactum:
    """Interface declaration."""
    nomen: str
    methodi: list[PactumMethodus] = field(default_factory=list)
    generics: list[str] = field(default_factory=list)
    publica: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtPactum"


@dataclass
class StmtOrdo:
    """Enum declaration."""
    nomen: str
    membra: list[OrdoMembrum] = field(default_factory=list)
    publica: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtOrdo"


@dataclass
class StmtDiscretio:
    """Discriminated union declaration."""
    nomen: str
    variantes: list[VariansDecl] = field(default_factory=list)
    generics: list[str] = field(default_factory=list)
    publica: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtDiscretio"


@dataclass
class StmtImporta:
    """Import statement."""
    fons: str
    specs: list[ImportSpec] = field(default_factory=list)
    totum: bool = False
    alias: str | None = None
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtImporta"


@dataclass
class StmtTypusAlias:
    """Type alias declaration."""
    nomen: str
    typus: Any = None  # Typus
    publica: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtTypusAlias"


@dataclass
class StmtSi:
    """If statement."""
    cond: Any  # Expr
    cons: Any  # Stmt
    alt: Any = None  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtSi"


@dataclass
class StmtDum:
    """While loop."""
    cond: Any  # Expr
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtDum"


@dataclass
class StmtFacDum:
    """Do-while loop."""
    corpus: Any  # Stmt
    cond: Any  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtFacDum"


@dataclass
class StmtIteratio:
    """For loop."""
    binding: str
    iter: Any  # Expr
    corpus: Any  # Stmt
    species: str = "In"  # "In" or "De"
    asynca: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtIteratio"


@dataclass
class StmtElige:
    """Switch statement."""
    discrim: Any  # Expr
    casus: list[EligeCasus] = field(default_factory=list)
    default: Any = None  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtElige"


@dataclass
class StmtDiscerne:
    """Pattern match statement."""
    discrim: list[Any] = field(default_factory=list)  # list[Expr]
    casus: list[DiscerneCasus] = field(default_factory=list)
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtDiscerne"


@dataclass
class StmtCustodi:
    """Guard statement."""
    clausulae: list[CustodiClausula] = field(default_factory=list)
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtCustodi"


@dataclass
class StmtTempta:
    """Try statement."""
    corpus: Any  # Stmt
    cape: CapeClausula | None = None
    demum: Any = None  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtTempta"


@dataclass
class StmtRedde:
    """Return statement."""
    valor: Any = None  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtRedde"


@dataclass
class StmtIace:
    """Throw statement."""
    arg: Any = None  # Expr
    fatale: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtIace"


@dataclass
class StmtScribe:
    """Print statement."""
    args: list[Any] = field(default_factory=list)  # list[Expr]
    gradus: str = "Scribe"  # Scribe, Vide, Mone
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtScribe"


@dataclass
class StmtAdfirma:
    """Assert statement."""
    cond: Any  # Expr
    msg: Any = None  # Expr
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtAdfirma"


@dataclass
class StmtRumpe:
    """Break statement."""
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtRumpe"


@dataclass
class StmtPerge:
    """Continue statement."""
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtPerge"


@dataclass
class StmtIncipit:
    """Entry point."""
    corpus: Any  # Stmt
    asynca: bool = False
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtIncipit"


@dataclass
class StmtProbandum:
    """Test suite (describe)."""
    nomen: str
    corpus: list[Any] = field(default_factory=list)  # list[Stmt]
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtProbandum"


@dataclass
class StmtProba:
    """Test case (it)."""
    nomen: str
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtProba"


@dataclass
class StmtIn:
    """With statement (in expr { ... })."""
    expr: Any  # Expr
    corpus: Any  # Stmt
    locus: Locus = field(default_factory=Locus)
    tag: str = "StmtIn"


Stmt = (StmtMassa | StmtExpressia | StmtVaria | StmtFunctio | StmtGenus |
        StmtPactum | StmtOrdo | StmtDiscretio | StmtImporta | StmtTypusAlias |
        StmtSi | StmtDum | StmtFacDum | StmtIteratio | StmtElige | StmtDiscerne |
        StmtCustodi | StmtTempta | StmtRedde | StmtIace | StmtScribe | StmtAdfirma |
        StmtRumpe | StmtPerge | StmtIncipit | StmtProbandum | StmtProba | StmtIn | None)


@dataclass
class Modulus:
    """Top-level compilation unit."""
    corpus: list[Stmt] = field(default_factory=list)
    locus: Locus = field(default_factory=Locus)
