//! Explicit conversion expression emission for the Rust backend.
//!
//! Faber's `conversio` surface is more policy-heavy than a plain Rust `as`.
//! Text-to-number conversions use Rust parsing, text/number-to-bool conversions
//! encode Faber truthiness, and only the remaining low-level primitive cases
//! fall back to Rust casts. Optional fallbacks are emitted at the conversion
//! site so lowering and typechecking can stay responsible for deciding whether
//! such a fallback exists.
//!
//! ERROR POLICY
//! ============
//! - Parse conversions without a fallback emit `.unwrap()` and therefore keep
//!   parse failure loud in generated Rust.
//! - Parse conversions with a fallback emit `.unwrap_or(fallback)`.
//! - Unsupported target-specific casts are not claimed here; the final `as`
//!   branch is only the Rust backend escape hatch for cases already accepted by
//!   earlier phases.

use super::super::types::type_to_rust;
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_conversio_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    target: TypeId,
    params: &[Symbol],
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // The source type is advisory but important: text conversions get parsing
    // and truthiness semantics, while other primitive conversions are emitted
    // as formatting or casts only after the typechecker has accepted them.
    let target_resolved = types.get(target);
    let source_ty = source.ty.map(|t| types.get(t));
    match (source_ty, target_resolved) {
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Numerus)) => {
            if let Some(radix) = radix_hint(codegen, params) {
                generate_radix_parse_expr(
                    codegen,
                    source,
                    radix,
                    fallback,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )
            } else {
                generate_parse_expr(
                    codegen,
                    source,
                    "i64",
                    fallback,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )
            }
        }
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Fractus)) => generate_parse_expr(
            codegen,
            source,
            "f64",
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Bivalens)) => generate_text_bool_expr(
            codegen,
            source,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Textus))
        | (Some(Type::Primitive(Primitive::Fractus)), Type::Primitive(Primitive::Textus)) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(".to_string()");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Fractus)) => {
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" as f64");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Bivalens)) => generate_number_bool_expr(
            codegen,
            source,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        ),
        (_, Type::Primitive(Primitive::Textus)) => {
            w.write("format!(\"{}\", ");
            generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
            Ok(())
        }
        _ => match target_resolved {
            Type::Primitive(Primitive::Numerus) | Type::Primitive(Primitive::Fractus) => {
                let target_text = type_to_rust(codegen, target, types);
                generate_parse_expr(
                    codegen,
                    source,
                    &target_text,
                    fallback,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )
            }
            _ => {
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" as ");
                w.write(&type_to_rust(codegen, target, types));
                Ok(())
            }
        },
    }
}

fn radix_hint(codegen: &RustCodegen<'_>, params: &[Symbol]) -> Option<u32> {
    params
        .iter()
        .find_map(|param| match codegen.resolve_symbol(*param) {
            "Hex" => Some(16),
            "Bin" => Some(2),
            "Oct" => Some(8),
            _ => None,
        })
}

#[allow(clippy::too_many_arguments)]
fn generate_parse_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    target: &str,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // `conversio` fallbacks are value fallbacks, not diagnostics. Without one,
    // the generated Rust intentionally preserves parse failure as a panic.
    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write(".parse::<");
    w.write(target);
    if let Some(fallback) = fallback {
        w.write(">().unwrap_or(");
        generate_expr(
            codegen,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(")");
    } else {
        w.write(">().unwrap()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_radix_parse_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    radix: u32,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("i64::from_str_radix(&(");
    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    w.write("), ");
    w.write(&radix.to_string());
    if let Some(fallback) = fallback {
        w.write(").unwrap_or(");
        generate_expr(
            codegen,
            fallback,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(")");
    } else {
        w.write(").unwrap()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_text_bool_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        w.write("if ");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".is_empty() { ");
        generate_expr(codegen, fb, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" } else { true }");
    } else {
        w.write("!");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(".is_empty()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_number_bool_expr(
    codegen: &RustCodegen<'_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        w.write("if ");
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" == 0 { ");
        generate_expr(codegen, fb, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" } else { true }");
    } else {
        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        w.write(" != 0");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_ref_expr(
    codegen: &RustCodegen<'_>,
    kind: HirRefKind,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match kind {
        HirRefKind::Shared => w.write("&"),
        HirRefKind::Mutable => w.write("&mut "),
    }
    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_deref_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("*");
    generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)
}
