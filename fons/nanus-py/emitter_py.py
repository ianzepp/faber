"""Python source code emitter for Faber AST."""

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
from semantic import analyze
from scope import SemanticContext


PY_BINARY_OPS = {
    "et": "and",
    "aut": "or",
    "==": "==",
    "!=": "!=",
    "===": "is",
    "!==": "is not",
}

PY_UNARY_OPS = {
    "non": "not ",
    "nihil": "",
    "nonnihil": "",
    "positivum": "+",
    "negativum": "-",
}

PY_TYPE_MAP = {
    "textus": "str",
    "numerus": "int",
    "fractus": "float",
    "bivalens": "bool",
    "nihil": "None",
    "vacuum": "None",
    "vacuus": "None",
    "ignotum": "Any",
    "quodlibet": "Any",
    "quidlibet": "Any",
    "lista": "list",
    "tabula": "dict",
    "copia": "set",
}


class PyEmitter:
    """Emits Python code from Faber AST."""

    def __init__(self, ctx: SemanticContext):
        self.ctx = ctx
        self.indent_char = "    "

    def emit(self, mod: Modulus) -> str:
        """Emit Python code for a module."""
        lines = []

        imports = self._collect_imports(mod)
        if imports:
            for imp in sorted(imports):
                lines.append(imp)
            lines.append("")

        for stmt in mod.corpus:
            code = self._stmt(stmt, "")
            if code:
                lines.append(code)
                lines.append("")

        return "\n".join(lines)

    def _collect_imports(self, mod: Modulus) -> set[str]:
        """Collect needed Python imports."""
        imports = set()

        for stmt in mod.corpus:
            self._scan_imports(stmt, imports)

        return imports

    def _scan_imports(self, stmt: Stmt, imports: set[str]) -> None:
        """Recursively scan for needed imports."""
        match stmt:
            case StmtPactum():
                imports.add("from typing import Protocol")
            case StmtOrdo():
                imports.add("from enum import Enum, auto")
            case StmtDiscretio():
                imports.add("from dataclasses import dataclass")
            case StmtGenus():
                imports.add("from dataclasses import dataclass")
                for m in stmt.methodi:
                    self._scan_imports(m, imports)
            case StmtFunctio():
                if stmt.corpus:
                    self._scan_imports(stmt.corpus, imports)
            case StmtMassa():
                for s in stmt.corpus:
                    self._scan_imports(s, imports)
            case StmtSi():
                self._scan_imports(stmt.cons, imports)
                if stmt.alt:
                    self._scan_imports(stmt.alt, imports)
            case StmtDum():
                self._scan_imports(stmt.corpus, imports)
            case StmtIteratio():
                self._scan_imports(stmt.corpus, imports)
            case StmtElige():
                for c in stmt.casus:
                    self._scan_imports(c.corpus, imports)
                if stmt.default:
                    self._scan_imports(stmt.default, imports)
            case StmtTempta():
                self._scan_imports(stmt.corpus, imports)
                if stmt.cape:
                    self._scan_imports(stmt.cape.corpus, imports)
                if stmt.demum:
                    self._scan_imports(stmt.demum, imports)
            case StmtProbandum():
                for s in stmt.corpus:
                    self._scan_imports(s, imports)
            case StmtProba():
                self._scan_imports(stmt.corpus, imports)
            case StmtIncipit():
                self._scan_imports(stmt.corpus, imports)

    def _stmt(self, s: Stmt, indent: str) -> str:
        """Emit a statement."""
        match s:
            case StmtVaria():
                return self._varia(s, indent)
            case StmtFunctio():
                return self._functio(s, indent)
            case StmtGenus():
                return self._genus(s, indent)
            case StmtPactum():
                return self._pactum(s, indent)
            case StmtOrdo():
                return self._ordo(s, indent)
            case StmtDiscretio():
                return self._discretio(s, indent)
            case StmtImporta():
                return self._importa(s, indent)
            case StmtRedde():
                return self._redde(s, indent)
            case StmtSi():
                return self._si(s, indent)
            case StmtDum():
                return self._dum(s, indent)
            case StmtFacDum():
                return self._fac_dum(s, indent)
            case StmtIteratio():
                return self._iteratio(s, indent)
            case StmtElige():
                return self._elige(s, indent)
            case StmtDiscerne():
                return self._discerne(s, indent)
            case StmtCustodi():
                return self._custodi(s, indent)
            case StmtTempta():
                return self._tempta(s, indent)
            case StmtIace():
                if s.fatale:
                    if s.arg:
                        return f"{indent}raise SystemExit({self._expr(s.arg)})"
                    return f"{indent}raise SystemExit()"
                return f"{indent}raise Exception({self._expr(s.arg)})"
            case StmtRumpe():
                return f"{indent}break"
            case StmtPerge():
                return f"{indent}continue"
            case StmtScribe():
                return self._scribe(s, indent)
            case StmtAdfirma():
                code = f"{indent}assert {self._expr(s.cond)}"
                if s.msg:
                    code += f", {self._expr(s.msg)}"
                return code
            case StmtExpressia():
                return f"{indent}{self._expr(s.expr)}"
            case StmtMassa():
                return self._massa(s, indent)
            case StmtIncipit():
                return self._incipit(s, indent)
            case StmtProbandum():
                return self._probandum(s, indent)
            case StmtProba():
                return self._proba(s, indent)
            case _:
                return f"{indent}# unknown statement"

    def _massa(self, s: StmtMassa, indent: str) -> str:
        """Emit a block as indented statements."""
        if not s.corpus:
            return f"{indent}pass"
        lines = []
        for stmt in s.corpus:
            lines.append(self._stmt(stmt, indent))
        return "\n".join(lines)

    def _varia(self, s: StmtVaria, indent: str) -> str:
        """Emit a variable declaration."""
        if s.externa:
            return f"{indent}# extern: {s.nomen}"

        typ_ann = ""
        if s.typus:
            typ_ann = f": {self._typus(s.typus)}"

        if s.valor:
            return f"{indent}{s.nomen}{typ_ann} = {self._expr(s.valor)}"
        if typ_ann:
            return f"{indent}{s.nomen}{typ_ann} = None"
        return f"{indent}{s.nomen} = None"

    def _functio(self, s: StmtFunctio, indent: str) -> str:
        """Emit a function definition."""
        if s.externa:
            return f"{indent}# extern: {s.nomen}"

        lines = []
        if s.asynca:
            lines.append(f"{indent}async def {s.nomen}(")
        else:
            lines.append(f"{indent}def {s.nomen}(")

        params = []
        for p in s.params:
            params.append(self._param(p))

        lines[-1] += ", ".join(params) + ")"

        if s.typus_reditus:
            lines[-1] += f" -> {self._typus(s.typus_reditus)}"

        lines[-1] += ":"

        if s.corpus:
            body = self._stmt(s.corpus, indent + self.indent_char)
            lines.append(body)
        else:
            lines.append(f"{indent}{self.indent_char}pass")

        return "\n".join(lines)

    def _genus(self, s: StmtGenus, indent: str) -> str:
        """Emit a class definition."""
        lines = []
        lines.append(f"{indent}@dataclass")
        lines.append(f"{indent}class {s.nomen}:")

        inner = indent + self.indent_char

        if not s.campi and not s.methodi:
            lines.append(f"{inner}pass")
            return "\n".join(lines)

        for c in s.campi:
            typ = self._typus(c.typus) if c.typus else "Any"
            if c.valor:
                lines.append(f"{inner}{c.nomen}: {typ} = {self._expr(c.valor)}")
            else:
                lines.append(f"{inner}{c.nomen}: {typ}")

        for m in s.methodi:
            if isinstance(m, StmtFunctio):
                lines.append("")
                method_lines = self._method(m, s.nomen, inner)
                lines.append(method_lines)

        return "\n".join(lines)

    def _method(self, s: StmtFunctio, class_name: str, indent: str) -> str:
        """Emit a method definition."""
        lines = []
        if s.asynca:
            lines.append(f"{indent}async def {s.nomen}(self")
        else:
            lines.append(f"{indent}def {s.nomen}(self")

        for p in s.params:
            lines[-1] += f", {self._param(p)}"

        lines[-1] += ")"

        if s.typus_reditus:
            lines[-1] += f" -> {self._typus(s.typus_reditus)}"

        lines[-1] += ":"

        if s.corpus:
            body = self._stmt(s.corpus, indent + self.indent_char)
            lines.append(body)
        else:
            lines.append(f"{indent}{self.indent_char}pass")

        return "\n".join(lines)

    def _pactum(self, s: StmtPactum, indent: str) -> str:
        """Emit a protocol (interface) definition."""
        lines = []
        lines.append(f"{indent}class {s.nomen}(Protocol):")

        inner = indent + self.indent_char

        if not s.methodi:
            lines.append(f"{inner}pass")
            return "\n".join(lines)

        for m in s.methodi:
            if m.asynca:
                lines.append(f"{inner}async def {m.nomen}(self")
            else:
                lines.append(f"{inner}def {m.nomen}(self")

            for p in m.params:
                lines[-1] += f", {self._param(p)}"
            lines[-1] += ")"

            if m.typus_reditus:
                lines[-1] += f" -> {self._typus(m.typus_reditus)}"
            lines[-1] += ": ..."

        return "\n".join(lines)

    def _ordo(self, s: StmtOrdo, indent: str) -> str:
        """Emit an enum definition."""
        lines = []
        lines.append(f"{indent}class {s.nomen}(Enum):")

        inner = indent + self.indent_char

        if not s.membra:
            lines.append(f"{inner}pass")
            return "\n".join(lines)

        for m in s.membra:
            if m.valor is not None:
                lines.append(f"{inner}{m.nomen} = {m.valor}")
            else:
                lines.append(f"{inner}{m.nomen} = auto()")

        return "\n".join(lines)

    def _discretio(self, s: StmtDiscretio, indent: str) -> str:
        """Emit a discriminated union as dataclasses."""
        lines = []

        for v in s.variantes:
            lines.append(f"{indent}@dataclass")
            lines.append(f"{indent}class {v.nomen}:")
            inner = indent + self.indent_char
            if v.campi:
                for f in v.campi:
                    typ = self._typus(f.typus) if f.typus else "Any"
                    lines.append(f"{inner}{f.nomen}: {typ}")
            else:
                lines.append(f"{inner}pass")
            lines.append("")

        variants = " | ".join(v.nomen for v in s.variantes)
        lines.append(f"{indent}{s.nomen} = {variants}")

        return "\n".join(lines)

    def _importa(self, s: StmtImporta, indent: str) -> str:
        """Emit an import statement."""
        module = s.fons.replace("/", ".").replace("-", "_")
        if s.totum:
            return f"{indent}import {module} as {s.local}"

        if s.imported:
            if s.imported != s.local:
                spec = f"{s.imported} as {s.local}"
            else:
                spec = s.imported
            return f"{indent}from {module} import {spec}"

        return f"{indent}# empty import from {module}"

    def _redde(self, s: StmtRedde, indent: str) -> str:
        """Emit a return statement."""
        if s.valor is None:
            return f"{indent}return"
        return f"{indent}return {self._expr(s.valor)}"

    def _si(self, s: StmtSi, indent: str) -> str:
        """Emit an if statement."""
        lines = []
        lines.append(f"{indent}if {self._expr(s.cond)}:")
        lines.append(self._stmt(s.cons, indent + self.indent_char))
        if s.alt:
            if isinstance(s.alt, StmtSi):
                lines.append(f"{indent}el" + self._si(s.alt, "")[len(indent):])
            else:
                lines.append(f"{indent}else:")
                lines.append(self._stmt(s.alt, indent + self.indent_char))
        return "\n".join(lines)

    def _dum(self, s: StmtDum, indent: str) -> str:
        """Emit a while loop."""
        lines = []
        lines.append(f"{indent}while {self._expr(s.cond)}:")
        lines.append(self._stmt(s.corpus, indent + self.indent_char))
        return "\n".join(lines)

    def _fac_dum(self, s: StmtFacDum, indent: str) -> str:
        """Emit a do-while loop (Python doesn't have this, emulate)."""
        lines = []
        lines.append(f"{indent}while True:")
        inner = indent + self.indent_char
        lines.append(self._stmt(s.corpus, inner))
        lines.append(f"{inner}if not ({self._expr(s.cond)}):")
        lines.append(f"{inner}{self.indent_char}break")
        return "\n".join(lines)

    def _iteratio(self, s: StmtIteratio, indent: str) -> str:
        """Emit a for loop."""
        lines = []
        if s.asynca:
            lines.append(f"{indent}async for {s.binding} in {self._expr(s.iter)}:")
        else:
            lines.append(f"{indent}for {s.binding} in {self._expr(s.iter)}:")
        lines.append(self._stmt(s.corpus, indent + self.indent_char))
        return "\n".join(lines)

    def _elige(self, s: StmtElige, indent: str) -> str:
        """Emit a switch as match statement."""
        lines = []
        lines.append(f"{indent}match {self._expr(s.discrim)}:")
        inner = indent + self.indent_char
        for c in s.casus:
            lines.append(f"{inner}case {self._expr(c.cond)}:")
            lines.append(self._stmt(c.corpus, inner + self.indent_char))
        if s.default:
            lines.append(f"{inner}case _:")
            lines.append(self._stmt(s.default, inner + self.indent_char))
        return "\n".join(lines)

    def _discerne(self, s: StmtDiscerne, indent: str) -> str:
        """Emit pattern matching."""
        lines = []
        discrim = ", ".join(self._expr(d) for d in s.discrim)
        if len(s.discrim) > 1:
            lines.append(f"{indent}match ({discrim}):")
        else:
            lines.append(f"{indent}match {discrim}:")

        inner = indent + self.indent_char
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
                        pat += f" as {p.alias}"
                    patterns.append(pat)
            lines.append(f"{inner}case {', '.join(patterns)}:")
            lines.append(self._stmt(c.corpus, inner + self.indent_char))

        return "\n".join(lines)

    def _custodi(self, s: StmtCustodi, indent: str) -> str:
        """Emit guard clauses as if statements."""
        lines = []
        for c in s.clausulae:
            lines.append(f"{indent}if {self._expr(c.cond)}:")
            lines.append(self._stmt(c.corpus, indent + self.indent_char))
        return "\n".join(lines)

    def _tempta(self, s: StmtTempta, indent: str) -> str:
        """Emit try-except-finally."""
        lines = []
        lines.append(f"{indent}try:")
        lines.append(self._stmt(s.corpus, indent + self.indent_char))
        if s.cape:
            lines.append(f"{indent}except Exception as {s.cape.param}:")
            lines.append(self._stmt(s.cape.corpus, indent + self.indent_char))
        if s.demum:
            lines.append(f"{indent}finally:")
            lines.append(self._stmt(s.demum, indent + self.indent_char))
        return "\n".join(lines)

    def _scribe(self, s: StmtScribe, indent: str) -> str:
        """Emit a print statement."""
        args = ", ".join(self._expr(a) for a in s.args)
        if s.gradus == "Mone":
            return f"{indent}import sys; print({args}, file=sys.stderr)"
        return f"{indent}print({args})"

    def _incipit(self, s: StmtIncipit, indent: str) -> str:
        """Emit main entry point."""
        lines = []
        if s.asynca:
            lines.append(f"{indent}async def main():")
            lines.append(self._stmt(s.corpus, indent + self.indent_char))
            lines.append("")
            lines.append(f'{indent}if __name__ == "__main__":')
            lines.append(f"{indent}{self.indent_char}import asyncio")
            lines.append(f"{indent}{self.indent_char}asyncio.run(main())")
        else:
            lines.append(f'{indent}if __name__ == "__main__":')
            lines.append(self._stmt(s.corpus, indent + self.indent_char))
        return "\n".join(lines)

    def _probandum(self, s: StmtProbandum, indent: str) -> str:
        """Emit a test suite as a class."""
        lines = []
        safe_name = s.nomen.replace(" ", "_").replace("-", "_")
        lines.append(f"{indent}class Test{safe_name}:")
        inner = indent + self.indent_char
        if not s.corpus:
            lines.append(f"{inner}pass")
        else:
            for stmt in s.corpus:
                lines.append(self._stmt(stmt, inner))
        return "\n".join(lines)

    def _proba(self, s: StmtProba, indent: str) -> str:
        """Emit a test case as a method."""
        lines = []
        safe_name = s.nomen.replace(" ", "_").replace("-", "_")
        lines.append(f"{indent}def test_{safe_name}(self):")
        lines.append(self._stmt(s.corpus, indent + self.indent_char))
        return "\n".join(lines)

    def _expr(self, e: Expr | None) -> str:
        """Emit an expression."""
        if e is None:
            return "None"

        match e:
            case ExprNomen():
                return e.valor

            case ExprEgo():
                return "self"

            case ExprLittera():
                match e.species:
                    case LitteraSpecies.TEXTUS:
                        return repr(e.valor)
                    case LitteraSpecies.VERUM:
                        return "True"
                    case LitteraSpecies.FALSUM:
                        return "False"
                    case LitteraSpecies.NIHIL:
                        return "None"
                    case _:
                        return e.valor

            case ExprBinaria():
                op = PY_BINARY_OPS.get(e.signum, e.signum)
                if e.signum == "vel":
                    left = self._expr(e.sin)
                    right = self._expr(e.dex)
                    return f"({left} if {left} is not None else {right})"
                if e.signum == "inter":
                    return f"{self._expr(e.sin)} in {self._expr(e.dex)}"
                if e.signum == "intra":
                    return f"{self._expr(e.sin)} not in {self._expr(e.dex)}"
                return f"({self._expr(e.sin)} {op} {self._expr(e.dex)})"

            case ExprUnaria():
                if e.signum == "nonnihil":
                    return f"({self._expr(e.arg)} is not None)"
                if e.signum == "nihil":
                    return f"({self._expr(e.arg)} is None)"
                op = PY_UNARY_OPS.get(e.signum, e.signum)
                return f"({op}{self._expr(e.arg)})"

            case ExprAssignatio():
                return f"{self._expr(e.sin)} {e.signum} {self._expr(e.dex)}"

            case ExprVocatio():
                if isinstance(e.callee, ExprMembrum) and not e.callee.computed:
                    prop = e.callee.prop
                    if isinstance(prop, ExprLittera):
                        method = prop.valor
                        obj = self._expr(e.callee.obj)
                        args = [self._expr(a) for a in e.args]

                        if method == "longitudo":
                            return f"len({obj})"
                        if method == "appende":
                            return f"{obj}.append({', '.join(args)})"
                        if method == "adde":
                            return f"{obj}.add({', '.join(args)})"
                        if method == "coniunge":
                            return f"{', '.join(args)}.join({obj})"
                        if method == "continet":
                            return f"{', '.join(args)} in {obj}"
                        if method == "initium":
                            return f"{obj}.startswith({', '.join(args)})"
                        if method == "finis":
                            return f"{obj}.endswith({', '.join(args)})"
                        if method == "maiuscula":
                            return f"{obj}.upper()"
                        if method == "minuscula":
                            return f"{obj}.lower()"
                        if method == "recide":
                            return f"{obj}.strip()"
                        if method == "divide":
                            return f"{obj}.split({', '.join(args)})"
                        if method == "muta":
                            return f"{obj}.replace({', '.join(args)})"
                        if method == "sectio":
                            if len(args) == 2:
                                return f"{obj}[{args[0]}:{args[1]}]"
                            if len(args) == 1:
                                return f"{obj}[{args[0]}:]"

                args = ", ".join(self._expr(a) for a in e.args)
                return f"{self._expr(e.callee)}({args})"

            case ExprMembrum():
                obj = self._expr(e.obj)
                if e.computed:
                    return f"{obj}[{self._expr(e.prop)}]"
                prop = e.prop.valor if isinstance(e.prop, ExprLittera) else self._expr(e.prop)

                if prop == "longitudo":
                    return f"len({obj})"
                if prop == "primus":
                    return f"{obj}[0]"
                if prop == "ultimus":
                    return f"{obj}[-1]"

                return f"{obj}.{prop}"

            case ExprCondicio():
                return f"({self._expr(e.cons)} if {self._expr(e.cond)} else {self._expr(e.alt)})"

            case ExprSeries():
                items = ", ".join(self._expr(i) for i in e.elementa)
                return f"[{items}]"

            case ExprObiectum():
                pairs = []
                for p in e.props:
                    if p.shorthand:
                        key = self._expr(p.key)
                        pairs.append(f'"{key}": {key}')
                    else:
                        key = self._expr(p.key)
                        if isinstance(p.key, ExprLittera):
                            key = repr(p.key.valor)
                        pairs.append(f"{key}: {self._expr(p.valor)}")
                return "{" + ", ".join(pairs) + "}"

            case ExprClausura():
                params = ", ".join(p.nomen for p in e.params)
                if isinstance(e.corpus, Expr):
                    return f"lambda {params}: {self._expr(e.corpus)}"
                return f"(lambda {params}: None)"

            case ExprNovum():
                args = ", ".join(self._expr(a) for a in e.args)
                callee = self._expr(e.callee)
                if e.init and isinstance(e.init, ExprObiectum):
                    fields = []
                    for p in e.init.props:
                        key = p.key.valor if isinstance(p.key, ExprLittera) else self._expr(p.key)
                        fields.append(f"{key}={self._expr(p.valor)}")
                    return f"{callee}({', '.join(fields)})"
                return f"{callee}({args})"

            case ExprPostfixNovum():
                typ = self._typus(e.typus)
                if isinstance(e.expr, ExprObiectum):
                    fields = []
                    for p in e.expr.props:
                        key = p.key.valor if isinstance(p.key, ExprLittera) else self._expr(p.key)
                        fields.append(f"{key}={self._expr(p.valor)}")
                    return f"{typ}({', '.join(fields)})"
                return f"{typ}({self._expr(e.expr)})"

            case ExprQua():
                return self._expr(e.expr)

            case ExprInnatum():
                return self._expr(e.expr)

            case ExprCede():
                return f"await {self._expr(e.arg)}"

            case ExprFinge():
                fields = []
                for p in e.campi:
                    key = p.key.valor if isinstance(p.key, ExprLittera) else self._expr(p.key)
                    fields.append(f"{key}={self._expr(p.valor)}")
                return f"{e.variant}({', '.join(fields)})"

            case ExprScriptum():
                template = e.template.replace("§", "{}")
                args = ", ".join(self._expr(a) for a in e.args)
                return f'f"{template}".format({args})'

            case ExprAmbitus():
                start = self._expr(e.start)
                end = self._expr(e.end)
                if e.inclusive:
                    return f"range({start}, {end} + 1)"
                return f"range({start}, {end})"

            case _:
                return "# unknown expr"

    def _typus(self, t: Typus | None) -> str:
        """Emit a type annotation."""
        if t is None:
            return "Any"

        match t:
            case TypusNomen():
                return PY_TYPE_MAP.get(t.nomen, t.nomen)

            case TypusNullabilis():
                inner = self._typus(t.inner)
                return f"{inner} | None"

            case TypusGenericus():
                base = PY_TYPE_MAP.get(t.nomen, t.nomen)
                args = ", ".join(self._typus(a) for a in t.args)
                return f"{base}[{args}]"

            case TypusFunctio():
                params = ", ".join(self._typus(p) for p in t.params)
                ret = self._typus(t.returns) if t.returns else "None"
                return f"Callable[[{params}], {ret}]"

            case TypusUnio():
                return " | ".join(self._typus(m) for m in t.members)

            case TypusLitteralis():
                return t.valor

            case _:
                return "Any"

    def _param(self, p: Param) -> str:
        """Emit a parameter."""
        typ = self._typus(p.typus) if p.typus else ""
        name = p.nomen
        if p.rest:
            name = f"*{name}"

        result = name
        if typ:
            result += f": {typ}"
        if p.default:
            result += f" = {self._expr(p.default)}"

        return result


def emit_py(mod: Modulus) -> str:
    """Emit Python code for a module."""
    ctx = analyze(mod)
    emitter = PyEmitter(ctx)
    return emitter.emit(mod)
