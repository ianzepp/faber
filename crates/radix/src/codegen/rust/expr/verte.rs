//! `verte` target-conversion emission for the Rust backend.
//!
//! `verte` is the explicit target-shaped construction path: it can turn object
//! entries into structs or maps, array literals into typed vectors, and selected
//! primitives into Rust casts or formatted text. It is not a general inference
//! engine. Typechecking and lowering decide the target type; this module emits
//! the Rust construction policy for that already-known target.
//!
//! TRADE-OFFS
//! ==========
//! - Struct targets reuse normal struct field emission so optional wrapping and
//!   omitted-field initializers stay identical between literals and `verte`.
//! - Array and map targets use block-local temporaries when spreads require
//!   mutation; simple arrays stay compact `vec![...]` expressions.
//! - Primitive numeric and boolean targets use Rust `as` casts as the backend
//!   escape hatch. Text targets use formatting because `as String` is not a
//!   Rust conversion.

use super::super::type_shape::{resolve_type, type_id_is_faber_value};
use super::super::types::type_to_rust;
use super::*;

pub(super) fn emit_verte_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    source: &HirExpr,
    target: TypeId,
    entries: Option<&[HirObjectField]>,
) -> Result<(), CodegenError> {
    // Dispatch is target-first. Source shape only matters inside target-specific
    // constructors, which keeps fallback behavior explicit when the source does
    // not match a literal form this backend can construct.
    match emitter.types.get(target) {
        Type::Struct(def_id) => emit_verte_struct_expr(emitter, expr_id, source, *def_id, entries),
        Type::Array(elem) => emit_verte_array_expr(emitter, expr_id, source, *elem),
        Type::Map(key_ty, value_ty) => emit_verte_map_expr(emitter, expr_id, *key_ty, *value_ty, entries),
        Type::Primitive(Primitive::Numerus)
        | Type::Primitive(Primitive::Fractus)
        | Type::Primitive(Primitive::Bivalens) => {
            emitter.expr(source)?;
            emitter.writer.write(" as ");
            emitter
                .writer
                .write(&type_to_rust(emitter.codegen, target, emitter.types));
            Ok(())
        }
        Type::Primitive(Primitive::Textus) => {
            emitter.writer.write("format!(\"{}\", ");
            emitter.expr(source)?;
            emitter.writer.write(")");
            Ok(())
        }
        Type::Enum(_) | Type::Interface(_) => emitter.expr(source),
        _ => emitter.expr(source),
    }
}

fn emit_verte_struct_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    source: &HirExpr,
    def_id: DefId,
    entries: Option<&[HirObjectField]>,
) -> Result<(), CodegenError> {
    let Some(entries) = entries else {
        return emitter.expr(source);
    };

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
            result = emit_verte_struct_literal_expr(&mut nested, def_id, entries);
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

    emit_verte_struct_literal_expr(emitter, def_id, entries)
}

fn emit_verte_struct_literal_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    def_id: DefId,
    entries: &[HirObjectField],
) -> Result<(), CodegenError> {
    // Entry-backed struct construction mirrors `generate_struct_expr` so
    // defaulted and optional fields do not diverge under `verte`.
    emitter.writer.write(emitter.codegen.resolve_def(def_id));
    emitter.writer.writeln(" {");
    let mut struct_result = Ok(());
    let provided = entries
        .iter()
        .filter_map(|field| match (&field.key, &field.value) {
            (HirObjectKey::Ident(name) | HirObjectKey::String(name), Some(_)) => Some(*name),
            _ => None,
        })
        .collect::<rustc_hash::FxHashSet<_>>();
    let codegen = emitter.codegen;
    let types = emitter.types;
    let policy = emitter.policy;
    emitter.writer.indented(|writer| {
        let mut nested = ExprEmitter::new(codegen, types, writer, policy);
        for field in entries {
            let (name, value) = match (&field.key, &field.value) {
                (HirObjectKey::Ident(name) | HirObjectKey::String(name), Some(value)) => (name, value),
                _ => continue,
            };
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

fn emit_verte_array_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    source: &HirExpr,
    elem: TypeId,
) -> Result<(), CodegenError> {
    if let HirExprKind::Array(elements) = &source.kind {
        if elements.is_empty() {
            // Empty vectors need the target element type because Rust cannot
            // infer it from `vec![]` in all generated contexts.
            emitter.writer.write("Vec::<");
            emitter
                .writer
                .write(&type_to_rust(emitter.codegen, elem, emitter.types));
            emitter.writer.write(">::new()");
            return Ok(());
        }
        if elements
            .iter()
            .any(|element| matches!(element, HirArrayElement::Spread(_)))
        {
            // Spread arrays need mutation; the HIR id makes the generated
            // temporary stable and unique within the emitted expression tree.
            let temp = format!("__faber_verte_vec_{}", expr_id.0);
            emitter.writer.writeln("{");
            let mut array_result = Ok(());
            let codegen = emitter.codegen;
            let types = emitter.types;
            let policy = emitter.policy;
            emitter.writer.indented(|writer| {
                let mut nested = ExprEmitter::new(codegen, types, writer, policy);
                nested.writer.write("let mut ");
                nested.writer.write(&temp);
                nested.writer.writeln(" = Vec::new();");
                for element in elements {
                    if array_result.is_err() {
                        return;
                    }
                    match element {
                        HirArrayElement::Expr(elem_expr) => {
                            nested.writer.write(&temp);
                            nested.writer.write(".push(");
                            array_result = emit_verte_value_expr(&mut nested, elem_expr, elem);
                            nested.writer.writeln(");");
                        }
                        HirArrayElement::Spread(elem_expr) => {
                            nested.writer.write(&temp);
                            nested.writer.write(".extend((");
                            array_result = nested.expr(elem_expr);
                            nested.writer.writeln(").iter().cloned());");
                        }
                    }
                }
                nested.writer.write(&temp);
                nested.writer.newline();
            });
            array_result?;
            emitter.writer.write("}");
            return Ok(());
        }

        emitter.writer.write("vec![");
        for (i, elem_expr) in elements.iter().enumerate() {
            if i > 0 {
                emitter.writer.write(", ");
            }
            let HirArrayElement::Expr(elem_expr) = elem_expr else {
                continue;
            };
            emit_verte_value_expr(emitter, elem_expr, elem)?;
        }
        emitter.writer.write("]");
        return Ok(());
    }

    emitter.writer.write("Vec::<");
    emitter
        .writer
        .write(&type_to_rust(emitter.codegen, elem, emitter.types));
    emitter.writer.write(">::new()");
    Ok(())
}

fn emit_verte_map_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    expr_id: HirId,
    key_ty: TypeId,
    value_ty: TypeId,
    entries: Option<&[HirObjectField]>,
) -> Result<(), CodegenError> {
    if let Some(entries) = entries {
        // Map construction always states key/value types up front. That avoids
        // depending on insert order or spread contents for Rust type inference.
        let map_name = format!("__faber_verte_map_{}", expr_id.0);
        emitter.writer.writeln("{");
        let mut map_result = Ok(());
        let codegen = emitter.codegen;
        let types = emitter.types;
        let policy = emitter.policy;
        emitter.writer.indented(|writer| {
            let mut nested = ExprEmitter::new(codegen, types, writer, policy);
            nested.writer.write("let mut ");
            nested.writer.write(&map_name);
            nested.writer.write(" = std::collections::HashMap::<");
            nested.writer.write(&type_to_rust(codegen, key_ty, types));
            nested.writer.write(", ");
            nested.writer.write(&type_to_rust(codegen, value_ty, types));
            nested.writer.writeln(">::new();");
            for field in entries {
                match (&field.key, &field.value) {
                    (HirObjectKey::Spread(expr), _) => {
                        nested.writer.write(&map_name);
                        nested.writer.write(".extend(");
                        if map_result.is_err() {
                            return;
                        }
                        map_result = nested.expr(expr);
                        nested.writer.writeln(");");
                    }
                    (_, Some(value)) => {
                        nested.writer.write(&map_name);
                        nested.writer.write(".insert(");
                        if map_result.is_err() {
                            return;
                        }
                        map_result = emit_object_map_key(&mut nested, &field.key, key_ty);
                        if map_result.is_err() {
                            return;
                        }
                        nested.writer.write(", ");
                        map_result = emit_verte_value_expr(&mut nested, value, value_ty);
                        nested.writer.writeln(");");
                    }
                    (_, None) => {}
                }
            }
            nested.writer.write(&map_name);
            nested.writer.newline();
        });
        map_result?;
        emitter.writer.write("}");
        return Ok(());
    }

    emitter.writer.write("std::collections::HashMap::<");
    emitter
        .writer
        .write(&type_to_rust(emitter.codegen, key_ty, emitter.types));
    emitter.writer.write(", ");
    emitter
        .writer
        .write(&type_to_rust(emitter.codegen, value_ty, emitter.types));
    emitter.writer.write(">::new()");
    Ok(())
}

fn emit_verte_value_expr(
    emitter: &mut ExprEmitter<'_, '_>,
    value: &HirExpr,
    target_ty: TypeId,
) -> Result<(), CodegenError> {
    if let Type::Array(elem_ty) = resolve_type(target_ty, emitter.types) {
        if type_id_is_faber_value(elem_ty, emitter.types) {
            return emit_verte_array_expr(emitter, value.id, value, elem_ty);
        }
    }

    if type_id_is_faber_value(target_ty, emitter.types) {
        return emitter.expr_as_type(value, target_ty);
    }

    emitter.expr(value)
}
