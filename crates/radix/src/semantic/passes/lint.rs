//! Final semantic lint pass over typed HIR.
//!
//! Linting is the last non-codegen semantic pass. It runs after collection,
//! resolution, lowering, typechecking, and any target-enabled borrow and
//! exhaustiveness checks. Its job is to turn fully resolved HIR identity and
//! `TypeTable` facts into user-facing diagnostics for suspicious but often
//! recoverable code: unused declarations, unreachable statements, redundant
//! casts, and explicit `ignotum` annotations.
//!
//! The pass is target-configurable through `PassConfig`. When enabled, it
//! returns both warning-kind diagnostics and hard semantic errors through the
//! same vector because the semantic result owns final severity policy. In
//! particular, shadowing is intentionally a hard error here: resolved `DefId`s
//! make the program unambiguous to the compiler, but the source remains
//! misleading enough that the language rejects it.
//!
//! INVARIANTS
//! ==========
//! - Usage is tracked by `DefId`, never by spelling, so imports, functions, and
//!   locals follow the resolver's identity model.
//! - Scope shadowing checks use a lightweight lexical stack local to this pass;
//!   they do not mutate resolver scopes or type information.
//! - The `TypeTable` is read-only and used only where lint policy needs type
//!   facts, such as redundant `verte` casts or explicit `ignotum` annotations.
//! - Warnings produced here should not block compilation unless the surrounding
//!   diagnostic policy promotes warning-kind diagnostics.

use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirImport, HirItem, HirItemKind, HirLocal, HirProgram, HirStmt,
    HirStmtKind,
};
use crate::lexer::Span;
use crate::semantic::{error::WarningKind, Primitive, Resolver, SemanticError, SemanticErrorKind, Type, TypeTable};
use rustc_hash::FxHashSet;

/// Run the target-enabled semantic lint suite.
///
/// The returned `Err` vector is a diagnostic carrier, not a guarantee that
/// every finding is fatal. Warning-kind diagnostics are wrapped as
/// `SemanticErrorKind::Warning`, while shadowing remains a hard semantic error.
pub fn lint(hir: &HirProgram, _resolver: &Resolver, types: &TypeTable) -> Result<(), Vec<SemanticError>> {
    let mut warnings = Vec::new();

    let mut ctx = LintContext::new(types);
    ctx.collect_items(hir);
    ctx.check_program(hir);

    for warning in ctx.warnings {
        warnings.push(warning);
    }

    let mut errors = ctx.errors;

    // Convert warnings to errors (with warning kind)
    let mut warnings: Vec<SemanticError> = warnings
        .into_iter()
        .map(|(kind, msg, span): (WarningKind, String, Span)| {
            SemanticError::new(SemanticErrorKind::Warning(kind), msg, span)
        })
        .collect();

    errors.append(&mut warnings);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

struct LintContext<'a> {
    /// Read-only semantic type facts for lints that need more than identity,
    /// such as redundant casts and explicit `ignotum` annotations.
    types: &'a TypeTable,
    warnings: Vec<(WarningKind, String, Span)>,
    errors: Vec<SemanticError>,
    used: FxHashSet<crate::hir::DefId>,
    defs: Vec<(crate::hir::DefId, Span, WarningKind)>,
    imports: Vec<(crate::hir::DefId, Span)>,
    functions: Vec<(crate::hir::DefId, Span)>,
    scope: Vec<rustc_hash::FxHashMap<crate::lexer::Symbol, crate::hir::DefId>>,
    in_loop: bool,
}

impl<'a> LintContext<'a> {
    fn new(types: &'a TypeTable) -> Self {
        Self {
            types,
            warnings: Vec::new(),
            errors: Vec::new(),
            used: FxHashSet::default(),
            defs: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            scope: Vec::new(),
            in_loop: false,
        }
    }

    fn collect_items(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Import(import) => self.collect_import(import),
                HirItemKind::Function(func) if item.def_id.0 < 1_000_000 => {
                    self.functions
                        .push((item.def_id, func.body.as_ref().map(|b| b.span).unwrap_or(item.span)));
                }
                _ => {}
            }
        }
    }

    fn collect_import(&mut self, import: &HirImport) {
        for item in &import.items {
            self.imports.push((item.def_id, Span::default()));
        }
    }

    fn check_program(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            self.check_item(item);
        }

        if let Some(entry) = &hir.entry {
            self.check_block(entry, false);
        }

        for (def_id, span, kind) in &self.defs {
            if !self.used.contains(def_id) {
                let message = match kind {
                    WarningKind::UnusedVariable => "unused variable",
                    _ => "unused binding",
                };
                self.warnings.push((*kind, message.to_owned(), *span));
            }
        }

        for (def_id, span) in &self.imports {
            if !self.used.contains(def_id) {
                self.warnings
                    .push((WarningKind::UnusedImport, "unused import".to_owned(), *span));
            }
        }

        for (def_id, span) in &self.functions {
            if !self.used.contains(def_id) {
                self.warnings
                    .push((WarningKind::UnusedFunction, "unused function".to_owned(), *span));
            }
        }
    }

    fn check_item(&mut self, item: &HirItem) {
        match &item.kind {
            HirItemKind::Function(func) => self.check_function(func),
            HirItemKind::Struct(strukt) => {
                for method in &strukt.methods {
                    self.check_function(&method.func);
                }
            }
            HirItemKind::Const(const_item) => self.check_expr(&const_item.value, false),
            _ => {}
        }
    }

    fn check_function(&mut self, func: &HirFunction) {
        self.push_scope();
        for param in &func.params {
            self.defs
                .push((param.def_id, param.span, WarningKind::UnusedVariable));
            self.check_shadowing(param.name, param.def_id, param.span);
            self.insert_name(param.name, param.def_id);
            self.warn_on_explicit_ignotum(param.ty, param.span);
        }
        if let Some(body) = &func.body {
            self.check_block(body, false);
        }
        self.pop_scope();
    }

    fn check_block(&mut self, block: &HirBlock, in_loop: bool) {
        self.push_scope();
        let mut terminated = false;
        for stmt in &block.stmts {
            if terminated {
                self.warnings
                    .push((WarningKind::UnreachableCode, "unreachable code".to_owned(), stmt.span));
                continue;
            }
            self.check_stmt(stmt, in_loop);
            if matches!(stmt.kind, HirStmtKind::Redde(_))
                || (in_loop && matches!(stmt.kind, HirStmtKind::Rumpe | HirStmtKind::Perge))
            {
                terminated = true;
            }
        }
        if let Some(expr) = &block.expr {
            self.check_expr(expr, in_loop);
        }
        self.pop_scope();
    }

    fn check_stmt(&mut self, stmt: &HirStmt, in_loop: bool) {
        match &stmt.kind {
            HirStmtKind::Local(local) => self.check_local(local),
            HirStmtKind::Expr(expr) => self.check_expr(expr, in_loop),
            HirStmtKind::Ad(ad) => {
                for arg in &ad.args {
                    self.check_expr(arg, in_loop);
                }
                if let Some(body) = &ad.body {
                    self.check_block(body, in_loop);
                }
                if let Some(catch) = &ad.catch {
                    self.check_block(catch, in_loop);
                }
            }
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr, in_loop);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge | HirStmtKind::Tacet => {}
        }
    }

    fn check_local(&mut self, local: &HirLocal) {
        let span = local
            .init
            .as_ref()
            .map(|expr| expr.span)
            .unwrap_or_default();
        self.defs
            .push((local.def_id, span, WarningKind::UnusedVariable));
        self.check_shadowing(local.name, local.def_id, span);
        self.insert_name(local.name, local.def_id);
        if let Some(ty) = local.ty {
            self.warn_on_explicit_ignotum(ty, span);
        }
        if let Some(init) = &local.init {
            self.check_expr(init, false);
        }
    }

    fn check_expr(&mut self, expr: &HirExpr, in_loop: bool) {
        let previous = self.in_loop;
        self.in_loop = in_loop;
        self.visit_expr(expr);
        self.in_loop = previous;
    }

    fn check_shadowing(&mut self, name: crate::lexer::Symbol, def_id: crate::hir::DefId, span: Span) {
        for scope in self.scope.iter().rev() {
            if let Some(existing) = scope.get(&name) {
                if existing != &def_id {
                    self.errors.push(SemanticError::new(
                        SemanticErrorKind::ShadowedVariable,
                        "shadowed variable",
                        span,
                    ));
                }
                break;
            }
        }
    }

    fn insert_name(&mut self, name: crate::lexer::Symbol, def_id: crate::hir::DefId) {
        if let Some(scope) = self.scope.last_mut() {
            scope.insert(name, def_id);
        }
    }

    fn push_scope(&mut self) {
        self.scope.push(rustc_hash::FxHashMap::default());
    }

    fn pop_scope(&mut self) {
        self.scope.pop();
    }

    fn warn_on_explicit_ignotum(&mut self, ty: crate::semantic::TypeId, span: Span) {
        if matches!(self.types.get(ty), Type::Primitive(Primitive::Ignotum)) {
            self.warnings.push((
                WarningKind::ExplicitIgnotumAnnotation,
                "explicit ignotum annotation disables precise type-checking".to_owned(),
                span,
            ));
        }
    }
}

impl HirVisitor for LintContext<'_> {
    fn visit_block(&mut self, block: &HirBlock) {
        self.check_block(block, self.in_loop);
    }

    fn visit_expr(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                self.used.insert(*def_id);
            }
            HirExprKind::Loop(block) => self.check_block(block, true),
            HirExprKind::Dum(cond, block) => {
                self.visit_expr(cond);
                self.check_block(block, true);
            }
            HirExprKind::Itera(_, _, _, iter, block) => {
                self.visit_expr(iter);
                self.check_block(block, true);
            }
            HirExprKind::Verte { source, target, entries } => {
                if entries.is_none() {
                    if let Some(inner_ty) = source.ty {
                        if self.types.equals(inner_ty, *target) {
                            self.warnings.push((
                                WarningKind::UnnecessaryCast,
                                "unnecessary cast: type is already known".to_owned(),
                                expr.span,
                            ));
                        }
                    }
                }
                walk_expr(self, expr);
            }
            _ => walk_expr(self, expr),
        }
    }
}

#[cfg(test)]
#[path = "lint_test.rs"]
mod tests;
