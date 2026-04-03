"""Compile error types and formatting."""

from dataclasses import dataclass


@dataclass
class Locus:
    """Source location for error reporting."""
    linea: int = 1
    columna: int = 1
    index: int = 0


class CompileError(Exception):
    """A positioned compile error."""

    def __init__(self, message: str, locus: Locus | None = None, filename: str = "<stdin>"):
        self.message = message
        self.locus = locus or Locus()
        self.filename = filename
        super().__init__(self._format())

    def _format(self) -> str:
        return f"{self.filename}:{self.locus.linea}:{self.locus.columna}: {self.message}"


def format_error(err: Exception, source: str, filename: str = "<stdin>") -> str:
    """Render a human-friendly error message with source context."""
    if not isinstance(err, CompileError):
        return str(err)

    lines = source.split("\n")
    line_idx = err.locus.linea - 1
    src_line = lines[line_idx] if 0 <= line_idx < len(lines) else ""
    pointer = " " * max(0, err.locus.columna - 1) + "^"

    return "\n".join([
        f"{err.filename}:{err.locus.linea}:{err.locus.columna}: error: {err.message}",
        "",
        f"  {src_line}",
        f"  {pointer}",
    ])
