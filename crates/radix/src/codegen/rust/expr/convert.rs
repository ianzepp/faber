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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_conversio_expr_with_emitter(&mut emitter, source, target, params, fallback)
}

#[allow(clippy::too_many_arguments)]
fn generate_conversio_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    source: &HirExpr,
    target: TypeId,
    params: &[Symbol],
    fallback: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    // The source type is advisory but important: text conversions get parsing
    // and truthiness semantics, while other primitive conversions are emitted
    // as formatting or casts only after the typechecker has accepted them.
    let target_resolved = emitter.types.get(target);
    let source_ty = source.ty.map(|t| emitter.types.get(t));
    match (source_ty, target_resolved) {
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Numerus)) => {
            if let Some(radix) = radix_hint(emitter.codegen, params) {
                generate_radix_parse_expr(emitter, source, radix, fallback)
            } else {
                generate_parse_expr(emitter, source, "i64", fallback)
            }
        }
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Fractus)) => {
            generate_parse_expr(emitter, source, "f64", fallback)
        }
        (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Bivalens)) => {
            generate_text_bool_expr(emitter, source, fallback)
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Textus))
        | (Some(Type::Primitive(Primitive::Fractus)), Type::Primitive(Primitive::Textus)) => {
            emitter.expr(source)?;
            emitter.writer.write(".to_string()");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Fractus)) => {
            emitter.expr(source)?;
            emitter.writer.write(" as f64");
            Ok(())
        }
        (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Bivalens)) => {
            generate_number_bool_expr(emitter, source, fallback)
        }
        (_, Type::Primitive(Primitive::Textus)) => {
            emitter.writer.write("format!(\"{}\", ");
            emitter.expr(source)?;
            emitter.writer.write(")");
            Ok(())
        }
        _ => match target_resolved {
            Type::Primitive(Primitive::Numerus) | Type::Primitive(Primitive::Fractus) => {
                let target_text = type_to_rust(emitter.codegen, target, emitter.types);
                generate_parse_expr(emitter, source, &target_text, fallback)
            }
            _ => {
                emitter.expr(source)?;
                emitter.writer.write(" as ");
                emitter
                    .writer
                    .write(&type_to_rust(emitter.codegen, target, emitter.types));
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
    emitter: &mut ExprEmitter<'_, '_>,
    source: &HirExpr,
    target: &str,
    fallback: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    // `conversio` fallbacks are value fallbacks, not diagnostics. Without one,
    // the generated Rust intentionally preserves parse failure as a panic.
    emitter.expr(source)?;
    emitter.writer.write(".parse::<");
    emitter.writer.write(target);
    if let Some(fallback) = fallback {
        emitter.writer.write(">().unwrap_or(");
        emitter.expr(fallback)?;
        emitter.writer.write(")");
    } else {
        emitter.writer.write(">().unwrap()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_radix_parse_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    source: &HirExpr,
    radix: u32,
    fallback: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    emitter.writer.write("i64::from_str_radix(&(");
    emitter.expr(source)?;
    emitter.writer.write("), ");
    emitter.writer.write(&radix.to_string());
    if let Some(fallback) = fallback {
        emitter.writer.write(").unwrap_or(");
        emitter.expr(fallback)?;
        emitter.writer.write(")");
    } else {
        emitter.writer.write(").unwrap()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_text_bool_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        emitter.writer.write("if ");
        emitter.expr(source)?;
        emitter.writer.write(".is_empty() { ");
        emitter.expr(fb)?;
        emitter.writer.write(" } else { true }");
    } else {
        emitter.writer.write("!");
        emitter.expr(source)?;
        emitter.writer.write(".is_empty()");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_number_bool_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    source: &HirExpr,
    fallback: Option<&HirExpr>,
) -> Result<(), CodegenError> {
    if let Some(fb) = fallback {
        emitter.writer.write("if ");
        emitter.expr(source)?;
        emitter.writer.write(" == 0 { ");
        emitter.expr(fb)?;
        emitter.writer.write(" } else { true }");
    } else {
        emitter.expr(source)?;
        emitter.writer.write(" != 0");
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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_ref_expr_with_emitter(&mut emitter, kind, expr)
}

fn generate_ref_expr_with_emitter(
    emitter: &mut ExprEmitter<'_, '_>,
    kind: HirRefKind,
    expr: &HirExpr,
) -> Result<(), CodegenError> {
    match kind {
        HirRefKind::Shared => emitter.writer.write("&"),
        HirRefKind::Mutable => emitter.writer.write("&mut "),
    }
    emitter.expr(expr)
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
    let mut emitter = ExprEmitter::new(
        codegen,
        types,
        w,
        ExprEmitPolicy::new(in_failable_fn, in_entry, suppress_error_propagation),
    );
    generate_deref_expr_with_emitter(&mut emitter, expr)
}

fn generate_deref_expr_with_emitter(emitter: &mut ExprEmitter<'_, '_>, expr: &HirExpr) -> Result<(), CodegenError> {
    emitter.writer.write("*");
    emitter.expr(expr)
}
