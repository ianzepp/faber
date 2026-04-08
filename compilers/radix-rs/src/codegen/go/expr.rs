use super::stmt;
use super::types;
use super::{CodeWriter, CodegenError, GoCodegen};
use crate::hir::{
    HirArrayElement, HirBinOp, HirExpr, HirExprKind, HirIteraMode, HirLiteral, HirObjectKey, HirUnOp,
};
use crate::semantic::{Primitive, Type, TypeTable};

pub fn generate_expr(
    codegen: &GoCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => w.write(codegen.resolve_def(*def_id)),
        HirExprKind::Literal(lit) => generate_literal(codegen, lit, w),
        HirExprKind::Binary(op, lhs, rhs) => {
            if matches!(op, HirBinOp::Coalesce) {
                generate_coalesce_expr(codegen, lhs, rhs, types, w)?;
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
            generate_expr(codegen, receiver, types, w)?;
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
            if matches!(object.ty.map(|ty| normalize_receiver_type(types.get(ty), types)), Some(Type::Map(_, _))) {
                generate_expr(codegen, object, types, w)?;
                w.write("[");
                w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                w.write("]");
                return Ok(());
            }
            generate_expr(codegen, object, types, w)?;
            w.write(".");
            w.write(&capitalize(codegen.resolve_symbol(*field)));
        }
        HirExprKind::Index(object, index) => {
            generate_expr(codegen, object, types, w)?;
            w.write("[");
            generate_expr(codegen, index, types, w)?;
            w.write("]");
        }
        HirExprKind::OptionalChain(object, chain) => {
            // WHY: Go has no optional chaining — emit a plain access.
            // A full implementation would use nil-check wrappers.
            generate_expr(codegen, object, types, w)?;
            match chain {
                crate::hir::HirOptionalChainKind::Member(field) => {
                    if matches!(object.ty.map(|ty| normalize_receiver_type(types.get(ty), types)), Some(Type::Map(_, _))) {
                        w.write("[");
                        w.write(&format!("{:?}", codegen.resolve_symbol(*field)));
                        w.write("]");
                    } else {
                        w.write(".");
                        w.write(&capitalize(codegen.resolve_symbol(*field)));
                    }
                }
                crate::hir::HirOptionalChainKind::Index(index) => {
                    w.write("[");
                    generate_expr(codegen, index, types, w)?;
                    w.write("]");
                }
                crate::hir::HirOptionalChainKind::Call(args) => {
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
        HirExprKind::NonNull(object, chain) => {
            // WHY: Go has no non-null assertion — emit plain access.
            generate_expr(codegen, object, types, w)?;
            match chain {
                crate::hir::HirNonNullKind::Member(field) => {
                    if matches!(object.ty.map(|ty| normalize_receiver_type(types.get(ty), types)), Some(Type::Map(_, _))) {
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
                generate_expr(codegen, value, types, w)?;
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
            w.write(&format!("{:?}", render_scriptum_template(codegen.resolve_symbol(*template), args.len())));
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
                stmt::generate_block(codegen, catch, types, w, |_| {})?;
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
                            generate_expr(codegen, value, types, w)?;
                            wrote_any = true;
                        }
                        w.write("}");
                    } else {
                        generate_expr(codegen, source, types, w)?;
                    }
                }
                Type::Map(key_ty, value_ty) => {
                    w.write("map[");
                    w.write(&types::type_to_go(codegen, *key_ty, types));
                    w.write("]");
                    w.write(&types::type_to_go(codegen, *value_ty, types));
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
                                HirObjectKey::Spread(_) => continue,
                            }
                            w.write(": ");
                            generate_expr(codegen, value, types, w)?;
                            wrote_any = true;
                        }
                    }
                    w.write("}");
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
            stmt::generate_block(codegen, block, types, w, |_| {})?;
            w.write("()");
        }
        HirExprKind::Si(cond, then_block, else_block) => {
            // WHY: Go's if is a statement, not an expression. Wrap in IIFE.
            w.write("func() ");
            w.write(&expr_return_type(expr, types, codegen));
            w.write(" { if ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            stmt::generate_block(codegen, then_block, types, w, |_| {})?;
            if let Some(else_block) = else_block {
                w.write(" else ");
                stmt::generate_block(codegen, else_block, types, w, |_| {})?;
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
            // WHY: Go has no method chaining on slices. Emit the source and add
            // a TODO comment. Full pipeline codegen requires helper functions.
            generate_expr(codegen, source, types, w)?;
            if filter.is_some() || !transforms.is_empty() {
                w.write(" /* TODO: ab pipeline */");
            }
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

fn try_generate_intrinsic_call(
    codegen: &GoCodegen<'_>,
    callee: &HirExpr,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let HirExprKind::Path(def_id) = callee.kind else {
        return Ok(false);
    };
    let name = codegen.resolve_def(def_id);
    let mapped = match name {
        "scribe" => Some("fmt.Println"),
        "vide" => Some("fmt.Println"),
        "mone" => Some("fmt.Fprintln(os.Stderr, "),
        _ => None,
    };

    let Some(target) = mapped else {
        return Ok(false);
    };

    if name == "mone" {
        // Special: fmt.Fprintln(os.Stderr, args...)
        w.write("fmt.Fprintln(os.Stderr, ");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(codegen, arg, types, w)?;
        }
        w.write(")");
    } else {
        w.write(target);
        w.write("(");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                w.write(", ");
            }
            generate_expr(codegen, arg, types, w)?;
        }
        w.write(")");
    }
    Ok(true)
}

fn try_generate_translated_method_call(
    codegen: &GoCodegen<'_>,
    receiver: &HirExpr,
    method: crate::lexer::Symbol,
    args: &[HirExpr],
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<bool, CodegenError> {
    let method_name = codegen.resolve_symbol(method);
    let receiver_type = receiver.ty.map(|ty| normalize_receiver_type(types.get(ty), types));
    let is_lista = matches!(receiver_type, Some(Type::Array(_)));
    let is_textus = matches!(receiver_type, Some(Type::Primitive(Primitive::Textus)));

    if method_name == "longitudo" && args.is_empty() && (is_lista || is_textus) {
        w.write("len(");
        generate_expr(codegen, receiver, types, w)?;
        w.write(")");
        return Ok(true);
    }

    if method_name == "accipe" && args.len() == 1 && (is_lista || is_textus) {
        generate_expr(codegen, receiver, types, w)?;
        w.write("[");
        generate_expr(codegen, &args[0], types, w)?;
        w.write("]");
        return Ok(true);
    }

    Ok(false)
}

fn generate_array_expr(
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

    if elements.iter().any(|element| matches!(element, HirArrayElement::Spread(_))) {
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

fn normalize_receiver_type<'a>(mut ty: &'a Type, types: &'a TypeTable) -> &'a Type {
    loop {
        match ty {
            Type::Ref(_, inner) | Type::Alias(_, inner) => ty = types.get(*inner),
            _ => return ty,
        }
    }
}

fn expr_return_type(expr: &HirExpr, types: &TypeTable, codegen: &GoCodegen<'_>) -> String {
    expr.ty.map(|ty| types::type_to_go(codegen, ty, types)).filter(|ty| !ty.is_empty()).unwrap_or_else(|| "any".to_owned())
}

fn generate_coalesce_expr(
    codegen: &GoCodegen<'_>,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match lhs.ty.map(|ty| normalize_receiver_type(types.get(ty), types)) {
        Some(Type::Option(_)) => {
            let ret_ty = lhs.ty.or(rhs.ty).map(|ty| types::type_to_go(codegen, ty, types)).unwrap_or_else(|| "any".to_owned());
            w.write("func() ");
            w.write(&ret_ty);
            w.write(" { v := ");
            generate_expr(codegen, lhs, types, w)?;
            w.write("; if v != nil { return ");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" }; return ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(" }()");
        }
        Some(Type::Primitive(Primitive::Textus)) => {
            w.write("func() string { v := ");
            generate_expr(codegen, lhs, types, w)?;
            w.write("; if v != \"\" { return v }; return ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(" }()");
        }
        Some(Type::Primitive(Primitive::Numerus)) => {
            w.write("func() int { v := ");
            generate_expr(codegen, lhs, types, w)?;
            w.write("; if v != 0 { return v }; return ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(" }()");
        }
        _ => {
            w.write("(");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" || ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(")");
        }
    }
    Ok(())
}

fn generate_literal(codegen: &GoCodegen<'_>, literal: &HirLiteral, w: &mut CodeWriter) {
    match literal {
        HirLiteral::Int(v) => w.write(&v.to_string()),
        HirLiteral::Float(v) => w.write(&v.to_string()),
        HirLiteral::String(sym) => w.write(&format!("{:?}", codegen.resolve_symbol(*sym))),
        HirLiteral::Regex(pattern, flags) => {
            w.write("regexp.MustCompile(`");
            w.write(codegen.resolve_symbol(*pattern));
            if let Some(flags) = flags {
                w.write("(?");
                w.write(codegen.resolve_symbol(*flags));
                w.write(")");
            }
            w.write("`)");
        }
        HirLiteral::Bool(v) => w.write(if *v { "true" } else { "false" }),
        HirLiteral::Nil => w.write("nil"),
    }
}

fn generate_unary_expr(
    codegen: &GoCodegen<'_>,
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
            w.write("^");
            generate_expr(codegen, operand, types, w)?;
        }
        HirUnOp::IsNull | HirUnOp::IsNil => {
            if !matches!(operand.ty.map(|ty| normalize_receiver_type(types.get(ty), types)), Some(Type::Option(_)) | Some(Type::Primitive(Primitive::Ignotum)) | Some(Type::Primitive(Primitive::Nihil)) | Some(Type::Param(_))) {
                w.write("false");
                return Ok(());
            }
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == nil)");
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            if !matches!(operand.ty.map(|ty| normalize_receiver_type(types.get(ty), types)), Some(Type::Option(_)) | Some(Type::Primitive(Primitive::Ignotum)) | Some(Type::Primitive(Primitive::Nihil)) | Some(Type::Param(_))) {
                w.write("true");
                return Ok(());
            }
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" != nil)");
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
            w.write(" == true)");
        }
        HirUnOp::IsFalse => {
            w.write("(");
            generate_expr(codegen, operand, types, w)?;
            w.write(" == false)");
        }
    }
    Ok(())
}

fn binary_op_to_go(op: HirBinOp) -> &'static str {
    use HirBinOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        Eq | StrictEq | Is => "==",
        NotEq | StrictNotEq | IsNot => "!=",
        Lt => "<",
        Gt => ">",
        LtEq => "<=",
        GtEq => ">=",
        And => "&&",
        Or => "||",
        Coalesce => "||", // WHY: Go has no ?? — || is the closest for booleans
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        InRange | Between => "&&",
    }
}

fn assign_op_to_go(op: HirBinOp) -> &'static str {
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

fn render_scriptum_template(template: &str, arg_count: usize) -> String {
    let mut rendered = template.to_owned();
    // Replace §N placeholders with Go's %v format verbs
    for idx in (1..=arg_count).rev() {
        rendered = rendered.replace(&format!("§{}", idx), "%v");
    }
    // Replace bare § with %v
    rendered = rendered.replace('§', "%v");
    rendered
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}
