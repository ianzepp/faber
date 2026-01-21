"""Scope tracking and semantic types for Faber."""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

from errors import Locus


class SymbolSpecies(Enum):
    """What kind of symbol this is."""
    VARIABILIS = "variabilis"
    FUNCTIO = "functio"
    PARAMETRUM = "parametrum"
    TYPUS = "typus"
    GENUS = "genus"
    ORDO = "ordo"
    DISCRETIO = "discretio"
    PACTUM = "pactum"
    VARIANS = "varians"


class ScopusSpecies(Enum):
    """What kind of scope this is."""
    GLOBAL = "global"
    FUNCTIO = "functio"
    MASSA = "massa"
    GENUS = "genus"


# Semantic types (resolved types, distinct from AST Typus nodes)

@dataclass
class SemPrimitivus:
    """Primitive type: textus, numerus, fractus, bivalens, nihil, vacuum."""
    species: str
    nullabilis: bool = False

    def __str__(self) -> str:
        s = self.species
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemLista:
    """List type: lista<T>."""
    elementum: SemanticTypus
    nullabilis: bool = False

    def __str__(self) -> str:
        s = f"lista<{self.elementum}>"
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemTabula:
    """Map type: tabula<K, V>."""
    clavis: SemanticTypus
    valor: SemanticTypus
    nullabilis: bool = False

    def __str__(self) -> str:
        s = f"tabula<{self.clavis}, {self.valor}>"
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemCopia:
    """Set type: copia<T>."""
    elementum: SemanticTypus
    nullabilis: bool = False

    def __str__(self) -> str:
        s = f"copia<{self.elementum}>"
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemFunctio:
    """Function type."""
    params: list[SemanticTypus] = field(default_factory=list)
    reditus: SemanticTypus | None = None
    nullabilis: bool = False

    def __str__(self) -> str:
        params_str = ", ".join(str(p) for p in self.params)
        s = f"functio({params_str})"
        if self.reditus:
            s += f" -> {self.reditus}"
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemGenus:
    """Class/struct type."""
    nomen: str
    agri: dict[str, SemanticTypus] = field(default_factory=dict)
    methodi: dict[str, SemFunctio] = field(default_factory=dict)
    nullabilis: bool = False

    def __str__(self) -> str:
        s = self.nomen
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemOrdo:
    """Enum type."""
    nomen: str
    membra: dict[str, int] = field(default_factory=dict)

    def __str__(self) -> str:
        return self.nomen


@dataclass
class SemDiscretio:
    """Discriminated union type."""
    nomen: str
    variantes: dict[str, SemGenus] = field(default_factory=dict)

    def __str__(self) -> str:
        return self.nomen


@dataclass
class SemPactum:
    """Interface/protocol type."""
    nomen: str
    methodi: dict[str, SemFunctio] = field(default_factory=dict)

    def __str__(self) -> str:
        return self.nomen


@dataclass
class SemUsitatum:
    """Reference to a user-defined type (unresolved or simple reference)."""
    nomen: str
    nullabilis: bool = False

    def __str__(self) -> str:
        s = self.nomen
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemUnio:
    """Union type: A | B | C."""
    membra: list[SemanticTypus] = field(default_factory=list)
    nullabilis: bool = False

    def __str__(self) -> str:
        s = " | ".join(str(m) for m in self.membra)
        if self.nullabilis:
            s += "?"
        return s


@dataclass
class SemParametrum:
    """Generic type parameter (e.g., T in lista<T>)."""
    nomen: str

    def __str__(self) -> str:
        return self.nomen


@dataclass
class SemIgnotum:
    """Unknown/error type for unresolved cases."""
    ratio: str = "unresolved"

    def __str__(self) -> str:
        return "ignotum"


SemanticTypus = (SemPrimitivus | SemLista | SemTabula | SemCopia | SemFunctio |
                 SemGenus | SemOrdo | SemDiscretio | SemPactum | SemUsitatum |
                 SemUnio | SemParametrum | SemIgnotum | None)


# Primitive type constants
TEXTUS = SemPrimitivus("textus")
NUMERUS = SemPrimitivus("numerus")
FRACTUS = SemPrimitivus("fractus")
BIVALENS = SemPrimitivus("bivalens")
NIHIL = SemPrimitivus("nihil")
VACUUM = SemPrimitivus("vacuum")
IGNOTUM = SemIgnotum("unresolved")


def nullabilis(t: SemanticTypus) -> SemanticTypus:
    """Make a type nullable."""
    if t is None:
        return None
    if isinstance(t, SemPrimitivus):
        return SemPrimitivus(t.species, nullabilis=True)
    if isinstance(t, SemLista):
        return SemLista(t.elementum, nullabilis=True)
    if isinstance(t, SemTabula):
        return SemTabula(t.clavis, t.valor, nullabilis=True)
    if isinstance(t, SemCopia):
        return SemCopia(t.elementum, nullabilis=True)
    if isinstance(t, SemFunctio):
        return SemFunctio(t.params, t.reditus, nullabilis=True)
    if isinstance(t, SemGenus):
        return SemGenus(t.nomen, t.agri, t.methodi, nullabilis=True)
    if isinstance(t, SemUsitatum):
        return SemUsitatum(t.nomen, nullabilis=True)
    if isinstance(t, SemUnio):
        return SemUnio(t.membra, nullabilis=True)
    return t


@dataclass
class Symbolum:
    """A named entity in the symbol table."""
    nomen: str
    typus: SemanticTypus = None
    species: SymbolSpecies = SymbolSpecies.VARIABILIS
    mutabilis: bool = True
    locus: Locus = field(default_factory=Locus)
    node: Any = None


class Scopus:
    """A lexical scope with a symbol table."""

    def __init__(self, parent: Scopus | None, species: ScopusSpecies, nomen: str = ""):
        self.parent = parent
        self.symbola: dict[str, Symbolum] = {}
        self.species = species
        self.nomen = nomen

    def definie(self, sym: Symbolum) -> None:
        """Add a symbol to this scope."""
        self.symbola[sym.nomen] = sym

    def quaere(self, nomen: str) -> Symbolum | None:
        """Look up a symbol in this scope and parent scopes."""
        if nomen in self.symbola:
            return self.symbola[nomen]
        if self.parent:
            return self.parent.quaere(nomen)
        return None

    def quaere_localis(self, nomen: str) -> Symbolum | None:
        """Look up a symbol only in this scope (not parents)."""
        return self.symbola.get(nomen)

    def quaere_typus(self, nomen: str) -> Symbolum | None:
        """Look up a type symbol (genus, ordo, discretio, pactum)."""
        sym = self.quaere(nomen)
        if sym is None:
            return None
        if sym.species in (SymbolSpecies.GENUS, SymbolSpecies.ORDO,
                           SymbolSpecies.DISCRETIO, SymbolSpecies.PACTUM,
                           SymbolSpecies.TYPUS):
            return sym
        return None


@dataclass
class SemanticError:
    """An error found during semantic analysis."""
    nuntius: str
    locus: Locus


class SemanticContext:
    """Holds the state during semantic analysis."""

    def __init__(self):
        self.global_scope = Scopus(None, ScopusSpecies.GLOBAL, "")
        self.current = self.global_scope
        self.typi: dict[str, SemanticTypus] = {}
        self.ordo_registry: dict[str, SemOrdo] = {}
        self.disc_registry: dict[str, SemDiscretio] = {}
        self.genus_registry: dict[str, SemGenus] = {}
        self.errores: list[SemanticError] = []
        self.expr_types: dict[int, SemanticTypus] = {}  # id(expr) -> type

    def intra_scopum(self, species: ScopusSpecies, nomen: str = "") -> None:
        """Enter a new scope."""
        self.current = Scopus(self.current, species, nomen)

    def exi_scopum(self) -> None:
        """Exit the current scope, returning to parent."""
        if self.current.parent:
            self.current = self.current.parent

    def definie(self, sym: Symbolum) -> None:
        """Add a symbol to the current scope."""
        self.current.definie(sym)

    def quaere(self, nomen: str) -> Symbolum | None:
        """Look up a symbol in the current scope chain."""
        return self.current.quaere(nomen)

    def error(self, nuntius: str, locus: Locus) -> None:
        """Record a semantic error."""
        self.errores.append(SemanticError(nuntius, locus))

    def register_typus(self, nomen: str, typus: SemanticTypus) -> None:
        """Register a resolved type by name."""
        self.typi[nomen] = typus

    def resolve_typus_nomen(self, nomen: str) -> SemanticTypus:
        """Resolve a type name to its semantic type."""
        # Check primitive types first
        primitives = {
            "textus": TEXTUS,
            "numerus": NUMERUS,
            "fractus": FRACTUS,
            "bivalens": BIVALENS,
            "nihil": NIHIL,
            "vacuum": VACUUM,
            "vacuus": VACUUM,
            "ignotum": IGNOTUM,
            "quodlibet": IGNOTUM,
            "quidlibet": IGNOTUM,
        }
        if nomen in primitives:
            return primitives[nomen]

        # Check registered types
        if nomen in self.typi:
            return self.typi[nomen]

        # Check registries
        if nomen in self.ordo_registry:
            return self.ordo_registry[nomen]
        if nomen in self.disc_registry:
            return self.disc_registry[nomen]
        if nomen in self.genus_registry:
            return self.genus_registry[nomen]

        # Return unresolved reference
        return SemUsitatum(nomen)

    def set_expr_type(self, expr: Any, typus: SemanticTypus) -> None:
        """Record the resolved type for an expression."""
        self.expr_types[id(expr)] = typus

    def get_expr_type(self, expr: Any) -> SemanticTypus:
        """Retrieve the resolved type for an expression."""
        return self.expr_types.get(id(expr), IGNOTUM)
