//! Read-only HIR visitor trait and walking functions.
//!
//! WHY: Most post-lowering passes need the same recursive traversal over HIR.
//! Keeping that traversal in one place reduces duplicated `match Hir*Kind`
//! walkers and makes small analyses easier to test.

use super::nodes::*;

pub trait HirVisitor: Sized {
    fn visit_program(&mut self, program: &HirProgram) {
        walk_program(self, program);
    }

    fn visit_item(&mut self, item: &HirItem) {
        walk_item(self, item);
    }

    fn visit_function(&mut self, function: &HirFunction) {
        walk_function(self, function);
    }

    fn visit_type_param(&mut self, type_param: &HirTypeParam) {
        self.visit_def(type_param.def_id, type_param.name);
    }

    fn visit_param(&mut self, param: &HirParam) {
        self.visit_def(param.def_id, param.name);
    }

    fn visit_field(&mut self, field: &HirField) {
        walk_field(self, field);
    }

    fn visit_method(&mut self, method: &HirMethod) {
        walk_method(self, method);
    }

    fn visit_variant(&mut self, variant: &HirVariant) {
        self.visit_def(variant.def_id, variant.name);
    }

    fn visit_import_item(&mut self, item: &HirImportItem) {
        self.visit_def(item.def_id, item.alias.unwrap_or(item.name));
    }

    fn visit_block(&mut self, block: &HirBlock) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &HirStmt) {
        walk_stmt(self, stmt);
    }

    fn visit_local(&mut self, local: &HirLocal) {
        walk_local(self, local);
    }

    fn visit_ad(&mut self, ad: &HirAd) {
        walk_ad(self, ad);
    }

    fn visit_expr(&mut self, expr: &HirExpr) {
        walk_expr(self, expr);
    }

    fn visit_cape(&mut self, cape: &HirCape) {
        walk_cape(self, cape);
    }

    fn visit_casu_arm(&mut self, arm: &HirCasuArm) {
        walk_casu_arm(self, arm);
    }

    fn visit_pattern(&mut self, pattern: &HirPattern) {
        walk_pattern(self, pattern);
    }

    fn visit_object_field(&mut self, field: &HirObjectField) {
        walk_object_field(self, field);
    }

    fn visit_def(&mut self, _def_id: DefId, _name: crate::lexer::Symbol) {}
}

pub fn walk_program<V: HirVisitor>(visitor: &mut V, program: &HirProgram) {
    for item in &program.items {
        visitor.visit_item(item);
    }
    if let Some(entry) = &program.entry {
        visitor.visit_block(entry);
    }
}

pub fn walk_item<V: HirVisitor>(visitor: &mut V, item: &HirItem) {
    match &item.kind {
        HirItemKind::Function(function) => {
            visitor.visit_def(item.def_id, function.name);
            visitor.visit_function(function);
        }
        HirItemKind::Struct(strukt) => {
            visitor.visit_def(item.def_id, strukt.name);
            for type_param in &strukt.type_params {
                visitor.visit_type_param(type_param);
            }
            for field in &strukt.fields {
                visitor.visit_field(field);
            }
            for method in &strukt.methods {
                visitor.visit_method(method);
            }
        }
        HirItemKind::Enum(enum_item) => {
            visitor.visit_def(item.def_id, enum_item.name);
            for type_param in &enum_item.type_params {
                visitor.visit_type_param(type_param);
            }
            for variant in &enum_item.variants {
                visitor.visit_variant(variant);
            }
        }
        HirItemKind::Interface(interface) => {
            visitor.visit_def(item.def_id, interface.name);
            for type_param in &interface.type_params {
                visitor.visit_type_param(type_param);
            }
            for method in &interface.methods {
                for param in &method.params {
                    visitor.visit_param(param);
                }
            }
        }
        HirItemKind::TypeAlias(alias) => {
            visitor.visit_def(item.def_id, alias.name);
        }
        HirItemKind::Const(const_item) => {
            visitor.visit_def(item.def_id, const_item.name);
            visitor.visit_expr(&const_item.value);
        }
        HirItemKind::Import(import) => {
            for item in &import.items {
                visitor.visit_import_item(item);
            }
        }
    }
}

pub fn walk_function<V: HirVisitor>(visitor: &mut V, function: &HirFunction) {
    for type_param in &function.type_params {
        visitor.visit_type_param(type_param);
    }
    for param in &function.params {
        visitor.visit_param(param);
    }
    if let Some(param) = &function.cli_args {
        visitor.visit_param(param);
    }
    if let Some(body) = &function.body {
        visitor.visit_block(body);
    }
}

pub fn walk_field<V: HirVisitor>(visitor: &mut V, field: &HirField) {
    visitor.visit_def(field.def_id, field.name);
    if let Some(init) = &field.init {
        visitor.visit_expr(init);
    }
}

pub fn walk_method<V: HirVisitor>(visitor: &mut V, method: &HirMethod) {
    visitor.visit_def(method.def_id, method.func.name);
    visitor.visit_function(&method.func);
}

pub fn walk_block<V: HirVisitor>(visitor: &mut V, block: &HirBlock) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
    if let Some(expr) = &block.expr {
        visitor.visit_expr(expr);
    }
}

pub fn walk_stmt<V: HirVisitor>(visitor: &mut V, stmt: &HirStmt) {
    match &stmt.kind {
        HirStmtKind::Local(local) => visitor.visit_local(local),
        HirStmtKind::Expr(expr) => visitor.visit_expr(expr),
        HirStmtKind::Ad(ad) => visitor.visit_ad(ad),
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                visitor.visit_expr(expr);
            }
        }
        HirStmtKind::Rumpe | HirStmtKind::Perge | HirStmtKind::Tacet => {}
    }
}

pub fn walk_local<V: HirVisitor>(visitor: &mut V, local: &HirLocal) {
    visitor.visit_def(local.def_id, local.name);
    if let Some(init) = &local.init {
        visitor.visit_expr(init);
    }
}

pub fn walk_ad<V: HirVisitor>(visitor: &mut V, ad: &HirAd) {
    for arg in &ad.args {
        visitor.visit_expr(arg);
    }
    if let Some(body) = &ad.body {
        visitor.visit_block(body);
    }
    if let Some(catch) = &ad.catch {
        visitor.visit_block(catch);
    }
}

pub fn walk_expr<V: HirVisitor>(visitor: &mut V, expr: &HirExpr) {
    match &expr.kind {
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Vacua | HirExprKind::Error => {}
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        HirExprKind::Unary(_, operand)
        | HirExprKind::Cede(operand)
        | HirExprKind::Ref(_, operand)
        | HirExprKind::Deref(operand)
        | HirExprKind::Panic(operand)
        | HirExprKind::Throw(operand) => visitor.visit_expr(operand),
        HirExprKind::Call(callee, args) => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        HirExprKind::MethodCall(receiver, _, args) => {
            visitor.visit_expr(receiver);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        HirExprKind::Field(object, _) => visitor.visit_expr(object),
        HirExprKind::Index(object, index) => {
            visitor.visit_expr(object);
            visitor.visit_expr(index);
        }
        HirExprKind::OptionalChain(object, chain) => {
            visitor.visit_expr(object);
            walk_optional_chain(visitor, chain);
        }
        HirExprKind::NonNull(object, chain) => {
            visitor.visit_expr(object);
            walk_non_null(visitor, chain);
        }
        HirExprKind::Ab { source, filter, transforms } => {
            visitor.visit_expr(source);
            if let Some(filter) = filter {
                walk_collection_filter(visitor, filter);
            }
            for transform in transforms {
                if let Some(arg) = &transform.arg {
                    visitor.visit_expr(arg);
                }
            }
        }
        HirExprKind::Block(block) | HirExprKind::Loop(block) => visitor.visit_block(block),
        HirExprKind::Si { cond, then_block, then_catch, else_block } => {
            visitor.visit_expr(cond);
            visitor.visit_block(then_block);
            if let Some(catch) = then_catch {
                visitor.visit_cape(catch);
            }
            if let Some(else_block) = else_block {
                visitor.visit_block(else_block);
            }
        }
        HirExprKind::Discerne(scrutinees, arms) => {
            for scrutinee in scrutinees {
                visitor.visit_expr(scrutinee);
            }
            for arm in arms {
                visitor.visit_casu_arm(arm);
            }
        }
        HirExprKind::Dum(cond, block) => {
            visitor.visit_expr(cond);
            visitor.visit_block(block);
        }
        HirExprKind::Itera(_, def_id, name, iter, block) => {
            visitor.visit_def(*def_id, *name);
            visitor.visit_expr(iter);
            visitor.visit_block(block);
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            visitor.visit_expr(start);
            visitor.visit_expr(end);
            if let Some(step) = step {
                visitor.visit_expr(step);
            }
        }
        HirExprKind::Array(elements) => {
            for element in elements {
                match element {
                    HirArrayElement::Expr(expr) | HirArrayElement::Spread(expr) => visitor.visit_expr(expr),
                }
            }
        }
        HirExprKind::Struct(_, fields) => {
            for (_, value) in fields {
                visitor.visit_expr(value);
            }
        }
        HirExprKind::Tuple(elements) | HirExprKind::Scribe(_, elements) | HirExprKind::Scriptum(_, elements) => {
            for element in elements {
                visitor.visit_expr(element);
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            visitor.visit_expr(cond);
            if let Some(message) = message {
                visitor.visit_expr(message);
            }
        }
        HirExprKind::Handled { body, catch } => {
            visitor.visit_block(body);
            visitor.visit_cape(catch);
        }
        HirExprKind::Tempta { body, catch, finally } => {
            visitor.visit_block(body);
            if let Some(catch) = catch {
                visitor.visit_block(catch);
            }
            if let Some(finally) = finally {
                visitor.visit_block(finally);
            }
        }
        HirExprKind::Clausura(params, _, body) => {
            for param in params {
                visitor.visit_param(param);
            }
            visitor.visit_expr(body);
        }
        HirExprKind::Verte { source, entries, .. } => {
            visitor.visit_expr(source);
            if let Some(entries) = entries {
                for field in entries {
                    visitor.visit_object_field(field);
                }
            }
        }
        HirExprKind::Conversio { source, fallback, .. } => {
            visitor.visit_expr(source);
            if let Some(fallback) = fallback {
                visitor.visit_expr(fallback);
            }
        }
    }
}

pub fn walk_cape<V: HirVisitor>(visitor: &mut V, cape: &HirCape) {
    visitor.visit_def(cape.binding_def_id, cape.binding_name);
    visitor.visit_block(&cape.body);
}

pub fn walk_casu_arm<V: HirVisitor>(visitor: &mut V, arm: &HirCasuArm) {
    for pattern in &arm.patterns {
        visitor.visit_pattern(pattern);
    }
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard);
    }
    visitor.visit_expr(&arm.body);
}

pub fn walk_pattern<V: HirVisitor>(visitor: &mut V, pattern: &HirPattern) {
    match pattern {
        HirPattern::Wildcard | HirPattern::Literal(_) => {}
        HirPattern::Binding(def_id, name) => visitor.visit_def(*def_id, *name),
        HirPattern::Alias(def_id, name, pattern) => {
            visitor.visit_def(*def_id, *name);
            visitor.visit_pattern(pattern);
        }
        HirPattern::Variant(_, patterns) => {
            for pattern in patterns {
                visitor.visit_pattern(pattern);
            }
        }
    }
}

pub fn walk_object_field<V: HirVisitor>(visitor: &mut V, field: &HirObjectField) {
    match &field.key {
        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
        HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => visitor.visit_expr(expr),
    }
    if let Some(value) = &field.value {
        visitor.visit_expr(value);
    }
}

pub fn walk_optional_chain<V: HirVisitor>(visitor: &mut V, chain: &HirOptionalChainKind) {
    match chain {
        HirOptionalChainKind::Member(_) => {}
        HirOptionalChainKind::Index(index) => visitor.visit_expr(index),
        HirOptionalChainKind::Call(args) => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
    }
}

pub fn walk_non_null<V: HirVisitor>(visitor: &mut V, chain: &HirNonNullKind) {
    match chain {
        HirNonNullKind::Member(_) => {}
        HirNonNullKind::Index(index) => visitor.visit_expr(index),
        HirNonNullKind::Call(args) => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
    }
}

pub fn walk_collection_filter<V: HirVisitor>(visitor: &mut V, filter: &HirCollectionFilter) {
    match &filter.kind {
        HirCollectionFilterKind::Condition(cond) => visitor.visit_expr(cond),
        HirCollectionFilterKind::Property(_) => {}
    }
}
