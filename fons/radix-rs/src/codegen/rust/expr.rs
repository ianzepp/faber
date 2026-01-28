//! Rust expression generation

use super::super::CodeWriter;
use super::types::type_to_rust;
use super::CodegenError;
use crate::hir::*;
use crate::semantic::TypeTable;

pub fn generate_expr(
    expr: &HirExpr,
    types: &TypeTable,
    w: &mut CodeWriter,
) -> Result<(), CodegenError> {
    match &expr.kind {
        HirExprKind::Path(_def_id) => {
            // TODO: Resolve path to name
            w.write("todo_var");
        }
        HirExprKind::Literal(lit) => {
            generate_literal(lit, w);
        }
        HirExprKind::Binary(op, lhs, rhs) => {
            w.write("(");
            generate_expr(lhs, types, w)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write(" ");
            generate_expr(rhs, types, w)?;
            w.write(")");
        }
        HirExprKind::Unary(op, operand) => {
            w.write(unop_to_rust(*op));
            generate_expr(operand, types, w)?;
        }
        HirExprKind::Call(callee, args) => {
            generate_expr(callee, types, w)?;
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            generate_expr(receiver, types, w)?;
            w.write(".");
            // TODO: Write method name
            w.write("method");
            w.write("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(arg, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Field(obj, _field) => {
            generate_expr(obj, types, w)?;
            w.write(".");
            // TODO: Write field name
            w.write("field");
        }
        HirExprKind::Index(obj, idx) => {
            generate_expr(obj, types, w)?;
            w.write("[");
            generate_expr(idx, types, w)?;
            w.write("]");
        }
        HirExprKind::Block(block) => {
            w.writeln("{");
            w.indented(|w| {
                for stmt in &block.stmts {
                    let _ = super::stmt::generate_stmt(stmt, types, w);
                }
                if let Some(expr) = &block.expr {
                    let _ = generate_expr(expr, types, w);
                }
            });
            w.write("}");
        }
        HirExprKind::If(cond, then, else_) => {
            w.write("if ");
            generate_expr(cond, types, w)?;
            w.write(" ");
            generate_block(then, types, w)?;
            if let Some(else_block) = else_ {
                w.write(" else ");
                generate_block(else_block, types, w)?;
            }
        }
        HirExprKind::Match(scrutinee, arms) => {
            w.write("match ");
            generate_expr(scrutinee, types, w)?;
            w.writeln(" {");
            w.indented(|w| {
                for arm in arms {
                    generate_pattern(&arm.pattern, w);
                    if let Some(guard) = &arm.guard {
                        w.write(" if ");
                        let _ = generate_expr(guard, types, w);
                    }
                    w.write(" => ");
                    let _ = generate_expr(&arm.body, types, w);
                    w.writeln(",");
                }
            });
            w.write("}");
        }
        HirExprKind::Loop(block) => {
            w.write("loop ");
            generate_block(block, types, w)?;
        }
        HirExprKind::While(cond, block) => {
            w.write("while ");
            generate_expr(cond, types, w)?;
            w.write(" ");
            generate_block(block, types, w)?;
        }
        HirExprKind::For(_binding, iter, block) => {
            w.write("for var in ");
            generate_expr(iter, types, w)?;
            w.write(" ");
            generate_block(block, types, w)?;
        }
        HirExprKind::Assign(target, value) => {
            generate_expr(target, types, w)?;
            w.write(" = ");
            generate_expr(value, types, w)?;
        }
        HirExprKind::AssignOp(op, target, value) => {
            generate_expr(target, types, w)?;
            w.write(" ");
            w.write(binop_to_rust(*op));
            w.write("= ");
            generate_expr(value, types, w)?;
        }
        HirExprKind::Array(elements) => {
            w.write("vec![");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                generate_expr(elem, types, w)?;
            }
            w.write("]");
        }
        HirExprKind::Struct(_def_id, fields) => {
            // TODO: Write struct name
            w.write("Struct");
            w.writeln(" {");
            w.indented(|w| {
                for (_name, value) in fields {
                    // TODO: Write field name
                    w.write("field: ");
                    let _ = generate_expr(value, types, w);
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
                generate_expr(elem, types, w)?;
            }
            w.write(")");
        }
        HirExprKind::Closure(params, _ret, body) => {
            w.write("|");
            for (i, _param) in params.iter().enumerate() {
                if i > 0 {
                    w.write(", ");
                }
                // TODO: Write param
                w.write("p");
            }
            w.write("| ");
            generate_expr(body, types, w)?;
        }
        HirExprKind::Await(expr) => {
            generate_expr(expr, types, w)?;
            w.write(".await");
        }
        HirExprKind::Cast(expr, ty) => {
            generate_expr(expr, types, w)?;
            w.write(" as ");
            w.write(&type_to_rust(*ty, types));
        }
        HirExprKind::Ref(kind, expr) => {
            match kind {
                HirRefKind::Shared => w.write("&"),
                HirRefKind::Mutable => w.write("&mut "),
            }
            generate_expr(expr, types, w)?;
        }
        HirExprKind::Deref(expr) => {
            w.write("*");
            generate_expr(expr, types, w)?;
        }
        HirExprKind::Error => {
            w.write("todo!(\"error\")");
        }
    }
    Ok(())
}

fn generate_literal(lit: &HirLiteral, w: &mut CodeWriter) {
    match lit {
        HirLiteral::Int(n) => {
            w.write(&n.to_string());
        }
        HirLiteral::Float(f) => {
            w.write(&f.to_string());
        }
        HirLiteral::String(_s) => {
            // TODO: Escape string
            w.write("\"todo\"");
        }
        HirLiteral::Bool(b) => {
            w.write(if *b { "true" } else { "false" });
        }
        HirLiteral::Nil => {
            w.write("None");
        }
    }
}

fn generate_pattern(pattern: &HirPattern, w: &mut CodeWriter) {
    match pattern {
        HirPattern::Wildcard => {
            w.write("_");
        }
        HirPattern::Binding(_def_id, _name) => {
            // TODO: Write binding name
            w.write("var");
        }
        HirPattern::Variant(_def_id, fields) => {
            // TODO: Write variant name
            w.write("Variant");
            if !fields.is_empty() {
                w.write(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        w.write(", ");
                    }
                    generate_pattern(field, w);
                }
                w.write(" }");
            }
        }
        HirPattern::Literal(lit) => {
            generate_literal(lit, w);
        }
    }
}

fn generate_block(block: &HirBlock, types: &TypeTable, w: &mut CodeWriter) -> Result<(), CodegenError> {
    w.writeln("{");
    w.indented(|w| {
        for stmt in &block.stmts {
            let _ = super::stmt::generate_stmt(stmt, types, w);
        }
        if let Some(expr) = &block.expr {
            let _ = generate_expr(expr, types, w);
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
