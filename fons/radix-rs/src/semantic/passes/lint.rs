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
    types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
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
    types: &'a TypeTable,
    warnings: Vec<(WarningKind, String, Span)>,
    errors: Vec<SemanticError>,
    used: FxHashSet<crate::hir::DefId>,
    defs: Vec<(crate::hir::DefId, Span, WarningKind)>,
    imports: Vec<(crate::hir::DefId, Span)>,
    functions: Vec<(crate::hir::DefId, Span)>,
    scope: Vec<rustc_hash::FxHashMap<crate::lexer::Symbol, crate::hir::DefId>>,
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
                self.warnings.push((
                    WarningKind::UnreachableCode,
                    "unreachable code".to_owned(),
                    stmt.span,
                ));
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
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr, in_loop);
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
        self.check_shadowing(
            local.name,
            local.def_id,
            local
                .init
                .as_ref()
                .map(|expr| expr.span)
                .unwrap_or_default(),
        );
        self.insert_name(local.name, local.def_id);
        if let Some(init) = &local.init {
            self.check_expr(init, false);
        }
    }

    fn check_expr(&mut self, expr: &HirExpr, in_loop: bool) {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                self.used.insert(*def_id);
            }
            HirExprKind::Binary(_, lhs, rhs) => {
                self.check_expr(lhs, in_loop);
                self.check_expr(rhs, in_loop);
            }
            HirExprKind::Unary(_, operand) => self.check_expr(operand, in_loop),
            HirExprKind::Call(callee, args) => {
                self.check_expr(callee, in_loop);
                for arg in args {
                    self.check_expr(arg, in_loop);
                }
            }
            HirExprKind::MethodCall(receiver, _name, args) => {
                self.check_expr(receiver, in_loop);
                for arg in args {
                    self.check_expr(arg, in_loop);
                }
            }
            HirExprKind::Field(object, _) => self.check_expr(object, in_loop),
            HirExprKind::Index(object, index) => {
                self.check_expr(object, in_loop);
                self.check_expr(index, in_loop);
            }
            HirExprKind::Block(block) => self.check_block(block, in_loop),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_expr(cond, in_loop);
                self.check_block(then_block, in_loop);
                if let Some(block) = else_block {
                    self.check_block(block, in_loop);
                }
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.check_expr(scrutinee, in_loop);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expr(guard, in_loop);
                    }
                    self.check_expr(&arm.body, in_loop);
                }
            }
            HirExprKind::Loop(block) => self.check_block(block, true),
            HirExprKind::Dum(cond, block) => {
                self.check_expr(cond, in_loop);
                self.check_block(block, true);
            }
            HirExprKind::Itera(_, iter, block) => {
                self.check_expr(iter, in_loop);
                self.check_block(block, true);
            }
            HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.check_expr(lhs, in_loop);
                self.check_expr(rhs, in_loop);
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    self.check_expr(element, in_loop);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.check_expr(value, in_loop);
                }
            }
            HirExprKind::Tuple(elements) => {
                for element in elements {
                    self.check_expr(element, in_loop);
                }
            }
            HirExprKind::Clausura(_, _, body) => self.check_expr(body, in_loop),
            HirExprKind::Cede(expr) | HirExprKind::Ref(_, expr) | HirExprKind::Deref(expr) => {
                self.check_expr(expr, in_loop)
            }
            HirExprKind::Qua(inner, target) => {
                self.check_expr(inner, in_loop);
                if let Some(inner_ty) = inner.ty {
                    if inner_ty == *target {
                        self.warnings.push((
                            WarningKind::UnnecessaryCast,
                            "unnecessary cast".to_owned(),
                            expr.span,
                        ));
                    }
                }
            }
            HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    fn check_shadowing(
        &mut self,
        name: crate::lexer::Symbol,
        def_id: crate::hir::DefId,
        span: Span,
    ) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{HirExpr, HirExprKind, HirLiteral, HirProgram, HirStmt, HirStmtKind};
    use crate::lexer::Span;
    use crate::semantic::Primitive;

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

    #[test]
    fn warns_on_shadowed_variable() {
        let program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![
                    HirStmt {
                        id: crate::hir::HirId(1),
                        kind: HirStmtKind::Local(HirLocal {
                            def_id: crate::hir::DefId(1),
                            name: crate::lexer::Symbol(1),
                            ty: None,
                            init: Some(lit_expr()),
                            mutable: false,
                        }),
                        span: span(),
                    },
                    HirStmt {
                        id: crate::hir::HirId(2),
                        kind: HirStmtKind::Local(HirLocal {
                            def_id: crate::hir::DefId(2),
                            name: crate::lexer::Symbol(1),
                            ty: None,
                            init: Some(lit_expr()),
                            mutable: false,
                        }),
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
            .any(|err| err.kind == SemanticErrorKind::ShadowedVariable));
    }

    #[test]
    fn warns_on_unnecessary_cast() {
        let mut types = TypeTable::new();
        let numerus = types.primitive(Primitive::Numerus);
        let program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(1),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: crate::hir::HirId(2),
                        kind: HirExprKind::Qua(
                            Box::new(HirExpr {
                                id: crate::hir::HirId(3),
                                kind: HirExprKind::Literal(HirLiteral::Int(1)),
                                ty: Some(numerus),
                                span: span(),
                            }),
                            numerus,
                        ),
                        ty: Some(numerus),
                        span: span(),
                    }),
                    span: span(),
                }],
                expr: None,
                span: span(),
            }),
        };

        let result = lint(&program, &Resolver::new(), &types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnnecessaryCast)));
    }

    #[test]
    fn warns_on_unreachable_after_break() {
        let loop_block = HirBlock {
            stmts: vec![
                HirStmt {
                    id: crate::hir::HirId(1),
                    kind: HirStmtKind::Rumpe,
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
        };
        let program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(3),
                    kind: HirStmtKind::Expr(HirExpr {
                        id: crate::hir::HirId(4),
                        kind: HirExprKind::Loop(loop_block),
                        ty: None,
                        span: span(),
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
            .any(|err| err.kind == SemanticErrorKind::Warning(WarningKind::UnreachableCode)));
    }
}
