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
            let is_failable_call =
                matches!(&callee.kind, HirExprKind::Path(def_id) if codegen.is_failable_def(*def_id));
            generate_expr(codegen, callee, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
            if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
                w.write("?");
            }
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            let is_failable_call = codegen.is_failable_method_name(*method);
            generate_expr(
                codegen,
                receiver,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.write(".");
            w.write(codegen.resolve_symbol(*method));
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr_unwrapped(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
            if is_failable_call && in_failable_fn && !in_entry && !suppress_error_propagation {
                w.write("?");
            }
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
        HirExprKind::OptionalChain(object, chain) => {
            w.write("(");
            generate_expr(codegen, object, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            match chain {
                HirOptionalChainKind::Member(field) => {
                    w.write(").as_ref().map(|__faber_opt| __faber_opt.");
                    w.write(codegen.resolve_symbol(*field));
                    w.write(")");
                }
                HirOptionalChainKind::Index(index) => {
                    w.write(").as_ref().map(|__faber_opt| __faber_opt[");
                    generate_expr(codegen, index, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write("])");
                }
                HirOptionalChainKind::Call(args) => {
                    w.write(").and_then(|__faber_opt| Some(__faber_opt(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            w.write(", ");
                        }
                        generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    }
                    w.write(")))");
                }
            }
        }
        HirExprKind::Ab { source, filter, transforms } => {
            let suffix = expr.id.0;
            let vec_name = format!("__faber_ab_vec_{}", suffix);
            let n_name = format!("__faber_ab_n_{}", suffix);
            let len_name = format!("__faber_ab_len_{}", suffix);
            let keep_name = format!("__faber_ab_keep_{}", suffix);
            let sum_name = format!("__faber_ab_sum_{}", suffix);
            let item_name = format!("__faber_ab_item_{}", suffix);

            w.writeln("{");
            let mut ab_result = Ok(());
            w.indented(|w| {
                w.write("let mut ");
                w.write(&vec_name);
                w.write(" = (");
                if ab_result.is_err() {
                    return;
                }
                ab_result =
                    generate_expr(codegen, source, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                w.write(").iter()");
                if let Some(filter) = filter {
                    match &filter.kind {
                        HirCollectionFilterKind::Property(name) => {
                            w.write(".filter(|");
                            w.write(&item_name);
                            w.write("| ");
                            if filter.negated {
                                w.write("!");
                            }
                            w.write(&item_name);
                            w.write(".");
                            w.write(codegen.resolve_symbol(*name));
                            w.write(")");
                        }
                        HirCollectionFilterKind::Condition(cond) => {
                            w.write(".filter(|_| ");
                            if ab_result.is_err() {
                                return;
                            }
                            ab_result = generate_expr(
                                codegen,
                                cond,
                                types,
                                w,
                                in_failable_fn,
                                in_entry,
                                suppress_error_propagation,
                            );
                            w.write(")");
                        }
                    }
                }
                w.writeln(".collect::<Vec<_>>();");

                let mut terminal_sum = false;
                for transform in transforms {
                    if terminal_sum {
                        break;
                    }
                    match transform.kind {
                        HirTransformKind::First => {
                            w.write("let ");
                            w.write(&n_name);
                            w.write(" = ");
                            if let Some(arg) = &transform.arg {
                                if ab_result.is_err() {
                                    return;
                                }
                                ab_result = generate_expr(
                                    codegen,
                                    arg,
                                    types,
                                    w,
                                    in_failable_fn,
                                    in_entry,
                                    suppress_error_propagation,
                                );
                            } else {
                                w.write("1");
                            }
                            w.writeln(" as usize;");
                            w.write(&vec_name);
                            w.write(" = ");
                            w.write(&vec_name);
                            w.write(".into_iter().take(");
                            w.write(&n_name);
                            w.writeln(").collect::<Vec<_>>();");
                        }
                        HirTransformKind::Last => {
                            w.write("let ");
                            w.write(&n_name);
                            w.write(" = ");
                            if let Some(arg) = &transform.arg {
                                if ab_result.is_err() {
                                    return;
                                }
                                ab_result = generate_expr(
                                    codegen,
                                    arg,
                                    types,
                                    w,
                                    in_failable_fn,
                                    in_entry,
                                    suppress_error_propagation,
                                );
                            } else {
                                w.write("1");
                            }
                            w.writeln(" as usize;");
                            w.write("let ");
                            w.write(&len_name);
                            w.write(" = ");
                            w.write(&vec_name);
                            w.writeln(".len();");
                            w.write("let ");
                            w.write(&keep_name);
                            w.write(" = ");
                            w.write(&n_name);
                            w.write(".min(");
                            w.write(&len_name);
                            w.writeln(");");
                            w.write(&vec_name);
                            w.write(" = ");
                            w.write(&vec_name);
                            w.write(".into_iter().skip(");
                            w.write(&len_name);
                            w.write(".saturating_sub(");
                            w.write(&keep_name);
                            w.writeln(")).collect::<Vec<_>>();");
                        }
                        HirTransformKind::Sum => {
                            w.write("let ");
                            w.write(&sum_name);
                            w.write(" = ");
                            w.write(&vec_name);
                            w.writeln(".into_iter().copied().sum::<i64>();");
                            terminal_sum = true;
                        }
                    }
                }

                if transforms
                    .iter()
                    .any(|t| matches!(t.kind, HirTransformKind::Sum))
                {
                    w.write(&sum_name);
                    w.newline();
                } else {
                    w.write(&vec_name);
                    w.newline();
                }
            });
            ab_result?;
            w.write("}");
        }
        HirExprKind::Block(block) => {
            w.writeln("{");
            let mut block_result = Ok(());
            w.indented(|w| {
                for stmt in &block.stmts {
                    if block_result.is_err() {
                        return;
                    }
                    block_result = super::stmt::generate_stmt(
                        codegen,
                        stmt,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    );
                }
                if let Some(expr) = &block.expr {
                    if block_result.is_err() {
                        return;
                    }
                    block_result =
                        generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                }
            });
            block_result?;
            w.write("}");
        }
        HirExprKind::Tempta { body, catch, finally } => {
            w.writeln("{");
            let mut tempta_result = Ok(());
            w.indented(|w| {
                for stmt in &body.stmts {
                    if tempta_result.is_err() {
                        return;
                    }
                    tempta_result = super::stmt::generate_stmt(
                        codegen,
                        stmt,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation || catch.is_some(),
                    );
                }
                if let Some(expr) = &body.expr {
                    if tempta_result.is_err() {
                        return;
                    }
                    tempta_result = generate_expr(
                        codegen,
                        expr,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation || catch.is_some(),
                    );
                    w.writeln(";");
                }
                if let Some(catch) = catch {
                    for stmt in &catch.stmts {
                        if tempta_result.is_err() {
                            return;
                        }
                        tempta_result = super::stmt::generate_stmt(
                            codegen,
                            stmt,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                    }
                    if let Some(expr) = &catch.expr {
                        if tempta_result.is_err() {
                            return;
                        }
                        tempta_result = generate_expr(
                            codegen,
                            expr,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(";");
                    }
                }
                if let Some(finally) = finally {
                    for stmt in &finally.stmts {
                        if tempta_result.is_err() {
                            return;
                        }
                        tempta_result = super::stmt::generate_stmt(
                            codegen,
                            stmt,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                    }
                    if let Some(expr) = &finally.expr {
                        if tempta_result.is_err() {
                            return;
                        }
                        tempta_result = generate_expr(
                            codegen,
                            expr,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                        w.writeln(";");
                    }
                }
            });
            tempta_result?;
            w.write("}");
        }
        HirExprKind::Si(cond, then, else_) => {
            w.write("if ");
            generate_expr_unwrapped(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            generate_block(codegen, then, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if let Some(else_block) = else_ {
                w.write(" else ");
                generate_block(
                    codegen,
                    else_block,
                    types,
                    w,
                    in_failable_fn,
                    in_entry,
                    suppress_error_propagation,
                )?;
            }
        }
        HirExprKind::Discerne(scrutinee, arms) => {
            w.write("match ");
            generate_expr(
                codegen,
                scrutinee,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            )?;
            w.writeln(" {");
            let mut discerne_result = Ok(());
            w.indented(|w| {
                for arm in arms {
                    generate_pattern(codegen, &arm.pattern, w);
                    if let Some(guard) = &arm.guard {
                        w.write(" if ");
                        if discerne_result.is_err() {
                            return;
                        }
                        discerne_result = generate_expr(
                            codegen,
                            guard,
                            types,
                            w,
                            in_failable_fn,
                            in_entry,
                            suppress_error_propagation,
                        );
                    }
                    w.write(" => ");
                    if discerne_result.is_err() {
                        return;
                    }
                    discerne_result = generate_expr(
                        codegen,
                        &arm.body,
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    );
                    w.writeln(",");
                }
            });
            discerne_result?;
            w.write("}");
        }
        HirExprKind::Loop(block) => {
            w.write("loop ");
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Dum(cond, block) => {
            w.write("while ");
            generate_expr_unwrapped(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Itera(_, binding, iter, block) => {
            w.write("for ");
            w.write(codegen.resolve_def(*binding));
            w.write(" in ");
            generate_expr(codegen, iter, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
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
        HirExprKind::Array(elements) => {
            w.write("vec![");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write("]");
        }
        HirExprKind::Struct(def_id, fields) => {
            w.write(codegen.resolve_def(*def_id));
            w.writeln(" {");
            let mut struct_result = Ok(());
            w.indented(|w| {
                for (name, value) in fields {
                    w.write(codegen.resolve_symbol(*name));
                    w.write(": ");
                    if struct_result.is_err() {
                        return;
                    }
                    struct_result =
                        generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                    w.writeln(",");
                }
            });
            struct_result?;
            w.write("}");
        }
        HirExprKind::Tuple(elements) => {
            w.write("(");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, elem, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
            w.write(")");
        }
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
                        for (name, value) in entries {
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
                    } else {
                        w.write("vec![");
                        for (i, elem_expr) in elements.iter().enumerate() {
                            if i > 0 {
                                w.write(", ");
                            }
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
                        for (key, value) in entries {
                            w.write(&map_name);
                            w.write(".insert(");
                            write_innatum_map_key(codegen, types, *key, *key_ty, w);
                            w.write(", ");
                            if map_result.is_err() {
                                return;
                            }
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

#[allow(clippy::too_many_arguments)]
fn generate_binary_expr(
    codegen: &RustCodegen<'_>,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    wrap: bool,
) -> Result<(), CodegenError> {
    match op {
        HirBinOp::Coalesce => {
            let lhs_ty = lhs.ty.map(|ty| resolve_type(ty, types));
            match lhs_ty {
                Some(Type::Option(_)) => {
                    w.write("(");
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    let rhs_ty = rhs.ty.map(|ty| resolve_type(ty, types));
                    if matches!(rhs_ty, Some(Type::Option(_))) {
                        w.write(").or(");
                    } else {
                        w.write(").unwrap_or(");
                    }
                    generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(")");
                }
                Some(Type::Primitive(Primitive::Nihil)) => {
                    generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                }
                _ => {
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                }
            }
        }
        HirBinOp::InRange => {
            if let HirExprKind::Tuple(bounds) = &rhs.kind {
                if bounds.len() >= 2 {
                    if wrap {
                        w.write("(");
                    }
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(" >= ");
                    generate_expr(
                        codegen,
                        &bounds[0],
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    w.write(" && ");
                    generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                    w.write(" < ");
                    generate_expr(
                        codegen,
                        &bounds[1],
                        types,
                        w,
                        in_failable_fn,
                        in_entry,
                        suppress_error_propagation,
                    )?;
                    if wrap {
                        w.write(")");
                    }
                } else {
                    w.write("false");
                }
            } else {
                w.write("false");
            }
        }
        HirBinOp::Between => {
            w.write("(");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(").contains(&");
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(")");
        }
        _ => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            w.write(binop_to_rust(op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            if wrap {
                w.write(")");
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_unary_expr(
    codegen: &RustCodegen<'_>,
    op: HirUnOp,
    operand: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
    wrap: bool,
) -> Result<(), CodegenError> {
    match op {
        HirUnOp::IsNull | HirUnOp::IsNil => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == None");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" != None");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsNeg => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" < 0");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsPos => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" > 0");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsTrue => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == true");
            if wrap {
                w.write(")");
            }
        }
        HirUnOp::IsFalse => {
            if wrap {
                w.write("(");
            }
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" == false");
            if wrap {
                w.write(")");
            }
        }
        _ => {
            w.write(unop_to_rust(op));
            generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
    }

    Ok(())
}

/// Generate a Rust literal.
///
/// TRANSFORMS:
///   HirLiteral::String("hello") -> "hello" (with escaping)
///   HirLiteral::Bool(true)      -> true
///   HirLiteral::Nil             -> None
///
/// TARGET: Rust string escaping (\n, \t, \\, \").
fn generate_literal(codegen: &RustCodegen<'_>, lit: &HirLiteral, w: &mut CodeWriter) {
    match lit {
        HirLiteral::Int(n) => {
            w.write(&n.to_string());
        }
        HirLiteral::Float(f) => {
            w.write(&f.to_string());
        }
        HirLiteral::String(s) => {
            write_rust_string_literal(codegen.resolve_symbol(*s), w);
        }
        HirLiteral::Regex(pattern, flags) => {
            let pattern_text = codegen.resolve_symbol(*pattern);
            let regex_pattern = apply_regex_flags(pattern_text, flags.map(|f| codegen.resolve_symbol(f)));
            w.write("regex::Regex::new(");
            write_rust_string_literal(&regex_pattern, w);
            w.write(").unwrap()");
        }
        HirLiteral::Bool(b) => {
            w.write(if *b { "true" } else { "false" });
        }
        HirLiteral::Nil => {
            w.write("None");
        }
    }
}

fn write_rust_string_literal(text: &str, w: &mut CodeWriter) {
    w.write("\"");
    for ch in text.chars() {
        match ch {
            '\\' => w.write("\\\\"),
            '"' => w.write("\\\""),
            '\n' => w.write("\\n"),
            '\r' => w.write("\\r"),
            '\t' => w.write("\\t"),
            _ => w.write(&ch.to_string()),
        }
    }
    w.write("\"");
}

fn apply_regex_flags(pattern: &str, flags: Option<&str>) -> String {
    let Some(flags) = flags else {
        return pattern.to_owned();
    };

    let mapped: String = flags
        .chars()
        .filter(|flag| matches!(flag, 'i' | 'm' | 's' | 'u' | 'U' | 'x'))
        .collect();

    if mapped.is_empty() {
        pattern.to_owned()
    } else {
        format!("(?{}){}", mapped, pattern)
    }
}

fn rust_scribe_format(expr: &HirExpr, types: &TypeTable) -> &'static str {
    if matches!(
        expr.kind,
        HirExprKind::Literal(HirLiteral::String(_))
            | HirExprKind::Literal(HirLiteral::Int(_))
            | HirExprKind::Literal(HirLiteral::Float(_))
            | HirExprKind::Literal(HirLiteral::Bool(_))
    ) {
        return "{}";
    }

    match expr.ty.map(|ty| resolve_type(ty, types)) {
        Some(Type::Primitive(Primitive::Textus))
        | Some(Type::Primitive(Primitive::Numerus))
        | Some(Type::Primitive(Primitive::Fractus))
        | Some(Type::Primitive(Primitive::Bivalens))
        | Some(Type::Primitive(Primitive::Vacuum))
        | Some(Type::Primitive(Primitive::Nihil)) => "{}",
        _ => "{:?}",
    }
}

fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        ty => ty.clone(),
    }
}

fn generate_pattern(codegen: &RustCodegen<'_>, pattern: &HirPattern, w: &mut CodeWriter) {
    match pattern {
        HirPattern::Wildcard => {
            w.write("_");
        }
        HirPattern::Binding(def_id, name) => {
            let resolved = codegen.resolve_def(*def_id);
            if resolved == "unresolved_def" {
                w.write(codegen.resolve_symbol(*name));
            } else {
                w.write(resolved);
            }
        }
        HirPattern::Variant(def_id, fields) => {
            w.write(codegen.resolve_def(*def_id));
            if !fields.is_empty() {
                w.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        w.write(", ");
                    }
                    generate_pattern(codegen, field, w);
                }
                w.write(" }");
            }
        }
        HirPattern::Literal(lit) => {
            generate_literal(codegen, lit, w);
        }
    }
}

fn generate_block(
    codegen: &RustCodegen<'_>,
    block: &HirBlock,
    types: &TypeTable,
    w: &mut CodeWriter,
    in_failable_fn: bool,
    in_entry: bool,
    suppress_error_propagation: bool,
) -> Result<(), CodegenError> {
    w.writeln("{");
    let mut block_result = Ok(());
    w.indented(|w| {
        for stmt in &block.stmts {
            if block_result.is_err() {
                return;
            }
            block_result = super::stmt::generate_stmt(
                codegen,
                stmt,
                types,
                w,
                in_failable_fn,
                in_entry,
                suppress_error_propagation,
            );
        }
        if let Some(expr) = &block.expr {
            if block_result.is_err() {
                return;
            }
            block_result = generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
        }
    });
    block_result?;
    w.write("}");
    Ok(())
}

fn binop_to_rust(op: HirBinOp) -> &'static str {
    match op {
        HirBinOp::Add => "+",
        HirBinOp::Sub => "-",
        HirBinOp::Mul => "*",
        HirBinOp::Div => "/",
        HirBinOp::Mod => "%",
        HirBinOp::Eq => "==",
        HirBinOp::NotEq => "!=",
        HirBinOp::StrictEq => "==",
        HirBinOp::StrictNotEq => "!=",
        HirBinOp::Lt => "<",
        HirBinOp::Gt => ">",
        HirBinOp::LtEq => "<=",
        HirBinOp::GtEq => ">=",
        HirBinOp::And => "&&",
        HirBinOp::Or => "||",
        HirBinOp::Coalesce => "??",
        HirBinOp::BitAnd => "&",
        HirBinOp::BitOr => "|",
        HirBinOp::BitXor => "^",
        HirBinOp::Shl => "<<",
        HirBinOp::Shr => ">>",
        HirBinOp::Is => "==",
        HirBinOp::IsNot => "!=",
        HirBinOp::InRange => "intra",
        HirBinOp::Between => "inter",
    }
}

fn unop_to_rust(op: HirUnOp) -> &'static str {
    match op {
        HirUnOp::Neg => "-",
        HirUnOp::Not => "!",
        HirUnOp::BitNot => "~",
        HirUnOp::IsNull => "nulla ",
        HirUnOp::IsNotNull => "nonnulla ",
        HirUnOp::IsNil => "nihil ",
        HirUnOp::IsNotNil => "nonnihil ",
        HirUnOp::IsNeg => "negativum ",
        HirUnOp::IsPos => "positivum ",
        HirUnOp::IsTrue => "verum ",
        HirUnOp::IsFalse => "falsum ",
    }
}

fn rust_format_template(template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    for ch in template.chars() {
        match ch {
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
            '§' => out.push_str("{}"),
            _ => out.push(ch),
        }
    }
    out
}

fn write_innatum_map_key(
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
