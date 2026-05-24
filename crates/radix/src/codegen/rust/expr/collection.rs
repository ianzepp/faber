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

pub(super) fn emit_array_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    expr_ty: Option<TypeId>,
    elements: &[HirArrayElement],
) -> Result<(), CodegenError> {
    let dynamic_elem_ty = dynamic_array_element_type(expr_ty, emitter.types);

    if elements
        .iter()
        .any(|element| matches!(element, HirArrayElement::Spread(_)))
    {
        // Rust's `vec![...]` cannot interleave single elements and spreads, so
        // spread arrays lower to a mutable accumulator and return that vector.
        let temp = format!("__faber_vec_{}", expr_id.0);
        emitter.writer.writeln("{");
        let mut result = Ok(());
        let codegen = emitter.codegen;
        let types = emitter.types;
        let policy = emitter.policy;
        emitter.writer.indented(|writer| {
            let mut nested = ExprEmitter::new(codegen, types, writer, policy);
            nested.writer.write("let mut ");
            nested.writer.write(&temp);
            nested.writer.writeln(" = Vec::new();");
            for element in elements {
                if result.is_err() {
                    return;
                }
                match element {
                    HirArrayElement::Expr(elem) => {
                        nested.writer.write(&temp);
                        nested.writer.write(".push(");
                        result = if let Some(elem_ty) = dynamic_elem_ty {
                            nested.expr_as_type(elem, elem_ty)
                        } else {
                            nested.expr(elem)
                        };
                        nested.writer.writeln(");");
                    }
                    HirArrayElement::Spread(elem) => {
                        nested.writer.write(&temp);
                        nested.writer.write(".extend((");
                        result = nested.expr(elem);
                        nested.writer.writeln(").iter().cloned());");
                    }
                }
            }
            nested.writer.write(&temp);
            nested.writer.newline();
        });
        result?;
        emitter.writer.write("}");
    } else {
        emitter.writer.write("vec![");
        for (i, elem) in elements.iter().enumerate() {
            if i > 0 {
                emitter.writer.write(", ");
            }
            let HirArrayElement::Expr(elem) = elem else {
                continue;
            };
            if let Some(elem_ty) = dynamic_elem_ty {
                emitter.expr_as_type(elem, elem_ty)?;
            } else {
                emitter.expr(elem)?;
            }
        }
        emitter.writer.write("]");
    }
    Ok(())
}

fn dynamic_array_element_type(expr_ty: Option<TypeId>, types: &TypeTable) -> Option<TypeId> {
    let Type::Array(elem_ty) = resolve_type(expr_ty?, types) else {
        return None;
    };
    type_id_is_faber_value(elem_ty, types).then_some(elem_ty)
}

pub(super) fn emit_struct_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    def_id: DefId,
    fields: &[(Symbol, HirExpr)],
) -> Result<(), CodegenError> {
    if emitter.codegen.struct_has_creo_hook(def_id) {
        let temp = format!("__faber_struct_{}", expr_id.0);
        emitter.writer.writeln("{");
        let mut result = Ok(());
        let codegen = emitter.codegen;
        let types = emitter.types;
        let policy = emitter.policy;
        emitter.writer.indented(|writer| {
            let mut nested = ExprEmitter::new(codegen, types, writer, policy);
            nested.writer.write("let mut ");
            nested.writer.write(&temp);
            nested.writer.write(" = ");
            result = emit_struct_literal_expr(&mut nested, def_id, fields);
            if result.is_err() {
                return;
            }
            nested.writer.writeln(";");
            nested.writer.write(&temp);
            nested.writer.writeln(".creo();");
            nested.writer.write(&temp);
            nested.writer.newline();
        });
        result?;
        emitter.writer.write("}");
        return Ok(());
    }

    emit_struct_literal_expr(emitter, def_id, fields)
}

fn emit_struct_literal_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    fields: &[(Symbol, HirExpr)],
) -> Result<(), CodegenError> {
    emitter.writer.write(emitter.codegen.resolve_def(def_id));
    emitter.writer.writeln(" {");
    let mut struct_result = Ok(());
    let provided: FxHashSet<Symbol> = fields.iter().map(|(n, _)| *n).collect();
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|writer| {
        let mut nested = ExprEmitter::new(codegen, types, writer, policy);
        for (name, value) in fields {
            nested.writer.write(codegen.resolve_symbol(*name));
            nested.writer.write(": ");
            if struct_result.is_err() {
                return;
            }
            struct_result = emit_struct_field_value(&mut nested, def_id, *name, value);
            nested.writer.writeln(",");
        }
        if struct_result.is_err() {
            return;
        }
        struct_result = emit_omitted_struct_fields(&mut nested, def_id, &provided);
    });
    struct_result?;
    emitter.writer.write("}");
    Ok(())
}

pub(super) fn emit_struct_field_value(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    // Optional field wrapping is decided against the Rust storage type, not the
    // source spelling. That keeps `T ∪ nihil` fields represented as `Option<T>`
    // even when the user supplies a bare non-option expression.
    if emitter
        .codegen
        .struct_field_stores_option(def_id, name, emitter.types)
        && !expr_may_already_produce_option(emitter.codegen, value, emitter.types)
    {
        emitter.writer.write("Some(");
        emit_struct_value_expr(emitter, def_id, name, value)?;
        emitter.writer.write(")");
        return Ok(());
    }

    emit_struct_value_expr(emitter, def_id, name, value)
}

pub(super) fn emit_omitted_struct_fields(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    provided: &FxHashSet<Symbol>,
) -> Result<(), CodegenError> {
    for field in emitter.codegen.sorted_struct_omittable_fields(def_id) {
        if provided.contains(&field.name) {
            continue;
        }
        emitter
            .writer
            .write(emitter.codegen.resolve_symbol(field.name));
        emitter.writer.write(": ");
        if let Some(init) = field.init {
            emit_struct_field_value(emitter, def_id, field.name, init)?;
        } else {
            emitter.writer.write("None");
        }
        emitter.writer.writeln(",");
    }
    Ok(())
}

fn emit_struct_value_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    name: Symbol,
    value: &HirExpr,
) -> Result<(), CodegenError> {
    if let HirExprKind::Literal(HirLiteral::Int(value)) = value.kind {
        if struct_field_value_is_fractus(emitter.codegen, def_id, name, emitter.types) {
            emitter.writer.write(&format!("{value}.0"));
            return Ok(());
        }
    }

    emitter.expr(value)?;
    if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_)))
        && value.ty.is_none()
        && struct_field_value_is_textus(emitter.codegen, def_id, name, emitter.types)
    {
        emitter.writer.write(".to_string()");
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

pub(super) fn emit_tuple_expr(emitter: &mut ExprEmitter<'_, '_>, elements: &[HirExpr]) -> Result<(), CodegenError> {
    emitter.writer.write("(");
    for (i, elem) in elements.iter().enumerate() {
        if i > 0 {
            emitter.writer.write(", ");
        }
        emitter.expr(elem)?;
    }
    emitter.writer.write(")");
    Ok(())
}

pub(super) fn emit_object_map_key(
    emitter: &mut ExprEmitter<'_, '_>,
    key: &HirObjectKey,
    key_ty: TypeId,
) -> Result<(), CodegenError> {
    // Object keys are shared by literal maps and `verte` map construction. Bare
    // identifier keys become text only when the target map key type says so;
    // computed keys are already expressions and must not be stringified here.
    match key {
        HirObjectKey::Ident(key) | HirObjectKey::String(key) => {
            write_innatum_map_key(emitter.codegen, emitter.types, *key, key_ty, emitter.writer);
        }
        HirObjectKey::Computed(expr) => {
            emitter.expr(expr)?;
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
    writer: &mut CodeWriter,
) {
    if matches!(types.get(key_ty), Type::Primitive(Primitive::Textus)) {
        writer.write("\"");
        for ch in codegen.resolve_symbol(key).chars() {
            match ch {
                '\\' => writer.write("\\\\"),
                '"' => writer.write("\\\""),
                '\n' => writer.write("\\n"),
                '\r' => writer.write("\\r"),
                '\t' => writer.write("\\t"),
                _ => writer.write(&ch.to_string()),
            }
        }
        writer.write("\".to_string()");
        return;
    }

    writer.write(codegen.resolve_symbol(key));
}
