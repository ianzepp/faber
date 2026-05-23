//! Literal, assertion, panic, and throw emission for the Rust backend.
//!
//! Literals are intentionally emitted close to Rust's native surface: numbers
//! are printed as numeric tokens, Faber strings become escaped Rust string
//! literals, `nihil` becomes `None`, and regex literals construct
//! `regex::Regex` values at runtime. This file does not validate regex syntax;
//! it preserves the backend contract that invalid literals fail where the
//! generated Rust constructs them.
//!
//! TARGET DETAILS
//! ==============
//! - Regex flags are filtered to the inline modifiers accepted by Rust regex.
//! - Unknown regex flags are ignored rather than translated to unsupported
//!   behavior.
//! - `throw` becomes `Err(String)` only inside failable non-entry functions
//!   where propagation has not been suppressed; otherwise it lowers to panic.

use super::*;
pub(super) fn apply_regex_flags(pattern: &str, flags: Option<&str>) -> String {
    // Rust regex exposes flags as inline modifiers. The frontend may preserve
    // broader source spellings, but this backend only emits modifiers the
    // target engine accepts.
    let Some(flags) = flags else {
        return pattern.to_owned();
    };

    let mapped: String = flags
        .chars()
        .filter(|flag| matches!(flag, 'i' | 'm' | 's' | 'u' | 'U' | 'x'))
        .collect();

    if mapped.is_empty() {
        pattern.to_owned()
    } else {
        format!("(?{}){}", mapped, pattern)
    }
}
pub(super) fn write_rust_string_literal(text: &str, w: &mut CodeWriter) {
    // Keep expression string escaping centralized so string and regex literal
    // emission do not drift in their Rust literal policy.
    w.write("\"");
    for ch in text.chars() {
        match ch {
            '\\' => w.write("\\\\"),
            '"' => w.write("\\\""),
            '\n' => w.write("\\n"),
            '\r' => w.write("\\r"),
            '\t' => w.write("\\t"),
            _ => w.write(&ch.to_string()),
        }
    }
    w.write("\"");
}
/// Generate a Rust literal.
///
/// TRANSFORMS:
///   HirLiteral::String("hello") -> "hello" (with escaping)
///   HirLiteral::Bool(true)      -> true
///   HirLiteral::Nil             -> None
///
/// TARGET: Rust string escaping (\n, \t, \\, \").
pub(super) fn generate_literal(codegen: &RustCodegen<'_>, lit: &HirLiteral, w: &mut CodeWriter) {
    match lit {
        HirLiteral::Int(n) => {
            w.write(&n.to_string());
        }
        HirLiteral::Float(f) => {
            w.write(&f.to_string());
        }
        HirLiteral::String(s) => {
            write_rust_string_literal(codegen.resolve_symbol(*s), w);
        }
        HirLiteral::Regex(pattern, flags) => {
            let pattern_text = codegen.resolve_symbol(*pattern);
            let regex_pattern = apply_regex_flags(pattern_text, flags.map(|f| codegen.resolve_symbol(f)));
            w.write("regex::Regex::new(");
            write_rust_string_literal(&regex_pattern, w);
            w.write(").unwrap()");
        }
        HirLiteral::Bool(b) => {
            w.write(if *b { "true" } else { "false" });
        }
        HirLiteral::Nil => {
            w.write("None");
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_assert_expr(
    codegen: &RustCodegen<'_>,
    cond: &HirExpr,
    message: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("assert!(");
    generate_expr(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if let Some(message) = message {
        w.write(", \"{}\", ");
        generate_expr(codegen, message, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_panic_expr(
    codegen: &RustCodegen<'_>,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("panic!(\"{}\", ");
    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(")");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_throw_expr(
    codegen: &RustCodegen<'_>,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if in_failable_fn && !in_entry && !suppress_error_propagation {
        w.write("return Err(");
        if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_))) {
            w.write("String::from(");
            generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        } else {
            w.write("format!(\"{}\", ");
            generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        }
        w.write(")");
    } else {
        generate_panic_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    Ok(())
}
