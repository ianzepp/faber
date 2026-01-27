"""Faber source code emitter for round-trip conversion."""

from nodes import (
    Modulus, Stmt, Expr, Typus, Param,
    StmtMassa, StmtVaria, StmtFunctio, StmtGenus, StmtPactum, StmtOrdo, StmtDiscretio,
    StmtImporta, StmtRedde, StmtSi, StmtDum, StmtFacDum, StmtIteratio,
    StmtElige, StmtDiscerne, StmtCustodi, StmtTempta, StmtIace, StmtRumpe, StmtPerge,
    StmtScribe, StmtAdfirma, StmtExpressia, StmtIncipit, StmtProbandum, StmtProba,
    ExprNomen, ExprEgo, ExprLittera, ExprBinaria, ExprUnaria, ExprAssignatio,
    ExprVocatio, ExprMembrum, ExprCondicio, ExprSeries, ExprObiectum, ExprClausura,
    ExprNovum, ExprQua, ExprInnatum, ExprCede, ExprFinge, ExprScriptum, ExprAmbitus,
    ExprPostfixNovum,
    TypusNomen, TypusGenericus, TypusFunctio, TypusNullabilis, TypusUnio, TypusLitteralis,
    LitteraSpecies, VariaSpecies,
)


def emit_faber(mod: Modulus) -> str:
    """Format an AST back to canonical Faber source."""
    lines = []
    for i, stmt in enumerate(mod.corpus):
        if i > 0:
            lines.append("")
        lines.append(_stmt(stmt, ""))
    return "\n".join(lines)


def _stmt(s: Stmt, indent: str) -> str:
    """Emit a statement."""
    match s:
        case StmtVaria():
            return _varia(s, indent)
        case StmtFunctio():
            return _functio(s, indent)
        case StmtGenus():
            return _genus(s, indent)
        case StmtPactum():
            return _pactum(s, indent)
        case StmtOrdo():
            return _ordo(s, indent)
        case StmtDiscretio():
            return _discretio(s, indent)
        case StmtImporta():
            return _importa(s, indent)
        case StmtRedde():
            return _redde(s, indent)
        case StmtSi():
            return _si(s, indent)
        case StmtDum():
            return _dum(s, indent)
        case StmtFacDum():
            return _fac_dum(s, indent)
        case StmtIteratio():
            return _iteratio(s, indent)
        case StmtElige():
            return _elige(s, indent)
        case StmtDiscerne():
            return _discerne(s, indent)
        case StmtCustodi():
            return _custodi(s, indent)
        case StmtTempta():
            return _tempta(s, indent)
        case StmtIace():
            if s.fatale:
                if s.arg:
                    return f"{indent}mori {_expr(s.arg)}"
                return f"{indent}mori"
            return f"{indent}iace {_expr(s.arg)}"
        case StmtRumpe():
            return f"{indent}rumpe"
        case StmtPerge():
            return f"{indent}perge"
        case StmtScribe():
            return _scribe(s, indent)
        case StmtAdfirma():
            code = f"{indent}adfirma {_expr(s.cond)}"
            if s.msg:
                code += f", {_expr(s.msg)}"
            return code
        case StmtExpressia():
            return f"{indent}{_expr(s.expr)}"
        case StmtMassa():
            return _massa(s, indent)
        case StmtIncipit():
            return _incipit(s, indent)
        case StmtProbandum():
            return _probandum(s, indent)
        case StmtProba():
            return _proba(s, indent)
        case _:
            return f"{indent}# unknown statement"


def _massa(s: StmtMassa, indent: str) -> str:
    """Emit a block statement."""
    lines = ["{"]
    for stmt in s.corpus:
        lines.append(_stmt(stmt, indent + "\t"))
    lines.append(f"{indent}}}")
    return "\n".join(lines)


def _varia(s: StmtVaria, indent: str) -> str:
    """Emit a variable declaration."""
    match s.species:
        case VariaSpecies.FIXUM:
            keyword = "fixum"
        case VariaSpecies.FIGENDUM:
            keyword = "figendum"
        case _:
            keyword = "varia"
    if s.typus:
        code = f"{indent}{keyword} {_typus(s.typus)} {s.nomen}"
    else:
        code = f"{indent}{keyword} {s.nomen}"
    if s.valor:
        code += f" = {_expr(s.valor)}"
    return code


def _functio(s: StmtFunctio, indent: str) -> str:
    """Emit a function declaration."""
    parts = [indent]
    if s.publica:
        parts.append("publica ")
    if s.asynca:
        parts.append("asynca ")
    parts.append("functio ")
    parts.append(s.nomen)
    if s.generics:
        parts.append("<")
        parts.append(", ".join(s.generics))
        parts.append(">")
    parts.append("(")
    parts.append(", ".join(_param(p) for p in s.params))
    parts.append(")")
    if s.typus_reditus:
        parts.append(": ")
        parts.append(_typus(s.typus_reditus))
    if s.corpus:
        parts.append(" ")
        parts.append(_stmt(s.corpus, indent))
    return "".join(parts)


def _genus(s: StmtGenus, indent: str) -> str:
    """Emit a class declaration."""
    parts = [indent]
    if s.publica:
        parts.append("publica ")
    parts.append("genus ")
    parts.append(s.nomen)
    if s.generics:
        parts.append("<")
        parts.append(", ".join(s.generics))
        parts.append(">")
    if s.implet:
        parts.append(" implet ")
        parts.append(", ".join(s.implet))
    parts.append(" {\n")
    for c in s.campi:
        if c.typus:
            parts.append(f"{indent}\t{_typus(c.typus)} {c.nomen}")
        else:
            parts.append(f"{indent}\t{c.nomen}")
        if c.valor:
            parts.append(f": {_expr(c.valor)}")
        parts.append("\n")
    for m in s.methodi:
        parts.append(_stmt(m, indent + "\t"))
        parts.append("\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _pactum(s: StmtPactum, indent: str) -> str:
    """Emit an interface declaration."""
    parts = [indent]
    if s.publica:
        parts.append("publica ")
    parts.append("pactum ")
    parts.append(s.nomen)
    if s.generics:
        parts.append("<")
        parts.append(", ".join(s.generics))
        parts.append(">")
    parts.append(" {\n")
    for m in s.methodi:
        parts.append(f"{indent}\t")
        if m.asynca:
            parts.append("asynca ")
        parts.append(f"functio {m.nomen}(")
        parts.append(", ".join(_param(p) for p in m.params))
        parts.append(")")
        if m.typus_reditus:
            parts.append(f": {_typus(m.typus_reditus)}")
        parts.append("\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _ordo(s: StmtOrdo, indent: str) -> str:
    """Emit an enum declaration."""
    parts = [indent]
    if s.publica:
        parts.append("publica ")
    parts.append("ordo ")
    parts.append(s.nomen)
    parts.append(" {\n")
    for m in s.membra:
        parts.append(f"{indent}\t{m.nomen}")
        if m.valor is not None:
            parts.append(f" = {m.valor}")
        parts.append("\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _discretio(s: StmtDiscretio, indent: str) -> str:
    """Emit a discriminated union declaration."""
    parts = [indent]
    if s.publica:
        parts.append("publica ")
    parts.append("discretio ")
    parts.append(s.nomen)
    if s.generics:
        parts.append("<")
        parts.append(", ".join(s.generics))
        parts.append(">")
    parts.append(" {\n")
    for v in s.variantes:
        parts.append(f"{indent}\t{v.nomen}")
        if v.campi:
            fields = ", ".join(f"{_typus(f.typus)} {f.nomen}" for f in v.campi)
            parts.append(f" {{ {fields} }}")
        parts.append("\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _importa(s: StmtImporta, indent: str) -> str:
    """Emit an import statement."""
    visibility = "publica" if s.publica else "privata"
    parts = [f'{indent}importa ex "{s.fons}" {visibility} ']
    if s.totum:
        parts.append(f"* ut {s.local}")
    elif s.imported:
        if s.imported != s.local:
            parts.append(f"{s.imported} ut {s.local}")
        else:
            parts.append(s.imported)
    return "".join(parts)


def _redde(s: StmtRedde, indent: str) -> str:
    """Emit a return statement."""
    if s.valor is None:
        return f"{indent}redde"
    return f"{indent}redde {_expr(s.valor)}"


def _si(s: StmtSi, indent: str) -> str:
    """Emit an if statement."""
    code = f"{indent}si {_expr(s.cond)} {_stmt(s.cons, indent)}"
    if s.alt:
        code += f" secus {_stmt(s.alt, indent)}"
    return code


def _dum(s: StmtDum, indent: str) -> str:
    """Emit a while statement."""
    return f"{indent}dum {_expr(s.cond)} {_stmt(s.corpus, indent)}"


def _fac_dum(s: StmtFacDum, indent: str) -> str:
    """Emit a do-while statement."""
    return f"{indent}fac {_stmt(s.corpus, indent)} dum {_expr(s.cond)}"


def _iteratio(s: StmtIteratio, indent: str) -> str:
    """Emit a for-loop statement."""
    parts = [indent]
    if s.asynca:
        parts.append("cede ")
    kw = "ex" if s.species == "Ex" else "de"
    parts.append(f"itera {kw} {_expr(s.iter)} fixum {s.binding} {_stmt(s.corpus, indent)}")
    return "".join(parts)


def _elige(s: StmtElige, indent: str) -> str:
    """Emit a switch statement."""
    parts = [f"{indent}elige {_expr(s.discrim)} {{\n"]
    for c in s.casus:
        parts.append(f"{indent}\tcasu {_expr(c.cond)} {_stmt(c.corpus, indent + chr(9))}\n")
    if s.default:
        parts.append(f"{indent}\tceterum {_stmt(s.default, indent + chr(9))}\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _discerne(s: StmtDiscerne, indent: str) -> str:
    """Emit a pattern-matching statement."""
    discrim_str = ", ".join(_expr(d) for d in s.discrim)
    parts = [f"{indent}discerne {discrim_str} {{\n"]
    for c in s.casus:
        patterns = []
        for p in c.patterns:
            if p.wildcard:
                patterns.append("_")
            else:
                pat = p.variant
                if p.bindings:
                    pat += "(" + ", ".join(p.bindings) + ")"
                if p.alias:
                    pat += f" ut {p.alias}"
                patterns.append(pat)
        parts.append(f"{indent}\tcasu {', '.join(patterns)} {_stmt(c.corpus, indent + chr(9))}\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _custodi(s: StmtCustodi, indent: str) -> str:
    """Emit a guard statement."""
    lines = []
    for c in s.clausulae:
        lines.append(f"{indent}custodi {_expr(c.cond)} {_stmt(c.corpus, indent)}")
    return "\n".join(lines)


def _tempta(s: StmtTempta, indent: str) -> str:
    """Emit a try-catch-finally statement."""
    parts = [f"{indent}tempta {_stmt(s.corpus, indent)}"]
    if s.cape:
        parts.append(f" cape {s.cape.param} {_stmt(s.cape.corpus, indent)}")
    if s.demum:
        parts.append(f" demum {_stmt(s.demum, indent)}")
    return "".join(parts)


def _scribe(s: StmtScribe, indent: str) -> str:
    """Emit a print statement."""
    keyword = "scribe"
    if s.gradus == "Vide":
        keyword = "vide"
    elif s.gradus == "Mone":
        keyword = "mone"
    args = ", ".join(_expr(a) for a in s.args)
    return f"{indent}{keyword} {args}"


def _incipit(s: StmtIncipit, indent: str) -> str:
    """Emit a main entry point."""
    keyword = "incipiet" if s.asynca else "incipit"
    return f"{indent}{keyword} {_stmt(s.corpus, indent)}"


def _probandum(s: StmtProbandum, indent: str) -> str:
    """Emit a test suite."""
    parts = [f'{indent}probandum "{s.nomen}" {{\n']
    for stmt in s.corpus:
        parts.append(_stmt(stmt, indent + "\t"))
        parts.append("\n")
    parts.append(f"{indent}}}")
    return "".join(parts)


def _proba(s: StmtProba, indent: str) -> str:
    """Emit a test case."""
    return f'{indent}proba "{s.nomen}" {_stmt(s.corpus, indent)}'


def _expr(e: Expr | None) -> str:
    """Emit an expression."""
    if e is None:
        return ""
    match e:
        case ExprNomen():
            return e.valor
        case ExprEgo():
            return "ego"
        case ExprLittera():
            match e.species:
                case LitteraSpecies.TEXTUS:
                    return '"' + _escape_string(e.valor) + '"'
                case LitteraSpecies.VERUM:
                    return "verum"
                case LitteraSpecies.FALSUM:
                    return "falsum"
                case LitteraSpecies.NIHIL:
                    return "nihil"
                case _:
                    return e.valor
        case ExprBinaria():
            return f"{_expr(e.sin)} {e.signum} {_expr(e.dex)}"
        case ExprUnaria():
            if e.signum in ("nihil", "non", "nonnihil"):
                return f"{e.signum} {_expr(e.arg)}"
            return f"{e.signum}{_expr(e.arg)}"
        case ExprAssignatio():
            return f"{_expr(e.sin)} {e.signum} {_expr(e.dex)}"
        case ExprVocatio():
            args = ", ".join(_expr(a) for a in e.args)
            return f"{_expr(e.callee)}({args})"
        case ExprMembrum():
            obj = _expr(e.obj)
            if e.computed:
                return f"{obj}[{_expr(e.prop)}]"
            prop = e.prop.valor if isinstance(e.prop, ExprLittera) else _expr(e.prop)
            if e.non_null:
                return f"{obj}!.{prop}"
            return f"{obj}.{prop}"
        case ExprCondicio():
            return f"{_expr(e.cond)} ? {_expr(e.cons)} : {_expr(e.alt)}"
        case ExprSeries():
            items = ", ".join(_expr(i) for i in e.elementa)
            return f"[{items}]"
        case ExprObiectum():
            pairs = []
            for p in e.props:
                if p.shorthand:
                    key = p.key.valor if isinstance(p.key, ExprLittera) else _expr(p.key)
                    pairs.append(key)
                else:
                    key = p.key.valor if isinstance(p.key, ExprLittera) else _expr(p.key)
                    pairs.append(f"{key}: {_expr(p.valor)}")
            return "{ " + ", ".join(pairs) + " }"
        case ExprClausura():
            params = ", ".join(_param(p) for p in e.params)
            if isinstance(e.corpus, Stmt):
                return f"({params}) => {_stmt(e.corpus, '')}"
            else:
                return f"({params}) => {_expr(e.corpus)}"
        case ExprNovum():
            args = ", ".join(_expr(a) for a in e.args)
            if e.args or not e.init:
                code = f"novum {_expr(e.callee)}({args})"
            else:
                code = f"novum {_expr(e.callee)}"
            if e.init:
                code += f" {_expr(e.init)}"
            return code
        case ExprQua():
            return f"{_expr(e.expr)} qua {_typus(e.typus)}"
        case ExprInnatum():
            return f"innatum {_expr(e.expr)}"
        case ExprCede():
            return f"cede {_expr(e.arg)}"
        case ExprFinge():
            pairs = []
            for p in e.campi:
                if p.shorthand:
                    pairs.append(_expr(p.key))
                else:
                    pairs.append(f"{_expr(p.key)}: {_expr(p.valor)}")
            return f"finge {e.variant} {{ {', '.join(pairs)} }}"
        case ExprScriptum():
            return f'scriptum("{e.template}")'
        case ExprAmbitus():
            if e.inclusive:
                return f"{_expr(e.start)} usque {_expr(e.end)}"
            return f"{_expr(e.start)} ante {_expr(e.end)}"
        case ExprPostfixNovum():
            return f"{_expr(e.expr)} novum {_typus(e.typus)}"
        case _:
            return "# unknown expr"


def _typus(t: Typus | None) -> str:
    """Emit a type annotation."""
    if t is None:
        return ""
    match t:
        case TypusNomen():
            return t.nomen
        case TypusGenericus():
            args = ", ".join(_typus(a) for a in t.args)
            return f"{t.nomen}<{args}>"
        case TypusFunctio():
            params = ", ".join(_typus(p) for p in t.params)
            return f"({params}) -> {_typus(t.returns)}"
        case TypusNullabilis():
            return f"si {_typus(t.inner)}"
        case TypusUnio():
            return " | ".join(_typus(m) for m in t.members)
        case TypusLitteralis():
            return t.valor
        case _:
            return "# unknown type"


def _param(p: Param) -> str:
    """Emit a parameter."""
    parts = []
    if p.rest:
        parts.append("...")
    if p.typus:
        parts.append(f"{_typus(p.typus)} ")
    parts.append(p.nomen)
    if p.default:
        parts.append(f" = {_expr(p.default)}")
    return "".join(parts)


def _escape_string(s: str) -> str:
    """Escape special characters in a string."""
    result = []
    for ch in s:
        if ch == '\n':
            result.append("\\n")
        elif ch == '\t':
            result.append("\\t")
        elif ch == '\r':
            result.append("\\r")
        elif ch == '\\':
            result.append("\\\\")
        elif ch == '"':
            result.append('\\"')
        else:
            result.append(ch)
    return "".join(result)
