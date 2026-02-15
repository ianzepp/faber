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
use crate::semantic::{Primitive, Type, TypeTable};

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
        }
        HirExprKind::Binary(op, lhs, rhs) => match op {
            HirBinOp::Coalesce => {
                let lhs_ty = lhs.ty.map(|ty| types.get(ty));
                match lhs_ty {
                    Some(Type::Option(_)) => {
                        w.write("(");
                        generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                        w.write(").unwrap_or(");
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
                        w.write("(");
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
                        w.write(")");
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
                w.write("(");
                generate_expr(codegen, lhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" ");
                w.write(binop_to_rust(*op));
                w.write(" ");
                generate_expr(codegen, rhs, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(")");
            }
        },
        HirExprKind::Unary(op, operand) => match op {
            HirUnOp::IsNull | HirUnOp::IsNil => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" == None)");
            }
            HirUnOp::IsNotNull | HirUnOp::IsNotNil => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" != None)");
            }
            HirUnOp::IsNeg => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" < 0)");
            }
            HirUnOp::IsPos => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" > 0)");
            }
            HirUnOp::IsTrue => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" == true)");
            }
            HirUnOp::IsFalse => {
                w.write("(");
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
                w.write(" == false)");
            }
            _ => {
                w.write(unop_to_rust(*op));
                generate_expr(codegen, operand, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            }
        },
        HirExprKind::Call(callee, args) => {
            let is_failable_call =
                matches!(&callee.kind, HirExprKind::Path(def_id) if codegen.is_failable_def(*def_id));
            generate_expr(codegen, callee, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
                generate_expr(codegen, arg, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
        HirExprKind::Block(block) => {
            w.writeln("{");
            w.indented(|w| {
                for stmt in &block.stmts {
                    let _ = super::stmt::generate_stmt(
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
                    let _ =
                        generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                }
            });
            w.write("}");
        }
        HirExprKind::Tempta { body, catch, finally } => {
            w.writeln("{");
            w.indented(|w| {
                for stmt in &body.stmts {
                    let _ = super::stmt::generate_stmt(
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
                    let _ = generate_expr(
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
                        let _ = super::stmt::generate_stmt(
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
                        let _ = generate_expr(
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
                        let _ = super::stmt::generate_stmt(
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
                        let _ = generate_expr(
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
            w.write("}");
        }
        HirExprKind::Si(cond, then, else_) => {
            w.write("if ");
            generate_expr(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
            w.indented(|w| {
                for arm in arms {
                    generate_pattern(codegen, &arm.pattern, w);
                    if let Some(guard) = &arm.guard {
                        w.write(" if ");
                        let _ = generate_expr(
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
                    let _ = generate_expr(
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
            w.write("}");
        }
        HirExprKind::Loop(block) => {
            w.write("loop ");
            generate_block(codegen, block, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::Dum(cond, block) => {
            w.write("while ");
            generate_expr(codegen, cond, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
            generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
        }
        HirExprKind::AssignOp(op, target, value) => {
            generate_expr(codegen, target, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write("= ");
            generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
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
            w.indented(|w| {
                for (name, value) in fields {
                    w.write(codegen.resolve_symbol(*name));
                    w.write(": ");
                    let _ =
                        generate_expr(codegen, value, types, w, in_failable_fn, in_entry, suppress_error_propagation);
                    w.writeln(",");
                }
            });
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
                let format = vec!["{}"; args.len()].join(" ");
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
        HirExprKind::Qua(expr, ty) => {
            generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation)?;
            w.write(" as ");
            w.write(&type_to_rust(codegen, *ty, types));
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
            w.write("todo!(\"error\")");
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
            w.write("\"");
            for ch in codegen.resolve_symbol(*s).chars() {
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
        HirLiteral::Bool(b) => {
            w.write(if *b { "true" } else { "false" });
        }
        HirLiteral::Nil => {
            w.write("None");
        }
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
    w.indented(|w| {
        for stmt in &block.stmts {
            let _ = super::stmt::generate_stmt(
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
            let _ = generate_expr(codegen, expr, types, w, in_failable_fn, in_entry, suppress_error_propagation);
        }
    });
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
