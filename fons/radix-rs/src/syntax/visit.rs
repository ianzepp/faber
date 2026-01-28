//! AST visitor trait and walking functions

use super::ast::*;

/// Visitor trait for traversing the AST
pub trait Visitor: Sized {
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_type_expr(&mut self, ty: &TypeExpr) {
        walk_type_expr(self, ty);
    }

    fn visit_ident(&mut self, _ident: &Ident) {}

    fn visit_block(&mut self, block: &BlockStmt) {
        walk_block(self, block);
    }
}

pub fn walk_program<V: Visitor>(visitor: &mut V, program: &Program) {
    for stmt in &program.stmts {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor>(visitor: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Var(decl) => {
            if let Some(ty) = &decl.ty {
                visitor.visit_type_expr(ty);
            }
            walk_binding_pattern(visitor, &decl.binding);
            if let Some(init) = &decl.init {
                visitor.visit_expr(init);
            }
        }
        StmtKind::Func(decl) => {
            visitor.visit_ident(&decl.name);
            for param in &decl.params {
                visitor.visit_type_expr(&param.ty);
                visitor.visit_ident(&param.name);
            }
            if let Some(ret) = &decl.ret {
                visitor.visit_type_expr(ret);
            }
            if let Some(body) = &decl.body {
                visitor.visit_block(body);
            }
        }
        StmtKind::Class(decl) => {
            visitor.visit_ident(&decl.name);
            for member in &decl.members {
                match &member.kind {
                    ClassMemberKind::Field(field) => {
                        visitor.visit_type_expr(&field.ty);
                        visitor.visit_ident(&field.name);
                        if let Some(init) = &field.init {
                            visitor.visit_expr(init);
                        }
                    }
                    ClassMemberKind::Method(method) => {
                        visitor.visit_ident(&method.name);
                        if let Some(body) = &method.body {
                            visitor.visit_block(body);
                        }
                    }
                }
            }
        }
        StmtKind::Block(block) => {
            visitor.visit_block(block);
        }
        StmtKind::Expr(expr_stmt) => {
            visitor.visit_expr(&expr_stmt.expr);
        }
        StmtKind::Si(if_stmt) => {
            visitor.visit_expr(&if_stmt.cond);
            walk_if_body(visitor, &if_stmt.then);
        }
        StmtKind::Dum(while_stmt) => {
            visitor.visit_expr(&while_stmt.cond);
            walk_if_body(visitor, &while_stmt.body);
        }
        StmtKind::Itera(iter_stmt) => {
            visitor.visit_expr(&iter_stmt.iterable);
            visitor.visit_ident(&iter_stmt.binding);
            walk_if_body(visitor, &iter_stmt.body);
        }
        StmtKind::Ex(extract_stmt) => {
            visitor.visit_expr(&extract_stmt.source);
            for field in &extract_stmt.fields {
                visitor.visit_ident(&field.name);
                if let Some(alias) = &field.alias {
                    visitor.visit_ident(alias);
                }
            }
            if let Some(rest) = &extract_stmt.rest {
                visitor.visit_ident(rest);
            }
        }
        StmtKind::Redde(ret) => {
            if let Some(value) = &ret.value {
                visitor.visit_expr(value);
            }
        }
        StmtKind::Proba(test) => {
            visitor.visit_block(&test.body);
        }
        StmtKind::Iace(throw) => {
            visitor.visit_expr(&throw.value);
        }
        StmtKind::Mori(panic) => {
            visitor.visit_expr(&panic.value);
        }
        // Add other statement kinds as needed
        _ => {}
    }
}

pub fn walk_expr<V: Visitor>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Ident(ident) => {
            visitor.visit_ident(ident);
        }
        ExprKind::Binary(bin) => {
            visitor.visit_expr(&bin.lhs);
            visitor.visit_expr(&bin.rhs);
        }
        ExprKind::Unary(un) => {
            visitor.visit_expr(&un.operand);
        }
        ExprKind::Ternary(tern) => {
            visitor.visit_expr(&tern.cond);
            visitor.visit_expr(&tern.then);
            visitor.visit_expr(&tern.else_);
        }
        ExprKind::Call(call) => {
            visitor.visit_expr(&call.callee);
            for arg in &call.args {
                visitor.visit_expr(&arg.value);
            }
        }
        ExprKind::Member(member) => {
            visitor.visit_expr(&member.object);
            visitor.visit_ident(&member.member);
        }
        ExprKind::Index(index) => {
            visitor.visit_expr(&index.object);
            visitor.visit_expr(&index.index);
        }
        ExprKind::Assign(assign) => {
            visitor.visit_expr(&assign.target);
            visitor.visit_expr(&assign.value);
        }
        ExprKind::Array(arr) => {
            for elem in &arr.elements {
                match elem {
                    ArrayElement::Expr(e) | ArrayElement::Spread(e) => {
                        visitor.visit_expr(e);
                    }
                }
            }
        }
        ExprKind::Object(obj) => {
            for field in &obj.fields {
                if let Some(value) = &field.value {
                    visitor.visit_expr(value);
                }
            }
        }
        ExprKind::Clausura(closure) => {
            for param in &closure.params {
                visitor.visit_type_expr(&param.ty);
            }
            if let Some(ret) = &closure.ret {
                visitor.visit_type_expr(ret);
            }
            match &closure.body {
                ClausuraBody::Expr(e) => visitor.visit_expr(e),
                ClausuraBody::Block(b) => visitor.visit_block(b),
            }
        }
        ExprKind::Cede(cede) => {
            visitor.visit_expr(&cede.expr);
        }
        ExprKind::Qua(qua) => {
            visitor.visit_expr(&qua.expr);
            visitor.visit_type_expr(&qua.ty);
        }
        ExprKind::Paren(inner) => {
            visitor.visit_expr(inner);
        }
        // Add other expression kinds as needed
        _ => {}
    }
}

fn walk_binding_pattern<V: Visitor>(visitor: &mut V, pattern: &BindingPattern) {
    match pattern {
        BindingPattern::Ident(ident) => visitor.visit_ident(ident),
        BindingPattern::Wildcard(_) => {}
        BindingPattern::Array { elements, rest, .. } => {
            for element in elements {
                walk_binding_pattern(visitor, element);
            }
            if let Some(rest) = rest {
                visitor.visit_ident(rest);
            }
        }
    }
}

pub fn walk_type_expr<V: Visitor>(visitor: &mut V, ty: &TypeExpr) {
    match &ty.kind {
        TypeExprKind::Named(name, params) => {
            visitor.visit_ident(name);
            for param in params {
                visitor.visit_type_expr(param);
            }
        }
        TypeExprKind::Array(inner) => {
            visitor.visit_type_expr(inner);
        }
        TypeExprKind::Func(func) => {
            for param in &func.params {
                visitor.visit_type_expr(param);
            }
            visitor.visit_type_expr(&func.ret);
        }
    }
}

pub fn walk_block<V: Visitor>(visitor: &mut V, block: &BlockStmt) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
}

fn walk_if_body<V: Visitor>(visitor: &mut V, body: &IfBody) {
    match body {
        IfBody::Block(block) => visitor.visit_block(block),
        IfBody::Ergo(stmt) => visitor.visit_stmt(stmt),
        IfBody::InlineReturn(ret) => match ret {
            InlineReturn::Reddit(e) | InlineReturn::Iacit(e) | InlineReturn::Moritor(e) => {
                visitor.visit_expr(e);
            }
            InlineReturn::Tacet => {}
        },
    }
}
