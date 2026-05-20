use super::*;
pub(super) fn apply_regex_flags(pattern: &str, flags: Option<&str>) -> String {
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
