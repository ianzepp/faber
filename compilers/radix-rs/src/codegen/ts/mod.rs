mod decl;
mod expr;
mod stmt;
mod types;

use super::{CodeWriter, Codegen, CodegenError};
use crate::hir::{
    DefId, HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirItem, HirItemKind, HirOptionalChainKind,
    HirPattern, HirProgram, HirStmtKind,
};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use crate::TypeScriptOutput;
use rustc_hash::FxHashMap;

pub struct TsCodegen<'a> {
    names: FxHashMap<DefId, Symbol>,
    interner: &'a Interner,
}

impl<'a> TsCodegen<'a> {
    pub fn new(hir: &HirProgram, interner: &'a Interner) -> Self {
        let mut codegen = Self { names: FxHashMap::default(), interner };
        codegen.names = codegen.collect_names(hir);
        codegen
    }

    pub(super) fn resolve_symbol(&self, sym: Symbol) -> &str {
        self.interner.resolve(sym)
    }

    pub(super) fn resolve_def(&self, def_id: DefId) -> &str {
        self.names
            .get(&def_id)
            .map(|sym| self.resolve_symbol(*sym))
            .unwrap_or("unresolved_def")
    }

    fn collect_names(&self, hir: &HirProgram) -> FxHashMap<DefId, Symbol> {
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
                    self.collect_block_names(&mut names, func.body.as_ref());
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
                        self.collect_block_names(&mut names, method.func.body.as_ref());
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
            self.collect_block_names(&mut names, Some(entry));
        }

        names
    }

    fn collect_block_names(&self, names: &mut FxHashMap<DefId, Symbol>, block: Option<&HirBlock>) {
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

    fn collect_expr_names(&self, names: &mut FxHashMap<DefId, Symbol>, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.collect_expr_names(names, lhs);
                self.collect_expr_names(names, rhs);
            }
            HirExprKind::Unary(_, operand)
            | HirExprKind::Cede(operand)
            | HirExprKind::Ref(_, operand)
            | HirExprKind::Deref(operand)
            | HirExprKind::Panic(operand)
            | HirExprKind::Throw(operand) => self.collect_expr_names(names, operand),
            HirExprKind::Verte { source, entries, .. } => {
                self.collect_expr_names(names, source);
                if let Some(entries) = entries {
                    for (_, value) in entries {
                        self.collect_expr_names(names, value);
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
            HirExprKind::Field(object, _) => self.collect_expr_names(names, object),
            HirExprKind::Index(object, index) => {
                self.collect_expr_names(names, object);
                self.collect_expr_names(names, index);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.collect_expr_names(names, object);
                match chain {
                    HirOptionalChainKind::Member(_) => {}
                    HirOptionalChainKind::Index(index) => self.collect_expr_names(names, index),
                    HirOptionalChainKind::Call(args) => {
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
                    if let HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.collect_expr_names(names, cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.collect_expr_names(names, arg);
                    }
                }
            }
            HirExprKind::Block(block) | HirExprKind::Loop(block) => self.collect_block_names(names, Some(block)),
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
            HirExprKind::Dum(cond, block) => {
                self.collect_expr_names(names, cond);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Itera(_, _, iter, block) => {
                self.collect_expr_names(names, iter);
                self.collect_block_names(names, Some(block));
            }
            HirExprKind::Array(elements) | HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
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
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.collect_expr_names(names, value);
                }
            }
            HirExprKind::Tempta { body, catch, finally } => {
                self.collect_block_names(names, Some(body));
                self.collect_block_names(names, catch.as_ref());
                self.collect_block_names(names, finally.as_ref());
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

    #[allow(clippy::only_used_in_recursion)]
    fn collect_pattern_names(&self, names: &mut FxHashMap<DefId, Symbol>, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard | HirPattern::Literal(_) => {}
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
        }
    }

    fn generate_item(&self, item: &HirItem, types: &TypeTable, w: &mut CodeWriter) -> Result<(), CodegenError> {
        match &item.kind {
            HirItemKind::Function(func) => decl::generate_function(self, func, types, w)?,
            HirItemKind::Struct(strukt) => decl::generate_class(self, strukt, types, w)?,
            HirItemKind::Enum(enum_item) => decl::generate_enum(self, enum_item, types, w)?,
            HirItemKind::Interface(interface) => decl::generate_interface(self, interface, types, w)?,
            HirItemKind::TypeAlias(alias) => decl::generate_type_alias(self, alias, types, w)?,
            HirItemKind::Const(constant) => decl::generate_const(self, constant, types, w)?,
            HirItemKind::Import(import) => decl::generate_import(self, import, w)?,
        }
        Ok(())
    }
}

impl Codegen for TsCodegen<'_> {
    type Output = TypeScriptOutput;

    fn generate(
        &self,
        hir: &HirProgram,
        types: &TypeTable,
        _interner: &Interner,
    ) -> Result<TypeScriptOutput, CodegenError> {
        let mut w = CodeWriter::new();
        w.writeln("// Generated by radix - do not edit");
        w.newline();

        for item in &hir.items {
            self.generate_item(item, types, &mut w)?;
            w.newline();
        }

        if let Some(entry) = &hir.entry {
            let entry_is_async = expr::contains_await_in_block(entry);
            if entry_is_async {
                w.writeln("(async () => {");
            } else {
                w.writeln("(() => {");
            }
            let mut block_result = Ok(());
            w.indented(|w| {
                block_result = stmt::generate_block(self, entry, types, w);
            });
            block_result?;
            w.writeln("})();");
        }

        Ok(TypeScriptOutput { code: w.finish() })
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
