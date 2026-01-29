//! Pass 6: Linting
//!
//! Produces warnings for common issues.

use crate::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirImport, HirImportItem, HirItem, HirItemKind,
    HirLocal, HirProgram, HirStmt, HirStmtKind,
};
use crate::lexer::Span;
use crate::semantic::{error::WarningKind, Resolver, SemanticError, SemanticErrorKind, TypeTable};
use rustc_hash::FxHashSet;

/// Run lint checks
pub fn lint(
    hir: &HirProgram,
    _resolver: &Resolver,
    _types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut warnings = Vec::new();

    let mut ctx = LintContext::new();
    ctx.collect_items(hir);
    ctx.check_program(hir);

    for warning in ctx.warnings {
        warnings.push(warning);
    }

    // Convert warnings to errors (with warning kind)
    let errors: Vec<SemanticError> = warnings
        .into_iter()
        .map(|(kind, msg, span): (WarningKind, String, Span)| {
            SemanticError::new(SemanticErrorKind::Warning(kind), msg, span)
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

struct LintContext {
    warnings: Vec<(WarningKind, String, Span)>,
    used: FxHashSet<crate::hir::DefId>,
    defs: Vec<(crate::hir::DefId, Span, WarningKind)>,
    imports: Vec<(crate::hir::DefId, Span)>,
    functions: Vec<(crate::hir::DefId, Span)>,
}

impl LintContext {
    fn new() -> Self {
        Self {
            warnings: Vec::new(),
            used: FxHashSet::default(),
            defs: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
        }
    }

    fn collect_items(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Import(import) => self.collect_import(import),
                HirItemKind::Function(func) => self.functions.push((
                    item.def_id,
                    func.body.as_ref().map(|b| b.span).unwrap_or(item.span),
                )),
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
            self.check_block(entry);
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
                self.warnings.push((
                    WarningKind::UnusedFunction,
                    "unused function".to_owned(),
                    *span,
                ));
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
            HirItemKind::Const(const_item) => self.check_expr(&const_item.value),
            _ => {}
        }
    }

    fn check_function(&mut self, func: &HirFunction) {
        for param in &func.params {
            self.defs
                .push((param.def_id, param.span, WarningKind::UnusedVariable));
        }
        if let Some(body) = &func.body {
            self.check_block(body);
        }
    }

    fn check_block(&mut self, block: &HirBlock) {
        let mut terminated = false;
        for stmt in &block.stmts {
            if terminated {
                self.warnings.push((
                    WarningKind::UnreachableCode,
                    "unreachable code".to_owned(),
                    stmt.span,
                ));
                continue;
            }
            self.check_stmt(stmt);
            if matches!(stmt.kind, HirStmtKind::Redde(_)) {
                terminated = true;
            }
        }
        if let Some(expr) = &block.expr {
            self.check_expr(expr);
        }
    }

    fn check_stmt(&mut self, stmt: &HirStmt) {
        match &stmt.kind {
            HirStmtKind::Local(local) => self.check_local(local),
            HirStmtKind::Expr(expr) => self.check_expr(expr),
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    fn check_local(&mut self, local: &HirLocal) {
        self.defs.push((
            local.def_id,
            local
                .init
                .as_ref()
                .map(|expr| expr.span)
                .unwrap_or_default(),
            WarningKind::UnusedVariable,
        ));
        if let Some(init) = &local.init {
            self.check_expr(init);
        }
    }

    fn check_expr(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                self.used.insert(*def_id);
            }
            HirExprKind::Binary(_, lhs, rhs) => {
                self.check_expr(lhs);
                self.check_expr(rhs);
            }
            HirExprKind::Unary(_, operand) => self.check_expr(operand),
            HirExprKind::Call(callee, args) => {
                self.check_expr(callee);
                for arg in args {
                    self.check_expr(arg);
                }
            }
            HirExprKind::MethodCall(receiver, _name, args) => {
                self.check_expr(receiver);
                for arg in args {
                    self.check_expr(arg);
                }
            }
            HirExprKind::Field(object, _) => self.check_expr(object),
            HirExprKind::Index(object, index) => {
                self.check_expr(object);
                self.check_expr(index);
            }
            HirExprKind::Block(block) => self.check_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_expr(cond);
                self.check_block(then_block);
                if let Some(block) = else_block {
                    self.check_block(block);
                }
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.check_expr(scrutinee);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expr(guard);
                    }
                    self.check_expr(&arm.body);
                }
            }
            HirExprKind::Loop(block) => self.check_block(block),
            HirExprKind::Dum(cond, block) => {
                self.check_expr(cond);
                self.check_block(block);
            }
            HirExprKind::Itera(_, iter, block) => {
                self.check_expr(iter);
                self.check_block(block);
            }
            HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.check_expr(lhs);
                self.check_expr(rhs);
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    self.check_expr(element);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.check_expr(value);
                }
            }
            HirExprKind::Tuple(elements) => {
                for element in elements {
                    self.check_expr(element);
                }
            }
            HirExprKind::Clausura(_, _, body) => self.check_expr(body),
            HirExprKind::Cede(expr)
            | HirExprKind::Qua(expr, _)
            | HirExprKind::Ref(_, expr)
            | HirExprKind::Deref(expr) => self.check_expr(expr),
            HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt, HirStmtKind};
    use crate::lexer::Span;

    fn span() -> Span {
        Span::default()
    }

    fn lit_expr() -> HirExpr {
        HirExpr {
            id: crate::hir::HirId(0),
            kind: HirExprKind::Literal(HirLiteral::Int(1)),
            ty: None,
            span: span(),
        }
    }

    #[test]
    fn warns_on_unused_local() {
        let program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(1),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: crate::hir::DefId(1),
                        name: crate::lexer::Symbol(1),
                        ty: None,
                        init: Some(lit_expr()),
                        mutable: false,
                    }),
                    span: span(),
                }],
                expr: None,
                span: span(),
            }),
        };

        let result = lint(&program, &Resolver::new(), &TypeTable::new());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnusedVariable)));
    }

    #[test]
    fn warns_on_unreachable_code() {
        let program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![
                    HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Redde(Some(lit_expr())),
                        span: span(),
                    },
                    HirStmt {
                        id: crate::hir::HirId(2),
                        kind: HirStmtKind::Expr(lit_expr()),
                        span: span(),
                    },
                ],
                expr: None,
                span: span(),
            }),
        };

        let result = lint(&program, &Resolver::new(), &TypeTable::new());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnreachableCode)));
    }

    #[test]
    fn warns_on_unused_import() {
        let program = HirProgram {
            items: vec![HirItem {
                id: crate::hir::HirId(0),
                def_id: crate::hir::DefId(10),
                kind: HirItemKind::Import(HirImport {
                    path: crate::lexer::Symbol(1),
                    items: vec![HirImportItem {
                        def_id: crate::hir::DefId(11),
                        name: crate::lexer::Symbol(2),
                        alias: None,
                    }],
                }),
                span: span(),
            }],
            entry: None,
        };

        let result = lint(&program, &Resolver::new(), &TypeTable::new());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnusedImport)));
    }
}
