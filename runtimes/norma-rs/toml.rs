//! toml.rs - TOML Encoding/Decoding Implementation
//!
//! Native Rust implementation of the TOML interface.
//! Uses the toml crate.
//!
//! Note: TOML root must be a table (object), not array or primitive.
//!
//! Verb meanings:
//!   - pange (compose): serialize table to TOML string
//!   - solve (untangle): parse TOML string to table
//!   - tempta (try): attempt to parse, return None on error

use toml::Value;

// =============================================================================
// SERIALIZATION
// =============================================================================

/// Serialize table to TOML string
pub fn pange(valor: &Value) -> String {
    toml::to_string(valor).expect("failed to serialize TOML")
}

// =============================================================================
// PARSING
// =============================================================================

/// Parse TOML string to table (panics on error)
pub fn solve(toml_str: &str) -> Value {
    toml::from_str(toml_str).expect("failed to parse TOML")
}

/// Attempt to parse TOML string (returns None on error)
pub fn tempta(toml_str: &str) -> Option<Value> {
    toml::from_str(toml_str).ok()
}

// =============================================================================
// TYPE CHECKING
// =============================================================================

/// Check if value is null (TOML doesn't have null, always false)
pub fn est_nihil(_valor: &Value) -> bool {
    false // TOML has no null type
}

/// Check if value is boolean
pub fn est_bivalens(valor: &Value) -> bool {
    valor.is_bool()
}

/// Check if value is string
pub fn est_textus(valor: &Value) -> bool {
    valor.is_str()
}

/// Check if value is integer
pub fn est_integer(valor: &Value) -> bool {
    valor.is_integer()
}

/// Check if value is float
pub fn est_fractus(valor: &Value) -> bool {
    valor.is_float()
}

/// Check if value is datetime
pub fn est_tempus(valor: &Value) -> bool {
    valor.is_datetime()
}

/// Check if value is array
pub fn est_lista(valor: &Value) -> bool {
    valor.is_array()
}

/// Check if value is table
pub fn est_tabula(valor: &Value) -> bool {
    valor.is_table()
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
    valor
        .as_float()
        .or_else(|| valor.as_integer().map(|i| i as f64))
        .unwrap_or(def_val)
}

/// Extract as boolean with default
pub fn ut_bivalens(valor: &Value, def_val: bool) -> bool {
    valor.as_bool().unwrap_or(def_val)
}

// =============================================================================
// VALUE ACCESS
// =============================================================================

/// Get value by key (returns integer 0 as sentinel for missing - TOML has no null)
pub fn cape(valor: &Value, clavis: &str) -> Option<Value> {
    valor.get(clavis).cloned()
}

/// Pluck value by array index (returns None if out of bounds)
pub fn carpe(valor: &Value, index: usize) -> Option<Value> {
    valor.get(index).cloned()
}

/// Find value by dotted path (returns None if not found)
pub fn inveni(valor: &Value, via: &str) -> Option<Value> {
    let parts: Vec<&str> = via.split('.').collect();
    let mut current = valor;

    for part in parts {
        match current {
            Value::Table(map) => {
                current = match map.get(part) {
                    Some(v) => v,
                    None => return None,
                };
            }
            Value::Array(arr) => {
                let idx: usize = match part.parse() {
                    Ok(i) => i,
                    Err(_) => return None,
                };
                current = match arr.get(idx) {
                    Some(v) => v,
                    None => return None,
                };
            }
            _ => return None,
        }
    }

    Some(current.clone())
}
