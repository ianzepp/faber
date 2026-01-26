//! yaml.rs - YAML Encoding/Decoding Implementation
//!
//! Native Rust implementation of the YAML interface.
//! Uses serde_yaml.
//!
//! Verb meanings:
//!   - pange (compose): serialize value to YAML string
//!   - necto (bind): bind multiple documents into multi-doc YAML
//!   - solve (untangle): parse YAML string to value
//!   - tempta (try): attempt to parse, return None on error
//!   - collige (gather): gather all documents from multi-doc YAML

use serde::Deserialize;
use serde_yaml::Value;

// =============================================================================
// SERIALIZATION
// =============================================================================

/// Serialize value to YAML string
pub fn pange(valor: &Value) -> String {
    serde_yaml::to_string(valor).expect("failed to serialize YAML")
}

/// Bind multiple documents into multi-doc YAML string
pub fn necto(documenta: &[Value]) -> String {
    documenta
        .iter()
        .map(|doc| serde_yaml::to_string(doc).expect("failed to serialize YAML"))
        .collect::<Vec<_>>()
        .join("---\n")
}

// =============================================================================
// PARSING
// =============================================================================

/// Parse YAML string to value (panics on error)
pub fn solve(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).expect("failed to parse YAML")
}

/// Attempt to parse YAML string (returns None on error)
pub fn tempta(yaml: &str) -> Option<Value> {
    serde_yaml::from_str(yaml).ok()
}

/// Gather all documents from multi-doc YAML string
pub fn collige(yaml: &str) -> Vec<Value> {
    let mut docs = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(yaml) {
        if let Ok(value) = Value::deserialize(doc) {
            docs.push(value);
        }
    }
    docs
}

// =============================================================================
// TYPE CHECKING
// =============================================================================

/// Check if value is null
pub fn est_nihil(valor: &Value) -> bool {
    valor.is_null()
}

/// Check if value is boolean
pub fn est_bivalens(valor: &Value) -> bool {
    valor.is_bool()
}

/// Check if value is number
pub fn est_numerus(valor: &Value) -> bool {
    valor.is_number()
}

/// Check if value is string
pub fn est_textus(valor: &Value) -> bool {
    valor.is_string()
}

/// Check if value is array/sequence
pub fn est_lista(valor: &Value) -> bool {
    valor.is_sequence()
}

/// Check if value is object/mapping
pub fn est_tabula(valor: &Value) -> bool {
    valor.is_mapping()
}

// =============================================================================
// VALUE EXTRACTION
// =============================================================================

/// Extract as string with default
pub fn ut_textus(valor: &Value, def_val: &str) -> String {
    valor
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| def_val.to_string())
}

/// Extract as number with default
pub fn ut_numerus(valor: &Value, def_val: f64) -> f64 {
    valor.as_f64().unwrap_or(def_val)
}

/// Extract as boolean with default
pub fn ut_bivalens(valor: &Value, def_val: bool) -> bool {
    valor.as_bool().unwrap_or(def_val)
}

// =============================================================================
// VALUE ACCESS
// =============================================================================

/// Get value by key (returns null if missing)
pub fn cape(valor: &Value, clavis: &str) -> Value {
    valor
        .get(clavis)
        .cloned()
        .unwrap_or(Value::Null)
}

/// Pluck value by array index (returns null if out of bounds)
pub fn carpe(valor: &Value, index: usize) -> Value {
    valor
        .get(index)
        .cloned()
        .unwrap_or(Value::Null)
}

/// Find value by dotted path (returns null if not found)
pub fn inveni(valor: &Value, via: &str) -> Value {
    let parts: Vec<&str> = via.split('.').collect();
    let mut current = valor;

    for part in parts {
        match current {
            Value::Mapping(map) => {
                let key = Value::String(part.to_string());
                current = match map.get(&key) {
                    Some(v) => v,
                    None => return Value::Null,
                };
            }
            Value::Sequence(arr) => {
                let idx: usize = match part.parse() {
                    Ok(i) => i,
                    Err(_) => return Value::Null,
                };
                current = match arr.get(idx) {
                    Some(v) => v,
                    None => return Value::Null,
                };
            }
            _ => return Value::Null,
        }
    }

    current.clone()
}
