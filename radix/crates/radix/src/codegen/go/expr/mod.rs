use super::stmt;
use super::types;
use super::{CodeWriter, CodegenError, GoCodegen};
use crate::hir::{HirArrayElement, HirBinOp, HirExpr, HirExprKind, HirIteraMode, HirLiteral, HirObjectKey, HirUnOp};
use crate::semantic::{Primitive, Type, TypeTable};

mod access;
mod call;
mod collection;
mod convert;
mod literal;
mod ops;
mod option;
mod variants;

use access::*;
use call::*;
use collection::*;
use convert::*;
use literal::*;
use ops::*;
use option::*;
use variants::*;

pub fn generate_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            if codegen.is_variant_def(*def_id) {
                w.write(codegen.resolve_def(*def_id));
                w.write("{}");
            } else if codegen.is_struct_def(*def_id) {
                w.write("*self");
            } else {
                w.write(codegen.resolve_def(*def_id));
            }
        }
        HirExprKind::Literal(lit) => generate_literal(codegen, lit, w),
        HirExprKind::Binary(op, lhs, rhs) => {
            if matches!(op, HirBinOp::Coalesce) {
                generate_coalesce_expr(codegen, expr, lhs, rhs, types, w)?;
                return Ok(());
            }
            if matches!(op, HirBinOp::Div)
                && matches!(
                    expr.ty
                        .map(|ty| normalize_receiver_type(types.get(ty), types)),
                    Some(Type::Primitive(Primitive::Fractus))
                )
                && matches!(
                    lhs.ty
                        .map(|ty| normalize_receiver_type(types.get(ty), types)),
                    Some(Type::Primitive(Primitive::Numerus))
                )
                && matches!(
                    rhs.ty
                        .map(|ty| normalize_receiver_type(types.get(ty), types)),
                    Some(Type::Primitive(Primitive::Numerus))
                )
            {
                w.write("(float64(");
                generate_expr(codegen, lhs, types, w)?;
                w.write(") / float64(");
                generate_expr(codegen, rhs, types, w)?;
                w.write("))");
                return Ok(());
            }
            w.write("(");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(binary_op_to_go(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(")");
        }
        HirExprKind::Unary(op, operand) => generate_unary_expr(codegen, *op, operand, types, w)?,
        HirExprKind::Call(callee, args) => {
            if let HirExprKind::Path(def_id) = callee.kind {
                if codegen.is_variant_def(def_id) {
                    generate_variant_constructor(codegen, def_id, args, types, w)?;
                    return Ok(());
                }
            }
            if try_generate_spread_call_recovery(codegen, callee, args, types, w)? {
                return Ok(());
            }
            if try_generate_intrinsic_call(codegen, callee, args, types, w)? {
                return Ok(());
            }
            generate_expr(codegen, callee, types, w)?;
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            if try_generate_translated_method_call(codegen, receiver, *method, args, types, w)? {
                return Ok(());
            }
            write_method_receiver(codegen, receiver, types, w)?;
            w.write(".");
            w.write(&capitalize(codegen.resolve_symbol(*method)));
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Field(object, field) => {
            let field_name = codegen.resolve_symbol(*field);
            if let HirExprKind::Path(def_id) = object.kind {
                if codegen.is_struct_def(def_id) {
                    w.write("self.");
                    w.write(&capitalize(field_name));
                    return Ok(());
                }
            }
            let object_ty = object
                .ty
                .map(|ty| normalize_receiver_type(types.get(ty), types));
            if matches!(field_name, "length" | "longitudo")
                && matches!(object_ty, Some(Type::Array(_)) | Some(Type::Primitive(Primitive::Textus)))
            {
                w.write("len(");
                generate_expr(codegen, object, types, w)?;
                w.write(")");
                return Ok(());
            }
            if let Some(Type::Map(_, value_ty)) = object_ty {
                write_map_member_expr(codegen, object, field_name, *value_ty, expr.ty, types, w)?;
                return Ok(());
            }
            if matches!(object_ty, Some(Type::Primitive(Primitive::Ignotum))) {
                w.write("func() any { if m, ok := ");
                generate_expr(codegen, object, types, w)?;
                w.write(".(map[string]any); ok { return m[");
                w.write(&format!("{:?}", field_name));
                w.write("] }; return nil }()");
                return Ok(());
            }
            generate_expr(codegen, object, types, w)?;
            w.write(".");
            w.write(&capitalize(field_name));
        }
        HirExprKind::Index(object, index) => {
            generate_expr(codegen, object, types, w)?;
            w.write("[");
            generate_expr(codegen, index, types, w)?;
            w.write("]");
        }
        HirExprKind::OptionalChain(object, chain) => {
            generate_optional_chain_expr(codegen, object, chain, expr, types, w)?;
        }
        HirExprKind::NonNull(object, chain) => {
            // WHY: Go has no non-null assertion — emit plain access.
            generate_expr(codegen, object, types, w)?;
            match chain {
                crate::hir::HirNonNullKind::Member(field) => {
                    if matches!(
                        object
                            .ty
                            .map(|ty| normalize_receiver_type(types.get(ty), types)),
                        Some(Type::Map(_, _))
                    ) {
                        w.write("[");
                        w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                        w.write("]");
                    } else {
                        w.write(".");
                        w.write(&capitalize(codegen.resolve_symbol(*field)));
                    }
                }
                crate::hir::HirNonNullKind::Index(index) => {
                    w.write("[");
                    generate_expr(codegen, index, types, w)?;
                    w.write("]");
                }
                crate::hir::HirNonNullKind::Call(args) => {
                    w.write("(");
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        generate_expr(codegen, arg, types, w)?;
                    }
                    w.write(")");
                }
            }
        }
        HirExprKind::Assign(lhs, rhs) => {
            generate_expr(codegen, lhs, types, w)?;
            w.write(" = ");
            generate_expr(codegen, rhs, types, w)?;
        }
        HirExprKind::AssignOp(op, lhs, rhs) => {
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(assign_op_to_go(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
        }
        HirExprKind::Array(elements) => {
            generate_array_expr(codegen, expr, elements, types, w)?;
        }
        HirExprKind::Struct(def_id, fields) => {
            w.write(codegen.resolve_def(*def_id));
            w.write("{");
            for (idx, (name, value)) in fields.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(&capitalize(codegen.resolve_symbol(*name)));
                w.write(": ");
                if let Some(field_ty) = codegen.struct_field_type(*def_id, *name) {
                    generate_expr_for_go_type(codegen, value, field_ty, types, w)?;
                } else {
                    generate_expr(codegen, value, types, w)?;
                }
            }
            w.write("}");
        }
        HirExprKind::Tuple(elements) => {
            // WHY: Go has no tuples — emit as a slice literal.
            w.write("[]any{");
            for (idx, element) in elements.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, element, types, w)?;
            }
            w.write("}");
        }
        HirExprKind::Scribe(args) => {
            w.write("fmt.Println(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Scriptum(template, args) => {
            w.write("fmt.Sprintf(");
            w.write(&format!(
                "{:?}",
                render_scriptum_template(codegen.resolve_symbol(*template), args.len())
            ));
            for arg in args {
                w.write(", ");
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Panic(value) => {
            w.write("panic(fmt.Sprint(");
            generate_expr(codegen, value, types, w)?;
            w.write("))");
        }
        HirExprKind::Throw(value) => {
            // WHY: Go has no throw — panic is the closest equivalent.
            w.write("panic(fmt.Sprint(");
            generate_expr(codegen, value, types, w)?;
            w.write("))");
        }
        HirExprKind::Tempta { body, catch, .. } => {
            // WHY: Go has no try/catch. Emit a defer/recover pattern.
            w.write("func() { defer func() { if r := recover(); r != nil { _ = r ");
            if let Some(catch) = catch {
                w.write("; ");
                stmt::generate_error_binding_block(codegen, catch, "r", types, w)?;
            }
            w.write(" } }(); ");
            stmt::generate_block(codegen, body, types, w, |_| {})?;
            w.write(" }()");
        }
        HirExprKind::Clausura(params, ret_ty, body) => {
            w.write("func(");
            for (idx, param) in params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                w.write(" ");
                w.write(&types::type_to_go(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = ret_ty.or(body.ty) {
                let ret = types::type_to_go(codegen, ret_ty, types);
                if !ret.is_empty() {
                    w.write(" ");
                    w.write(&ret);
                }
            }
            w.write(" { return ");
            generate_expr(codegen, body, types, w)?;
            w.write(" }");
        }
        HirExprKind::Cede(inner) => {
            // WHY: Go has no await — goroutines handle concurrency differently.
            // Emit the inner expression as-is; async support is a future concern.
            generate_expr(codegen, inner, types, w)?;
        }
        HirExprKind::Verte { source, target, entries } => {
            match types.get(*target) {
                Type::Struct(_) => {
                    if let Some(entries) = entries {
                        w.write(codegen.resolve_def(match types.get(*target) {
                            Type::Struct(def_id) => *def_id,
                            _ => unreachable!(),
                        }));
                        w.write("{");
                        let mut wrote_any = false;
                        for field in entries {
                            let Some(value) = &field.value else { continue };
                            if wrote_any {
                                w.write(", ");
                            }
                            match &field.key {
                                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                                    w.write(&capitalize(codegen.resolve_symbol(*name)));
                                }
                                HirObjectKey::Computed(_) | HirObjectKey::Spread(_) => {
                                    // Go struct literals don't support computed keys
                                    continue;
                                }
                            }
                            w.write(": ");
                            if let Some(struct_ty) = match types.get(*target) {
                                Type::Struct(def_id) => Some(*def_id),
                                _ => None,
                            } {
                                if let Some(field_ty) = match &field.key {
                                    HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                                        codegen.struct_field_type(struct_ty, *name)
                                    }
                                    HirObjectKey::Computed(_) | HirObjectKey::Spread(_) => None,
                                } {
                                    generate_expr_for_go_type(codegen, value, field_ty, types, w)?;
                                } else {
                                    generate_expr(codegen, value, types, w)?;
                                }
                            } else {
                                generate_expr(codegen, value, types, w)?;
                            }
                            wrote_any = true;
                        }
                        w.write("}");
                    } else {
                        generate_expr(codegen, source, types, w)?;
                    }
                }
                Type::Map(key_ty, value_ty) => {
                    generate_map_literal(codegen, *key_ty, *value_ty, entries.as_deref(), types, w)?;
                }
                Type::Array(elem_ty) => {
                    generate_verte_array_expr(codegen, source, *elem_ty, types, w)?;
                }
                Type::Option(inner_ty) => {
                    generate_option_wrapped_expr(codegen, source, *inner_ty, types, w)?;
                }
                Type::Primitive(Primitive::Textus) => {
                    w.write("fmt.Sprint(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")");
                }
                Type::Primitive(Primitive::Numerus) => {
                    w.write("func() int { v, _ := strconv.Atoi(fmt.Sprint(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")); return v }()");
                }
                Type::Primitive(Primitive::Fractus) => {
                    w.write("func() float64 { v, _ := strconv.ParseFloat(fmt.Sprint(");
                    generate_expr(codegen, source, types, w)?;
                    w.write("), 64); return v }()");
                }
                Type::Primitive(Primitive::Bivalens) => {
                    generate_bool_conversion_expr(codegen, source, types, w)?;
                }
                Type::Enum(_) | Type::Interface(_) if variant_value_expr(source, codegen) => {
                    generate_expr(codegen, source, types, w)?;
                }
                Type::Enum(_) | Type::Interface(_) => {
                    generate_expr(codegen, source, types, w)?;
                    w.write(".(");
                    w.write(&types::type_to_go(codegen, *target, types));
                    w.write(")");
                }
                _ => {
                    generate_expr(codegen, source, types, w)?;
                    w.write(".(");
                    w.write(&types::type_to_go(codegen, *target, types));
                    w.write(")");
                }
            }
        }
        HirExprKind::Conversio { source, target, fallback, .. } => {
            let target_resolved = types.get(*target);
            match target_resolved {
                Type::Primitive(Primitive::Numerus) => {
                    // WHY: strconv.Atoi is the idiomatic Go int parse.
                    if let Some(fb) = fallback {
                        w.write("func() int { v, err := strconv.Atoi(fmt.Sprint(");
                        generate_expr(codegen, source, types, w)?;
                        w.write(")); if err != nil { return ");
                        generate_expr(codegen, fb, types, w)?;
                        w.write(" }; return v }()");
                    } else {
                        w.write("func() int { v, _ := strconv.Atoi(fmt.Sprint(");
                        generate_expr(codegen, source, types, w)?;
                        w.write(")); return v }()");
                    }
                }
                Type::Primitive(Primitive::Fractus) => {
                    if let Some(fb) = fallback {
                        w.write("func() float64 { v, err := strconv.ParseFloat(fmt.Sprint(");
                        generate_expr(codegen, source, types, w)?;
                        w.write("), 64); if err != nil { return ");
                        generate_expr(codegen, fb, types, w)?;
                        w.write(" }; return v }()");
                    } else {
                        w.write("func() float64 { v, _ := strconv.ParseFloat(fmt.Sprint(");
                        generate_expr(codegen, source, types, w)?;
                        w.write("), 64); return v }()");
                    }
                }
                Type::Primitive(Primitive::Textus) => {
                    w.write("fmt.Sprint(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")");
                }
                Type::Primitive(Primitive::Bivalens) => {
                    w.write("func() bool { v, _ := strconv.ParseBool(fmt.Sprint(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")); return v }()");
                }
                _ => {
                    generate_expr(codegen, source, types, w)?;
                    w.write(".(");
                    w.write(&types::type_to_go(codegen, *target, types));
                    w.write(")");
                }
            }
        }
        // WHY: Go is GC'd — ref/deref are no-ops, just emit the inner expression.
        HirExprKind::Ref(_, inner) | HirExprKind::Deref(inner) => generate_expr(codegen, inner, types, w)?,
        HirExprKind::Block(block) => {
            w.write("func() ");
            w.write(&expr_return_type(expr, types, codegen));
            w.write(" ");
            if let Some(result_ty) = expr.ty {
                stmt::generate_value_block(codegen, block, result_ty, types, w)?;
            } else {
                stmt::generate_block(codegen, block, types, w, |_| {})?;
            }
            w.write("()");
        }
        HirExprKind::Si(cond, then_block, else_block) => {
            // WHY: Go's if is a statement, not an expression. Wrap in IIFE.
            w.write("func() ");
            w.write(&expr_return_type(expr, types, codegen));
            w.write(" { if ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            if let Some(result_ty) = expr.ty {
                stmt::generate_value_block(codegen, then_block, result_ty, types, w)?;
            } else {
                stmt::generate_block(codegen, then_block, types, w, |_| {})?;
            }
            if let Some(else_block) = else_block {
                w.write(" else ");
                if let Some(result_ty) = expr.ty {
                    stmt::generate_value_block(codegen, else_block, result_ty, types, w)?;
                } else {
                    stmt::generate_block(codegen, else_block, types, w, |_| {})?;
                }
            } else {
                w.write(" else { return nil }");
            }
            w.write(" }()");
        }
        HirExprKind::Discerne(_, _) => {
            // TODO: switch statement codegen
            w.write("nil");
        }
        HirExprKind::Loop(block) => {
            w.write("func() { for ");
            stmt::generate_block(codegen, block, types, w, |_| {})?;
            w.write(" }()");
        }
        HirExprKind::Dum(cond, block) => {
            w.write("func() { for ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            stmt::generate_block(codegen, block, types, w, |_| {})?;
            w.write(" }()");
        }
        HirExprKind::Itera(mode, def_id, _binding_name, iter, block) => {
            w.write("func() { for ");
            match mode {
                HirIteraMode::De => {
                    w.write(codegen.resolve_def(*def_id));
                    w.write(", _ := range ");
                }
                HirIteraMode::Ex | HirIteraMode::Pro => {
                    w.write("_, ");
                    w.write(codegen.resolve_def(*def_id));
                    w.write(" := range ");
                }
            }
            generate_expr(codegen, iter, types, w)?;
            w.write(" ");
            stmt::generate_block(codegen, block, types, w, |_| {})?;
            w.write(" }()");
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            // WHY: Go has no range literals — emit as a slice for now.
            w.write("[]any{");
            generate_expr(codegen, start, types, w)?;
            w.write(", ");
            generate_expr(codegen, end, types, w)?;
            if let Some(step) = step {
                w.write(", ");
                generate_expr(codegen, step, types, w)?;
            }
            w.write("}");
        }
        HirExprKind::Ab { source, filter, transforms } => {
            generate_ab_expr(codegen, expr, source, filter.as_ref(), transforms, types, w)?;
        }
        HirExprKind::Adfirma(cond, message) => {
            w.write("func() { if !(");
            generate_expr(codegen, cond, types, w)?;
            w.write(") { panic(");
            if let Some(message) = message {
                w.write("fmt.Sprint(");
                generate_expr(codegen, message, types, w)?;
                w.write(")");
            } else {
                w.write("\"assertion failed\"");
            }
            w.write(") } }()");
        }
        HirExprKind::Error => {
            return Err(CodegenError { message: "cannot emit Go for error expression".to_owned() });
        }
    }
    Ok(())
}

pub(super) fn generate_expr_for_go_type(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    expected_ty: crate::semantic::TypeId,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match (&expr.kind, types.get(expected_ty)) {
        (_, Type::Option(inner)) => generate_option_wrapped_expr(codegen, expr, *inner, types, w),
        (HirExprKind::Array(elements), Type::Array(elem_ty)) => {
            generate_typed_array_expr(codegen, *elem_ty, elements, types, w)
        }
        (HirExprKind::Verte { entries: Some(entries), .. }, Type::Map(key_ty, value_ty)) => {
            generate_map_literal(codegen, *key_ty, *value_ty, Some(entries), types, w)
        }
        _ => generate_expr(codegen, expr, types, w),
    }
}

pub(super) fn normalize_receiver_type<'a>(mut ty: &'a Type, types: &'a TypeTable) -> &'a Type {
    loop {
        match ty {
            Type::Ref(_, inner) | Type::Alias(_, inner) => ty = types.get(*inner),
            _ => return ty,
        }
    }
}

pub(super) fn expr_return_type(expr: &HirExpr, types: &TypeTable, codegen: &GoCodegen<'_>) -> String {
    expr.ty
        .map(|ty| types::type_to_go(codegen, ty, types))
        .filter(|ty| !ty.is_empty())
        .unwrap_or_else(|| "any".to_owned())
}
pub(super) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}
