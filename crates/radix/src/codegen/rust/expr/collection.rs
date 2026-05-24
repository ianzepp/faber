//! Collection, tuple, and struct expression emission for the Rust backend.
//!
//! This module owns Faber collection construction policy after lowering has
//! already decided the expression shape. It deliberately keeps Rust temporaries
//! local to emitted blocks and names them from the HIR id so spread expansion
//! and shared object-key emission remain deterministic without
//! requiring a backend-wide name allocator.
//!
//! INVARIANTS
//! ==========
//! - Empty collection element/key/value types must arrive from earlier phases;
//!   this backend does not infer missing type information.
//! - Spread-bearing arrays are block expressions because Rust needs a mutable
//!   accumulator; non-spread arrays stay compact `vec![...]` expressions.
//! - Struct field omission is semantic, not cosmetic: omitted optional fields
//!   become `None`, and omitted fields with initializers are emitted through the
//!   same field-value wrapper path as provided fields.

use super::super::type_shape::{option_inner_or_self, resolve_type, type_id_is_faber_value};
use super::*;
use rustc_hash::FxHashSet;

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_array_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    expr_ty: Option<TypeId>,
    elements: &[HirArrayElement],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    let dynamic_elem_ty = dynamic_array_element_type(expr_ty, types);

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        // Rust's `vec![...]` cannot interleave single elements and spreads, so
        // spread arrays lower to a mutable accumulator and return that vector.
        let temp = format!("__faber_vec_{}", expr_id.0);
        w.writeln("{");
        let mut result = Ok(());
        w.indented(|w| {
            w.write("let mut ");
            w.write(&temp);
            w.writeln(" = Vec::new();");
            for element in elements {
                if result.is_err() {
                    return;
                }
                match element {
                    HirArrayElement::Expr(elem) => {
                        w.write(&temp);
                        w.write(".push(");
                        result = if let Some(elem_ty) = dynamic_elem_ty {
                            generate_expr_as_type(
                                codegen,
                                elem,
                                elem_ty,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            )
                        } else {
                            generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)
                        };
                        w.writeln(");");
                    }
                    HirArrayElement::Spread(elem) => {
                        w.write(&temp);
                        w.write(".extend((");
                        result = generate_expr(
                            codegen,
                            elem,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(").iter().cloned());");
                    }
                }
            }
            w.write(&temp);
            w.newline();
        });
        result?;
        w.write("}");
    } else {
        w.write("vec![");
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                w.write(", ");
            }
            let HirArrayElement::Expr(elem) = elem else {
                continue;
            };
            if let Some(elem_ty) = dynamic_elem_ty {
                generate_expr_as_type(
                    codegen,
                    elem,
                    elem_ty,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            } else {
                generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
        }
        w.write("]");
    }
    Ok(())
}

fn dynamic_array_element_type(expr_ty: Option<TypeId>, types: &TypeTable) -> Option<TypeId> {
    let Type::Array(elem_ty) = resolve_type(expr_ty?, types) else {
        return None;
    };
    type_id_is_faber_value(elem_ty, types).then_some(elem_ty)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_struct_expr(
    codegen: &RustCodegen<'_>,
    expr_id: HirId,
    def_id: DefId,
    fields: &[(Symbol, HirExpr)],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if codegen.struct_has_creo_hook(def_id) {
        let temp = format!("__faber_struct_{}", expr_id.0);
        w.writeln("{");
        let mut result = Ok(());
        w.indented(|w| {
            w.write("let mut ");
            w.write(&temp);
            w.write(" = ");
            result = generate_struct_literal_expr(
                codegen,
                def_id,
                fields,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            if result.is_err() {
                return;
            }
            w.writeln(";");
            w.write(&temp);
            w.writeln(".creo();");
            w.write(&temp);
            w.newline();
        });
        result?;
        w.write("}");
        return Ok(());
    }

    generate_struct_literal_expr(
        codegen,
        def_id,
        fields,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_struct_literal_expr(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    fields: &[(Symbol, HirExpr)],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write(codegen.resolve_def(def_id));
    w.writeln(" {");
    let mut struct_result = Ok(());
    let provided: FxHashSet<Symbol> = fields.iter().map(|(n, _)| *n).collect();
    w.indented(|w| {
        for (name, value) in fields {
            w.write(codegen.resolve_symbol(*name));
            w.write(": ");
            if struct_result.is_err() {
                return;
            }
            struct_result = generate_struct_field_value(
                codegen,
                def_id,
                *name,
                value,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
            w.writeln(",");
        }
        if struct_result.is_err() {
            return;
        }
        struct_result = generate_omitted_struct_fields(
            codegen,
            def_id,
            &provided,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        );
    });
    struct_result?;
    w.write("}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_struct_field_value(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // Optional field wrapping is decided against the Rust storage type, not the
    // source spelling. That keeps `T ∪ nihil` fields represented as `Option<T>`
    // even when the user supplies a bare non-option expression.
    if codegen.struct_field_stores_option(def_id, name, types)
        && !expr_may_already_produce_option(codegen, value, types)
    {
        w.write("Some(");
        generate_struct_value_expr(
            codegen,
            def_id,
            name,
            value,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?;
        w.write(")");
        return Ok(());
    }

    generate_struct_value_expr(
        codegen,
        def_id,
        name,
        value,
        types,
        w,
        in_failable_fn,
        in_entry,
        suppress_error_propagation,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_omitted_struct_fields(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    provided: &FxHashSet<Symbol>,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    for field in codegen.sorted_struct_omittable_fields(def_id) {
        if provided.contains(&field.name) {
            continue;
        }
        w.write(codegen.resolve_symbol(field.name));
        w.write(": ");
        if let Some(init) = field.init {
            generate_struct_field_value(
                codegen,
                def_id,
                field.name,
                init,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        } else {
            w.write("None");
        }
        w.writeln(",");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_struct_value_expr(
    codegen: &RustCodegen<'_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    if let HirExprKind::Literal(HirLiteral::Int(value)) = value.kind {
        if struct_field_value_is_fractus(codegen, def_id, name, types) {
            w.write(&format!("{value}.0"));
            return Ok(());
        }
    }

    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_)))
        && value.ty.is_none()
        && struct_field_value_is_textus(codegen, def_id, name, types)
    {
        w.write(".to_string()");
    }
    Ok(())
}

fn struct_field_value_is_fractus(codegen: &RustCodegen<'_>, def_id: DefId, name: Symbol, types: &TypeTable) -> bool {
    let Some(field) = codegen.struct_field_info(def_id, name) else {
        return false;
    };

    matches!(
        types.get(option_inner_or_self(field.ty, types)),
        Type::Primitive(Primitive::Fractus)
    )
}

fn struct_field_value_is_textus(codegen: &RustCodegen<'_>, def_id: DefId, name: Symbol, types: &TypeTable) -> bool {
    let Some(field) = codegen.struct_field_info(def_id, name) else {
        return false;
    };

    let value_type = match types.get(field.ty) {
        Type::Option(inner) => *inner,
        Type::Alias(_, resolved) => option_inner_or_self(*resolved, types),
        _ => field.ty,
    };
    matches!(types.get(value_type), Type::Primitive(Primitive::Textus))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn generate_tuple_expr(
    codegen: &RustCodegen<'_>,
    elements: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.write("(");
    for (i, elem) in elements.iter().enumerate() {
        if i > 0 {
            w.write(", ");
        }
        generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
    }
    w.write(")");
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(super) fn write_object_map_key(
    codegen: &RustCodegen<'_>,
    types: &TypeTable,
    key: &HirObjectKey,
    key_ty: TypeId,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    // Object keys are shared by literal maps and `verte` map construction. Bare
    // identifier keys become text only when the target map key type says so;
    // computed keys are already expressions and must not be stringified here.
    match key {
        HirObjectKey::Ident(key) | HirObjectKey::String(key) => {
            write_innatum_map_key(codegen, types, *key, key_ty, w);
        }
        HirObjectKey::Computed(expr) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirObjectKey::Spread(_) => {}
    }
    Ok(())
}
pub(super) fn write_innatum_map_key(
    codegen: &RustCodegen<'_>,
    types: &TypeTable,
    key: Symbol,
    key_ty: TypeId,
    w: &mut CodeWriter,
) {
    if matches!(types.get(key_ty), Type::Primitive(Primitive::Textus)) {
        w.write("\"");
        for ch in codegen.resolve_symbol(key).chars() {
            match ch {
                '\\' => w.write("\\\\"),
                '"' => w.write("\\\""),
                '\n' => w.write("\\n"),
                '\r' => w.write("\\r"),
                '\t' => w.write("\\t"),
                _ => w.write(&ch.to_string()),
            }
        }
        w.write("\".to_string()");
        return;
    }

    w.write(codegen.resolve_symbol(key));
}
