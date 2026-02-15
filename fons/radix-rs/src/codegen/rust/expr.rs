//! Rust expression generation

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::{CodegenError, RustCodegen};
use crate::hir::*;
use crate::semantic::TypeTable;

pub fn generate_expr(
    codegen: &RustCodegen<'_>,
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(def_id) => {
            w.write(codegen.resolve_def(*def_id));
        }
        HirExprKind::Literal(lit) => {
            generate_literal(codegen, lit, w);
        }
        HirExprKind::Binary(op, lhs, rhs) => {
            w.write("(");
            generate_expr(codegen, lhs, types, w)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write(" ");
            generate_expr(codegen, rhs, types, w)?;
            w.write(")");
        }
        HirExprKind::Unary(op, operand) => {
            w.write(unop_to_rust(*op));
            generate_expr(codegen, operand, types, w)?;
        }
        HirExprKind::Call(callee, args) => {
            generate_expr(codegen, callee, types, w)?;
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_expr(codegen, receiver, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*method));
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Field(obj, field) => {
            generate_expr(codegen, obj, types, w)?;
            w.write(".");
            w.write(codegen.resolve_symbol(*field));
        }
        HirExprKind::Index(obj, idx) => {
            generate_expr(codegen, obj, types, w)?;
            w.write("[");
            generate_expr(codegen, idx, types, w)?;
            w.write("]");
        }
        HirExprKind::Block(block) => {
            w.writeln("{");
            w.indented(|w| {
                for stmt in &block.stmts {
                    let _ = super::stmt::generate_stmt(codegen, stmt, types, w);
                }
                if let Some(expr) = &block.expr {
                    let _ = generate_expr(codegen, expr, types, w);
                }
            });
            w.write("}");
        }
        HirExprKind::Si(cond, then, else_) => {
            w.write("if ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_block(codegen, then, types, w)?;
            if let Some(else_block) = else_ {
                w.write(" else ");
                generate_block(codegen, else_block, types, w)?;
            }
        }
        HirExprKind::Discerne(scrutinee, arms) => {
            w.write("match ");
            generate_expr(codegen, scrutinee, types, w)?;
            w.writeln(" {");
            w.indented(|w| {
                for arm in arms {
                    generate_pattern(codegen, &arm.pattern, w);
                    if let Some(guard) = &arm.guard {
                        w.write(" if ");
                        let _ = generate_expr(codegen, guard, types, w);
                    }
                    w.write(" => ");
                    let _ = generate_expr(codegen, &arm.body, types, w);
                    w.writeln(",");
                }
            });
            w.write("}");
        }
        HirExprKind::Loop(block) => {
            w.write("loop ");
            generate_block(codegen, block, types, w)?;
        }
        HirExprKind::Dum(cond, block) => {
            w.write("while ");
            generate_expr(codegen, cond, types, w)?;
            w.write(" ");
            generate_block(codegen, block, types, w)?;
        }
        HirExprKind::Itera(binding, iter, block) => {
            w.write("for ");
            w.write(codegen.resolve_def(*binding));
            w.write(" in ");
            generate_expr(codegen, iter, types, w)?;
            w.write(" ");
            generate_block(codegen, block, types, w)?;
        }
        HirExprKind::Assign(target, value) => {
            generate_expr(codegen, target, types, w)?;
            w.write(" = ");
            generate_expr(codegen, value, types, w)?;
        }
        HirExprKind::AssignOp(op, target, value) => {
            generate_expr(codegen, target, types, w)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write("= ");
            generate_expr(codegen, value, types, w)?;
        }
        HirExprKind::Array(elements) => {
            w.write("vec![");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(codegen, elem, types, w)?;
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
                    let _ = generate_expr(codegen, value, types, w);
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
                generate_expr(codegen, elem, types, w)?;
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
                    generate_expr(codegen, arg, types, w)?;
                }
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
            generate_expr(codegen, body, types, w)?;
        }
        HirExprKind::Cede(expr) => {
            generate_expr(codegen, expr, types, w)?;
            w.write(".await");
        }
        HirExprKind::Qua(expr, ty) => {
            generate_expr(codegen, expr, types, w)?;
            w.write(" as ");
            w.write(&type_to_rust(codegen, *ty, types));
        }
        HirExprKind::Ref(kind, expr) => {
            match kind {
                HirRefKind::Shared => w.write("&"),
                HirRefKind::Mutable => w.write("&mut "),
            }
            generate_expr(codegen, expr, types, w)?;
        }
        HirExprKind::Deref(expr) => {
            w.write("*");
            generate_expr(codegen, expr, types, w)?;
        }
        HirExprKind::Error => {
            w.write("todo!(\"error\")");
        }
    }
    Ok(())
}

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
) -> Result<(), CodegenError> {
    w.writeln("{");
    w.indented(|w| {
        for stmt in &block.stmts {
            let _ = super::stmt::generate_stmt(codegen, stmt, types, w);
        }
        if let Some(expr) = &block.expr {
            let _ = generate_expr(codegen, expr, types, w);
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
        HirBinOp::Lt => "<",
        HirBinOp::Gt => ">",
        HirBinOp::LtEq => "<=",
        HirBinOp::GtEq => ">=",
        HirBinOp::And => "&&",
        HirBinOp::Or => "||",
        HirBinOp::BitAnd => "&",
        HirBinOp::BitOr => "|",
        HirBinOp::BitXor => "^",
        HirBinOp::Shl => "<<",
        HirBinOp::Shr => ">>",
    }
}

fn unop_to_rust(op: HirUnOp) -> &'static str {
    match op {
        HirUnOp::Neg => "-",
        HirUnOp::Not => "!",
        HirUnOp::BitNot => "!",
    }
}
