//! json.rs - JSON Encoding/Decoding Implementation
//!
//! Native Rust implementation of the JSON interface.
//! Uses serde_json.
//!
//! Verb meanings:
//!   - pange (compose): serialize value to JSON string
//!   - solve (untangle): parse JSON string to value
//!   - tempta (try): attempt to parse, return None on error

use serde_json::Value;

// =============================================================================
// SERIALIZATION
// =============================================================================

/// Serialize value to JSON string (indentum > 0 for pretty-print)
pub fn pange(valor: &Value, indentum: Option<usize>) -> String {
    match indentum {
        Some(n) if n > 0 => serde_json::to_string_pretty(valor).expect("failed to serialize JSON"),
        _ => serde_json::to_string(valor).expect("failed to serialize JSON"),
    }
}

// =============================================================================
// PARSING
// =============================================================================

/// Parse JSON string to value (panics on error)
pub fn solve(json: &str) -> Value {
    serde_json::from_str(json).expect("failed to parse JSON")
}

/// Attempt to parse JSON string (returns None on error)
pub fn tempta(json: &str) -> Option<Value> {
    serde_json::from_str(json).ok()
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
    valor.is_boolean()
}

/// Check if value is number
pub fn est_numerus(valor: &Value) -> bool {
    valor.is_number()
}

/// Check if value is string
pub fn est_textus(valor: &Value) -> bool {
    valor.is_string()
}

/// Check if value is array
pub fn est_lista(valor: &Value) -> bool {
    valor.is_array()
}

/// Check if value is object
pub fn est_tabula(valor: &Value) -> bool {
    valor.is_object()
}

// =============================================================================
// VALUE EXTRACTION
// =============================================================================

/// Extract as string with default
pub fn ut_textus(valor: &Value, def_val: &str) -> String {
    valor.as_str().map(|s| s.to_string()).unwrap_or_else(|| def_val.to_string())
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
    valor.get(clavis).cloned().unwrap_or(Value::Null)
}

/// Pluck value by array index (returns null if out of bounds)
pub fn carpe(valor: &Value, index: usize) -> Value {
    valor.get(index).cloned().unwrap_or(Value::Null)
}

/// Find value by dotted path (returns null if not found)
pub fn inveni(valor: &Value, via: &str) -> Value {
    let parts: Vec<&str> = via.split('.').collect();
    let mut current = valor;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = match map.get(part) {
                    Some(v) => v,
                    None => return Value::Null,
                };
            }
            Value::Array(arr) => {
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
