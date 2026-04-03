"""Semantic analysis for Faber AST."""

from __future__ import annotations
from typing import Any

from errors import Locus
from nodes import (
    Modulus, Typus, TypusNomen, TypusNullabilis, TypusGenericus, TypusFunctio, TypusUnio, TypusLitteralis,
    Expr, ExprNomen, ExprEgo, ExprLittera, ExprBinaria, ExprUnaria, ExprAssignatio,
    ExprCondicio, ExprVocatio, ExprMembrum, ExprSeries, ExprObiectum, ExprClausura,
    ExprNovum, ExprCede, ExprQua, ExprInnatum, ExprPostfixNovum, ExprFinge, ExprScriptum,
    ExprAmbitus, ExprConversio,
    Stmt, StmtMassa, StmtExpressia, StmtVaria, StmtFunctio, StmtGenus, StmtPactum,
    StmtOrdo, StmtDiscretio, StmtImporta, StmtTypusAlias, StmtSi, StmtDum, StmtFacDum,
    StmtIteratio, StmtElige, StmtDiscerne, StmtCustodi, StmtTempta, StmtRedde, StmtIace,
    StmtScribe, StmtAdfirma, StmtIncipit, StmtIn,
    Param, PactumMethodus, VariansPattern,
    LitteraSpecies, VariaSpecies,
)
from scope import (
    SemanticContext, ScopusSpecies, Symbolum, SymbolSpecies,
    SemanticTypus, SemPrimitivus, SemLista, SemTabula, SemCopia, SemFunctio,
    SemGenus, SemOrdo, SemDiscretio, SemPactum, SemUsitatum, SemUnio,
    TEXTUS, NUMERUS, FRACTUS, BIVALENS, NIHIL, VACUUM, IGNOTUM, nullabilis,
)


def analyze(mod: Modulus) -> SemanticContext:
    """Perform semantic analysis on a parsed module."""
    ctx = SemanticContext()

    # Pass 1: Collect all type declarations
    for stmt in mod.corpus:
        _collect_declaration(ctx, stmt)

    # Pass 2: Resolve types in all declarations and bodies
    for stmt in mod.corpus:
        _analyze_statement(ctx, stmt)

    return ctx


def _collect_declaration(ctx: SemanticContext, stmt: Stmt) -> None:
    """Register type declarations in the first pass."""
    if isinstance(stmt, StmtGenus):
        _collect_genus(ctx, stmt)
    elif isinstance(stmt, StmtOrdo):
        _collect_ordo(ctx, stmt)
    elif isinstance(stmt, StmtDiscretio):
        _collect_discretio(ctx, stmt)
    elif isinstance(stmt, StmtPactum):
        _collect_pactum(ctx, stmt)
    elif isinstance(stmt, StmtFunctio):
        _collect_functio(ctx, stmt)


def _collect_genus(ctx: SemanticContext, s: StmtGenus) -> None:
    genus = SemGenus(nomen=s.nomen, agri={}, methodi={})

    for campo in s.campi:
        if campo.typus is not None:
            genus.agri[campo.nomen] = _resolve_typus_annotatio(ctx, campo.typus)
        else:
            genus.agri[campo.nomen] = IGNOTUM

    for method in s.methodi:
        if isinstance(method, StmtFunctio):
            genus.methodi[method.nomen] = _resolve_functio_typus(ctx, method)

    ctx.genus_registry[s.nomen] = genus
    ctx.register_typus(s.nomen, genus)

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=genus,
        species=SymbolSpecies.GENUS,
        locus=s.locus,
        node=s,
    ))


def _collect_ordo(ctx: SemanticContext, s: StmtOrdo) -> None:
    ordo = SemOrdo(nomen=s.nomen, membra={})

    for i, m in enumerate(s.membra):
        ordo.membra[m.nomen] = i

    ctx.ordo_registry[s.nomen] = ordo
    ctx.register_typus(s.nomen, ordo)

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=ordo,
        species=SymbolSpecies.ORDO,
        locus=s.locus,
        node=s,
    ))


def _collect_discretio(ctx: SemanticContext, s: StmtDiscretio) -> None:
    disc = SemDiscretio(nomen=s.nomen, variantes={})

    for v in s.variantes:
        variant = SemGenus(nomen=v.nomen, agri={})
        for f in v.campi:
            if f.typus is not None:
                variant.agri[f.nomen] = _resolve_typus_annotatio(ctx, f.typus)
            else:
                variant.agri[f.nomen] = IGNOTUM
        disc.variantes[v.nomen] = variant

        ctx.definie(Symbolum(
            nomen=v.nomen,
            typus=variant,
            species=SymbolSpecies.VARIANS,
            locus=v.locus,
            node=v,
        ))

    ctx.disc_registry[s.nomen] = disc
    ctx.register_typus(s.nomen, disc)

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=disc,
        species=SymbolSpecies.DISCRETIO,
        locus=s.locus,
        node=s,
    ))


def _collect_pactum(ctx: SemanticContext, s: StmtPactum) -> None:
    pactum = SemPactum(nomen=s.nomen, methodi={})

    for m in s.methodi:
        pactum.methodi[m.nomen] = _resolve_pactum_method_typus(ctx, m)

    ctx.register_typus(s.nomen, pactum)

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=pactum,
        species=SymbolSpecies.PACTUM,
        locus=s.locus,
        node=s,
    ))


def _collect_functio(ctx: SemanticContext, s: StmtFunctio) -> None:
    if s.externa:
        return

    func_typus = _resolve_functio_typus(ctx, s)

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=func_typus,
        species=SymbolSpecies.FUNCTIO,
        locus=s.locus,
        node=s,
    ))


def _resolve_functio_typus(ctx: SemanticContext, s: StmtFunctio) -> SemFunctio:
    params = []
    for p in s.params:
        if p.typus is not None:
            params.append(_resolve_typus_annotatio(ctx, p.typus))
        else:
            params.append(IGNOTUM)

    reditus = None
    if s.typus_reditus is not None:
        reditus = _resolve_typus_annotatio(ctx, s.typus_reditus)

    return SemFunctio(params=params, reditus=reditus)


def _resolve_pactum_method_typus(ctx: SemanticContext, m: PactumMethodus) -> SemFunctio:
    params = []
    for p in m.params:
        if p.typus is not None:
            params.append(_resolve_typus_annotatio(ctx, p.typus))
        else:
            params.append(IGNOTUM)

    reditus = None
    if m.typus_reditus is not None:
        reditus = _resolve_typus_annotatio(ctx, m.typus_reditus)

    return SemFunctio(params=params, reditus=reditus)


def _resolve_typus_annotatio(ctx: SemanticContext, typus: Typus) -> SemanticTypus:
    """Convert an AST type annotation to a SemanticTypus."""
    if typus is None:
        return IGNOTUM

    if isinstance(typus, TypusNomen):
        return ctx.resolve_typus_nomen(typus.nomen)

    if isinstance(typus, TypusNullabilis):
        inner = _resolve_typus_annotatio(ctx, typus.inner)
        return nullabilis(inner)

    if isinstance(typus, TypusGenericus):
        base = typus.nomen
        if base == "lista":
            elem = IGNOTUM
            if typus.args:
                elem = _resolve_typus_annotatio(ctx, typus.args[0])
            return SemLista(elementum=elem)
        if base == "tabula":
            clavis = TEXTUS
            valor = IGNOTUM
            if typus.args:
                clavis = _resolve_typus_annotatio(ctx, typus.args[0])
            if len(typus.args) > 1:
                valor = _resolve_typus_annotatio(ctx, typus.args[1])
            return SemTabula(clavis=clavis, valor=valor)
        if base in ("copia", "collectio"):
            elem = IGNOTUM
            if typus.args:
                elem = _resolve_typus_annotatio(ctx, typus.args[0])
            return SemCopia(elementum=elem)
        return SemUsitatum(nomen=base)

    if isinstance(typus, TypusFunctio):
        params = [_resolve_typus_annotatio(ctx, p) for p in typus.params]
        reditus = _resolve_typus_annotatio(ctx, typus.returns) if typus.returns else None
        return SemFunctio(params=params, reditus=reditus)

    if isinstance(typus, TypusUnio):
        membra = [_resolve_typus_annotatio(ctx, m) for m in typus.members]
        return SemUnio(membra=membra)

    if isinstance(typus, TypusLitteralis):
        return TEXTUS

    return IGNOTUM


def _analyze_statement(ctx: SemanticContext, stmt: Stmt) -> None:
    """Perform semantic analysis on a statement."""
    if isinstance(stmt, StmtMassa):
        ctx.intra_scopum(ScopusSpecies.MASSA)
        for inner in stmt.corpus:
            _analyze_statement(ctx, inner)
        ctx.exi_scopum()

    elif isinstance(stmt, StmtVaria):
        _analyze_varia(ctx, stmt)

    elif isinstance(stmt, StmtFunctio):
        _analyze_functio(ctx, stmt)

    elif isinstance(stmt, StmtGenus):
        _analyze_genus(ctx, stmt)

    elif isinstance(stmt, StmtSi):
        _analyze_expression(ctx, stmt.cond)
        _analyze_statement(ctx, stmt.cons)
        if stmt.alt is not None:
            _analyze_statement(ctx, stmt.alt)

    elif isinstance(stmt, StmtDum):
        _analyze_expression(ctx, stmt.cond)
        _analyze_statement(ctx, stmt.corpus)

    elif isinstance(stmt, StmtFacDum):
        _analyze_statement(ctx, stmt.corpus)
        _analyze_expression(ctx, stmt.cond)

    elif isinstance(stmt, StmtIteratio):
        _analyze_expression(ctx, stmt.iter)
        ctx.intra_scopum(ScopusSpecies.MASSA)
        iter_type = ctx.get_expr_type(stmt.iter)
        elem_type = IGNOTUM
        if isinstance(iter_type, SemLista):
            elem_type = iter_type.elementum
        ctx.definie(Symbolum(nomen=stmt.binding, typus=elem_type, species=SymbolSpecies.VARIABILIS))
        _analyze_statement(ctx, stmt.corpus)
        ctx.exi_scopum()

    elif isinstance(stmt, StmtElige):
        _analyze_expression(ctx, stmt.discrim)
        for c in stmt.casus:
            _analyze_expression(ctx, c.cond)
            _analyze_statement(ctx, c.corpus)
        if stmt.default is not None:
            _analyze_statement(ctx, stmt.default)

    elif isinstance(stmt, StmtDiscerne):
        for d in stmt.discrim:
            _analyze_expression(ctx, d)
        for c in stmt.casus:
            ctx.intra_scopum(ScopusSpecies.MASSA)
            for p in c.patterns:
                _analyze_pattern(ctx, p, stmt.discrim)
            _analyze_statement(ctx, c.corpus)
            ctx.exi_scopum()

    elif isinstance(stmt, StmtRedde):
        if stmt.valor is not None:
            _analyze_expression(ctx, stmt.valor)

    elif isinstance(stmt, StmtExpressia):
        _analyze_expression(ctx, stmt.expr)

    elif isinstance(stmt, StmtScribe):
        for arg in stmt.args:
            _analyze_expression(ctx, arg)

    elif isinstance(stmt, StmtAdfirma):
        _analyze_expression(ctx, stmt.cond)
        if stmt.msg is not None:
            _analyze_expression(ctx, stmt.msg)

    elif isinstance(stmt, StmtIace):
        if stmt.arg is not None:
            _analyze_expression(ctx, stmt.arg)

    elif isinstance(stmt, StmtCustodi):
        for c in stmt.clausulae:
            _analyze_expression(ctx, c.cond)
            _analyze_statement(ctx, c.corpus)

    elif isinstance(stmt, StmtIncipit):
        _analyze_statement(ctx, stmt.corpus)

    elif isinstance(stmt, StmtTempta):
        _analyze_statement(ctx, stmt.corpus)
        if stmt.cape is not None:
            _analyze_statement(ctx, stmt.cape.corpus)
        if stmt.demum is not None:
            _analyze_statement(ctx, stmt.demum)

    elif isinstance(stmt, StmtTypusAlias):
        target_type = _resolve_typus_annotatio(ctx, stmt.typus)
        ctx.register_typus(stmt.nomen, target_type)
        ctx.definie(Symbolum(
            nomen=stmt.nomen,
            typus=target_type,
            species=SymbolSpecies.TYPUS,
            locus=stmt.locus,
            node=stmt,
        ))

    elif isinstance(stmt, StmtIn):
        _analyze_expression(ctx, stmt.expr)
        _analyze_statement(ctx, stmt.corpus)


def _analyze_varia(ctx: SemanticContext, s: StmtVaria) -> None:
    if s.externa:
        return

    var_type: SemanticTypus = IGNOTUM

    if s.typus is not None:
        var_type = _resolve_typus_annotatio(ctx, s.typus)

    if s.valor is not None:
        _analyze_expression(ctx, s.valor)
        init_type = ctx.get_expr_type(s.valor)
        if s.typus is None:
            var_type = init_type

    ctx.definie(Symbolum(
        nomen=s.nomen,
        typus=var_type,
        species=SymbolSpecies.VARIABILIS,
        mutabilis=s.species == VariaSpecies.VARIA,
        locus=s.locus,
        node=s,
    ))


def _analyze_functio(ctx: SemanticContext, s: StmtFunctio) -> None:
    if s.externa or s.corpus is None:
        return

    ctx.intra_scopum(ScopusSpecies.FUNCTIO, s.nomen)

    for p in s.params:
        param_type = IGNOTUM
        if p.typus is not None:
            param_type = _resolve_typus_annotatio(ctx, p.typus)
        ctx.definie(Symbolum(
            nomen=p.nomen,
            typus=param_type,
            species=SymbolSpecies.PARAMETRUM,
        ))

    _analyze_statement(ctx, s.corpus)

    ctx.exi_scopum()


def _analyze_genus(ctx: SemanticContext, s: StmtGenus) -> None:
    for method in s.methodi:
        if isinstance(method, StmtFunctio):
            ctx.intra_scopum(ScopusSpecies.GENUS, s.nomen)
            genus = ctx.genus_registry.get(s.nomen)
            ctx.definie(Symbolum(
                nomen="ego",
                typus=genus,
                species=SymbolSpecies.VARIABILIS,
            ))
            _analyze_functio(ctx, method)
            ctx.exi_scopum()


def _analyze_pattern(ctx: SemanticContext, p: VariansPattern, discrim: list[Expr]) -> None:
    if p.wildcard:
        return

    if discrim:
        discrim_type = ctx.get_expr_type(discrim[0])
        if isinstance(discrim_type, SemDiscretio):
            if p.variant in discrim_type.variantes:
                variant = discrim_type.variantes[p.variant]
                for b in p.bindings:
                    field_type = variant.agri.get(b, IGNOTUM)
                    ctx.definie(Symbolum(
                        nomen=b,
                        typus=field_type,
                        species=SymbolSpecies.VARIABILIS,
                    ))

    if p.alias is not None:
        ctx.definie(Symbolum(
            nomen=p.alias,
            typus=IGNOTUM,
            species=SymbolSpecies.VARIABILIS,
        ))


def _analyze_expression(ctx: SemanticContext, expr: Expr) -> SemanticTypus:
    """Resolve the type of an expression and record it."""
    if expr is None:
        return IGNOTUM

    result: SemanticTypus

    if isinstance(expr, ExprLittera):
        result = _analyze_littera(expr)

    elif isinstance(expr, ExprNomen):
        result = _analyze_nomen(ctx, expr)

    elif isinstance(expr, ExprEgo):
        sym = ctx.quaere("ego")
        result = sym.typus if sym else IGNOTUM

    elif isinstance(expr, ExprBinaria):
        result = _analyze_binaria(ctx, expr)

    elif isinstance(expr, ExprUnaria):
        result = _analyze_unaria(ctx, expr)

    elif isinstance(expr, ExprAssignatio):
        _analyze_expression(ctx, expr.sin)
        _analyze_expression(ctx, expr.dex)
        result = ctx.get_expr_type(expr.sin)

    elif isinstance(expr, ExprCondicio):
        _analyze_expression(ctx, expr.cond)
        cons_type = _analyze_expression(ctx, expr.cons)
        _analyze_expression(ctx, expr.alt)
        result = cons_type

    elif isinstance(expr, ExprVocatio):
        result = _analyze_vocatio(ctx, expr)

    elif isinstance(expr, ExprMembrum):
        result = _analyze_membrum(ctx, expr)

    elif isinstance(expr, ExprSeries):
        result = _analyze_series(ctx, expr)

    elif isinstance(expr, ExprObiectum):
        result = _analyze_obiectum(ctx, expr)

    elif isinstance(expr, ExprClausura):
        result = _analyze_clausura(ctx, expr)

    elif isinstance(expr, ExprNovum):
        result = _analyze_novum(ctx, expr)

    elif isinstance(expr, ExprFinge):
        result = _analyze_finge(ctx, expr)

    elif isinstance(expr, ExprCede):
        result = _analyze_expression(ctx, expr.arg)

    elif isinstance(expr, ExprQua):
        result = _resolve_typus_annotatio(ctx, expr.typus)

    elif isinstance(expr, ExprInnatum):
        result = _resolve_typus_annotatio(ctx, expr.typus)

    elif isinstance(expr, ExprPostfixNovum):
        result = _resolve_typus_annotatio(ctx, expr.typus)

    elif isinstance(expr, ExprScriptum):
        for arg in expr.args:
            _analyze_expression(ctx, arg)
        result = TEXTUS

    elif isinstance(expr, ExprAmbitus):
        _analyze_expression(ctx, expr.start)
        _analyze_expression(ctx, expr.end)
        result = SemLista(elementum=NUMERUS)

    elif isinstance(expr, ExprConversio):
        _analyze_expression(ctx, expr.expr)
        if expr.fallback is not None:
            _analyze_expression(ctx, expr.fallback)
        match expr.species:
            case "numeratum":
                result = NUMERUS
            case "fractatum":
                result = FRACTUS
            case "textatum":
                result = TEXTUS
            case "bivalentum":
                result = BIVALENS
            case _:
                result = IGNOTUM

    else:
        result = IGNOTUM

    ctx.set_expr_type(expr, result)
    return result


def _analyze_littera(e: ExprLittera) -> SemanticTypus:
    match e.species:
        case LitteraSpecies.TEXTUS:
            return TEXTUS
        case LitteraSpecies.NUMERUS:
            return NUMERUS
        case LitteraSpecies.FRACTUS:
            return FRACTUS
        case LitteraSpecies.VERUM | LitteraSpecies.FALSUM:
            return BIVALENS
        case LitteraSpecies.NIHIL:
            return NIHIL
        case _:
            return IGNOTUM


def _analyze_nomen(ctx: SemanticContext, e: ExprNomen) -> SemanticTypus:
    sym = ctx.quaere(e.valor)
    if sym is not None:
        return sym.typus

    t = ctx.resolve_typus_nomen(e.valor)
    if t is not None and not isinstance(t, SemUsitatum):
        return t

    ctx.error(f"undefined identifier: {e.valor}", e.locus)
    return IGNOTUM


def _analyze_binaria(ctx: SemanticContext, e: ExprBinaria) -> SemanticTypus:
    left_type = _analyze_expression(ctx, e.sin)
    right_type = _analyze_expression(ctx, e.dex)

    if e.signum in ("+", "-", "*", "/", "%"):
        if _is_numeric(left_type) and _is_numeric(right_type):
            if _is_fractus(left_type) or _is_fractus(right_type):
                return FRACTUS
            return NUMERUS
        if e.signum == "+" and _is_textus(left_type):
            return TEXTUS
        return IGNOTUM

    if e.signum in ("==", "!=", "<", ">", "<=", ">="):
        return BIVALENS

    if e.signum in ("et", "aut", "&&", "||"):
        return BIVALENS

    if e.signum == "vel":
        return left_type

    return IGNOTUM


def _analyze_unaria(ctx: SemanticContext, e: ExprUnaria) -> SemanticTypus:
    arg_type = _analyze_expression(ctx, e.arg)

    if e.signum in ("non", "!"):
        return BIVALENS
    if e.signum in ("nihil", "nonnihil", "nulla", "nonnulla"):
        return BIVALENS
    if e.signum in ("-", "+", "~", "positivum", "negativum"):
        return arg_type
    return arg_type


def _analyze_vocatio(ctx: SemanticContext, e: ExprVocatio) -> SemanticTypus:
    for arg in e.args:
        _analyze_expression(ctx, arg)

    callee_type = _analyze_expression(ctx, e.callee)

    if isinstance(callee_type, SemFunctio):
        return callee_type.reditus if callee_type.reditus else VACUUM

    if isinstance(e.callee, ExprMembrum):
        obj_type = ctx.get_expr_type(e.callee.obj)
        if isinstance(obj_type, SemGenus):
            if isinstance(e.callee.prop, ExprLittera):
                method = obj_type.methodi.get(e.callee.prop.valor)
                if method:
                    return method.reditus if method.reditus else VACUUM

    if isinstance(e.callee, ExprNomen):
        genus = ctx.genus_registry.get(e.callee.valor)
        if genus:
            return genus

    return IGNOTUM


def _analyze_membrum(ctx: SemanticContext, e: ExprMembrum) -> SemanticTypus:
    obj_type = _analyze_expression(ctx, e.obj)

    if e.computed:
        _analyze_expression(ctx, e.prop)
        if isinstance(obj_type, SemLista):
            return obj_type.elementum
        if isinstance(obj_type, SemTabula):
            return obj_type.valor
        if isinstance(obj_type, SemCopia):
            return BIVALENS
        return IGNOTUM

    prop_name = ""
    if isinstance(e.prop, ExprLittera):
        prop_name = e.prop.valor
    else:
        return IGNOTUM

    if prop_name == "longitudo":
        if isinstance(obj_type, (SemLista, SemTabula, SemCopia)):
            return NUMERUS
        if _is_textus(obj_type):
            return NUMERUS

    if prop_name in ("primus", "ultimus"):
        if isinstance(obj_type, SemLista):
            return obj_type.elementum

    if isinstance(obj_type, SemGenus):
        if prop_name in obj_type.agri:
            return obj_type.agri[prop_name]
        if prop_name in obj_type.methodi:
            return obj_type.methodi[prop_name]

    if isinstance(obj_type, SemUsitatum):
        genus = ctx.genus_registry.get(obj_type.nomen)
        if genus:
            if prop_name in genus.agri:
                return genus.agri[prop_name]
            if prop_name in genus.methodi:
                return genus.methodi[prop_name]

    if isinstance(obj_type, SemOrdo):
        if prop_name in obj_type.membra:
            return obj_type

    if isinstance(obj_type, SemDiscretio):
        if prop_name in obj_type.variantes:
            return obj_type.variantes[prop_name]

    return IGNOTUM


def _analyze_series(ctx: SemanticContext, e: ExprSeries) -> SemanticTypus:
    elem_type: SemanticTypus = IGNOTUM

    for i, elem in enumerate(e.elementa):
        t = _analyze_expression(ctx, elem)
        if i == 0:
            elem_type = t

    return SemLista(elementum=elem_type)


def _analyze_obiectum(ctx: SemanticContext, e: ExprObiectum) -> SemanticTypus:
    fields: dict[str, SemanticTypus] = {}

    for p in e.props:
        value_type = _analyze_expression(ctx, p.valor)
        if isinstance(p.key, ExprLittera):
            fields[p.key.valor] = value_type

    return SemGenus(nomen="", agri=fields)


def _analyze_clausura(ctx: SemanticContext, e: ExprClausura) -> SemanticTypus:
    params: list[SemanticTypus] = []

    ctx.intra_scopum(ScopusSpecies.FUNCTIO)

    for p in e.params:
        param_type = IGNOTUM
        if p.typus is not None:
            param_type = _resolve_typus_annotatio(ctx, p.typus)
        params.append(param_type)

        ctx.definie(Symbolum(
            nomen=p.nomen,
            typus=param_type,
            species=SymbolSpecies.PARAMETRUM,
        ))

    reditus: SemanticTypus = None
    if isinstance(e.corpus, Stmt):
        _analyze_statement(ctx, e.corpus)
    elif isinstance(e.corpus, Expr):
        reditus = _analyze_expression(ctx, e.corpus)

    ctx.exi_scopum()

    return SemFunctio(params=params, reditus=reditus)


def _analyze_novum(ctx: SemanticContext, e: ExprNovum) -> SemanticTypus:
    for arg in e.args:
        _analyze_expression(ctx, arg)

    if e.init is not None:
        _analyze_expression(ctx, e.init)

    if isinstance(e.callee, ExprNomen):
        genus = ctx.genus_registry.get(e.callee.valor)
        if genus:
            return genus
        sym = ctx.quaere(e.callee.valor)
        if sym and sym.species == SymbolSpecies.VARIANS:
            return sym.typus
        return SemUsitatum(nomen=e.callee.valor)

    return IGNOTUM


def _analyze_finge(ctx: SemanticContext, e: ExprFinge) -> SemanticTypus:
    for p in e.campi:
        _analyze_expression(ctx, p.valor)

    sym = ctx.quaere(e.variant)
    if sym and sym.species == SymbolSpecies.VARIANS:
        return sym.typus

    for disc in ctx.disc_registry.values():
        if e.variant in disc.variantes:
            return disc.variantes[e.variant]

    return SemUsitatum(nomen=e.variant)


def _is_numeric(t: SemanticTypus) -> bool:
    if isinstance(t, SemPrimitivus):
        return t.species in ("numerus", "fractus")
    return False


def _is_fractus(t: SemanticTypus) -> bool:
    if isinstance(t, SemPrimitivus):
        return t.species == "fractus"
    return False


def _is_textus(t: SemanticTypus) -> bool:
    if isinstance(t, SemPrimitivus):
        return t.species == "textus"
    return False
