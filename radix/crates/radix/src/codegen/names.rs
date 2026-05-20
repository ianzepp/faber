use crate::hir::{
    HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirItemKind, HirNonNullKind, HirObjectKey,
    HirOptionalChainKind, HirPattern, HirProgram, HirStmtKind,
};
use crate::lexer::{Interner, Symbol};
use rustc_hash::FxHashMap;

pub(crate) struct NameCatalog<'a> {
    names: FxHashMap<crate::hir::DefId, Symbol>,
    interner: &'a Interner,
}

impl<'a> NameCatalog<'a> {
    pub(crate) fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        Self { names: collect_names(hir), interner }
    }

    pub(crate) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }

    pub(crate) fn resolve_def(&self, def_id: crate::hir::DefId) -> &str {
        self.names
            .get(&def_id)
            .map(|sym| self.resolve_symbol(*sym))
            .unwrap_or("unresolved_def")
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&crate::hir::DefId, &Symbol)> {
        self.names.iter()
    }
}

pub(crate) fn collect_names(hir: &HirProgram) -> FxHashMap<crate::hir::DefId, Symbol> {
    let mut names = FxHashMap::default();
    for item in &hir.items {
        match &item.kind {
            HirItemKind::Function(func) => {
                names.insert(item.def_id, func.name);
                for type_param in &func.type_params {
                    names.insert(type_param.def_id, type_param.name);
                }
                for param in &func.params {
                    names.insert(param.def_id, param.name);
                }
                collect_block_names(&mut names, func.body.as_ref());
            }
            HirItemKind::Struct(strukt) => {
                names.insert(item.def_id, strukt.name);
                for type_param in &strukt.type_params {
                    names.insert(type_param.def_id, type_param.name);
                }
                for field in &strukt.fields {
                    names.insert(field.def_id, field.name);
                }
                for method in &strukt.methods {
                    names.insert(method.def_id, method.func.name);
                    for type_param in &method.func.type_params {
                        names.insert(type_param.def_id, type_param.name);
                    }
                    for param in &method.func.params {
                        names.insert(param.def_id, param.name);
                    }
                    collect_block_names(&mut names, method.func.body.as_ref());
                }
            }
            HirItemKind::Enum(enum_item) => {
                names.insert(item.def_id, enum_item.name);
                for type_param in &enum_item.type_params {
                    names.insert(type_param.def_id, type_param.name);
                }
                for variant in &enum_item.variants {
                    names.insert(variant.def_id, variant.name);
                }
            }
            HirItemKind::Interface(interface) => {
                names.insert(item.def_id, interface.name);
                for type_param in &interface.type_params {
                    names.insert(type_param.def_id, type_param.name);
                }
            }
            HirItemKind::TypeAlias(alias) => {
                names.insert(item.def_id, alias.name);
            }
            HirItemKind::Const(const_item) => {
                names.insert(item.def_id, const_item.name);
            }
            HirItemKind::Import(import) => {
                for item in &import.items {
                    names.insert(item.def_id, item.alias.unwrap_or(item.name));
                }
            }
        }
    }

    if let Some(entry) = &hir.entry {
        collect_block_names(&mut names, Some(entry));
    }

    names
}

fn collect_block_names(names: &mut FxHashMap<crate::hir::DefId, Symbol>, block: Option<&HirBlock>) {
    let Some(block) = block else {
        return;
    };

    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Local(local) => {
                names.insert(local.def_id, local.name);
                if let Some(init) = &local.init {
                    collect_expr_names(names, init);
                }
            }
            HirStmtKind::Ad(ad) => {
                for arg in &ad.args {
                    collect_expr_names(names, arg);
                }
                collect_block_names(names, ad.body.as_ref());
                collect_block_names(names, ad.catch.as_ref());
            }
            HirStmtKind::Expr(expr) => collect_expr_names(names, expr),
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    collect_expr_names(names, expr);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    if let Some(expr) = &block.expr {
        collect_expr_names(names, expr);
    }
}

fn collect_expr_names(names: &mut FxHashMap<crate::hir::DefId, Symbol>, expr: &HirExpr) {
    match &expr.kind {
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            collect_expr_names(names, lhs);
            collect_expr_names(names, rhs);
        }
        HirExprKind::Unary(_, operand)
        | HirExprKind::Cede(operand)
        | HirExprKind::Ref(_, operand)
        | HirExprKind::Deref(operand)
        | HirExprKind::Panic(operand)
        | HirExprKind::Throw(operand) => collect_expr_names(names, operand),
        HirExprKind::Verte { source, entries, .. } => {
            collect_expr_names(names, source);
            if let Some(entries) = entries {
                for field in entries {
                    match &field.key {
                        HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => collect_expr_names(names, expr),
                        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
                    }
                    if let Some(value) = &field.value {
                        collect_expr_names(names, value);
                    }
                }
            }
        }
        HirExprKind::Conversio { source, fallback, .. } => {
            collect_expr_names(names, source);
            if let Some(fallback) = fallback {
                collect_expr_names(names, fallback);
            }
        }
        HirExprKind::Call(callee, args) => {
            collect_expr_names(names, callee);
            for arg in args {
                collect_expr_names(names, arg);
            }
        }
        HirExprKind::MethodCall(receiver, _, args) => {
            collect_expr_names(names, receiver);
            for arg in args {
                collect_expr_names(names, arg);
            }
        }
        HirExprKind::Field(object, _) => collect_expr_names(names, object),
        HirExprKind::Index(object, index) => {
            collect_expr_names(names, object);
            collect_expr_names(names, index);
        }
        HirExprKind::OptionalChain(object, chain) => {
            collect_expr_names(names, object);
            match chain {
                HirOptionalChainKind::Member(_) => {}
                HirOptionalChainKind::Index(index) => collect_expr_names(names, index),
                HirOptionalChainKind::Call(args) => {
                    for arg in args {
                        collect_expr_names(names, arg);
                    }
                }
            }
        }
        HirExprKind::NonNull(object, chain) => {
            collect_expr_names(names, object);
            match chain {
                HirNonNullKind::Member(_) => {}
                HirNonNullKind::Index(index) => collect_expr_names(names, index),
                HirNonNullKind::Call(args) => {
                    for arg in args {
                        collect_expr_names(names, arg);
                    }
                }
            }
        }
        HirExprKind::Ab { source, filter, transforms } => {
            collect_expr_names(names, source);
            if let Some(filter) = filter {
                if let HirCollectionFilterKind::Condition(cond) = &filter.kind {
                    collect_expr_names(names, cond);
                }
            }
            for transform in transforms {
                if let Some(arg) = &transform.arg {
                    collect_expr_names(names, arg);
                }
            }
        }
        HirExprKind::Block(block) | HirExprKind::Loop(block) => collect_block_names(names, Some(block)),
        HirExprKind::Si(cond, then_block, else_block) => {
            collect_expr_names(names, cond);
            collect_block_names(names, Some(then_block));
            collect_block_names(names, else_block.as_ref());
        }
        HirExprKind::Discerne(scrutinees, arms) => {
            for scrutinee in scrutinees {
                collect_expr_names(names, scrutinee);
            }
            for arm in arms {
                for pattern in &arm.patterns {
                    collect_pattern_names(names, pattern);
                }
                if let Some(guard) = &arm.guard {
                    collect_expr_names(names, guard);
                }
                collect_expr_names(names, &arm.body);
            }
        }
        HirExprKind::Dum(cond, block) => {
            collect_expr_names(names, cond);
            collect_block_names(names, Some(block));
        }
        HirExprKind::Itera(_, _, _, iter, block) => {
            collect_expr_names(names, iter);
            collect_block_names(names, Some(block));
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            collect_expr_names(names, start);
            collect_expr_names(names, end);
            if let Some(step) = step {
                collect_expr_names(names, step);
            }
        }
        HirExprKind::Array(elements) => {
            for element in elements {
                match element {
                    crate::hir::HirArrayElement::Expr(expr) | crate::hir::HirArrayElement::Spread(expr) => {
                        collect_expr_names(names, expr)
                    }
                }
            }
        }
        HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
            for element in elements {
                collect_expr_names(names, element);
            }
        }
        HirExprKind::Scriptum(_, args) => {
            for arg in args {
                collect_expr_names(names, arg);
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            collect_expr_names(names, cond);
            if let Some(message) = message {
                collect_expr_names(names, message);
            }
        }
        HirExprKind::Struct(_, fields) => {
            for (_, value) in fields {
                collect_expr_names(names, value);
            }
        }
        HirExprKind::Tempta { body, catch, finally } => {
            collect_block_names(names, Some(body));
            collect_block_names(names, catch.as_ref());
            collect_block_names(names, finally.as_ref());
        }
        HirExprKind::Clausura(params, _, body) => {
            for param in params {
                names.insert(param.def_id, param.name);
            }
            collect_expr_names(names, body);
        }
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
    }
}

fn collect_pattern_names(names: &mut FxHashMap<crate::hir::DefId, Symbol>, pattern: &HirPattern) {
    match pattern {
        HirPattern::Wildcard | HirPattern::Literal(_) => {}
        HirPattern::Binding(def_id, name) => {
            names.insert(*def_id, *name);
        }
        HirPattern::Alias(def_id, name, pattern) => {
            names.insert(*def_id, *name);
            collect_pattern_names(names, pattern);
        }
        HirPattern::Variant(_, patterns) => {
            for pattern in patterns {
                collect_pattern_names(names, pattern);
            }
        }
    }
}
