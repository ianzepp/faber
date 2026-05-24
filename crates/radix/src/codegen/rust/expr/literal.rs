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
pub(super) fn write_rust_string_literal(text: &str, writer: &mut CodeWriter) {
    // Keep expression string escaping centralized so string and regex literal
    // emission do not drift in their Rust literal policy.
    writer.write("\"");
    for ch in text.chars() {
        match ch {
            '\\' => writer.write("\\\\"),
            '"' => writer.write("\\\""),
            '\n' => writer.write("\\n"),
            '\r' => writer.write("\\r"),
            '\t' => writer.write("\\t"),
            _ => writer.write(&ch.to_string()),
        }
    }
    writer.write("\"");
}
/// Generate a Rust literal.
///
/// TRANSFORMS:
///   HirLiteral::String("hello") -> "hello" (with escaping)
///   HirLiteral::Bool(true)      -> true
///   HirLiteral::Nil             -> None
///
/// TARGET: Rust string escaping (\n, \t, \\, \").
pub(super) fn generate_literal(codegen: &RustCodegen<'_>, lit: &HirLiteral, writer: &mut CodeWriter) {
    match lit {
        HirLiteral::Int(n) => {
            writer.write(&n.to_string());
        }
        HirLiteral::Float(f) => {
            writer.write(&rust_float_literal(*f));
        }
        HirLiteral::String(s) => {
            write_rust_string_literal(codegen.resolve_symbol(*s), writer);
        }
        HirLiteral::Regex(pattern, flags) => {
            let pattern_text = codegen.resolve_symbol(*pattern);
            let regex_pattern = apply_regex_flags(pattern_text, flags.map(|f| codegen.resolve_symbol(f)));
            writer.write("regex::Regex::new(");
            write_rust_string_literal(&regex_pattern, writer);
            writer.write(").unwrap()");
        }
        HirLiteral::Bool(b) => {
            writer.write(if *b { "true" } else { "false" });
        }
        HirLiteral::Nil => {
            writer.write("None");
        }
    }
}

fn rust_float_literal(value: f64) -> String {
    let rendered = value.to_string();
    if value.is_finite() && !rendered.contains(['.', 'e', 'E']) {
        format!("{rendered}.0")
    } else {
        rendered
    }
}

pub(super) fn generate_assert_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    cond: &HirExpr,
    message: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    emitter.writer.write("assert!(");
    emitter.expr(cond)?;
    if let Some(message) = message {
        emitter.writer.write(", \"{}\", ");
        emitter.expr(message)?;
    }
    emitter.writer.write(")");
    Ok(())
}

pub(super) fn generate_panic_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    emitter.writer.write("panic!(\"{}\", ");
    emitter.expr(value)?;
    emitter.writer.write(")");
    Ok(())
}

pub(super) fn generate_throw_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    if emitter.policy.permits_question_mark() {
        emitter.writer.write("return Err(");
        if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_))) {
            emitter.writer.write("String::from(");
            emitter.expr(value)?;
            emitter.writer.write(")");
        } else {
            emitter.writer.write("format!(\"{}\", ");
            emitter.expr(value)?;
            emitter.writer.write(")");
        }
        emitter.writer.write(")");
    } else {
        generate_panic_expr_with_emitter(emitter, value)?;
    }
    Ok(())
}
