//! Traversal contracts for HIR analyses and tree rewrites.
//!
//! This module centralizes the recursive shape of HIR so analysis passes do not
//! each re-encode which child nodes belong to which construct. The default
//! visitor methods are pre-order at the semantic boundary: declaration hooks
//! fire when a definition is encountered, then walking proceeds into the
//! lowered children in source/evaluation order where that order is meaningful.
//!
//! INVARIANTS
//! ==========
//! - Walkers visit HIR children, not type arena contents or symbol interner
//!   data. `TypeId` and `Symbol` values are references into other compiler
//!   state, not subtrees.
//! - `visit_def` is called for definitions introduced by items, parameters,
//!   locals, patterns, imports, methods, fields, handlers, and loop bindings.
//! - Read-only and mutable walkers should preserve the same traversal shape.
//! - Walkers do not perform semantic validation; they only expose traversal
//!   order for passes that choose to validate, collect, or mutate.
//!
//! TRADE-OFFS
//! ==========
//! The walkers are intentionally explicit `match` statements instead of a macro
//! or generated visitor. HIR changes are compiler phase changes, and reviewing
//! traversal updates beside new variants is more valuable than hiding the shape
//! behind abstraction.

use super::nodes::*;

/// Read-only visitor over HIR.
///
/// Override the narrowest hook needed by the analysis. Default implementations
/// recursively walk children, so a pass can observe only expressions, only
/// definitions, or only a specific declaration kind without duplicating the
/// rest of the tree traversal.
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
        if let Some(default) = &param.default {
            self.visit_expr(default);
        }
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

    /// Observe a definition introduced by the current subtree.
    ///
    /// This hook is intentionally lighter than visiting a declaration node:
    /// locals, pattern bindings, handler bindings, imported aliases, and loop
    /// variables also introduce `DefId` values and should be visible to passes
    /// that build definition indexes.
    fn visit_def(&mut self, _def_id: DefId, _name: crate::lexer::Symbol) {}
}

/// Walk a whole HIR program in item order, then entry-block order.
pub fn walk_program<V: HirVisitor>(visitor: &mut V, program: &HirProgram) {
    for item in &program.items {
        visitor.visit_item(item);
    }
    if let Some(entry) = &program.entry {
        visitor.visit_block(entry);
    }
}

/// Walk a top-level item and any definitions it introduces.
///
/// Item-level definitions are reported before child signatures/bodies so
/// definition collectors can see the outer declaration before nested bindings.
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

/// Walk a function-like body, including generic parameters and CLI argument metadata.
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

/// Walk a field declaration and optional initializer.
pub fn walk_field<V: HirVisitor>(visitor: &mut V, field: &HirField) {
    visitor.visit_def(field.def_id, field.name);
    if let Some(init) = &field.init {
        visitor.visit_expr(init);
    }
}

/// Walk a method as both a method definition and a function payload.
pub fn walk_method<V: HirVisitor>(visitor: &mut V, method: &HirMethod) {
    visitor.visit_def(method.def_id, method.func.name);
    visitor.visit_function(&method.func);
}

/// Walk a block in execution order, with the tail expression last.
pub fn walk_block<V: HirVisitor>(visitor: &mut V, block: &HirBlock) {
    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }
    if let Some(expr) = &block.expr {
        visitor.visit_expr(expr);
    }
}

/// Walk the child subtree of one statement.
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

/// Walk a local binding and initializer.
pub fn walk_local<V: HirVisitor>(visitor: &mut V, local: &HirLocal) {
    visitor.visit_def(local.def_id, local.name);
    if let Some(init) = &local.init {
        visitor.visit_expr(init);
    }
}

/// Walk an endpoint form's arguments and body/handler blocks.
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

/// Walk an expression's child expressions, blocks, patterns, and introduced bindings.
///
/// Leaf values such as resolved paths, literals, and type references are not
/// walked because they point into semantic side tables rather than owning HIR
/// subtrees.
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
                visitor.visit_expr(&arg.expr);
            }
        }
        HirExprKind::MethodCall(receiver, _, args) => {
            visitor.visit_expr(receiver);
            for arg in args {
                visitor.visit_expr(&arg.expr);
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

/// Walk a recoverable handler binding and body.
pub fn walk_cape<V: HirVisitor>(visitor: &mut V, cape: &HirCape) {
    visitor.visit_def(cape.binding_def_id, cape.binding_name);
    visitor.visit_block(&cape.body);
}

/// Walk one pattern arm: patterns first, then guard, then body.
pub fn walk_casu_arm<V: HirVisitor>(visitor: &mut V, arm: &HirCasuArm) {
    for pattern in &arm.patterns {
        visitor.visit_pattern(pattern);
    }
    if let Some(guard) = &arm.guard {
        visitor.visit_expr(guard);
    }
    visitor.visit_expr(&arm.body);
}

/// Walk bindings introduced by a pattern.
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

/// Walk expressions owned by an object field key or value.
pub fn walk_object_field<V: HirVisitor>(visitor: &mut V, field: &HirObjectField) {
    match &field.key {
        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
        HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => visitor.visit_expr(expr),
    }
    if let Some(value) = &field.value {
        visitor.visit_expr(value);
    }
}

/// Walk expression children of an optional-chain operation.
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

/// Walk expression children of a non-null assertion operation.
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

/// Mutable visitor over HIR.
///
/// This mirrors [`HirVisitor`] but grants mutable access to nodes that own HIR
/// children. Use it for normalization and repair passes that preserve node
/// identity and tree shape contracts; rebuilding IDs or redefining semantic
/// ownership belongs in lowering or a dedicated transformation phase.
pub trait HirVisitorMut: Sized {
    fn visit_program_mut(&mut self, program: &mut HirProgram) {
        walk_program_mut(self, program);
    }

    fn visit_item_mut(&mut self, item: &mut HirItem) {
        walk_item_mut(self, item);
    }

    fn visit_function_mut(&mut self, function: &mut HirFunction) {
        walk_function_mut(self, function);
    }

    fn visit_type_param_mut(&mut self, type_param: &mut HirTypeParam) {
        self.visit_def(type_param.def_id, type_param.name);
    }

    fn visit_param_mut(&mut self, param: &mut HirParam) {
        self.visit_def(param.def_id, param.name);
        if let Some(default) = &mut param.default {
            self.visit_expr_mut(default);
        }
    }

    fn visit_field_mut(&mut self, field: &mut HirField) {
        walk_field_mut(self, field);
    }

    fn visit_method_mut(&mut self, method: &mut HirMethod) {
        walk_method_mut(self, method);
    }

    fn visit_variant_mut(&mut self, variant: &mut HirVariant) {
        self.visit_def(variant.def_id, variant.name);
    }

    fn visit_import_item_mut(&mut self, item: &mut HirImportItem) {
        self.visit_def(item.def_id, item.alias.unwrap_or(item.name));
    }

    fn visit_block_mut(&mut self, block: &mut HirBlock) {
        walk_block_mut(self, block);
    }

    fn visit_stmt_mut(&mut self, stmt: &mut HirStmt) {
        walk_stmt_mut(self, stmt);
    }

    fn visit_local_mut(&mut self, local: &mut HirLocal) {
        walk_local_mut(self, local);
    }

    fn visit_ad_mut(&mut self, ad: &mut HirAd) {
        walk_ad_mut(self, ad);
    }

    fn visit_expr_mut(&mut self, expr: &mut HirExpr) {
        walk_expr_mut(self, expr);
    }

    fn visit_cape_mut(&mut self, cape: &mut HirCape) {
        walk_cape_mut(self, cape);
    }

    fn visit_casu_arm_mut(&mut self, arm: &mut HirCasuArm) {
        walk_casu_arm_mut(self, arm);
    }

    fn visit_pattern_mut(&mut self, pattern: &mut HirPattern) {
        walk_pattern_mut(self, pattern);
    }

    fn visit_object_field_mut(&mut self, field: &mut HirObjectField) {
        walk_object_field_mut(self, field);
    }

    /// Observe a definition introduced by the current subtree.
    ///
    /// The hook receives copied identity/name data rather than mutable access
    /// because changing definitions in place would invalidate resolver/typecheck
    /// side tables.
    fn visit_def(&mut self, _def_id: DefId, _name: crate::lexer::Symbol) {}
}

/// Mutably walk a whole HIR program in item order, then entry-block order.
pub fn walk_program_mut<V: HirVisitorMut>(visitor: &mut V, program: &mut HirProgram) {
    for item in &mut program.items {
        visitor.visit_item_mut(item);
    }
    if let Some(entry) = &mut program.entry {
        visitor.visit_block_mut(entry);
    }
}

/// Mutably walk a top-level item and any definitions it introduces.
pub fn walk_item_mut<V: HirVisitorMut>(visitor: &mut V, item: &mut HirItem) {
    match &mut item.kind {
        HirItemKind::Function(function) => {
            visitor.visit_def(item.def_id, function.name);
            visitor.visit_function_mut(function);
        }
        HirItemKind::Struct(strukt) => {
            visitor.visit_def(item.def_id, strukt.name);
            for type_param in &mut strukt.type_params {
                visitor.visit_type_param_mut(type_param);
            }
            for field in &mut strukt.fields {
                visitor.visit_field_mut(field);
            }
            for method in &mut strukt.methods {
                visitor.visit_method_mut(method);
            }
        }
        HirItemKind::Enum(enum_item) => {
            visitor.visit_def(item.def_id, enum_item.name);
            for type_param in &mut enum_item.type_params {
                visitor.visit_type_param_mut(type_param);
            }
            for variant in &mut enum_item.variants {
                visitor.visit_variant_mut(variant);
            }
        }
        HirItemKind::Interface(interface) => {
            visitor.visit_def(item.def_id, interface.name);
            for type_param in &mut interface.type_params {
                visitor.visit_type_param_mut(type_param);
            }
            for method in &mut interface.methods {
                for param in &mut method.params {
                    visitor.visit_param_mut(param);
                }
            }
        }
        HirItemKind::TypeAlias(alias) => {
            visitor.visit_def(item.def_id, alias.name);
        }
        HirItemKind::Const(const_item) => {
            visitor.visit_def(item.def_id, const_item.name);
            visitor.visit_expr_mut(&mut const_item.value);
        }
        HirItemKind::Import(import) => {
            for item in &mut import.items {
                visitor.visit_import_item_mut(item);
            }
        }
    }
}

/// Mutably walk a function-like body.
pub fn walk_function_mut<V: HirVisitorMut>(visitor: &mut V, function: &mut HirFunction) {
    for type_param in &mut function.type_params {
        visitor.visit_type_param_mut(type_param);
    }
    for param in &mut function.params {
        visitor.visit_param_mut(param);
    }
    if let Some(param) = &mut function.cli_args {
        visitor.visit_param_mut(param);
    }
    if let Some(body) = &mut function.body {
        visitor.visit_block_mut(body);
    }
}

/// Mutably walk a field declaration and optional initializer.
pub fn walk_field_mut<V: HirVisitorMut>(visitor: &mut V, field: &mut HirField) {
    visitor.visit_def(field.def_id, field.name);
    if let Some(init) = &mut field.init {
        visitor.visit_expr_mut(init);
    }
}

/// Mutably walk a method as both a method definition and a function payload.
pub fn walk_method_mut<V: HirVisitorMut>(visitor: &mut V, method: &mut HirMethod) {
    visitor.visit_def(method.def_id, method.func.name);
    visitor.visit_function_mut(&mut method.func);
}

/// Mutably walk a block in execution order.
pub fn walk_block_mut<V: HirVisitorMut>(visitor: &mut V, block: &mut HirBlock) {
    for stmt in &mut block.stmts {
        visitor.visit_stmt_mut(stmt);
    }
    if let Some(expr) = &mut block.expr {
        visitor.visit_expr_mut(expr);
    }
}

/// Mutably walk the child subtree of one statement.
pub fn walk_stmt_mut<V: HirVisitorMut>(visitor: &mut V, stmt: &mut HirStmt) {
    match &mut stmt.kind {
        HirStmtKind::Local(local) => visitor.visit_local_mut(local),
        HirStmtKind::Expr(expr) => visitor.visit_expr_mut(expr),
        HirStmtKind::Ad(ad) => visitor.visit_ad_mut(ad),
        HirStmtKind::Redde(value) => {
            if let Some(expr) = value {
                visitor.visit_expr_mut(expr);
            }
        }
        HirStmtKind::Rumpe | HirStmtKind::Perge | HirStmtKind::Tacet => {}
    }
}

/// Mutably walk a local binding and initializer.
pub fn walk_local_mut<V: HirVisitorMut>(visitor: &mut V, local: &mut HirLocal) {
    visitor.visit_def(local.def_id, local.name);
    if let Some(init) = &mut local.init {
        visitor.visit_expr_mut(init);
    }
}

/// Mutably walk an endpoint form's arguments and body/handler blocks.
pub fn walk_ad_mut<V: HirVisitorMut>(visitor: &mut V, ad: &mut HirAd) {
    for arg in &mut ad.args {
        visitor.visit_expr_mut(arg);
    }
    if let Some(body) = &mut ad.body {
        visitor.visit_block_mut(body);
    }
    if let Some(catch) = &mut ad.catch {
        visitor.visit_block_mut(catch);
    }
}

/// Mutably walk an expression's child expressions, blocks, patterns, and bindings.
pub fn walk_expr_mut<V: HirVisitorMut>(visitor: &mut V, expr: &mut HirExpr) {
    match &mut expr.kind {
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Vacua | HirExprKind::Error => {}
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            visitor.visit_expr_mut(lhs);
            visitor.visit_expr_mut(rhs);
        }
        HirExprKind::Unary(_, operand)
        | HirExprKind::Cede(operand)
        | HirExprKind::Ref(_, operand)
        | HirExprKind::Deref(operand)
        | HirExprKind::Panic(operand)
        | HirExprKind::Throw(operand) => visitor.visit_expr_mut(operand),
        HirExprKind::Call(callee, args) => {
            visitor.visit_expr_mut(callee);
            for arg in args {
                visitor.visit_expr_mut(&mut arg.expr);
            }
        }
        HirExprKind::MethodCall(receiver, _, args) => {
            visitor.visit_expr_mut(receiver);
            for arg in args {
                visitor.visit_expr_mut(&mut arg.expr);
            }
        }
        HirExprKind::Field(object, _) => visitor.visit_expr_mut(object),
        HirExprKind::Index(object, index) => {
            visitor.visit_expr_mut(object);
            visitor.visit_expr_mut(index);
        }
        HirExprKind::OptionalChain(object, chain) => {
            visitor.visit_expr_mut(object);
            walk_optional_chain_mut(visitor, chain);
        }
        HirExprKind::NonNull(object, chain) => {
            visitor.visit_expr_mut(object);
            walk_non_null_mut(visitor, chain);
        }
        HirExprKind::Block(block) | HirExprKind::Loop(block) => visitor.visit_block_mut(block),
        HirExprKind::Si { cond, then_block, then_catch, else_block } => {
            visitor.visit_expr_mut(cond);
            visitor.visit_block_mut(then_block);
            if let Some(catch) = then_catch {
                visitor.visit_cape_mut(catch);
            }
            if let Some(else_block) = else_block {
                visitor.visit_block_mut(else_block);
            }
        }
        HirExprKind::Discerne(scrutinees, arms) => {
            for scrutinee in scrutinees {
                visitor.visit_expr_mut(scrutinee);
            }
            for arm in arms {
                visitor.visit_casu_arm_mut(arm);
            }
        }
        HirExprKind::Dum(cond, block) => {
            visitor.visit_expr_mut(cond);
            visitor.visit_block_mut(block);
        }
        HirExprKind::Itera(_, def_id, name, iter, block) => {
            visitor.visit_def(*def_id, *name);
            visitor.visit_expr_mut(iter);
            visitor.visit_block_mut(block);
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            visitor.visit_expr_mut(start);
            visitor.visit_expr_mut(end);
            if let Some(step) = step {
                visitor.visit_expr_mut(step);
            }
        }
        HirExprKind::Array(elements) => {
            for element in elements {
                match element {
                    HirArrayElement::Expr(expr) | HirArrayElement::Spread(expr) => visitor.visit_expr_mut(expr),
                }
            }
        }
        HirExprKind::Struct(_, fields) => {
            for (_, value) in fields {
                visitor.visit_expr_mut(value);
            }
        }
        HirExprKind::Tuple(elements) | HirExprKind::Scribe(_, elements) | HirExprKind::Scriptum(_, elements) => {
            for element in elements {
                visitor.visit_expr_mut(element);
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            visitor.visit_expr_mut(cond);
            if let Some(message) = message {
                visitor.visit_expr_mut(message);
            }
        }
        HirExprKind::Handled { body, catch } => {
            visitor.visit_block_mut(body);
            visitor.visit_cape_mut(catch);
        }
        HirExprKind::Tempta { body, catch, finally } => {
            visitor.visit_block_mut(body);
            if let Some(catch) = catch {
                visitor.visit_block_mut(catch);
            }
            if let Some(finally) = finally {
                visitor.visit_block_mut(finally);
            }
        }
        HirExprKind::Clausura(params, _, body) => {
            for param in params {
                visitor.visit_param_mut(param);
            }
            visitor.visit_expr_mut(body);
        }
        HirExprKind::Verte { source, entries, .. } => {
            visitor.visit_expr_mut(source);
            if let Some(entries) = entries {
                for field in entries {
                    visitor.visit_object_field_mut(field);
                }
            }
        }
        HirExprKind::Conversio { source, fallback, .. } => {
            visitor.visit_expr_mut(source);
            if let Some(fallback) = fallback {
                visitor.visit_expr_mut(fallback);
            }
        }
    }
}

/// Mutably walk a recoverable handler binding and body.
pub fn walk_cape_mut<V: HirVisitorMut>(visitor: &mut V, cape: &mut HirCape) {
    visitor.visit_def(cape.binding_def_id, cape.binding_name);
    visitor.visit_block_mut(&mut cape.body);
}

/// Mutably walk one pattern arm: patterns first, then guard, then body.
pub fn walk_casu_arm_mut<V: HirVisitorMut>(visitor: &mut V, arm: &mut HirCasuArm) {
    for pattern in &mut arm.patterns {
        visitor.visit_pattern_mut(pattern);
    }
    if let Some(guard) = &mut arm.guard {
        visitor.visit_expr_mut(guard);
    }
    visitor.visit_expr_mut(&mut arm.body);
}

/// Mutably walk bindings introduced by a pattern.
pub fn walk_pattern_mut<V: HirVisitorMut>(visitor: &mut V, pattern: &mut HirPattern) {
    match pattern {
        HirPattern::Wildcard | HirPattern::Literal(_) => {}
        HirPattern::Binding(def_id, name) => visitor.visit_def(*def_id, *name),
        HirPattern::Alias(def_id, name, pattern) => {
            visitor.visit_def(*def_id, *name);
            visitor.visit_pattern_mut(pattern);
        }
        HirPattern::Variant(_, patterns) => {
            for pattern in patterns {
                visitor.visit_pattern_mut(pattern);
            }
        }
    }
}

/// Mutably walk expressions owned by an object field key or value.
pub fn walk_object_field_mut<V: HirVisitorMut>(visitor: &mut V, field: &mut HirObjectField) {
    match &mut field.key {
        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
        HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => visitor.visit_expr_mut(expr),
    }
    if let Some(value) = &mut field.value {
        visitor.visit_expr_mut(value);
    }
}

/// Mutably walk expression children of an optional-chain operation.
pub fn walk_optional_chain_mut<V: HirVisitorMut>(visitor: &mut V, chain: &mut HirOptionalChainKind) {
    match chain {
        HirOptionalChainKind::Member(_) => {}
        HirOptionalChainKind::Index(index) => visitor.visit_expr_mut(index),
        HirOptionalChainKind::Call(args) => {
            for arg in args {
                visitor.visit_expr_mut(arg);
            }
        }
    }
}

/// Mutably walk expression children of a non-null assertion operation.
pub fn walk_non_null_mut<V: HirVisitorMut>(visitor: &mut V, chain: &mut HirNonNullKind) {
    match chain {
        HirNonNullKind::Member(_) => {}
        HirNonNullKind::Index(index) => visitor.visit_expr_mut(index),
        HirNonNullKind::Call(args) => {
            for arg in args {
                visitor.visit_expr_mut(arg);
            }
        }
    }
}
