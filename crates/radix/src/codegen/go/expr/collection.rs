//! Collection and aggregate expression lowering for the Go backend.
//!
//! This module owns expression forms whose Go representation depends on both
//! the HIR shape and the resolved Faber type: structs, tuples, arrays, and
//! maps. The code generator writes Go expressions directly, so the contracts
//! here are deliberately conservative:
//! known aggregate types are emitted with their target shape, spread support is
//! lowered only where Go can model it cleanly, and missing semantic type data
//! remains visible instead of being guessed late in codegen.
//!
//! TARGET CONTRACTS
//! ================
//! - Struct fields are emitted as exported Go field names and `sponte` fields
//!   are wrapped as pointers at construction time.
//! - Tuples are represented as `[]any` because Go has no tuple value type.
//! - Array spread lowers through an immediately invoked function that appends
//!   elements in source order.
//! - Object spread is not merged into Go struct or map literals here; unsupported
//!   spread entries are skipped rather than claiming partial merge semantics.
//! - Ident and string object keys are emitted as string keys for maps; computed
//!   keys keep their expression form.

use super::*;
use crate::hir::HirObjectKey;

pub(super) fn generate_struct_expr(
    codegen: &GoCodegen<'_>,
    def_id: crate::hir::DefId,
    fields: &[(crate::lexer::Symbol, HirExpr)],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.write("{");
    for (idx, (name, value)) in fields.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        w.write(&capitalize(codegen.resolve_symbol(*name)));
        w.write(": ");
        if let Some(field_ty) = codegen.struct_field_type(def_id, *name) {
            // WHY: Faber `sponte` fields are nullable declaration slots; Go
            // represents that optionality as a pointer to the concrete field type.
            if codegen.struct_field_is_sponte(def_id, *name) {
                generate_option_wrapped_expr(codegen, value, field_ty, types, w)?;
            } else {
                generate_expr_for_go_type(codegen, value, field_ty, types, w)?;
            }
        } else {
            generate_expr(codegen, value, types, w)?;
        }
    }
    w.write("}");
    Ok(())
}

pub(super) fn generate_tuple_expr(
    codegen: &GoCodegen<'_>,
    elements: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    // WHY: Go has no tuples; a slice preserves multi-value expression shape.
    w.write("[]any{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, element, types, w)?;
    }
    w.write("}");
    Ok(())
}

pub(super) fn generate_array_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let elem_ty = expr
        .ty
        .and_then(|ty| match normalize_receiver_type(types.get(ty), types) {
            Type::Array(elem) => Some(types::type_to_go(codegen, *elem, types)),
            _ => None,
        })
        .unwrap_or_else(|| "any".to_owned());

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        // WHY: Go composite literals cannot contain spread elements, so an IIFE
        // preserves source ordering while letting append expand spread operands.
        w.write("func() []");
        w.write(&elem_ty);
        w.write(" { acc := []");
        w.write(&elem_ty);
        w.write("{}; ");
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("); ");
                }
                HirArrayElement::Spread(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("...); ");
                }
            }
        }
        w.write("return acc }()");
        return Ok(());
    }

    w.write("[]");
    w.write(&elem_ty);
    w.write("{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        let HirArrayElement::Expr(expr) = element else { continue };
        generate_expr(codegen, expr, types, w)?;
    }
    w.write("}");
    Ok(())
}
pub(super) fn generate_typed_array_expr(
    codegen: &GoCodegen<'_>,
    elem_ty: crate::semantic::TypeId,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let elem_go_ty = types::type_to_go(codegen, elem_ty, types);

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        // WHY: Typed literals need the same spread policy as inferred arrays,
        // but scalar elements are converted to the declared element type first.
        w.write("func() []");
        w.write(&elem_go_ty);
        w.write(" { acc := []");
        w.write(&elem_go_ty);
        w.write("{}; ");
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr_for_go_type(codegen, expr, elem_ty, types, w)?;
                    w.write("); ");
                }
                HirArrayElement::Spread(expr) => {
                    w.write("acc = append(acc, ");
                    generate_expr(codegen, expr, types, w)?;
                    w.write("...); ");
                }
            }
        }
        w.write("return acc }()");
        return Ok(());
    }

    w.write("[]");
    w.write(&elem_go_ty);
    w.write("{");
    for (idx, element) in elements.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        let HirArrayElement::Expr(expr) = element else { continue };
        generate_expr_for_go_type(codegen, expr, elem_ty, types, w)?;
    }
    w.write("}");
    Ok(())
}
pub(super) fn generate_map_literal(
    codegen: &GoCodegen<'_>,
    key_ty: crate::semantic::TypeId,
    value_ty: crate::semantic::TypeId,
    entries: Option<&[crate::hir::HirObjectField]>,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    w.write("map[");
    w.write(&types::type_to_go(codegen, key_ty, types));
    w.write("]");
    w.write(&types::type_to_go(codegen, value_ty, types));
    w.write("{");
    if let Some(entries) = entries {
        let mut wrote_any = false;
        for field in entries {
            let Some(value) = &field.value else { continue };
            if wrote_any {
                w.write(", ");
            }
            match &field.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    w.write(&format!("{:?}", codegen.resolve_symbol(*name)));
                }
                HirObjectKey::Computed(expr) => generate_expr(codegen, expr, types, w)?,
                // EDGE: Map spread has no Go literal equivalent in this path.
                // Earlier phases must lower or reject real spread semantics.
                HirObjectKey::Spread(_) => continue,
            }
            w.write(": ");
            generate_expr_for_go_type(codegen, value, value_ty, types, w)?;
            wrote_any = true;
        }
    }
    w.write("}");
    Ok(())
}
