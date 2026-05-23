//! Shared traversal helpers for the Faber AST.
//!
//! Visitors separate "what to do at a node" from the recursive mechanics of
//! descending through syntax. That is useful for light analyses, diagnostics,
//! indexing, and tooling that need source-shaped traversal without owning a
//! full semantic pass.
//!
//! This module is intentionally conservative: the walkers visit the high-value
//! child expressions, statements, identifiers, and type syntax currently needed
//! by generic visitors. A compiler phase that relies on exhaustive semantics
//! should audit the relevant AST variants instead of assuming these helpers
//! encode every language rule.
//!
//! INVARIANTS
//! ==========
//! - Visitor callbacks observe AST nodes in source order where the walker covers
//!   that node family.
//! - Calling a `walk_*` function continues traversal after custom visitor logic.
//! - Traversal has no semantic side effects beyond what the visitor performs.

use super::ast::*;

// =============================================================================
// VISITOR TRAIT
// =============================================================================

/// Callback surface for source-shaped AST traversal.
///
/// Implementers override the callbacks they care about and usually call the
/// matching `walk_*` helper to keep descending. Omitting the helper is a valid
/// way to prune traversal for a subtree.
///
/// USAGE:
/// ```ignore
/// struct TypeChecker { ... }
/// impl Visitor for TypeChecker {
///     fn visit_expr(&mut self, expr: &Expr) {
///         // Custom logic here
///         walk_expr(self, expr); // Continue traversal
///     }
/// }
/// ```
pub trait Visitor: Sized {
    /// Visit a complete parsed source unit.
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }

    /// Visit one statement node.
    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    /// Visit one expression node.
    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    /// Visit one written type expression.
    fn visit_type_expr(&mut self, ty: &TypeExpr) {
        walk_type_expr(self, ty);
    }

    /// Visit an identifier occurrence or binding selected by a walker.
    fn visit_ident(&mut self, _ident: &Ident) {}

    /// Visit a statement block.
    fn visit_block(&mut self, block: &BlockStmt) {
        walk_block(self, block);
    }
}

// =============================================================================
// WALKING FUNCTIONS
// =============================================================================
//
// These functions are public so visitors can perform custom logic at a node and
// then explicitly resume the shared traversal policy.

/// Continue traversal through a parsed source unit.
pub fn walk_program<V: Visitor>(visitor: &mut V, program: &Program) {
    for stmt in &program.stmts {
        visitor.visit_stmt(stmt);
    }
}

/// Continue traversal through statement children covered by the generic walker.
///
/// NOTE: This walker is useful for source-shaped analyses, but it intentionally
/// does not claim semantic exhaustiveness for every statement variant.
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
        StmtKind::Tacet(_) => {}
        // NOTE: Add other statement kinds as generic visitors need them.
        _ => {}
    }
}

/// Continue traversal through expression children covered by the generic walker.
///
/// NOTE: Some expression variants are intentionally left to specialized passes
/// until a generic visitor needs a stable traversal contract for them.
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
            if let Some(err) = &closure.err {
                visitor.visit_type_expr(err);
            }
            match &closure.body {
                ClausuraBody::Expr(e) => visitor.visit_expr(e),
                ClausuraBody::Block(b) => visitor.visit_block(b),
                ClausuraBody::Fac(f) => {
                    visitor.visit_block(&f.body);
                    if let Some(catch) = &f.catch {
                        visitor.visit_block(&catch.body);
                    }
                    if let Some(while_) = &f.while_ {
                        visitor.visit_expr(while_);
                    }
                }
            }
        }
        ExprKind::Cede(cede) => {
            visitor.visit_expr(&cede.expr);
        }
        ExprKind::Verte(verte) => {
            visitor.visit_expr(&verte.expr);
            visitor.visit_type_expr(&verte.ty);
        }
        ExprKind::Conversio(conversio) => {
            visitor.visit_expr(&conversio.expr);
            let ConversioTarget::Explicit(ty) = &conversio.target;
            visitor.visit_type_expr(ty);
            for ty in &conversio.type_params {
                visitor.visit_type_expr(ty);
            }
            if let Some(fallback) = &conversio.fallback {
                visitor.visit_expr(fallback);
            }
        }
        ExprKind::Paren(inner) => {
            visitor.visit_expr(inner);
        }
        // NOTE: Add other expression kinds as generic visitors need them.
        _ => {}
    }
}

/// Walk a binding pattern, visiting all nested identifiers.
///
/// WHY: Destructuring patterns can contain nested patterns and rest elements.
/// This function recursively visits all identifiers bound by a pattern.
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
        BindingPattern::Object { fields, rest, .. } => {
            for field in fields {
                visitor.visit_ident(field.alias.as_ref().unwrap_or(&field.name));
            }
            if let Some(rest) = rest {
                visitor.visit_ident(rest);
            }
        }
    }
}

/// Continue traversal through written type syntax.
pub fn walk_type_expr<V: Visitor>(visitor: &mut V, ty: &TypeExpr) {
    match &ty.kind {
        TypeExprKind::Infer => {}
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
        TypeExprKind::Union(members) => {
            for m in members {
                visitor.visit_type_expr(m);
            }
        }
    }
}

/// Continue traversal through statements in a block.
pub fn walk_block<V: Visitor>(visitor: &mut V, block: &BlockStmt) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
}

/// Walk an if-statement body (which can be a block or single statement).
///
/// WHY: Faber allows `si cond ergo stmt` as a shorthand. This helper handles
/// both body styles uniformly.
fn walk_if_body<V: Visitor>(visitor: &mut V, body: &IfBody) {
    match body {
        IfBody::Block(block) => visitor.visit_block(block),
        IfBody::Ergo(stmt) => visitor.visit_stmt(stmt),
    }
}
