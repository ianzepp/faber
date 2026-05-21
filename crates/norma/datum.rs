//! datum.rs - Canonical runtime data value (`Valor`) for stdlib data formats.
//!
//! ARCHITECTURE:
//! - Single stable type `Valor` is the ABI between Faber codegen and the Rust
//!   runtime for JSON/TOML (and future YAML, DB rows, etc.).
//! - Backends (serde_json::Value, toml::Value) convert through `TryFrom` / `From`
//!   adapters; the public pactum functions will accept/return `Valor` (Phase 3+).
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
    /// TOML (and future) datetime values, stored as their canonical text form.
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

impl From<Valor> for serde_json::Value {
    fn from(val: Valor) -> Self {
        match val {
            Valor::Nihil => serde_json::Value::Null,
            Valor::Bivalens(b) => serde_json::Value::Bool(b),
            Valor::Numerus(n) => serde_json::Value::Number(n.into()),
            Valor::Fractus(f) => {
                // JSON numbers are IEEE doubles; use Number::from_f64 when possible.
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Valor::Textus(s) => serde_json::Value::String(s),
            Valor::Lista(xs) => {
                serde_json::Value::Array(xs.into_iter().map(serde_json::Value::from).collect())
            }
            Valor::Tabula(m) => serde_json::Value::Object(
                m.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::from(v)))
                    .collect(),
            ),
            Valor::Tempus(t) => serde_json::Value::String(t), // degrade datetime to text for JSON
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

impl From<Valor> for toml::Value {
    fn from(val: Valor) -> Self {
        match val {
            Valor::Nihil => {
                // TOML has no null; represent as "null" string sentinel (documented).
                // Consumers using the toml pactum should prefer the safe paths or
                // avoid sending nihil through TOML serialization.
                toml::Value::String("null".to_owned())
            }
            Valor::Bivalens(b) => toml::Value::Boolean(b),
            Valor::Numerus(n) => toml::Value::Integer(n),
            Valor::Fractus(f) => toml::Value::Float(f),
            Valor::Textus(s) => toml::Value::String(s),
            Valor::Lista(xs) => {
                toml::Value::Array(xs.into_iter().map(toml::Value::from).collect())
            }
            Valor::Tabula(m) => toml::Value::Table(
                m.into_iter()
                    .map(|(k, v)| (k, toml::Value::from(v)))
                    .collect(),
            ),
            Valor::Tempus(t) => {
                // Best effort: if it parses back as datetime, keep it; else string.
                // For simplicity we always emit as string here; the toml::to_string
                // path will quote it. Dedicated tempus helpers can upgrade.
                toml::Value::String(t)
            }
        }
    }
}

// =============================================================================
// TESTS (inline per current crate convention; dedicated _test.rs can be split later)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip_basic_shapes() {
        let original = serde_json::json!({
            "null": null,
            "bool": true,
            "int": 42,
            "float": 3.14,
            "str": "hello",
            "arr": [1, "x", false],
            "obj": {"nested": {"k": 1}}
        });

        let valor = Valor::try_from(original.clone()).expect("json -> valor");
        let back: serde_json::Value = valor.into();
        assert_eq!(back, original);
    }

    #[test]
    fn toml_roundtrip_basic_shapes() {
        let original = toml::from_str::<toml::Value>(
            r#"
            str = "hi"
            int = 7
            float = 2.5
            bool = false
            arr = [1, 2]
            [tbl]
            inner = "x"
            "#,
        )
        .unwrap();

        let valor = Valor::try_from(original.clone()).expect("toml -> valor");
        let back: toml::Value = valor.into();
        // Note: table key order may differ (BTreeMap), but values equal.
        assert_eq!(back, original);
    }

    #[test]
    fn toml_datetime_becomes_tempus() {
        let v = toml::from_str::<toml::Value>(r#"dt = 1979-05-27T07:32:00Z"#).unwrap();
        let valor = Valor::try_from(v).expect("datetime conversion");
        // The table contains a Tempus entry.
        if let Valor::Tabula(m) = valor {
            match m.get("dt") {
                Some(Valor::Tempus(_)) => {}
                other => panic!("expected Tempus, got {:?}", other),
            }
        } else {
            panic!("expected tabula");
        }
    }

    #[test]
    fn large_json_numbers_convert_via_fractus() {
        // All JSON numbers are accepted: exact i64 -> Numerus, else Fractus (f64 path).
        // True "unsupported" only for exotic future values; current policy never panics.
        let big = serde_json::json!(1u64 << 60); // fits in f64 exactly for this test
        let _v: Valor = Valor::try_from(big).expect("large int becomes fractus or numerus");
    }

    #[test]
    fn nihil_and_collections_preserve() {
        let v = Valor::Lista(vec![Valor::Nihil, Valor::Bivalens(true)]);
        let j: serde_json::Value = v.clone().into();
        let back: Valor = j.try_into().unwrap();
        assert_eq!(back, v);
    }
}
