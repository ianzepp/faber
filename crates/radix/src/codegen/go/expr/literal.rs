//! Literal, formatting, and console expression lowering for Go.
//!
//! This module owns expression forms that imply Go standard-library imports:
//! `fmt` for formatted strings, printing, and panic text; `os` for stderr
//! output; and `regexp` for regex literals. Import collection is handled by the
//! wider Go backend, but the expression choices here are the reason those
//! imports may be required.
//!
//! TARGET CONTRACTS
//! ================
//! - `scriptum` uses `fmt.Sprintf`; each `§` placeholder becomes `%v`, and
//!   indexed placeholders become Go's one-based `%[n]v` form.
//! - Literal percent signs are escaped before reaching `fmt`.
//! - `scribe`-family output lowers to Go's line-printing functions; `mone`
//!   writes to stderr.
//! - String literals use Rust debug escaping for the generated Go string token.
//! - Regex literals use `regexp.MustCompile`, so invalid generated patterns fail
//!   at Go program initialization or execution time.
//! - `nihil` lowers to Go `nil`.

use super::*;
use crate::hir::HirScribeKind;

pub(super) fn render_scriptum_template(template: &str, _arg_count: usize) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => rendered.push_str("%%"),
            '§' => {
                let mut index = String::new();
                while chars.peek().is_some_and(|next| next.is_ascii_digit()) {
                    index.push(chars.next().expect("peeked digit"));
                }
                if index.is_empty() {
                    rendered.push_str("%v");
                } else if let Ok(index) = index.parse::<usize>() {
                    rendered.push_str(&format!("%[{}]v", index + 1));
                } else {
                    rendered.push_str("%!v(BADINDEX)");
                }
            }
            _ => rendered.push(ch),
        }
    }
    rendered
}
pub(super) fn generate_scribe_expr(
    codegen: &GoCodegen<'_>,
    kind: HirScribeKind,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let function = match kind {
        HirScribeKind::Mone => "fmt.Fprintln(os.Stderr, ",
        HirScribeKind::Nota | HirScribeKind::Vide | HirScribeKind::Scribe => "fmt.Println(",
    };
    w.write(function);
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(())
}

pub(super) fn generate_scriptum_expr(
    codegen: &GoCodegen<'_>,
    template: crate::lexer::Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("fmt.Sprintf(");
    w.write(&format!(
        "{:?}",
        render_scriptum_template(codegen.resolve_symbol(template), args.len())
    ));
    for arg in args {
        w.write(", ");
        generate_expr(codegen, arg, types, w)?;
    }
    w.write(")");
    Ok(())
}

pub(super) fn generate_panic_expr(
    codegen: &GoCodegen<'_>,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("panic(fmt.Sprint(");
    generate_expr(codegen, value, types, w)?;
    w.write("))");
    Ok(())
}
pub(super) fn generate_literal(codegen: &GoCodegen<'_>, literal: &HirLiteral, w: &mut CodeWriter) {
    match literal {
        HirLiteral::Int(v) => w.write(&v.to_string()),
        HirLiteral::Float(v) => w.write(&v.to_string()),
        HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        HirLiteral::Regex(pattern, flags) => {
            // EDGE: This is a direct Go regexp literal path. The backend
            // does not pre-validate Go regexp syntax or escape raw backticks.
            w.write("regexp.MustCompile(`");
            w.write(codegen.resolve_symbol(*pattern));
            if let Some(flags) = flags {
                w.write("(?");
                w.write(codegen.resolve_symbol(*flags));
                w.write(")");
            }
            w.write("`)");
        }
        HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        HirLiteral::Nil => w.write("nil"),
    }
}
