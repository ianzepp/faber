"""toml.py - TOML Encoding/Decoding Implementation

Native Python implementation of the TOML interface.
Uses tomllib (read) and tomli_w (write) for Python 3.11+.

Note: TOML root must be a table (object), not array or primitive.

Verb meanings:
  - pange (compose): serialize table to TOML string
  - solve (untangle): parse TOML string to table
  - tempta (try): attempt to parse, return None on error
"""

import tomllib
from typing import Any, Optional

try:
    import tomli_w
    _HAS_WRITER = True
except ImportError:
    _HAS_WRITER = False


# =============================================================================
# SERIALIZATION
# =============================================================================

def pange(valor: Any) -> str:
    """Serialize table to TOML string."""
    if not _HAS_WRITER:
        raise RuntimeError("tomli_w package required for TOML serialization")
    return tomli_w.dumps(valor)


# =============================================================================
# PARSING
# =============================================================================

def solve(toml_str: str) -> Any:
    """Parse TOML string to table (raises on error)."""
    return tomllib.loads(toml_str)


def tempta(toml_str: str) -> Optional[Any]:
    """Attempt to parse TOML string (returns None on error)."""
    try:
        return tomllib.loads(toml_str)
    except (ValueError, tomllib.TOMLDecodeError):
        return None


# =============================================================================
# TYPE CHECKING
# =============================================================================

def est_nihil(valor: Any) -> bool:
    """Check if value is null (TOML doesn't have null, always False)."""
    return False


def est_bivalens(valor: Any) -> bool:
    """Check if value is boolean."""
    return isinstance(valor, bool)


def est_textus(valor: Any) -> bool:
    """Check if value is string."""
    return isinstance(valor, str)


def est_integer(valor: Any) -> bool:
    """Check if value is integer."""
    return isinstance(valor, int) and not isinstance(valor, bool)


def est_fractus(valor: Any) -> bool:
    """Check if value is float."""
    return isinstance(valor, float)


def est_tempus(valor: Any) -> bool:
    """Check if value is datetime."""
    from datetime import datetime, date, time
    return isinstance(valor, (datetime, date, time))


def est_lista(valor: Any) -> bool:
    """Check if value is array."""
    return isinstance(valor, list)


def est_tabula(valor: Any) -> bool:
    """Check if value is table."""
    return isinstance(valor, dict)


# =============================================================================
# VALUE EXTRACTION
# =============================================================================

def ut_textus(valor: Any, def_val: str) -> str:
    """Extract as string with default."""
    return valor if isinstance(valor, str) else def_val


def ut_numerus(valor: Any, def_val: float) -> float:
    """Extract as number with default."""
    if isinstance(valor, float):
        return valor
    if isinstance(valor, int) and not isinstance(valor, bool):
        return float(valor)
    return def_val


def ut_bivalens(valor: Any, def_val: bool) -> bool:
    """Extract as boolean with default."""
    return valor if isinstance(valor, bool) else def_val


# =============================================================================
# VALUE ACCESS
# =============================================================================

def cape(valor: Any, clavis: str) -> Optional[Any]:
    """Get value by key (returns None if missing)."""
    if isinstance(valor, dict):
        return valor.get(clavis)
    return None


def carpe(valor: Any, index: int) -> Optional[Any]:
    """Pluck value by array index (returns None if out of bounds)."""
    if isinstance(valor, list) and 0 <= index < len(valor):
        return valor[index]
    return None


def inveni(valor: Any, via: str) -> Optional[Any]:
    """Find value by dotted path (returns None if not found)."""
    parts = via.split(".")
    current = valor

    for part in parts:
        if current is None:
            return None
        if isinstance(current, dict):
            current = current.get(part)
        elif isinstance(current, list):
            try:
                idx = int(part)
                if 0 <= idx < len(current):
                    current = current[idx]
                else:
                    return None
            except ValueError:
                return None
        else:
            return None

    return current
