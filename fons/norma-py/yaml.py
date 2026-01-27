"""yaml.py - YAML Encoding/Decoding Implementation

Native Python implementation of the YAML interface.
Uses PyYAML.

Verb meanings:
  - pange (compose): serialize value to YAML string
  - necto (bind): bind multiple documents into multi-doc YAML
  - solve (untangle): parse YAML string to value
  - tempta (try): attempt to parse, return None on error
  - collige (gather): gather all documents from multi-doc YAML
"""

from typing import Any, Optional

import yaml as _yaml


# =============================================================================
# SERIALIZATION
# =============================================================================

def pange(valor: Any) -> str:
    """Serialize value to YAML string."""
    return _yaml.dump(valor, default_flow_style=False, allow_unicode=True)


def necto(documenta: list[Any]) -> str:
    """Bind multiple documents into multi-doc YAML string."""
    return _yaml.dump_all(documenta, default_flow_style=False, allow_unicode=True)


# =============================================================================
# PARSING
# =============================================================================

def solve(yaml_str: str) -> Any:
    """Parse YAML string to value (raises on error)."""
    return _yaml.safe_load(yaml_str)


def tempta(yaml_str: str) -> Optional[Any]:
    """Attempt to parse YAML string (returns None on error)."""
    try:
        return _yaml.safe_load(yaml_str)
    except _yaml.YAMLError:
        return None


def collige(yaml_str: str) -> list[Any]:
    """Gather all documents from multi-doc YAML string."""
    return list(_yaml.safe_load_all(yaml_str))


# =============================================================================
# TYPE CHECKING
# =============================================================================

def est_nihil(valor: Any) -> bool:
    """Check if value is null."""
    return valor is None


def est_bivalens(valor: Any) -> bool:
    """Check if value is boolean."""
    return isinstance(valor, bool)


def est_numerus(valor: Any) -> bool:
    """Check if value is number."""
    return isinstance(valor, (int, float)) and not isinstance(valor, bool)


def est_textus(valor: Any) -> bool:
    """Check if value is string."""
    return isinstance(valor, str)


def est_lista(valor: Any) -> bool:
    """Check if value is array/sequence."""
    return isinstance(valor, list)


def est_tabula(valor: Any) -> bool:
    """Check if value is object/mapping."""
    return isinstance(valor, dict)


# =============================================================================
# VALUE EXTRACTION
# =============================================================================

def ut_textus(valor: Any, def_val: str) -> str:
    """Extract as string with default."""
    return valor if isinstance(valor, str) else def_val


def ut_numerus(valor: Any, def_val: float) -> float:
    """Extract as number with default."""
    if isinstance(valor, (int, float)) and not isinstance(valor, bool):
        return float(valor)
    return def_val


def ut_bivalens(valor: Any, def_val: bool) -> bool:
    """Extract as boolean with default."""
    return valor if isinstance(valor, bool) else def_val


# =============================================================================
# VALUE ACCESS
# =============================================================================

def cape(valor: Any, clavis: str) -> Any:
    """Get value by key (returns None if missing)."""
    if isinstance(valor, dict):
        return valor.get(clavis)
    return None


def carpe(valor: Any, index: int) -> Any:
    """Pluck value by array index (returns None if out of bounds)."""
    if isinstance(valor, list) and 0 <= index < len(valor):
        return valor[index]
    return None


def inveni(valor: Any, via: str) -> Any:
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
