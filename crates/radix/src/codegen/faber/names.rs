use crate::hir::{
    DefId, HirArrayElement, HirBlock, HirExpr, HirExprKind, HirFunction, HirItemKind, HirObjectKey, HirPattern,
    HirProgram, HirStmtKind,
};
use crate::lexer::{Interner, Symbol};
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn collect_pattern_names(&self, names: &mut FxHashMap<DefId, Symbol>, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Binding(def_id, name) => {
                names.insert(*def_id, *name);
            }
            HirPattern::Alias(def_id, name, pattern) => {
                names.insert(*def_id, *name);
                self.collect_pattern_names(names, pattern);
            }
            HirPattern::Variant(_, patterns) => {
                for pattern in patterns {
                    self.collect_pattern_names(names, pattern);
                }
            }
            HirPattern::Literal(_) => {}
        }
    }
    pub(super) fn collect_expr_names(&self, names: &mut FxHashMap<DefId, Symbol>, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.collect_expr_names(names, lhs);
                self.collect_expr_names(names, rhs);
            }
            HirExprKind::Unary(_, operand)
            | HirExprKind::Cede(operand)
            | HirExprKind::Ref(_, operand)
            | HirExprKind::Deref(operand) => self.collect_expr_names(names, operand),
            HirExprKind::Verte { source, entries, .. } => {
                self.collect_expr_names(names, source);
                if let Some(entries) = entries {
                    for field in entries {
                        match &field.key {
                            HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => {
                                self.collect_expr_names(names, expr);
                            }
                            HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
                        }
                        if let Some(value) = &field.value {
                            self.collect_expr_names(names, value);
                        }
                    }
                }
            }
            HirExprKind::Conversio { source, fallback, .. } => {
                self.collect_expr_names(names, source);
                if let Some(fallback) = fallback {
                    self.collect_expr_names(names, fallback);
                }
            }
            HirExprKind::Call(callee, args) => {
                self.collect_expr_names(names, callee);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.collect_expr_names(names, receiver);
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Field(object, _) | HirExprKind::Index(object, _) => {
                self.collect_expr_names(names, object);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.collect_expr_names(names, object);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(_) => {}
                    crate::hir::HirOptionalChainKind::Index(index) => self.collect_expr_names(names, index),
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_names(names, arg);
                        }
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.collect_expr_names(names, object);
                match chain {
                    crate::hir::HirNonNullKind::Member(_) => {}
                    crate::hir::HirNonNullKind::Index(index) => self.collect_expr_names(names, index),
                    crate::hir::HirNonNullKind::Call(args) => {
                        for arg in args {
                            self.collect_expr_names(names, arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.collect_expr_names(names, source);
                if let Some(filter) = filter {
                    if let crate::hir::HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.collect_expr_names(names, cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.collect_expr_names(names, arg);
                    }
                }
            }
            HirExprKind::Block(block) => self.collect_block_names(names, Some(block)),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.collect_expr_names(names, cond);
                self.collect_block_names(names, Some(then_block));
                self.collect_block_names(names, else_block.as_ref());
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                for scrutinee in scrutinees {
                    self.collect_expr_names(names, scrutinee);
                }
                for arm in arms {
                    for pattern in &arm.patterns {
                        self.collect_pattern_names(names, pattern);
                    }
                    if let Some(guard) = &arm.guard {
                        self.collect_expr_names(names, guard);
                    }
                    self.collect_expr_names(names, &arm.body);
                }
            }
            HirExprKind::Loop(block) | HirExprKind::Dum(_, block) => {
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Itera(_, binding, binding_name, iter, block) => {
                names.insert(*binding, *binding_name);
                self.collect_expr_names(names, iter);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                self.collect_expr_names(names, start);
                self.collect_expr_names(names, end);
                if let Some(step) = step {
                    self.collect_expr_names(names, step);
                }
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    match element {
                        HirArrayElement::Expr(expr) | HirArrayElement::Spread(expr) => {
                            self.collect_expr_names(names, expr);
                        }
                    }
                }
            }
            HirExprKind::Tuple(elements) | HirExprKind::Scribe(_, elements) => {
                for element in elements {
                    self.collect_expr_names(names, element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.collect_expr_names(names, arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.collect_expr_names(names, cond);
                if let Some(message) = message {
                    self.collect_expr_names(names, message);
                }
            }
            HirExprKind::Panic(value) | HirExprKind::Throw(value) => self.collect_expr_names(names, value),
            HirExprKind::Tempta { body, catch, finally } => {
                self.collect_block_names(names, Some(body));
                self.collect_block_names(names, catch.as_ref());
                self.collect_block_names(names, finally.as_ref());
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.collect_expr_names(names, value);
                }
            }
            HirExprKind::Clausura(params, _, body) => {
                for param in params {
                    names.insert(param.def_id, param.name);
                }
                self.collect_expr_names(names, body);
            }
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }
    pub(super) fn name_for_def(&self, def_id: DefId, names: &FxHashMap<DefId, Symbol>, interner: &Interner) -> String {
        names
            .get(&def_id)
            .map(|sym| self.symbol_to_string(*sym, interner))
            .unwrap_or_else(|| format!("def_{}", def_id.0))
    }

    pub(super) fn symbol_to_string(&self, sym: Symbol, interner: &Interner) -> String {
        interner.resolve(sym).to_owned()
    }

    /// Collect DefId -> Symbol mappings for all definitions in the program.
    ///
    /// WHY: HIR uses DefIds for references; we need to map them back to their
    /// original names for source generation. This is a single upfront traversal
    /// rather than repeated lookups during generation.
    pub(super) fn collect_names(&self, hir: &HirProgram) -> FxHashMap<DefId, Symbol> {
        let mut names = FxHashMap::default();
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    self.collect_function_names(&mut names, item.def_id, func);
                }
                HirItemKind::Struct(strukt) => {
                    names.insert(item.def_id, strukt.name);
                    for field in &strukt.fields {
                        names.insert(field.def_id, field.name);
                    }
                    for method in &strukt.methods {
                        self.collect_function_names(&mut names, method.def_id, &method.func);
                    }
                }
                HirItemKind::Enum(enum_item) => {
                    names.insert(item.def_id, enum_item.name);
                    for variant in &enum_item.variants {
                        names.insert(variant.def_id, variant.name);
                    }
                }
                HirItemKind::Interface(interface) => {
                    names.insert(item.def_id, interface.name);
                }
                HirItemKind::TypeAlias(alias) => {
                    names.insert(item.def_id, alias.name);
                }
                HirItemKind::Const(const_item) => {
                    names.insert(item.def_id, const_item.name);
                }
                HirItemKind::Import(import) => {
                    for item in &import.items {
                        let name = item.alias.unwrap_or(item.name);
                        names.insert(item.def_id, name);
                    }
                }
            }
        }

        if let Some(entry) = &hir.entry {
            self.collect_block_names(&mut names, Some(entry));
        }

        names
    }

    pub(super) fn collect_function_names(
        &self,
        names: &mut FxHashMap<DefId, Symbol>,
        def_id: DefId,
        func: &HirFunction,
    ) {
        names.insert(def_id, func.name);
        for param in &func.params {
            names.insert(param.def_id, param.name);
        }
        self.collect_block_names(names, func.body.as_ref());
    }

    pub(super) fn collect_block_names(&self, names: &mut FxHashMap<DefId, Symbol>, block: Option<&HirBlock>) {
        let Some(block) = block else {
            return;
        };
        for stmt in &block.stmts {
            match &stmt.kind {
                HirStmtKind::Local(local) => {
                    names.insert(local.def_id, local.name);
                    if let Some(init) = &local.init {
                        self.collect_expr_names(names, init);
                    }
                }
                HirStmtKind::Ad(ad) => {
                    for arg in &ad.args {
                        self.collect_expr_names(names, arg);
                    }
                    self.collect_block_names(names, ad.body.as_ref());
                    self.collect_block_names(names, ad.catch.as_ref());
                }
                HirStmtKind::Expr(expr) => self.collect_expr_names(names, expr),
                HirStmtKind::Redde(value) => {
                    if let Some(expr) = value {
                        self.collect_expr_names(names, expr);
                    }
                }
                HirStmtKind::Rumpe | HirStmtKind::Perge => {}
            }
        }
        if let Some(expr) = &block.expr {
            self.collect_expr_names(names, expr);
        }
    }
}
