//! Rust Expression Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Generates Rust expressions from HIR, handling error propagation (`?` operator),
//! async await (`.await`), reference creation (`&` and `&mut`), and control flow.
//!
//! COMPILER PHASE: Codegen (submodule)
//! INPUT: HirExpr nodes
//! OUTPUT: Rust expression source text
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Error propagation context: `?` operator inserted only in failable functions.
//!   WHY: Rust requires `?` to appear only in functions returning Result/Option.
//! - Suppress propagation in entry blocks: Entry code uses panic! for throws.
//!   WHY: Faber's `incipit` has no error return type; crashes are appropriate.
//! - Catch blocks suppress `?`: Errors are handled locally, not propagated.
//!   WHY: `tempta { iace "err" } cape { ... }` should not add `?` to the throw.

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::lexer::Symbol;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

mod block;
mod call;
mod collection;
mod control;
mod format;
mod literal;
mod ops;
mod option;
mod pattern;

use block::*;
use call::*;
use collection::*;
use control::*;
use format::*;
use literal::*;
use ops::*;
use option::*;
use pattern::*;

/// Generate a Rust expression.
///
/// TRANSFORMS:
///   salve(n)           -> salve(n) or salve(n)? (if failable)
///   iace "err"         -> return Err(String::from("err")) or panic!("err")
///   obj.method()       -> obj.method() or obj.method()? (if failable)
///   cede future_expr   -> future_expr.await
///   de expr            -> &expr
///   in expr            -> &mut expr
///
/// TARGET: Rust-specific `?` operator for error propagation.
/// EDGE: suppress_error_propagation=true in tempta catch blocks.
pub fn generate_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            w.write(codegen.resolve_def(*def_id));
        }
        HirExprKind::Literal(lit) => {
            generate_literal(codegen, lit, w);
            if matches!(lit, HirLiteral::String(_))
                && expr
                    .ty
                    .is_some_and(|ty| matches!(types.get(ty), Type::Primitive(Primitive::Textus)))
            {
                w.write(".to_string()");
            }
        }
        HirExprKind::Binary(op, lhs, rhs) => {
            generate_binary_expr(
                codegen,
                *op,
                lhs,
                rhs,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
                true,
            )?;
        }
        HirExprKind::Unary(op, operand) => {
            generate_unary_expr(
                codegen,
                *op,
                operand,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
                true,
            )?;
        }
        HirExprKind::Call(callee, args) => {
            generate_call_expr(
                codegen,
                callee,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_method_call_expr(
                codegen,
                receiver,
                *method,
                args,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
        }
        HirExprKind::Field(obj, field) => {
            generate_expr(codegen, obj, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*field));
        }
        HirExprKind::Index(obj, idx) => {
            generate_expr(codegen, obj, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("[");
            generate_expr(codegen, idx, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("]");
        }
        HirExprKind::OptionalChain(object, chain) => generate_optional_chain_expr(
            codegen,
            object,
            chain,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::NonNull(object, chain) => generate_non_null_expr(
            codegen,
            object,
            chain,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Ab { source, filter, transforms } => generate_ab_expr(
            codegen,
            expr.id,
            source,
            filter.as_ref(),
            transforms,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Block(block) => {
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Tempta { body, catch, finally } => generate_tempta_expr(
            codegen,
            body,
            catch.as_ref(),
            finally.as_ref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Si(cond, then, else_) => generate_if_expr(
            codegen,
            cond,
            then,
            else_.as_ref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Discerne(scrutinees, arms) => generate_match_expr(
            codegen,
            scrutinees,
            arms,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Loop(block) => {
            generate_loop_expr(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?
        }
        HirExprKind::Dum(cond, block) => generate_while_expr(
            codegen,
            cond,
            block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Itera(_, binding, _binding_name, iter, block) => generate_for_expr(
            codegen,
            *binding,
            iter,
            block,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Intervallum { start, end, step, .. } => generate_range_tuple_expr(
            codegen,
            start,
            end,
            step.as_deref(),
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Assign(target, value) => {
            generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" = ");
            generate_expr_unwrapped(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::AssignOp(op, target, value) => {
            generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write("= ");
            generate_expr_unwrapped(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Array(elements) => generate_array_expr(
            codegen,
            expr.id,
            elements,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Struct(def_id, fields) => generate_struct_expr(
            codegen,
            *def_id,
            fields,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Tuple(elements) => generate_tuple_expr(
            codegen,
            elements,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
        )?,
        HirExprKind::Scribe(args) => {
            if args.is_empty() {
                w.write("println!()");
            } else {
                let format = args
                    .iter()
                    .map(|arg| rust_scribe_format(arg, types))
                    .collect::<Vec<_>>()
                    .join(" ");
                w.write("println!(\"");
                w.write(&format);
                w.write("\"");
                for arg in args {
                    w.write(", ");
                    generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                }
                w.write(")");
            }
        }
        HirExprKind::Scriptum(template, args) => {
            w.write("format!(\"");
            w.write(&rust_format_template(codegen.resolve_symbol(*template)));
            w.write("\"");
            for arg in args {
                w.write(", ");
                generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
        }
        HirExprKind::Adfirma(cond, message) => {
            w.write("assert!(");
            generate_expr(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if let Some(message) = message {
                w.write(", \"{}\", ");
                generate_expr(codegen, message, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
        }
        HirExprKind::Panic(value) => {
            w.write("panic!(\"{}\", ");
            generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        }
        HirExprKind::Throw(value) => {
            if in_failable_fn && !in_entry && !suppress_error_propagation {
                w.write("return Err(");
                if matches!(value.kind, HirExprKind::Literal(HirLiteral::String(_))) {
                    w.write("String::from(");
                    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                } else {
                    w.write("format!(\"{}\", ");
                    generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                }
                w.write(")");
            } else {
                w.write("panic!(\"{}\", ");
                generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(")");
            }
        }
        HirExprKind::Clausura(params, _ret, body) => {
            w.write("|");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
            }
            w.write("| ");
            generate_expr(codegen, body, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Cede(expr) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(".await");
        }
        HirExprKind::Verte { source, target, entries } => match types.get(*target) {
            Type::Struct(def_id) => {
                if let Some(entries) = entries {
                    w.write(codegen.resolve_def(*def_id));
                    w.writeln(" {");
                    let mut struct_result = Ok(());
                    w.indented(|w| {
                        for field in entries {
                            let (name, value) = match (&field.key, &field.value) {
                                (HirObjectKey::Ident(name) | HirObjectKey::String(name), Some(value)) => (name, value),
                                _ => continue,
                            };
                            w.write(codegen.resolve_symbol(*name));
                            w.write(": ");
                            if struct_result.is_err() {
                                return;
                            }
                            struct_result = generate_expr(
                                codegen,
                                value,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            );
                            w.writeln(",");
                        }
                    });
                    struct_result?;
                    w.write("}");
                } else {
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                }
            }
            Type::Array(elem) => {
                if let HirExprKind::Array(elements) = &source.kind {
                    if elements.is_empty() {
                        w.write("Vec::<");
                        w.write(&type_to_rust(codegen, *elem, types));
                        w.write(">::new()");
                    } else if elements
                        .iter()
                        .any(|element| matches!(element, HirArrayElement::Spread(_)))
                    {
                        let temp = format!("__faber_verte_vec_{}", expr.id.0);
                        w.writeln("{");
                        let mut array_result = Ok(());
                        w.indented(|w| {
                            w.write("let mut ");
                            w.write(&temp);
                            w.writeln(" = Vec::new();");
                            for element in elements {
                                if array_result.is_err() {
                                    return;
                                }
                                match element {
                                    HirArrayElement::Expr(elem_expr) => {
                                        w.write(&temp);
                                        w.write(".push(");
                                        array_result = generate_expr(
                                            codegen,
                                            elem_expr,
                                            types,
                                            w,
                                            in_failable_fn,
                                            in_entry,
                                            suppress_error_propagation,
                                        );
                                        w.writeln(");");
                                    }
                                    HirArrayElement::Spread(elem_expr) => {
                                        w.write(&temp);
                                        w.write(".extend(");
                                        array_result = generate_expr(
                                            codegen,
                                            elem_expr,
                                            types,
                                            w,
                                            in_failable_fn,
                                            in_entry,
                                            suppress_error_propagation,
                                        );
                                        w.writeln(");");
                                    }
                                }
                            }
                            w.write(&temp);
                            w.newline();
                        });
                        array_result?;
                        w.write("}");
                    } else {
                        w.write("vec![");
                        for (i, elem_expr) in elements.iter().enumerate() {
                            if i > 0 {
                                w.write(", ");
                            }
                            let HirArrayElement::Expr(elem_expr) = elem_expr else {
                                continue;
                            };
                            generate_expr(
                                codegen,
                                elem_expr,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            )?;
                        }
                        w.write("]");
                    }
                } else {
                    w.write("Vec::<");
                    w.write(&type_to_rust(codegen, *elem, types));
                    w.write(">::new()");
                }
            }
            Type::Map(key_ty, value_ty) => {
                if let Some(entries) = entries {
                    let suffix = expr.id.0;
                    let map_name = format!("__faber_verte_map_{}", suffix);
                    w.writeln("{");
                    let mut map_result = Ok(());
                    w.indented(|w| {
                        w.write("let mut ");
                        w.write(&map_name);
                        w.write(" = std::collections::HashMap::<");
                        w.write(&type_to_rust(codegen, *key_ty, types));
                        w.write(", ");
                        w.write(&type_to_rust(codegen, *value_ty, types));
                        w.writeln(">::new();");
                        for field in entries {
                            match (&field.key, &field.value) {
                                (HirObjectKey::Spread(expr), _) => {
                                    w.write(&map_name);
                                    w.write(".extend(");
                                    if map_result.is_err() {
                                        return;
                                    }
                                    map_result = generate_expr(
                                        codegen,
                                        expr,
                                        types,
                                        w,
                                        in_failable_fn,
                                        in_entry,
                                        suppress_error_propagation,
                                    );
                                    w.writeln(");");
                                }
                                (_, Some(value)) => {
                                    w.write(&map_name);
                                    w.write(".insert(");
                                    if map_result.is_err() {
                                        return;
                                    }
                                    map_result = write_object_map_key(
                                        codegen,
                                        types,
                                        &field.key,
                                        *key_ty,
                                        w,
                                        in_failable_fn,
                                        in_entry,
                                        suppress_error_propagation,
                                    );
                                    if map_result.is_err() {
                                        return;
                                    }
                                    w.write(", ");
                                    map_result = generate_expr(
                                        codegen,
                                        value,
                                        types,
                                        w,
                                        in_failable_fn,
                                        in_entry,
                                        suppress_error_propagation,
                                    );
                                    w.writeln(");");
                                }
                                (_, None) => {}
                            }
                        }
                        w.write(&map_name);
                        w.newline();
                    });
                    map_result?;
                    w.write("}");
                } else {
                    w.write("std::collections::HashMap::<");
                    w.write(&type_to_rust(codegen, *key_ty, types));
                    w.write(", ");
                    w.write(&type_to_rust(codegen, *value_ty, types));
                    w.write(">::new()");
                }
            }
            // Primitive cast — `as` is valid for numeric/bool conversions in Rust
            Type::Primitive(Primitive::Numerus)
            | Type::Primitive(Primitive::Fractus)
            | Type::Primitive(Primitive::Bivalens) => {
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" as ");
                w.write(&type_to_rust(codegen, *target, types));
            }
            // Textus cast — use .to_string() instead of `as`
            Type::Primitive(Primitive::Textus) => {
                w.write("format!(\"{}\", ");
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(")");
            }
            // Enum/Interface — passthrough (trust the type checker)
            Type::Enum(_) | Type::Interface(_) => {
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            // All other types — passthrough with the source expression
            _ => {
                generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
        },
        HirExprKind::Conversio { source, target, params: _, fallback } => {
            let target_resolved = types.get(*target);
            let source_ty = source.ty.map(|t| types.get(t));
            match (source_ty, target_resolved) {
                // textus → numerus: .parse::<i64>()
                (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Numerus)) => {
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    if let Some(fallback) = fallback {
                        w.write(".parse::<i64>().unwrap_or(");
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
                        w.write(".parse::<i64>().unwrap()");
                    }
                }
                // textus → fractus: .parse::<f64>()
                (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Fractus)) => {
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    if let Some(fallback) = fallback {
                        w.write(".parse::<f64>().unwrap_or(");
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
                        w.write(".parse::<f64>().unwrap()");
                    }
                }
                // textus → bivalens: !source.is_empty()
                (Some(Type::Primitive(Primitive::Textus)), Type::Primitive(Primitive::Bivalens)) => {
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
                }
                // numerus → textus: source.to_string()
                (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Textus))
                | (Some(Type::Primitive(Primitive::Fractus)), Type::Primitive(Primitive::Textus)) => {
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(".to_string()");
                }
                // numerus → fractus: source as f64
                (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Fractus)) => {
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(" as f64");
                }
                // numerus → bivalens: source != 0
                (Some(Type::Primitive(Primitive::Numerus)), Type::Primitive(Primitive::Bivalens)) => {
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
                }
                // any → textus: format!("{}", source)
                (_, Type::Primitive(Primitive::Textus)) => {
                    w.write("format!(\"{}\", ");
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                }
                // Fallback: .parse() for string-to-target, `as` for numeric
                _ => match target_resolved {
                    Type::Primitive(Primitive::Numerus) | Type::Primitive(Primitive::Fractus) => {
                        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                        if let Some(fallback) = fallback {
                            w.write(".parse::<");
                            w.write(&type_to_rust(codegen, *target, types));
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
                            w.write(".parse::<");
                            w.write(&type_to_rust(codegen, *target, types));
                            w.write(">().unwrap()");
                        }
                    }
                    _ => {
                        generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                        w.write(" as ");
                        w.write(&type_to_rust(codegen, *target, types));
                    }
                },
            }
        }
        HirExprKind::Ref(kind, expr) => {
            match kind {
                HirRefKind::Shared => w.write("&"),
                HirRefKind::Mutable => w.write("&mut "),
            }
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Deref(expr) => {
            w.write("*");
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Error => {
            return Err(CodegenError {
                message: format!(
                    "cannot generate Rust for HIR error expression at span {}..{}",
                    expr.span.start, expr.span.end
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn generate_expr_unwrapped(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Binary(op, lhs, rhs) => generate_binary_expr(
            codegen,
            *op,
            lhs,
            rhs,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
            false,
        ),
        HirExprKind::Unary(op, operand) => generate_unary_expr(
            codegen,
            *op,
            operand,
            types,
            w,
            in_failable_fn,
            in_entry,
            suppress_error_propagation,
            false,
        ),
        _ => generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation),
    }
}

fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        ty => ty.clone(),
    }
}
