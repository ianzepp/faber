"""Recursive descent parser for Faber source code."""

from __future__ import annotations
from typing import Any

from errors import Locus, CompileError
from nodes import (
    Token, TokenTag, Modulus,
    Typus, TypusNomen, TypusNullabilis, TypusGenericus, TypusFunctio, TypusUnio, TypusLitteralis,
    Expr, ExprNomen, ExprEgo, ExprLittera, ExprBinaria, ExprUnaria, ExprAssignatio,
    ExprCondicio, ExprVocatio, ExprMembrum, ExprSeries, ExprObiectum, ExprClausura,
    ExprNovum, ExprCede, ExprQua, ExprInnatum, ExprPostfixNovum, ExprFinge, ExprScriptum,
    ExprAmbitus, ExprConversio, ObiectumProp,
    Stmt, StmtMassa, StmtExpressia, StmtVaria, StmtFunctio, StmtGenus, StmtPactum,
    StmtOrdo, StmtDiscretio, StmtImporta, StmtTypusAlias, StmtSi, StmtDum, StmtFacDum,
    StmtIteratio, StmtElige, StmtDiscerne, StmtCustodi, StmtTempta, StmtRedde, StmtIace,
    StmtScribe, StmtAdfirma, StmtRumpe, StmtPerge, StmtIncipit, StmtProbandum, StmtProba, StmtIn,
    Param, CampusDecl, PactumMethodus, OrdoMembrum, VariansDecl, VariansCampus,
    ImportSpec, EligeCasus, DiscerneCasus, VariansPattern, CustodiClausula, CapeClausula,
    LitteraSpecies, VariaSpecies,
)


PRECEDENCE = {
    "=": 1, "+=": 1, "-=": 1, "*=": 1, "/=": 1,
    "vel": 2, "??": 2,
    "aut": 3, "||": 3,
    "et": 4, "&&": 4,
    "==": 5, "!=": 5, "===": 5, "!==": 5,
    "<": 6, ">": 6, "<=": 6, ">=": 6, "inter": 6, "intra": 6,
    "+": 7, "-": 7,
    "*": 8, "/": 8, "%": 8,
    "qua": 9, "innatum": 9, "novum": 9,
    "numeratum": 9, "fractatum": 9, "textatum": 9, "bivalentum": 9,
}

UNARY_OPS = frozenset(["-", "!", "~", "non", "nihil", "nonnihil", "positivum", "negativum", "nulla", "nonnulla"])
ASSIGN_OPS = frozenset(["=", "+=", "-=", "*=", "/="])


class Parser:
    """Parser for Faber source."""

    def __init__(self, tokens: list[Token], filename: str = "<stdin>"):
        self.tokens = tokens
        self.pos = 0
        self.filename = filename

    def peek(self, offset: int = 0) -> Token:
        idx = self.pos + offset
        if idx >= len(self.tokens):
            return self.tokens[-1]
        return self.tokens[idx]

    def advance(self) -> Token:
        tok = self.tokens[self.pos]
        self.pos += 1
        return tok

    def check(self, tag: TokenTag, valor: str | None = None) -> bool:
        tok = self.peek()
        if tok.tag != tag:
            return False
        if valor is not None and tok.valor != valor:
            return False
        return True

    def match(self, tag: TokenTag, valor: str | None = None) -> Token | None:
        if self.check(tag, valor):
            return self.advance()
        return None

    def expect(self, tag: TokenTag, valor: str | None = None) -> Token:
        tok = self.match(tag, valor)
        if tok is None:
            got = self.peek()
            msg = valor if valor else tag.value
            raise self.error(f"expected {msg}, got '{got.valor}'")
        return tok

    def error(self, msg: str) -> CompileError:
        return CompileError(msg, self.peek().locus, self.filename)

    def expect_name(self) -> Token:
        """Accept identifier OR keyword as a name."""
        tok = self.peek()
        if tok.tag in (TokenTag.IDENTIFIER, TokenTag.KEYWORD):
            return self.advance()
        raise self.error(f"expected identifier, got '{tok.valor}'")

    def check_name(self) -> bool:
        tok = self.peek()
        return tok.tag in (TokenTag.IDENTIFIER, TokenTag.KEYWORD)

    def parse(self) -> Modulus:
        """Parse entry point."""
        corpus: list[Stmt] = []
        while not self.check(TokenTag.EOF):
            corpus.append(self.parse_stmt())
        return Modulus(corpus=corpus, locus=Locus(linea=1, columna=1, index=0))

    def parse_stmt(self) -> Stmt:
        publica = False
        futura = False
        externa = False

        while self.match(TokenTag.PUNCTUATOR, "@"):
            pub, fut, ext = self.parse_annotatio()
            if pub:
                publica = True
            if fut:
                futura = True
            if ext:
                externa = True

        if self.match(TokenTag.PUNCTUATOR, "§"):
            return self.parse_sectio()

        tok = self.peek()
        if tok.tag == TokenTag.KEYWORD:
            match tok.valor:
                case "varia" | "fixum" | "figendum" | "variandum":
                    return self.parse_varia(publica, externa)
                case "ex":
                    return self.parse_ex_stmt(publica)
                case "functio":
                    return self.parse_functio(publica, futura, externa)
                case "abstractus":
                    self.advance()
                    if self.check(TokenTag.KEYWORD, "genus"):
                        return self.parse_genus(publica, True)
                    raise self.error("expected 'genus' after 'abstractus'")
                case "genus":
                    return self.parse_genus(publica, False)
                case "pactum":
                    return self.parse_pactum(publica)
                case "ordo":
                    return self.parse_ordo(publica)
                case "discretio":
                    return self.parse_discretio(publica)
                case "typus":
                    return self.parse_typus_alias(publica)
                case "in":
                    return self.parse_in_stmt()
                case "de":
                    return self.parse_de_stmt()
                case "si":
                    return self.parse_si()
                case "dum":
                    return self.parse_dum()
                case "fac":
                    return self.parse_fac()
                case "elige":
                    return self.parse_elige()
                case "discerne":
                    return self.parse_discerne()
                case "custodi":
                    return self.parse_custodi()
                case "tempta":
                    return self.parse_tempta()
                case "redde":
                    return self.parse_redde()
                case "iace" | "mori":
                    return self.parse_iace()
                case "scribe" | "vide" | "mone":
                    return self.parse_scribe()
                case "adfirma":
                    return self.parse_adfirma()
                case "rumpe":
                    return self.parse_rumpe()
                case "perge":
                    return self.parse_perge()
                case "incipit" | "incipiet":
                    return self.parse_incipit()
                case "probandum":
                    return self.parse_probandum()
                case "proba":
                    return self.parse_proba()

        if self.check(TokenTag.PUNCTUATOR, "{"):
            return self.parse_massa()

        return self.parse_expressia_stmt()

    def parse_sectio(self) -> Stmt:
        """Dispatch § annotations based on keyword."""
        tok = self.peek()
        if tok.tag not in (TokenTag.IDENTIFIER, TokenTag.KEYWORD):
            raise self.error("expected keyword after §")
        keyword = self.advance().valor
        if keyword == "importa":
            return self.parse_sectio_importa()
        elif keyword == "sectio":
            return self.parse_sectio_sectio()
        elif keyword == "ex":
            return self.parse_sectio_ex_legacy()
        else:
            raise self.error(f"unknown § keyword: {keyword}")

    def parse_sectio_importa(self) -> Stmt:
        """New syntax: § importa ex 'path' bindings"""
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "ex")
        fons = self.expect(TokenTag.TEXTUS).valor

        # Check for wildcard import: * or * ut alias
        if self.match(TokenTag.OPERATOR, "*"):
            alias: str | None = None
            if self.match(TokenTag.KEYWORD, "ut"):
                alias = self.expect(TokenTag.IDENTIFIER).valor
            return StmtImporta(fons=fons, specs=[], totum=True, alias=alias, locus=locus)

        specs: list[ImportSpec] = []
        while True:
            loc = self.peek().locus
            imported = self.expect(TokenTag.IDENTIFIER).valor
            local = imported
            if self.match(TokenTag.KEYWORD, "ut"):
                local = self.expect(TokenTag.IDENTIFIER).valor
            specs.append(ImportSpec(imported=imported, local=local, locus=loc))
            if not self.match(TokenTag.PUNCTUATOR, ","):
                break

        return StmtImporta(fons=fons, specs=specs, totum=False, alias=None, locus=locus)

    def parse_sectio_sectio(self) -> Stmt:
        """§ sectio 'name' - file section marker (ignored in nanus, but parsed)"""
        locus = self.peek().locus
        self.expect(TokenTag.TEXTUS)  # section name, ignored
        return StmtExpressia(expr=ExprLittera(species=LitteraSpecies.NIHIL, valor="null", locus=locus), locus=locus)

    def parse_sectio_ex_legacy(self) -> Stmt:
        """Legacy syntax: § ex 'path' importa bindings"""
        locus = self.peek().locus
        fons = self.expect(TokenTag.TEXTUS).valor
        self.expect(TokenTag.KEYWORD, "importa")

        # Check for wildcard import: * or * ut alias
        if self.match(TokenTag.OPERATOR, "*"):
            alias: str | None = None
            if self.match(TokenTag.KEYWORD, "ut"):
                alias = self.expect(TokenTag.IDENTIFIER).valor
            return StmtImporta(fons=fons, specs=[], totum=True, alias=alias, locus=locus)

        specs: list[ImportSpec] = []
        while True:
            loc = self.peek().locus
            imported = self.expect(TokenTag.IDENTIFIER).valor
            local = imported
            if self.match(TokenTag.KEYWORD, "ut"):
                local = self.expect(TokenTag.IDENTIFIER).valor
            specs.append(ImportSpec(imported=imported, local=local, locus=loc))
            if not self.match(TokenTag.PUNCTUATOR, ","):
                break

        return StmtImporta(fons=fons, specs=specs, totum=False, alias=None, locus=locus)

    def parse_annotatio(self) -> tuple[bool, bool, bool]:
        """Dispatch @ annotations based on keyword. Returns (publica, futura, externa)."""
        tok = self.peek()
        if tok.tag not in (TokenTag.IDENTIFIER, TokenTag.KEYWORD):
            raise self.error("expected keyword after @")
        keyword = self.advance().valor
        match keyword:
            case "publica" | "publicum":
                return (True, False, False)
            case "privata" | "privatum":
                return (False, False, False)
            case "futura":
                return (False, True, False)
            case "externa":
                return (False, False, True)
            # Stdlib annotations - skip their arguments
            case "innatum" | "subsidia" | "radix" | "verte":
                self._skip_annotatio_args()
                return (False, False, False)
            # CLI annotations - skip their arguments
            case "cli" | "versio" | "descriptio" | "optio" | "operandus" | "imperium" | "alias" | "imperia" | "nomen":
                self._skip_annotatio_args()
                return (False, False, False)
            # Formatter annotations - skip their arguments
            case "indentum" | "tabulae" | "latitudo" | "ordinatio" | "separaGroups" | "bracchiae" | "methodiSeparatio":
                self._skip_annotatio_args()
                return (False, False, False)
            case _:
                raise self.error(f"unknown @ keyword: {keyword}")

    def _skip_annotatio_args(self) -> None:
        """Skip annotation arguments until next @ or § or declaration keyword."""
        while (not self.check(TokenTag.EOF) and
               not self.check(TokenTag.PUNCTUATOR, "@") and
               not self.check(TokenTag.PUNCTUATOR, "§") and
               not self._is_declaration_keyword()):
            self.advance()

    def parse_varia(self, publica: bool, externa: bool) -> Stmt:
        locus = self.peek().locus
        kw = self.advance().valor
        species = VariaSpecies.VARIA
        if kw == "figendum":
            species = VariaSpecies.FIGENDUM
        elif kw == "fixum":
            species = VariaSpecies.FIXUM
        elif kw == "variandum":
            species = VariaSpecies.VARIANDUM

        typus: Typus = None
        first = self.expect_name().valor

        if self.check(TokenTag.OPERATOR, "<"):
            args: list[Typus] = []
            self.advance()
            while True:
                args.append(self.parse_typus())
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")
            typus = TypusGenericus(nomen=first, args=args)
            if self.match(TokenTag.PUNCTUATOR, "?"):
                typus = TypusNullabilis(inner=typus)
            nomen = self.expect_name().valor
        elif self.match(TokenTag.PUNCTUATOR, "?"):
            typus = TypusNullabilis(inner=TypusNomen(nomen=first))
            nomen = self.expect_name().valor
        elif self.check_name():
            typus = TypusNomen(nomen=first)
            nomen = self.expect_name().valor
        else:
            nomen = first

        valor: Expr = None
        if self.match(TokenTag.OPERATOR, "="):
            valor = self.parse_expr(0)

        return StmtVaria(nomen=nomen, species=species, typus=typus, valor=valor,
                         publica=publica, externa=externa, locus=locus)

    def parse_ex_stmt(self, publica: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "ex")
        expr = self.parse_expr(0)

        if self.check(TokenTag.KEYWORD, "fixum") or self.check(TokenTag.KEYWORD, "varia"):
            self.advance()
            binding = self.expect(TokenTag.IDENTIFIER).valor
            corpus = self.parse_massa()
            return StmtIteratio(binding=binding, iter=expr, corpus=corpus, species="Ex",
                                asynca=False, locus=locus)

        raise self.error("destructuring not supported in nanus")

    def parse_functio(self, publica: bool, futura: bool, externa: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "functio")
        asynca = futura

        nomen = self.expect_name().valor

        generics: list[str] = []
        if self.match(TokenTag.OPERATOR, "<"):
            while True:
                generics.append(self.expect(TokenTag.IDENTIFIER).valor)
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")

        self.expect(TokenTag.PUNCTUATOR, "(")
        params = self.parse_params()
        self.expect(TokenTag.PUNCTUATOR, ")")

        typus_reditus: Typus = None
        if self.match(TokenTag.OPERATOR, "->"):
            typus_reditus = self.parse_typus()

        corpus: Stmt = None
        if self.check(TokenTag.PUNCTUATOR, "{"):
            corpus = self.parse_massa()

        return StmtFunctio(nomen=nomen, params=params, typus_reditus=typus_reditus,
                           corpus=corpus, asynca=asynca, publica=publica,
                           generics=generics, externa=externa, locus=locus)

    def parse_params(self) -> list[Param]:
        params: list[Param] = []
        if self.check(TokenTag.PUNCTUATOR, ")"):
            return params

        while True:
            locus = self.peek().locus
            rest = bool(self.match(TokenTag.KEYWORD, "ceteri"))
            optional = bool(self.match(TokenTag.KEYWORD, "si"))

            ownership = ""
            if self.match(TokenTag.KEYWORD, "ex"):
                ownership = "ex"
            elif self.match(TokenTag.KEYWORD, "de"):
                ownership = "de"
            elif self.match(TokenTag.KEYWORD, "in"):
                ownership = "in"

            typus: Typus = None

            if self.check_name():
                first = self.expect_name().valor

                if self.match(TokenTag.OPERATOR, "<"):
                    args: list[Typus] = []
                    while True:
                        args.append(self.parse_typus())
                        if not self.match(TokenTag.PUNCTUATOR, ","):
                            break
                    self.expect(TokenTag.OPERATOR, ">")
                    typus = TypusGenericus(nomen=first, args=args)
                    if self.match(TokenTag.PUNCTUATOR, "?"):
                        typus = TypusNullabilis(inner=typus)
                    nomen = self.expect_name().valor
                elif self.match(TokenTag.PUNCTUATOR, "?"):
                    typus = TypusNullabilis(inner=TypusNomen(nomen=first))
                    nomen = self.expect_name().valor
                elif self.check_name():
                    typus = TypusNomen(nomen=first)
                    nomen = self.expect_name().valor
                else:
                    nomen = first
            else:
                raise self.error("expected parameter name")

            if optional and typus is not None:
                if not isinstance(typus, TypusNullabilis):
                    typus = TypusNullabilis(inner=typus)

            default: Expr = None
            if self.match(TokenTag.OPERATOR, "="):
                default = self.parse_expr(0)

            params.append(Param(nomen=nomen, typus=typus, default=default, rest=rest,
                                optional=optional, ownership=ownership, locus=locus))

            if not self.match(TokenTag.PUNCTUATOR, ","):
                break

        return params

    def parse_genus(self, publica: bool, abstractus: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "genus")
        nomen = self.expect(TokenTag.IDENTIFIER).valor

        generics: list[str] = []
        if self.match(TokenTag.OPERATOR, "<"):
            while True:
                generics.append(self.expect(TokenTag.IDENTIFIER).valor)
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")

        implet: list[str] = []
        if self.match(TokenTag.KEYWORD, "implet"):
            while True:
                implet.append(self.expect(TokenTag.IDENTIFIER).valor)
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break

        self.expect(TokenTag.PUNCTUATOR, "{")

        campi: list[CampusDecl] = []
        methodi: list[Stmt] = []

        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            while self.match(TokenTag.PUNCTUATOR, "@"):
                tok = self.peek()
                if tok.tag not in (TokenTag.IDENTIFIER, TokenTag.KEYWORD):
                    raise self.error("expected annotation name")
                self.advance()

            visibilitas = "Publica"
            if self.match(TokenTag.KEYWORD, "privata") or self.match(TokenTag.KEYWORD, "privatus"):
                visibilitas = "Privata"
            elif self.match(TokenTag.KEYWORD, "protecta") or self.match(TokenTag.KEYWORD, "protectus"):
                visibilitas = "Protecta"

            if self.check(TokenTag.KEYWORD, "functio"):
                methodi.append(self.parse_functio(False, False, False))
            else:
                loc = self.peek().locus
                first = self.expect_name().valor
                field_typus: Typus = None

                if self.match(TokenTag.OPERATOR, "<"):
                    args: list[Typus] = []
                    while True:
                        args.append(self.parse_typus())
                        if not self.match(TokenTag.PUNCTUATOR, ","):
                            break
                    self.expect(TokenTag.OPERATOR, ">")
                    field_typus = TypusGenericus(nomen=first, args=args)
                    if self.match(TokenTag.PUNCTUATOR, "?"):
                        field_typus = TypusNullabilis(inner=field_typus)
                    field_nomen = self.expect_name().valor
                else:
                    nullable = bool(self.match(TokenTag.PUNCTUATOR, "?"))
                    if self.check_name():
                        field_typus = TypusNomen(nomen=first)
                        if nullable:
                            field_typus = TypusNullabilis(inner=field_typus)
                        field_nomen = self.expect_name().valor
                    else:
                        raise self.error("expected field type or name")

                valor: Expr = None
                if self.match(TokenTag.OPERATOR, "="):
                    valor = self.parse_expr(0)

                campi.append(CampusDecl(nomen=field_nomen, typus=field_typus, valor=valor,
                                        visibilitas=visibilitas, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtGenus(nomen=nomen, campi=campi, methodi=methodi, implet=implet,
                         generics=generics, publica=publica, abstractus=abstractus, locus=locus)

    def parse_pactum(self, publica: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "pactum")
        nomen = self.expect(TokenTag.IDENTIFIER).valor

        generics: list[str] = []
        if self.match(TokenTag.OPERATOR, "<"):
            while True:
                generics.append(self.expect(TokenTag.IDENTIFIER).valor)
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")

        self.expect(TokenTag.PUNCTUATOR, "{")

        methodi: list[PactumMethodus] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            loc = self.peek().locus
            self.expect(TokenTag.KEYWORD, "functio")
            asynca = bool(self.match(TokenTag.KEYWORD, "asynca"))
            name = self.expect(TokenTag.IDENTIFIER).valor
            self.expect(TokenTag.PUNCTUATOR, "(")
            params = self.parse_params()
            self.expect(TokenTag.PUNCTUATOR, ")")
            typus_reditus: Typus = None
            if self.match(TokenTag.OPERATOR, "->"):
                typus_reditus = self.parse_typus()
            methodi.append(PactumMethodus(nomen=name, params=params, typus_reditus=typus_reditus,
                                          asynca=asynca, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtPactum(nomen=nomen, methodi=methodi, generics=generics, publica=publica, locus=locus)

    def parse_ordo(self, publica: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "ordo")
        nomen = self.expect(TokenTag.IDENTIFIER).valor
        self.expect(TokenTag.PUNCTUATOR, "{")

        membra: list[OrdoMembrum] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            loc = self.peek().locus
            name = self.expect(TokenTag.IDENTIFIER).valor
            valor: str | None = None
            if self.match(TokenTag.OPERATOR, "="):
                tok = self.peek()
                if tok.tag == TokenTag.TEXTUS:
                    valor = f'"{tok.valor}"'
                else:
                    valor = tok.valor
                self.advance()
            membra.append(OrdoMembrum(nomen=name, valor=valor, locus=loc))
            self.match(TokenTag.PUNCTUATOR, ",")

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtOrdo(nomen=nomen, membra=membra, publica=publica, locus=locus)

    def parse_discretio(self, publica: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "discretio")
        nomen = self.expect(TokenTag.IDENTIFIER).valor

        generics: list[str] = []
        if self.match(TokenTag.OPERATOR, "<"):
            while True:
                generics.append(self.expect(TokenTag.IDENTIFIER).valor)
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")

        self.expect(TokenTag.PUNCTUATOR, "{")

        variantes: list[VariansDecl] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            loc = self.peek().locus
            name = self.expect(TokenTag.IDENTIFIER).valor
            campi: list[VariansCampus] = []

            if self.match(TokenTag.PUNCTUATOR, "{"):
                while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
                    typ_nomen = self.expect_name().valor
                    field_typus: Typus

                    if self.match(TokenTag.OPERATOR, "<"):
                        args: list[Typus] = []
                        while True:
                            args.append(self.parse_typus())
                            if not self.match(TokenTag.PUNCTUATOR, ","):
                                break
                        self.expect(TokenTag.OPERATOR, ">")
                        field_typus = TypusGenericus(nomen=typ_nomen, args=args)
                    else:
                        field_typus = TypusNomen(nomen=typ_nomen)

                    if self.match(TokenTag.PUNCTUATOR, "?"):
                        field_typus = TypusNullabilis(inner=field_typus)

                    field_nomen = self.expect_name().valor
                    campi.append(VariansCampus(nomen=field_nomen, typus=field_typus))

                    if not self.match(TokenTag.PUNCTUATOR, ","):
                        break
                self.expect(TokenTag.PUNCTUATOR, "}")

            variantes.append(VariansDecl(nomen=name, campi=campi, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtDiscretio(nomen=nomen, variantes=variantes, generics=generics, publica=publica, locus=locus)

    def parse_typus_alias(self, publica: bool) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "typus")
        nomen = self.expect(TokenTag.IDENTIFIER).valor
        self.expect(TokenTag.OPERATOR, "=")
        typus = self.parse_typus()
        return StmtTypusAlias(nomen=nomen, typus=typus, publica=publica, locus=locus)

    def parse_in_stmt(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "in")
        expr = self.parse_expr(0)
        corpus = self.parse_massa()
        return StmtIn(expr=expr, corpus=corpus, locus=locus)

    def parse_de_stmt(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "de")
        expr = self.parse_expr(0)

        if not self.check(TokenTag.KEYWORD, "fixum") and not self.check(TokenTag.KEYWORD, "varia"):
            raise self.error("expected 'fixum' or 'varia' after 'de' expression")
        self.advance()
        binding = self.expect(TokenTag.IDENTIFIER).valor
        corpus = self.parse_massa()
        return StmtIteratio(binding=binding, iter=expr, corpus=corpus, species="De",
                            asynca=False, locus=locus)

    def parse_massa(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.PUNCTUATOR, "{")
        corpus: list[Stmt] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            corpus.append(self.parse_stmt())
        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtMassa(corpus=corpus, locus=locus)

    def parse_body(self) -> Stmt:
        locus = self.peek().locus

        if self.check(TokenTag.PUNCTUATOR, "{"):
            return self.parse_massa()

        if self.match(TokenTag.KEYWORD, "ergo"):
            stmt = self.parse_stmt()
            return StmtMassa(corpus=[stmt], locus=locus)

        if self.match(TokenTag.KEYWORD, "reddit"):
            valor = self.parse_expr(0)
            return StmtMassa(corpus=[StmtRedde(valor=valor, locus=locus)], locus=locus)

        if self.match(TokenTag.KEYWORD, "iacit"):
            arg = self.parse_expr(0)
            return StmtMassa(corpus=[StmtIace(arg=arg, fatale=False, locus=locus)], locus=locus)

        if self.match(TokenTag.KEYWORD, "moritor"):
            arg = self.parse_expr(0)
            return StmtMassa(corpus=[StmtIace(arg=arg, fatale=True, locus=locus)], locus=locus)

        if self.match(TokenTag.KEYWORD, "tacet"):
            return StmtMassa(corpus=[], locus=locus)

        return self.parse_massa()

    def parse_si(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "si")
        return self._parse_si_body(locus)

    def _parse_si_body(self, locus: Locus) -> Stmt:
        cond = self.parse_expr(0)
        cons = self.parse_body()
        alt: Stmt = None
        if self.match(TokenTag.KEYWORD, "sin"):
            sin_locus = self.peek().locus
            alt = self._parse_si_body(sin_locus)
        elif self.match(TokenTag.KEYWORD, "secus"):
            if self.check(TokenTag.KEYWORD, "si"):
                alt = self.parse_si()
            else:
                alt = self.parse_body()
        return StmtSi(cond=cond, cons=cons, alt=alt, locus=locus)

    def parse_dum(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "dum")
        cond = self.parse_expr(0)
        corpus = self.parse_body()
        return StmtDum(cond=cond, corpus=corpus, locus=locus)

    def parse_fac(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "fac")
        corpus = self.parse_massa()
        self.expect(TokenTag.KEYWORD, "dum")
        cond = self.parse_expr(0)
        return StmtFacDum(corpus=corpus, cond=cond, locus=locus)

    def parse_elige(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "elige")
        discrim = self.parse_expr(0)
        self.expect(TokenTag.PUNCTUATOR, "{")

        casus: list[EligeCasus] = []
        default: Stmt = None

        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            if self.match(TokenTag.KEYWORD, "ceterum"):
                if self.check(TokenTag.PUNCTUATOR, "{"):
                    default = self.parse_massa()
                elif self.match(TokenTag.KEYWORD, "reddit"):
                    red_loc = self.peek().locus
                    valor = self.parse_expr(0)
                    default = StmtMassa(corpus=[StmtRedde(valor=valor, locus=red_loc)], locus=red_loc)
                else:
                    raise self.error("expected { or reddit after ceterum")
            else:
                self.expect(TokenTag.KEYWORD, "casu")
                loc = self.peek().locus
                cond = self.parse_expr(0)
                if self.check(TokenTag.PUNCTUATOR, "{"):
                    corpus = self.parse_massa()
                elif self.match(TokenTag.KEYWORD, "reddit"):
                    red_loc = self.peek().locus
                    valor = self.parse_expr(0)
                    corpus = StmtMassa(corpus=[StmtRedde(valor=valor, locus=red_loc)], locus=red_loc)
                else:
                    raise self.error("expected { or reddit after casu condition")
                casus.append(EligeCasus(cond=cond, corpus=corpus, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtElige(discrim=discrim, casus=casus, default=default, locus=locus)

    def parse_discerne(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "discerne")
        discrim: list[Expr] = [self.parse_expr(0)]
        while self.match(TokenTag.PUNCTUATOR, ","):
            discrim.append(self.parse_expr(0))
        self.expect(TokenTag.PUNCTUATOR, "{")

        casus: list[DiscerneCasus] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            loc = self.peek().locus

            if self.match(TokenTag.KEYWORD, "ceterum"):
                patterns = [VariansPattern(variant="_", bindings=[], alias=None, wildcard=True, locus=loc)]
                corpus = self.parse_massa()
                casus.append(DiscerneCasus(patterns=patterns, corpus=corpus, locus=loc))
                continue

            self.expect(TokenTag.KEYWORD, "casu")
            patterns: list[VariansPattern] = []

            while True:
                p_loc = self.peek().locus
                variant = self.expect(TokenTag.IDENTIFIER).valor
                alias: str | None = None
                bindings: list[str] = []
                wildcard = variant == "_"

                if self.match(TokenTag.KEYWORD, "ut"):
                    alias = self.expect_name().valor
                elif self.match(TokenTag.KEYWORD, "pro") or self.match(TokenTag.KEYWORD, "fixum"):
                    while True:
                        bindings.append(self.expect_name().valor)
                        if not self.match(TokenTag.PUNCTUATOR, ","):
                            break

                patterns.append(VariansPattern(variant=variant, bindings=bindings, alias=alias,
                                               wildcard=wildcard, locus=p_loc))

                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break

            corpus = self.parse_massa()
            casus.append(DiscerneCasus(patterns=patterns, corpus=corpus, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtDiscerne(discrim=discrim, casus=casus, locus=locus)

    def parse_custodi(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "custodi")
        self.expect(TokenTag.PUNCTUATOR, "{")

        clausulae: list[CustodiClausula] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            loc = self.peek().locus
            self.expect(TokenTag.KEYWORD, "si")
            cond = self.parse_expr(0)
            corpus = self.parse_massa()
            clausulae.append(CustodiClausula(cond=cond, corpus=corpus, locus=loc))

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtCustodi(clausulae=clausulae, locus=locus)

    def parse_tempta(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "tempta")
        corpus = self.parse_massa()

        cape: CapeClausula | None = None
        if self.match(TokenTag.KEYWORD, "cape"):
            loc = self.peek().locus
            param = self.expect(TokenTag.IDENTIFIER).valor
            body = self.parse_massa()
            cape = CapeClausula(param=param, corpus=body, locus=loc)

        demum: Stmt = None
        if self.match(TokenTag.KEYWORD, "demum"):
            demum = self.parse_massa()

        return StmtTempta(corpus=corpus, cape=cape, demum=demum, locus=locus)

    def parse_redde(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "redde")
        valor: Expr = None
        if (not self.check(TokenTag.EOF) and
            not self.check(TokenTag.PUNCTUATOR, "}") and
            not self._is_statement_keyword()):
            valor = self.parse_expr(0)
        return StmtRedde(valor=valor, locus=locus)

    def _is_statement_keyword(self) -> bool:
        if not self.check(TokenTag.KEYWORD):
            return False
        kw = self.peek().valor
        stmt_keywords = {
            "si", "sin", "secus", "dum", "fac", "ex", "de", "in", "elige", "discerne", "custodi",
            "tempta", "cape", "demum", "redde", "rumpe", "perge", "iace", "mori",
            "scribe", "vide", "mone", "adfirma", "functio", "genus", "pactum", "ordo",
            "discretio", "varia", "fixum", "figendum", "variandum", "incipit", "probandum", "proba",
            "casu", "ceterum", "reddit", "ergo", "tacet", "iacit", "moritor", "typus", "abstractus",
        }
        return kw in stmt_keywords

    def _is_declaration_keyword(self) -> bool:
        if not self.check(TokenTag.KEYWORD):
            return False
        kw = self.peek().valor
        decl_keywords = {
            "functio", "genus", "pactum", "ordo", "discretio", "typus",
            "varia", "fixum", "figendum", "variandum", "incipit", "probandum", "abstractus",
        }
        return kw in decl_keywords

    def parse_iace(self) -> Stmt:
        locus = self.peek().locus
        fatale = self.advance().valor == "mori"
        arg = self.parse_expr(0)
        return StmtIace(arg=arg, fatale=fatale, locus=locus)

    def parse_scribe(self) -> Stmt:
        locus = self.peek().locus
        kw = self.advance().valor
        gradus = "Scribe"
        if kw == "vide":
            gradus = "Vide"
        elif kw == "mone":
            gradus = "Mone"
        args: list[Expr] = []
        if (not self.check(TokenTag.EOF) and
            not self.check(TokenTag.PUNCTUATOR, "}") and
            not self._is_statement_keyword()):
            while True:
                args.append(self.parse_expr(0))
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
        return StmtScribe(args=args, gradus=gradus, locus=locus)

    def parse_adfirma(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "adfirma")
        cond = self.parse_expr(0)
        msg: Expr = None
        if self.match(TokenTag.PUNCTUATOR, ","):
            msg = self.parse_expr(0)
        return StmtAdfirma(cond=cond, msg=msg, locus=locus)

    def parse_rumpe(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "rumpe")
        return StmtRumpe(locus=locus)

    def parse_perge(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "perge")
        return StmtPerge(locus=locus)

    def parse_incipit(self) -> Stmt:
        locus = self.peek().locus
        kw = self.advance().valor
        asynca = kw == "incipiet"
        corpus = self.parse_massa()
        return StmtIncipit(corpus=corpus, asynca=asynca, locus=locus)

    def parse_probandum(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "probandum")
        nomen = self.expect(TokenTag.TEXTUS).valor
        self.expect(TokenTag.PUNCTUATOR, "{")

        corpus: list[Stmt] = []
        while not self.check(TokenTag.PUNCTUATOR, "}") and not self.check(TokenTag.EOF):
            corpus.append(self.parse_stmt())

        self.expect(TokenTag.PUNCTUATOR, "}")
        return StmtProbandum(nomen=nomen, corpus=corpus, locus=locus)

    def parse_proba(self) -> Stmt:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "proba")
        nomen = self.expect(TokenTag.TEXTUS).valor
        corpus = self.parse_massa()
        return StmtProba(nomen=nomen, corpus=corpus, locus=locus)

    def parse_expressia_stmt(self) -> Stmt:
        locus = self.peek().locus
        expr = self.parse_expr(0)
        return StmtExpressia(expr=expr, locus=locus)

    def parse_typus(self) -> Typus:
        typus = self._parse_typus_primary()

        if self.match(TokenTag.PUNCTUATOR, "?"):
            typus = TypusNullabilis(inner=typus)

        if self.match(TokenTag.OPERATOR, "|"):
            members: list[Typus] = [typus]
            while True:
                members.append(self._parse_typus_primary())
                if not self.match(TokenTag.OPERATOR, "|"):
                    break
            typus = TypusUnio(members=members)

        return typus

    def _parse_typus_primary(self) -> Typus:
        nomen = self.expect(TokenTag.IDENTIFIER).valor

        if self.match(TokenTag.OPERATOR, "<"):
            args: list[Typus] = []
            while True:
                args.append(self.parse_typus())
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
            self.expect(TokenTag.OPERATOR, ">")
            return TypusGenericus(nomen=nomen, args=args)

        return TypusNomen(nomen=nomen)

    def parse_expr(self, min_prec: int) -> Expr:
        left = self._parse_unary()

        while True:
            tok = self.peek()
            op = tok.valor
            prec = PRECEDENCE.get(op)
            if prec is None or prec < min_prec:
                break

            self.advance()

            if op == "qua":
                typus = self.parse_typus()
                left = ExprQua(expr=left, typus=typus, locus=tok.locus)
                continue
            if op == "innatum":
                typus = self.parse_typus()
                left = ExprInnatum(expr=left, typus=typus, locus=tok.locus)
                continue
            if op == "novum":
                typus = self.parse_typus()
                left = ExprPostfixNovum(expr=left, typus=typus, locus=tok.locus)
                continue

            if op in ("numeratum", "fractatum", "textatum", "bivalentum"):
                fallback: Expr = None
                if op in ("numeratum", "fractatum") and self.match(TokenTag.KEYWORD, "vel"):
                    fallback = self._parse_unary()
                left = ExprConversio(expr=left, species=op, fallback=fallback, locus=tok.locus)
                continue

            right = self.parse_expr(prec + 1)

            if op in ASSIGN_OPS:
                left = ExprAssignatio(signum=op, sin=left, dex=right, locus=tok.locus)
            else:
                left = ExprBinaria(signum=op, sin=left, dex=right, locus=tok.locus)

        if self.match(TokenTag.KEYWORD, "sic"):
            cons = self.parse_expr(0)
            self.expect(TokenTag.KEYWORD, "secus")
            alt = self.parse_expr(0)
            left = ExprCondicio(cond=left, cons=cons, alt=alt, locus=_expr_locus(left))

        return left

    def _parse_unary(self) -> Expr:
        tok = self.peek()

        if tok.tag in (TokenTag.OPERATOR, TokenTag.KEYWORD):
            if tok.valor in UNARY_OPS:
                non_expr = {
                    "qua", "innatum", "et", "aut", "vel", "sic", "secus", "inter", "intra",
                    "perge", "rumpe", "redde", "reddit", "iace", "mori",
                    "si", "secussi", "dum", "ex", "de", "elige", "discerne", "custodi", "tempta",
                    "functio", "genus", "pactum", "ordo", "discretio",
                    "casu", "ceterum", "importa", "incipit", "incipiet", "probandum", "proba",
                }
                next_tok = self.peek(1)
                can_be_unary = (
                    next_tok.tag == TokenTag.IDENTIFIER or
                    (next_tok.tag == TokenTag.KEYWORD and next_tok.valor not in non_expr) or
                    next_tok.tag == TokenTag.NUMERUS or
                    next_tok.tag == TokenTag.TEXTUS or
                    next_tok.valor in ("(", "[", "{") or
                    next_tok.valor in UNARY_OPS
                )

                if can_be_unary:
                    self.advance()
                    arg = self._parse_unary()
                    return ExprUnaria(signum=tok.valor, arg=arg, locus=tok.locus)

        if self.match(TokenTag.KEYWORD, "cede"):
            arg = self._parse_unary()
            return ExprCede(arg=arg, locus=tok.locus)

        return self._parse_postfix()

    def _parse_postfix(self) -> Expr:
        expr = self._parse_primary()

        while True:
            tok = self.peek()

            if self.match(TokenTag.PUNCTUATOR, "("):
                args = self._parse_args()
                self.expect(TokenTag.PUNCTUATOR, ")")
                expr = ExprVocatio(callee=expr, args=args, locus=tok.locus)
                continue

            if self.match(TokenTag.PUNCTUATOR, "."):
                prop = ExprLittera(species=LitteraSpecies.TEXTUS, valor=self.expect_name().valor,
                                   locus=self.peek().locus)
                expr = ExprMembrum(obj=expr, prop=prop, computed=False, non_null=False, locus=tok.locus)
                continue

            if self.match(TokenTag.OPERATOR, "!.") or (tok.valor == "!" and self.peek(1).valor == "."):
                if tok.valor == "!":
                    self.advance()
                    self.advance()
                prop = ExprLittera(species=LitteraSpecies.TEXTUS, valor=self.expect_name().valor,
                                   locus=self.peek().locus)
                expr = ExprMembrum(obj=expr, prop=prop, computed=False, non_null=True, locus=tok.locus)
                continue

            if tok.valor == "!" and self.peek(1).valor == "[":
                self.advance()
                self.advance()
                prop = self.parse_expr(0)
                self.expect(TokenTag.PUNCTUATOR, "]")
                expr = ExprMembrum(obj=expr, prop=prop, computed=True, non_null=True, locus=tok.locus)
                continue

            if self.match(TokenTag.PUNCTUATOR, "["):
                prop = self.parse_expr(0)
                self.expect(TokenTag.PUNCTUATOR, "]")
                expr = ExprMembrum(obj=expr, prop=prop, computed=True, non_null=False, locus=tok.locus)
                continue

            break

        return expr

    def _parse_primary(self) -> Expr:
        tok = self.peek()

        if self.match(TokenTag.PUNCTUATOR, "("):
            expr = self.parse_expr(0)
            self.expect(TokenTag.PUNCTUATOR, ")")
            return expr

        if self.match(TokenTag.PUNCTUATOR, "["):
            elementa: list[Expr] = []
            if not self.check(TokenTag.PUNCTUATOR, "]"):
                while True:
                    elementa.append(self.parse_expr(0))
                    if not self.match(TokenTag.PUNCTUATOR, ","):
                        break
            self.expect(TokenTag.PUNCTUATOR, "]")
            return ExprSeries(elementa=elementa, locus=tok.locus)

        if self.match(TokenTag.PUNCTUATOR, "{"):
            props: list[ObiectumProp] = []
            if not self.check(TokenTag.PUNCTUATOR, "}"):
                while True:
                    loc = self.peek().locus
                    key: Expr
                    computed = False

                    if self.match(TokenTag.PUNCTUATOR, "["):
                        key = self.parse_expr(0)
                        self.expect(TokenTag.PUNCTUATOR, "]")
                        computed = True
                    elif self.check(TokenTag.TEXTUS):
                        str_key = self.advance().valor
                        key = ExprLittera(species=LitteraSpecies.TEXTUS, valor=str_key, locus=loc)
                    else:
                        name = self.expect_name().valor
                        key = ExprLittera(species=LitteraSpecies.TEXTUS, valor=name, locus=loc)

                    valor: Expr
                    shorthand = False

                    if self.match(TokenTag.PUNCTUATOR, ":"):
                        valor = self.parse_expr(0)
                    else:
                        shorthand = True
                        key_name = key.valor if isinstance(key, ExprLittera) else ""
                        valor = ExprNomen(valor=key_name, locus=loc)

                    props.append(ObiectumProp(key=key, valor=valor, shorthand=shorthand,
                                              computed=computed, locus=loc))

                    if not self.match(TokenTag.PUNCTUATOR, ","):
                        break
            self.expect(TokenTag.PUNCTUATOR, "}")
            return ExprObiectum(props=props, locus=tok.locus)

        if tok.tag == TokenTag.KEYWORD:
            match tok.valor:
                case "verum":
                    self.advance()
                    return ExprLittera(species=LitteraSpecies.VERUM, valor="true", locus=tok.locus)
                case "falsum":
                    self.advance()
                    return ExprLittera(species=LitteraSpecies.FALSUM, valor="false", locus=tok.locus)
                case "nihil":
                    self.advance()
                    return ExprLittera(species=LitteraSpecies.NIHIL, valor="null", locus=tok.locus)
                case "ego":
                    self.advance()
                    return ExprEgo(locus=tok.locus)
                case "novum":
                    return self._parse_novum()
                case "finge":
                    return self._parse_finge()
                case "clausura":
                    return self._parse_clausura()
                case "scriptum":
                    return self._parse_scriptum()
                case _:
                    self.advance()
                    return ExprNomen(valor=tok.valor, locus=tok.locus)

        if tok.tag == TokenTag.NUMERUS:
            self.advance()
            species = LitteraSpecies.NUMERUS
            if "." in tok.valor:
                species = LitteraSpecies.FRACTUS
            return ExprLittera(species=species, valor=tok.valor, locus=tok.locus)

        if tok.tag == TokenTag.TEXTUS:
            self.advance()
            return ExprLittera(species=LitteraSpecies.TEXTUS, valor=tok.valor, locus=tok.locus)

        if tok.tag == TokenTag.IDENTIFIER:
            self.advance()
            return ExprNomen(valor=tok.valor, locus=tok.locus)

        raise self.error(f"unexpected token '{tok.valor}'")

    def _parse_args(self) -> list[Expr]:
        args: list[Expr] = []
        if self.check(TokenTag.PUNCTUATOR, ")"):
            return args

        while True:
            args.append(self.parse_expr(0))
            if not self.match(TokenTag.PUNCTUATOR, ","):
                break

        return args

    def _parse_novum(self) -> Expr:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "novum")
        callee = self._parse_primary()
        args: list[Expr] = []
        if self.match(TokenTag.PUNCTUATOR, "("):
            args = self._parse_args()
            self.expect(TokenTag.PUNCTUATOR, ")")
        init: Expr = None
        if self.check(TokenTag.PUNCTUATOR, "{"):
            init = self._parse_primary()
        return ExprNovum(callee=callee, args=args, init=init, locus=locus)

    def _parse_finge(self) -> Expr:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "finge")
        variant = self.expect(TokenTag.IDENTIFIER).valor
        self.expect(TokenTag.PUNCTUATOR, "{")

        campi: list[ObiectumProp] = []
        if not self.check(TokenTag.PUNCTUATOR, "}"):
            while True:
                loc = self.peek().locus
                name = self.expect_name().valor
                key = ExprLittera(species=LitteraSpecies.TEXTUS, valor=name, locus=loc)
                self.expect(TokenTag.PUNCTUATOR, ":")
                valor = self.parse_expr(0)
                campi.append(ObiectumProp(key=key, valor=valor, shorthand=False, computed=False, locus=loc))
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break
        self.expect(TokenTag.PUNCTUATOR, "}")

        typus: Typus = None
        if self.match(TokenTag.KEYWORD, "qua"):
            typus = self.parse_typus()

        return ExprFinge(variant=variant, campi=campi, typus=typus, locus=locus)

    def _parse_clausura(self) -> Expr:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "clausura")

        params: list[Param] = []
        if self.check(TokenTag.IDENTIFIER):
            while True:
                loc = self.peek().locus
                nomen = self.expect(TokenTag.IDENTIFIER).valor
                typus: Typus = None
                if self.match(TokenTag.PUNCTUATOR, ":"):
                    typus = self.parse_typus()
                params.append(Param(nomen=nomen, typus=typus, default=None, rest=False, locus=loc))
                if not self.match(TokenTag.PUNCTUATOR, ","):
                    break

        corpus: Any
        if self.check(TokenTag.PUNCTUATOR, "{"):
            corpus = self.parse_massa()
        else:
            self.expect(TokenTag.PUNCTUATOR, ":")
            corpus = self.parse_expr(0)

        return ExprClausura(params=params, corpus=corpus, locus=locus)

    def _parse_scriptum(self) -> Expr:
        locus = self.peek().locus
        self.expect(TokenTag.KEYWORD, "scriptum")
        self.expect(TokenTag.PUNCTUATOR, "(")
        template = self.expect(TokenTag.TEXTUS).valor
        args: list[Expr] = []
        while self.match(TokenTag.PUNCTUATOR, ","):
            args.append(self.parse_expr(0))
        self.expect(TokenTag.PUNCTUATOR, ")")
        return ExprScriptum(template=template, args=args, locus=locus)


def _expr_locus(expr: Expr) -> Locus:
    """Extract the location from an expression."""
    if hasattr(expr, "locus"):
        return expr.locus
    return Locus()


def prepare(tokens: list[Token]) -> list[Token]:
    """Filter out comments and newlines from token stream."""
    return [tok for tok in tokens if tok.tag not in (TokenTag.COMMENT, TokenTag.NEWLINE)]


def parse(tokens: list[Token], filename: str = "<stdin>") -> Modulus:
    """Parse tokens into a module."""
    return Parser(tokens, filename).parse()
