//! datum.rs - Canonical runtime data value (`Valor`) for stdlib data formats.
//!
//! ARCHITECTURE:
//! - Single stable type `Valor` is the ABI between Faber codegen and the Rust
//!   runtime for JSON and TOML data formats.
//! - Backends (serde_json::Value, toml::Value) convert through `TryFrom` and
//!   fallible `try_to_*` helpers; the public pactum functions will accept/return
//!   `Valor` (Phase 3+).
//! - No leakage of crate-specific dynamic values into generated code or Faber `quidlibet`.
//!
//! VALUE SPACE (per plan):
//!   nihil | bivalens | numerus(i64) | fractus(f64) | textus
//!   | lista<Valor> | tabula<textus, Valor>
//!   | tempus (TOML datetime as RFC3339 text; JSON has none)
//!
//! ERROR CONTRACT:
//! - Conversions are total for supported shapes; unsupported shapes (e.g. JSON
//!   u128 out of i64 range, or exotic toml values) produce `DatumError`, never panic.
//! - Callers (pange/solve wrappers) surface via failable paths or `tempta` returning nihil.
//!
//! NUMERIC POLICY (initial):
//! - JSON Number -> prefer i64 (numerus) when exact, else f64 (fractus).
//! - TOML Integer -> numerus, Float -> fractus.
//! - Round-trip loss for very large integers is accepted and documented; callers
//!   that need exact bigints should not use the generic data value.
//!
//! TOML DATETIME:
//! - Serialized as text (to_string() of toml::Datetime, usually RFC3339).
//! - `est_tempus` in the toml module will recognize the Tempus variant.
//! - When going to JSON, Tempus becomes textus (lossy but safe).

use std::collections::BTreeMap;

use serde_json;
use toml;

/// Canonical dynamic data value for data-format stdlib modules.
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    Nihil,
    Bivalens(bool),
    Numerus(i64),
    Fractus(f64),
    Textus(String),
    Lista(Vec<Valor>),
    Tabula(BTreeMap<String, Valor>),
    /// TOML datetime values, stored as their canonical text form (RFC3339-ish).
    Tempus(String),
}

/// Error type for data conversion failures (never panics at runtime boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatumError {
    /// A backend value shape cannot be represented in `Valor` (e.g. out-of-range number).
    UnsupportedValue(String),
}

pub type DatumResult<T> = Result<T, DatumError>;

impl Valor {
    /// Convenience constructor.
    pub fn nihil() -> Self {
        Valor::Nihil
    }

    /// Returns true if this value is the null/nihil sentinel.
    pub fn is_nihil(&self) -> bool {
        matches!(self, Valor::Nihil)
    }
}

// =============================================================================
// JSON CONVERSIONS
// =============================================================================

impl TryFrom<serde_json::Value> for Valor {
    type Error = DatumError;

    fn try_from(value: serde_json::Value) -> DatumResult<Self> {
        match value {
            serde_json::Value::Null => Ok(Valor::Nihil),
            serde_json::Value::Bool(b) => Ok(Valor::Bivalens(b)),
            serde_json::Value::Number(n) => {
                // Prefer exact integer when possible (fits i64); otherwise fall to float.
                if let Some(i) = n.as_i64() {
                    Ok(Valor::Numerus(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Valor::Fractus(f))
                } else {
                    Err(DatumError::UnsupportedValue(format!(
                        "JSON number out of representable range: {n}"
                    )))
                }
            }
            serde_json::Value::String(s) => Ok(Valor::Textus(s)),
            serde_json::Value::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for v in items {
                    out.push(Valor::try_from(v)?);
                }
                Ok(Valor::Lista(out))
            }
            serde_json::Value::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    out.insert(k, Valor::try_from(v)?);
                }
                Ok(Valor::Tabula(out))
            }
        }
    }
}

// =============================================================================
// REVERSE CONVERSIONS (Valor → backend values)
// =============================================================================
//
// These are intentionally fallible. We do not provide `From<Valor>` impls
// because several shapes have no lossless representation in the target format
// (e.g. Nihil in TOML, NaN/∞ in JSON numbers). Callers must handle the error.

impl Valor {
    /// Convert to a `serde_json::Value`, returning an error for values that
    /// cannot be represented without loss or invention (e.g. `Fractus(NaN)`,
    /// `Fractus(±∞)`).
    pub fn try_to_json(&self) -> DatumResult<serde_json::Value> {
        match self {
            Valor::Nihil => Ok(serde_json::Value::Null),
            Valor::Bivalens(b) => Ok(serde_json::Value::Bool(*b)),
            Valor::Numerus(n) => Ok(serde_json::Value::Number((*n).into())),
            Valor::Fractus(f) => {
                if f.is_finite() {
                    match serde_json::Number::from_f64(*f) {
                        Some(num) => Ok(serde_json::Value::Number(num)),
                        None => Err(DatumError::UnsupportedValue(format!(
                            "fractus {f} cannot be represented as a JSON number"
                        ))),
                    }
                } else {
                    Err(DatumError::UnsupportedValue(format!(
                        "fractus value is NaN or infinite: {f}"
                    )))
                }
            }
            Valor::Textus(s) => Ok(serde_json::Value::String(s.clone())),
            Valor::Lista(xs) => {
                let mut out = Vec::with_capacity(xs.len());
                for v in xs {
                    out.push(v.try_to_json()?);
                }
                Ok(serde_json::Value::Array(out))
            }
            Valor::Tabula(m) => {
                let mut out = serde_json::Map::new();
                for (k, v) in m {
                    out.insert(k.clone(), v.try_to_json()?);
                }
                Ok(serde_json::Value::Object(out))
            }
            Valor::Tempus(t) => Ok(serde_json::Value::String(t.clone())),
        }
    }

    /// Convert to a `toml::Value`, returning an error for values that have no
    /// representation in TOML (currently `Nihil`).
    pub fn try_to_toml(&self) -> DatumResult<toml::Value> {
        match self {
            Valor::Nihil => Err(DatumError::UnsupportedValue(
                "Nihil (null) has no representation in TOML".to_string(),
            )),
            Valor::Bivalens(b) => Ok(toml::Value::Boolean(*b)),
            Valor::Numerus(n) => Ok(toml::Value::Integer(*n)),
            Valor::Fractus(f) => Ok(toml::Value::Float(*f)),
            Valor::Textus(s) => Ok(toml::Value::String(s.clone())),
            Valor::Lista(xs) => {
                let mut out = Vec::with_capacity(xs.len());
                for v in xs {
                    out.push(v.try_to_toml()?);
                }
                Ok(toml::Value::Array(out))
            }
            Valor::Tabula(m) => {
                let mut out = toml::Table::new();
                for (k, v) in m {
                    out.insert(k.clone(), v.try_to_toml()?);
                }
                Ok(toml::Value::Table(out))
            }
            Valor::Tempus(t) => {
                // Store as string; callers that want a native TOML datetime can
                // attempt to parse the text form.
                Ok(toml::Value::String(t.clone()))
            }
        }
    }
}

// =============================================================================
// TOML CONVERSIONS
// =============================================================================

impl TryFrom<toml::Value> for Valor {
    type Error = DatumError;

    fn try_from(value: toml::Value) -> DatumResult<Self> {
        match value {
            toml::Value::String(s) => Ok(Valor::Textus(s)),
            toml::Value::Integer(i) => Ok(Valor::Numerus(i)),
            toml::Value::Float(f) => Ok(Valor::Fractus(f)),
            toml::Value::Boolean(b) => Ok(Valor::Bivalens(b)),
            toml::Value::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for v in items {
                    out.push(Valor::try_from(v)?);
                }
                Ok(Valor::Lista(out))
            }
            toml::Value::Table(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    out.insert(k, Valor::try_from(v)?);
                }
                Ok(Valor::Tabula(out))
            }
            toml::Value::Datetime(dt) => {
                // Store the datetime in its standard text form (RFC3339 / toml canonical).
                Ok(Valor::Tempus(dt.to_string()))
            }
        }
    }
}
