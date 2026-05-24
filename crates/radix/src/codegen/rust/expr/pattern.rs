//! Pattern emission for Rust `match`-like backend surfaces.
//!
//! Patterns are intentionally narrower than expressions: they can bind, alias,
//! match variants, or reuse literal emission. This module does not invent
//! destructuring behavior beyond the HIR shape. When definition resolution is
//! unavailable, binding names fall back to their source symbols so diagnostics
//! and generated code remain readable instead of collapsing to a sentinel.

use super::*;
pub(super) fn generate_pattern(codegen: &RustCodegen<'_>, pattern: &HirPattern, writer: &mut CodeWriter) {
    match pattern {
        HirPattern::Wildcard => {
            writer.write("_");
        }
        HirPattern::Binding(def_id, name) => {
            let resolved = codegen.resolve_def(*def_id);
            // The unresolved sentinel is a backend recovery value, not a legal
            // Rust binding. Prefer the user's symbol when HIR resolution failed.
            if resolved == "unresolved_def" {
                writer.write(codegen.resolve_symbol(*name));
            } else {
                writer.write(resolved);
            }
        }
        HirPattern::Alias(def_id, name, pattern) => {
            let resolved = codegen.resolve_def(*def_id);
            // Alias bindings follow the same recovery rule as plain bindings,
            // then emit Rust's `name @ pattern` form.
            if resolved == "unresolved_def" {
                writer.write(codegen.resolve_symbol(*name));
            } else {
                writer.write(resolved);
            }
            writer.write(" @ ");
            generate_pattern(codegen, pattern, writer);
        }
        HirPattern::Variant(def_id, fields) => {
            if let Some(variant) = codegen.variant_info(*def_id) {
                writer.write(codegen.resolve_def(variant.enum_def));
                writer.write("::");
            }
            writer.write(codegen.resolve_def(*def_id));
            if !fields.is_empty() {
                writer.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        writer.write(", ");
                    }
                    generate_pattern(codegen, field, writer);
                }
                writer.write(" }");
            }
        }
        HirPattern::Literal(lit) => {
            generate_literal(codegen, lit, writer);
        }
    }
}
