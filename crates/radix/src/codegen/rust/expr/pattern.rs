//! Pattern emission for Rust `match`-like backend surfaces.
//!
//! Patterns are intentionally narrower than expressions: they can bind, alias,
//! match variants, or reuse literal emission. This module does not invent
//! destructuring behavior beyond the HIR shape. When definition resolution is
//! unavailable, binding names fall back to their source symbols so diagnostics
//! and generated code remain readable instead of collapsing to a sentinel.

use super::*;
pub(super) fn generate_pattern(codegen: &RustCodegen<'_>, pattern: &HirPattern, w: &mut CodeWriter) {
    match pattern {
        HirPattern::Wildcard => {
            w.write("_");
        }
        HirPattern::Binding(def_id, name) => {
            let resolved = codegen.resolve_def(*def_id);
            // The unresolved sentinel is a backend recovery value, not a legal
            // Rust binding. Prefer the user's symbol when HIR resolution failed.
            if resolved == "unresolved_def" {
                w.write(codegen.resolve_symbol(*name));
            } else {
                w.write(resolved);
            }
        }
        HirPattern::Alias(def_id, name, pattern) => {
            let resolved = codegen.resolve_def(*def_id);
            // Alias bindings follow the same recovery rule as plain bindings,
            // then emit Rust's `name @ pattern` form.
            if resolved == "unresolved_def" {
                w.write(codegen.resolve_symbol(*name));
            } else {
                w.write(resolved);
            }
            w.write(" @ ");
            generate_pattern(codegen, pattern, w);
        }
        HirPattern::Variant(def_id, fields) => {
            w.write(codegen.resolve_def(*def_id));
            if !fields.is_empty() {
                w.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        w.write(", ");
                    }
                    generate_pattern(codegen, field, w);
                }
                w.write(" }");
            }
        }
        HirPattern::Literal(lit) => {
            generate_literal(codegen, lit, w);
        }
    }
}
