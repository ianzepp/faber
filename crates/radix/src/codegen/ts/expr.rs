//! Expression emission for the TypeScript backend.
//!
//! This file is the expression half of the HIR-to-TypeScript boundary. It
//! assumes semantic analysis has already resolved names, assigned HIR types,
//! and rejected source programs that cannot be represented safely. The emitter's
//! job is therefore not to re-typecheck Faber, but to use those types when
//! choosing TypeScript spellings for expressions and standard-library calls.
//!
//! LOWERING POLICY
//! ===============
//! Expressions are emitted directly when TypeScript has a matching expression
//! form. Faber constructs that carry statement-like control flow, such as block
//! expressions, `si` branches, loops, assertions, and `tempta`, are wrapped in
//! IIFEs so they can still appear where HIR expects an expression value. The
//! statement emitter owns block layout; this module only decides when an
//! expression needs that block machinery.
//!
//! TARGET TRADE-OFFS
//! =================
//! - `nihil` becomes `null`, while optional access uses TypeScript optional
//!   chaining and null checks use loose `== null`/`!= null` intentionally so
//!   both JavaScript nullish sentinels behave as Faber optional values.
//! - `lista`, `textus`, and map-like stdlib calls are translated only when the
//!   receiver type proves the target surface. Unknown methods fall back to
//!   direct property calls instead of guessing a collection protocol.
//! - Unsupported HIR nodes return `CodegenError`; generated TypeScript should
//!   not silently stand in for compiler states that earlier phases failed to
//!   normalize.

use super::stmt;
use super::types;
use super::{CodeWriter, CodegenError, TsCodegen};
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{
    HirArrayElement, HirBinOp, HirBlock, HirCallArg, HirExpr, HirExprKind, HirIteraMode, HirLiteral, HirObjectKey,
    HirOptionalChainKind, HirRangeKind, HirUnOp,
};
use crate::semantic::{Primitive, Type, TypeTable};

/// Emits one typed HIR expression as TypeScript expression text.
///
/// The caller supplies the shared [`TypeTable`] so expression emission can
/// choose target-specific spellings for collections, option-like accesses,
/// casts, and empty literals without duplicating type reconstruction. Errors
/// are reserved for HIR surfaces this backend does not yet support; ordinary
/// source diagnostics should have been produced before codegen.
pub fn generate_expr(
    codegen: &TsCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => w.write(codegen.resolve_expr_def(*def_id)),
        HirExprKind::Literal(lit) => generate_literal(codegen, lit, w),
        HirExprKind::Binary(op, lhs, rhs) => {
            w.write("(");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(binary_op_to_ts(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(")");
        }
        HirExprKind::Unary(op, operand) => generate_unary_expr(codegen, *op, operand, types, w)?,
        HirExprKind::Call(callee, args) => {
            if try_generate_intrinsic_call(codegen, callee, args, types, w)? {
                return Ok(());
            }
            generate_expr(codegen, callee, types, w)?;
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, &arg.expr, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            if try_generate_translated_method_call(codegen, receiver, *method, args, types, w)? {
                return Ok(());
            }
            generate_expr(codegen, receiver, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*method));
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, &arg.expr, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Field(object, field) => {
            generate_expr(codegen, object, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*field));
        }
        HirExprKind::Index(object, index) => {
            if matches!(object.ty.map(|ty| types.get(ty)), Some(Type::Primitive(Primitive::Textus))) {
                generate_textus_index_expr(codegen, object, index, types, w)?;
                return Ok(());
            }
            generate_expr(codegen, object, types, w)?;
            w.write("[");
            generate_expr(codegen, index, types, w)?;
            w.write("]");
        }
        HirExprKind::OptionalChain(object, chain) => {
            generate_expr(codegen, object, types, w)?;
            match chain {
                HirOptionalChainKind::Member(field) => {
                    w.write("?.");
                    w.write(codegen.resolve_symbol(*field));
                }
                HirOptionalChainKind::Index(index) => {
                    w.write("?.[");
                    generate_expr(codegen, index, types, w)?;
                    w.write("]");
                }
                HirOptionalChainKind::Call(args) => {
                    w.write("?.(");
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        generate_expr(codegen, &arg.expr, types, w)?;
                    }
                    w.write(")");
                }
            }
        }
        HirExprKind::NonNull(object, chain) => {
            generate_expr(codegen, object, types, w)?;
            match chain {
                crate::hir::HirNonNullKind::Member(field) => {
                    w.write("!.");
                    w.write(codegen.resolve_symbol(*field));
                }
                crate::hir::HirNonNullKind::Index(index) => {
                    w.write("![");
                    generate_expr(codegen, index, types, w)?;
                    w.write("]");
                }
                crate::hir::HirNonNullKind::Call(args) => {
                    w.write("!(");
                    for (idx, arg) in args.iter().enumerate() {
                        if idx > 0 {
                            w.write(", ");
                        }
                        generate_expr(codegen, &arg.expr, types, w)?;
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
            w.write(assign_op_to_ts(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
        }
        HirExprKind::Array(elements) => {
            w.write("[");
            for (idx, element) in elements.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                match element {
                    HirArrayElement::Expr(expr) => generate_expr(codegen, expr, types, w)?,
                    HirArrayElement::Spread(expr) => {
                        w.write("...");
                        generate_expr(codegen, expr, types, w)?;
                    }
                }
            }
            w.write("]");
        }
        HirExprKind::Vacua => generate_vacua_expr(expr, types, w),
        HirExprKind::Struct(def_id, fields) => {
            w.write("Object.assign(new ");
            w.write(codegen.resolve_def(*def_id));
            w.write("(), { ");
            for (idx, (name, value)) in fields.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(*name));
                w.write(": ");
                generate_expr(codegen, value, types, w)?;
            }
            w.write(" })");
        }
        HirExprKind::Tuple(elements) => {
            w.write("[");
            for (idx, element) in elements.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, element, types, w)?;
            }
            w.write("]");
        }
        HirExprKind::Scribe(kind, args) => {
            let method = match kind {
                crate::hir::HirScribeKind::Vide => "debug",
                crate::hir::HirScribeKind::Mone => "warn",
                crate::hir::HirScribeKind::Nota | crate::hir::HirScribeKind::Scribe => "log",
            };
            w.write("console.");
            w.write(method);
            w.write("(");
            for (idx, arg) in args.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Scriptum(template, args) => {
            generate_scriptum_expr(codegen, codegen.resolve_symbol(*template), args, types, w)?;
        }
        HirExprKind::Panic(value) | HirExprKind::Throw(value) => {
            w.write("(() => { throw new Error(String(");
            generate_expr(codegen, value, types, w)?;
            w.write(")); })()");
        }
        HirExprKind::Tempta { body, catch, finally } => {
            w.write("(() => { try ");
            stmt::generate_inline_block(codegen, body, types, w)?;
            if let Some(catch) = catch {
                w.write(" catch (_err) ");
                stmt::generate_inline_block(codegen, catch, types, w)?;
            }
            if let Some(finally) = finally {
                w.write(" finally ");
                stmt::generate_inline_block(codegen, finally, types, w)?;
            }
            w.write(" })()");
        }
        HirExprKind::Clausura(params, ret_ty, _, body) => {
            w.write("(");
            for (idx, param) in params.iter().enumerate() {
                if idx > 0 {
                    w.write(", ");
                }
                w.write(codegen.resolve_symbol(param.name));
                if param.optional {
                    w.write("?");
                }
                w.write(": ");
                w.write(&types::type_to_ts(codegen, param.ty, types));
            }
            w.write(")");
            if let Some(ret_ty) = ret_ty {
                w.write(": ");
                w.write(&types::type_to_ts(codegen, *ret_ty, types));
            }
            w.write(" => ");
            generate_expr(codegen, body, types, w)?;
        }
        HirExprKind::Cede(inner) => {
            w.write("await ");
            generate_expr(codegen, inner, types, w)?;
        }
        HirExprKind::Verte { source, target, entries } => match types.get(*target) {
            Type::Struct(def_id) => {
                w.write("Object.assign(new ");
                w.write(codegen.resolve_def(*def_id));
                w.write("(), ");
                if let Some(entries) = entries {
                    w.write("{ ");
                    let mut wrote_any = false;
                    for field in entries {
                        let Some(value) = &field.value else {
                            continue;
                        };
                        if wrote_any {
                            w.write(", ");
                        }
                        match &field.key {
                            HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                                w.write(codegen.resolve_symbol(*name));
                            }
                            HirObjectKey::Computed(expr) => {
                                w.write("[");
                                generate_expr(codegen, expr, types, w)?;
                                w.write("]");
                            }
                            HirObjectKey::Spread(expr) => {
                                w.write("...");
                                generate_expr(codegen, expr, types, w)?;
                                wrote_any = true;
                                continue;
                            }
                        }
                        w.write(": ");
                        generate_expr(codegen, value, types, w)?;
                        wrote_any = true;
                    }
                    w.write(" }");
                } else {
                    generate_expr(codegen, source, types, w)?;
                }
                w.write(")");
            }
            Type::Map(_, _) => {
                if let Some(entries) = entries {
                    w.write("{ ");
                    let mut wrote_any = false;
                    for field in entries {
                        if wrote_any {
                            w.write(", ");
                        }
                        match &field.key {
                            HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                                w.write(codegen.resolve_symbol(*name));
                                if let Some(value) = &field.value {
                                    w.write(": ");
                                    generate_expr(codegen, value, types, w)?;
                                }
                            }
                            HirObjectKey::Computed(expr) => {
                                w.write("[");
                                generate_expr(codegen, expr, types, w)?;
                                w.write("]");
                                if let Some(value) = &field.value {
                                    w.write(": ");
                                    generate_expr(codegen, value, types, w)?;
                                }
                            }
                            HirObjectKey::Spread(expr) => {
                                w.write("...");
                                generate_expr(codegen, expr, types, w)?;
                            }
                        }
                        wrote_any = true;
                    }
                    w.write(" }");
                } else {
                    generate_expr(codegen, source, types, w)?;
                    w.write(" as ");
                    w.write(&types::type_to_ts(codegen, *target, types));
                }
            }
            _ => {
                generate_expr(codegen, source, types, w)?;
                w.write(" as ");
                w.write(&types::type_to_ts(codegen, *target, types));
            }
        },
        HirExprKind::Conversio { source, target, params: _, fallback } => {
            let target_resolved = types.get(*target);
            match target_resolved {
                Type::Primitive(Primitive::Numerus) => {
                    if let Some(fb) = fallback {
                        // WHY: Number("bad") returns NaN, not null — ?? won't catch it.
                        // Use an IIFE with isNaN to correctly apply the fallback.
                        w.write("((v) => isNaN(v) ? ");
                        generate_expr(codegen, fb, types, w)?;
                        w.write(" : v)(Number(");
                        generate_expr(codegen, source, types, w)?;
                        w.write("))");
                    } else {
                        w.write("Number(");
                        generate_expr(codegen, source, types, w)?;
                        w.write(")");
                    }
                }
                Type::Primitive(Primitive::Fractus) => {
                    if let Some(fb) = fallback {
                        w.write("((v) => isNaN(v) ? ");
                        generate_expr(codegen, fb, types, w)?;
                        w.write(" : v)(parseFloat(");
                        generate_expr(codegen, source, types, w)?;
                        w.write("))");
                    } else {
                        w.write("parseFloat(");
                        generate_expr(codegen, source, types, w)?;
                        w.write(")");
                    }
                }
                Type::Primitive(Primitive::Textus) => {
                    w.write("String(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")");
                }
                Type::Primitive(Primitive::Bivalens) => {
                    w.write("Boolean(");
                    generate_expr(codegen, source, types, w)?;
                    w.write(")");
                }
                _ => {
                    generate_expr(codegen, source, types, w)?;
                    w.write(" as ");
                    w.write(&types::type_to_ts(codegen, *target, types));
                }
            }
        }
        HirExprKind::Ref(_, inner) | HirExprKind::Deref(inner) => generate_expr(codegen, inner, types, w)?,
        HirExprKind::Block(block) => {
            w.write("(() => ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(")()");
        }
        HirExprKind::Si { cond, then_block, then_catch: None, else_block } => {
            w.write("(");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ? ");
            w.write("(() => ");
            stmt::generate_inline_block(codegen, then_block, types, w)?;
            w.write(")()");
            w.write(" : ");
            if let Some(else_block) = else_block {
                w.write("(() => ");
                stmt::generate_inline_block(codegen, else_block, types, w)?;
                w.write(")()");
            } else {
                w.write("undefined");
            }
            w.write(")");
        }
        HirExprKind::Si { then_catch: Some(_), .. } | HirExprKind::Handled { .. } => {
            return Err(CodegenError {
                message: "structured cape handlers are not emitted by TypeScript codegen in Phase 5C".to_owned(),
            });
        }
        HirExprKind::Discerne(_, _) => {
            w.write("undefined");
        }
        HirExprKind::Loop(block) => {
            w.write("(() => { while (true) ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Dum(cond, block) => {
            w.write("(() => { while (");
            generate_expr(codegen, cond, types, w)?;
            w.write(") ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Itera(mode, def_id, _binding_name, iter, block) => {
            w.write("(() => { for (const ");
            w.write(codegen.resolve_def(*def_id));
            match mode {
                HirIteraMode::Ex | HirIteraMode::Ab => w.write(" of "),
                HirIteraMode::De => w.write(" in "),
            }
            generate_expr(codegen, iter, types, w)?;
            w.write(") ");
            stmt::generate_inline_block(codegen, block, types, w)?;
            w.write(" })()");
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            w.write("[");
            generate_expr(codegen, start, types, w)?;
            w.write(", ");
            generate_expr(codegen, end, types, w)?;
            if let Some(step) = step {
                w.write(", ");
                generate_expr(codegen, step, types, w)?;
            }
            w.write("]");
        }
        HirExprKind::Adfirma(cond, message) => {
            w.write("(() => { if (!(");
            generate_expr(codegen, cond, types, w)?;
            w.write(")) { throw new Error(");
            if let Some(message) = message {
                generate_expr(codegen, message, types, w)?;
            } else {
                w.write("\"assertion failed\"");
            }
            w.write("); } })()");
        }
        HirExprKind::Error => {
            return Err(CodegenError { message: "cannot emit TS for error expression".to_owned() });
        }
    }
    Ok(())
}

/// Emits an explicitly typed empty collection literal.
///
/// HIR must carry the destination type for `vacua`; when it does not, the
/// backend falls back to an array literal instead of inventing semantic type
/// information during codegen.
fn generate_vacua_expr(expr: &HirExpr, types: &TypeTable, w: &mut CodeWriter) {
    match expr.ty.map(|ty| types.get(ty)) {
        Some(Type::Map(_, _)) => w.write("new Map()"),
        Some(Type::Set(_)) => w.write("new Set()"),
        Some(Type::Array(_)) => w.write("[]"),
        _ => w.write("[]"),
    }
}

fn try_generate_intrinsic_call(
    codegen: &TsCodegen<'_>,
    callee: &HirExpr,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let HirExprKind::Path(def_id) = callee.kind else {
        return Ok(false);
    };
    let name = codegen.resolve_def(def_id);
    // Stdlib intrinsics are name-based because these prelude functions do not
    // carry receiver types. Keep the table small and explicit so accidental
    // user functions do not inherit runtime behavior by shape alone.
    let mapped = match name {
        "scribe" => Some("console.log"),
        "vide" => Some("console.debug"),
        "mone" => Some("console.warn"),
        "lege" => Some("prompt"),
        "pavimentum" => Some("Math.floor"),
        "nunc" => None,
        _ => None,
    };

    if name == "nunc" && args.is_empty() {
        w.write("new Date()");
        return Ok(true);
    }

    let Some(target) = mapped else {
        return Ok(false);
    };
    w.write(target);
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, &arg.expr, types, w)?;
    }
    w.write(")");
    Ok(true)
}

fn try_generate_translated_method_call(
    codegen: &TsCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirCallArg],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let method_name = codegen.resolve_symbol(method);
    let receiver_type = receiver
        .ty
        .map(|ty| normalize_receiver_type(types.get(ty), types));

    let is_lista = matches!(receiver_type, Some(Type::Array(_)));
    let is_tabula = matches!(receiver_type, Some(Type::Map(_, _)));
    let is_textus = matches!(receiver_type, Some(Type::Primitive(Primitive::Textus)));

    if method_name == "longitudo" && args.is_empty() && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write(".length");
        return Ok(true);
    }

    if method_name == "accipe" && args.len() == 1 && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write("[");
        generate_expr(codegen, &args[0].expr, types, w)?;
        w.write("]");
        return Ok(true);
    }

    if is_tabula {
        // TypeScript records cover today's map-like lowering for keyed access
        // and enumeration. This deliberately avoids `Map` methods until the
        // target type and empty-map literal policy are unified.
        match method_name {
            "pone" if args.len() == 2 => {
                w.write("(");
                generate_expr(codegen, receiver, types, w)?;
                w.write("[");
                generate_expr(codegen, &args[0].expr, types, w)?;
                w.write("] = ");
                generate_expr(codegen, &args[1].expr, types, w)?;
                w.write(")");
                return Ok(true);
            }
            "accipe" if args.len() == 1 => {
                generate_expr(codegen, receiver, types, w)?;
                w.write("[");
                generate_expr(codegen, &args[0].expr, types, w)?;
                w.write("]");
                return Ok(true);
            }
            "habet" if args.len() == 1 => {
                w.write("(");
                generate_expr(codegen, &args[0].expr, types, w)?;
                w.write(" in ");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            "claves" if args.is_empty() => {
                w.write("Object.keys(");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            "valores" if args.is_empty() => {
                w.write("Object.values(");
                generate_expr(codegen, receiver, types, w)?;
                w.write(")");
                return Ok(true);
            }
            _ => {}
        }
    }

    let translated = if is_lista {
        match method_name {
            "appende" => Some("push"),
            "decapita" => Some("shift"),
            "detrahe" => Some("pop"),
            "filtrata" => Some("filter"),
            "mappata" => Some("map"),
            "reducta" => Some("reduce"),
            "ordinata" => Some("toSorted"),
            "inversa" => Some("reverse"),
            "coniunge" => Some("join"),
            "continet" => Some("includes"),
            "plana" => Some("flat"),
            "seca" => Some("slice"),
            _ => None,
        }
    } else if is_textus {
        match method_name {
            "maiuscula" => Some("toUpperCase"),
            "minuscula" => Some("toLowerCase"),
            "divide" => Some("split"),
            "continet" => Some("includes"),
            "incipe" => Some("startsWith"),
            "fini" => Some("endsWith"),
            "repone" => Some("replace"),
            "reseca" => Some("trim"),
            _ => None,
        }
    } else {
        None
    };

    let Some(translated) = translated else {
        return Ok(false);
    };
    generate_expr(codegen, receiver, types, w)?;
    w.write(".");
    w.write(translated);
    w.write("(");
    for (idx, arg) in args.iter().enumerate() {
        if idx > 0 {
            w.write(", ");
        }
        generate_expr(codegen, &arg.expr, types, w)?;
    }
    w.write(")");
    Ok(true)
}

fn normalize_receiver_type<'a>(mut ty: &'a Type, types: &'a TypeTable) -> &'a Type {
    loop {
        match ty {
            Type::Ref(_, inner) | Type::Alias(_, inner) => {
                ty = types.get(*inner);
            }
            _ => return ty,
        }
    }
}

fn generate_literal(codegen: &TsCodegen<'_>, literal: &HirLiteral, w: &mut CodeWriter) {
    match literal {
        HirLiteral::Int(v) => w.write(&v.to_string()),
        HirLiteral::Float(v) => w.write(&v.to_string()),
        HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        HirLiteral::Regex(pattern, flags) => {
            w.write("/");
            w.write(codegen.resolve_symbol(*pattern));
            w.write("/");
            if let Some(flags) = flags {
                w.write(codegen.resolve_symbol(*flags));
            }
        }
        HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        HirLiteral::Nil => w.write("null"),
    }
}

fn generate_unary_expr(
    codegen: &TsCodegen<'_>,
    op: HirUnOp,
    operand: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match op {
        HirUnOp::Neg => {
            w.write("-");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::Not => {
            w.write("!");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::BitNot => {
            w.write("~");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::IsNull | HirUnOp::IsNil => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == null)");
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" != null)");
        }
        HirUnOp::IsNeg => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" < 0)");
        }
        HirUnOp::IsPos => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" > 0)");
        }
        HirUnOp::IsTrue => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" === true)");
        }
        HirUnOp::IsFalse => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" === false)");
        }
    }
    Ok(())
}

fn binary_op_to_ts(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        Eq | StrictEq => "===",
        NotEq | StrictNotEq => "!==",
        Lt => "<",
        Gt => ">",
        LtEq => "<=",
        GtEq => ">=",
        And => "&&",
        Or => "||",
        Coalesce => "??",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        Is => "===",
        IsNot => "!==",
        InRange | Between => "&&",
    }
}

fn assign_op_to_ts(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
    match op {
        Add => "+=",
        Sub => "-=",
        Mul => "*=",
        Div => "/=",
        Mod => "%=",
        BitAnd => "&=",
        BitOr => "|=",
        BitXor => "^=",
        Shl => "<<=",
        Shr => ">>=",
        _ => "=",
    }
}

fn generate_scriptum_expr(
    codegen: &TsCodegen<'_>,
    template: &str,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    let mut auto_index = 0usize;
    let mut literal = String::new();
    let mut chars = template.chars().peekable();

    w.write("`");
    while let Some(ch) = chars.next() {
        if ch != '§' {
            push_ts_template_char(&mut literal, ch);
            continue;
        }

        w.write(&literal);
        literal.clear();

        let mut index = String::new();
        while chars.peek().is_some_and(|next| next.is_ascii_digit()) {
            index.push(chars.next().expect("peeked digit"));
        }

        // Missing template arguments stay visible as `undefined`; this keeps
        // codegen total for malformed templates without manufacturing a source
        // diagnostic after semantic analysis has already run.
        let arg_index = if index.is_empty() {
            let current = auto_index;
            auto_index += 1;
            current
        } else {
            index.parse::<usize>().unwrap_or(usize::MAX)
        };

        w.write("${");
        if let Some(arg) = args.get(arg_index) {
            generate_expr(codegen, arg, types, w)?;
        } else {
            w.write("undefined");
        }
        w.write("}");
    }
    w.write(&literal);
    w.write("`");
    Ok(())
}

fn generate_textus_index_expr(
    codegen: &TsCodegen<'_>,
    object: &HirExpr,
    index: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &index.kind {
        HirExprKind::Intervallum { start, end, step, kind } => {
            w.write("Array.from(");
            generate_expr(codegen, object, types, w)?;
            w.write(").slice(");
            generate_expr(codegen, start, types, w)?;
            w.write(", ");
            match kind {
                HirRangeKind::Exclusive => generate_expr(codegen, end, types, w)?,
                HirRangeKind::Inclusive => {
                    w.write("(");
                    generate_expr(codegen, end, types, w)?;
                    w.write(") + 1");
                }
            }
            w.write(")");
            if let Some(step) = step {
                w.write(".filter((_, __faber_i) => __faber_i % Math.max(1, ");
                generate_expr(codegen, step, types, w)?;
                w.write(") === 0)");
            }
            w.write(".join(\"\")");
        }
        _ => {
            w.write("(Array.from(");
            generate_expr(codegen, object, types, w)?;
            w.write(")[");
            generate_expr(codegen, index, types, w)?;
            w.write("] ?? \"\")");
        }
    }
    Ok(())
}

fn push_ts_template_char(out: &mut String, ch: char) {
    match ch {
        '`' => out.push_str("\\`"),
        '\\' => out.push_str("\\\\"),
        '$' => out.push_str("\\$"),
        _ => out.push(ch),
    }
}

/// Returns whether a block contains `cede` and therefore needs an async wrapper.
///
/// TypeScript only permits `await` inside async functions or modules with
/// top-level await. The backend wraps package entry blocks in an IIFE for
/// statement isolation, so it uses this scan to make that wrapper `async` only
/// when the HIR actually requires it.
pub fn contains_await_in_block(block: &HirBlock) -> bool {
    let mut visitor = AwaitDetector::default();
    visitor.visit_block(block);
    visitor.found
}

#[derive(Default)]
struct AwaitDetector {
    found: bool,
}

impl HirVisitor for AwaitDetector {
    fn visit_expr(&mut self, expr: &HirExpr) {
        if self.found {
            return;
        }
        if matches!(expr.kind, HirExprKind::Cede(_)) {
            self.found = true;
            return;
        }
        walk_expr(self, expr);
    }
}
