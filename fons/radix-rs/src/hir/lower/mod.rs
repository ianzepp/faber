//! AST to HIR lowering
//!
//! Transforms the AST into a simplified HIR representation.
//! Uses Latin function names matching Faber keywords.

mod decl;
mod expr;
mod pattern;
mod stmt;
mod types;

use super::{HirBlock, HirExpr, HirExprKind, HirId, HirItem, HirProgram, HirStmt, HirStmtKind};
use crate::lexer::{Interner, Span, Symbol};
use crate::semantic::{Resolver, TypeTable};
use crate::syntax::{Program, Stmt, StmtKind};
use rustc_hash::FxHashMap;

/// Lowerer state for AST to HIR transformation
pub struct Lowerer<'a> {
    /// Resolver for name resolution
    resolver: &'a Resolver,
    /// Type table for interning types
    types: &'a mut TypeTable,
    /// Interner for resolving symbols
    interner: &'a Interner,
    /// Next HIR ID to assign
    next_id: u32,
    /// Next synthetic DefId to assign
    next_def_id: u32,
    /// Collected errors during lowering
    errors: Vec<LowerError>,
    /// Current span for error reporting
    current_span: Span,
    /// Lowering-local scopes for parameters and local bindings
    local_scopes: Vec<FxHashMap<Symbol, crate::hir::DefId>>,
}

/// Lowering error
#[derive(Debug, Clone)]
pub struct LowerError {
    pub message: String,
    pub span: Span,
}

impl<'a> Lowerer<'a> {
    /// Create a new lowerer with the given resolver
    pub fn new(resolver: &'a Resolver, types: &'a mut TypeTable, interner: &'a Interner) -> Self {
        Self {
            resolver,
            types,
            interner,
            next_id: 0,
            next_def_id: 1_000_000,
            errors: Vec::new(),
            current_span: Span::default(),
            local_scopes: Vec::new(),
        }
    }

    /// Lower a program to HIR
    pub fn lower_program(&mut self, program: &Program) -> HirProgram {
        let mut items = Vec::new();
        let mut entry = None;
        let mut implicit_entry_stmts = Vec::new();
        let mut implicit_entry_scope = false;

        for stmt in &program.stmts {
            self.current_span = stmt.span;

            match &stmt.kind {
                StmtKind::Incipit(entry_stmt) => {
                    // Entry point gets special treatment
                    self.push_scope();
                    let mut args_binding = None;
                    if let Some(args) = &entry_stmt.args {
                        let def_id = self.next_def_id();
                        self.bind_local(args.name, def_id);
                        args_binding = Some((args.name, args.span, def_id));
                    }
                    let mut block = self.lower_ergo_body(&entry_stmt.body);
                    if let Some((name, span, def_id)) = args_binding {
                        block.stmts.insert(
                            0,
                            HirStmt {
                                id: self.next_hir_id(),
                                kind: HirStmtKind::Local(crate::hir::HirLocal {
                                    def_id,
                                    name,
                                    ty: None,
                                    init: Some(HirExpr {
                                        id: self.next_hir_id(),
                                        kind: HirExprKind::Error,
                                        ty: None,
                                        span,
                                    }),
                                    mutable: false,
                                }),
                                span,
                            },
                        );
                    }
                    self.pop_scope();
                    entry = Some(block);
                }
                _ => {
                    if let Some(item) = self.lower_stmt_item(stmt) {
                        items.push(item);
                    } else {
                        if !implicit_entry_scope {
                            self.push_scope();
                            implicit_entry_scope = true;
                        }
                        implicit_entry_stmts.push(stmt::lower_stmt(self, stmt));
                    }
                }
            }
        }

        if implicit_entry_scope {
            self.pop_scope();
        }

        if entry.is_none() && !implicit_entry_stmts.is_empty() {
            let span = implicit_entry_stmts
                .first()
                .map(|stmt| stmt.span)
                .unwrap_or_else(Span::default);
            entry = Some(HirBlock { stmts: implicit_entry_stmts, expr: None, span });
        }

        HirProgram { items, entry }
    }

    /// Take collected errors
    pub fn take_errors(&mut self) -> Vec<LowerError> {
        std::mem::take(&mut self.errors)
    }

    /// Generate a fresh HIR ID
    fn next_hir_id(&mut self) -> HirId {
        let id = self.next_id;
        self.next_id += 1;
        HirId(id)
    }

    /// Generate a fresh DefId (synthetic, for lowering only)
    pub(super) fn next_def_id(&mut self) -> crate::hir::DefId {
        let id = self.next_def_id;
        self.next_def_id += 1;
        crate::hir::DefId(id)
    }

    /// Resolve a DefId or generate a synthetic one
    pub(super) fn def_id_for(&mut self, name: Symbol) -> crate::hir::DefId {
        self.resolver
            .lookup(name)
            .unwrap_or_else(|| self.next_def_id())
    }

    pub(super) fn push_scope(&mut self) {
        self.local_scopes.push(FxHashMap::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.local_scopes.pop();
    }

    pub(super) fn bind_local(&mut self, name: Symbol, def_id: crate::hir::DefId) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name, def_id);
        }
    }

    pub(super) fn lookup_name(&self, name: Symbol) -> Option<crate::hir::DefId> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(def_id) = scope.get(&name) {
                return Some(*def_id);
            }
        }
        self.resolver.lookup(name)
    }

    /// Record an error
    fn error(&mut self, message: impl Into<String>) {
        self.errors
            .push(LowerError { message: message.into(), span: self.current_span });
    }

    /// Lower a statement to an item (top-level declarations)
    fn lower_stmt_item(&mut self, stmt: &Stmt) -> Option<HirItem> {
        match &stmt.kind {
            StmtKind::Var(decl) => self.lower_varia(stmt, decl),
            StmtKind::Func(decl) => self.lower_functio(stmt, decl),
            StmtKind::Class(decl) => self.lower_gens(stmt, decl),
            StmtKind::Enum(decl) => self.lower_ordo(stmt, decl),
            StmtKind::Union(decl) => self.lower_discretio(stmt, decl),
            StmtKind::Interface(decl) => self.lower_pactum(stmt, decl),
            StmtKind::TypeAlias(decl) => self.lower_typus(stmt, decl),
            StmtKind::Import(decl) => self.lower_importa(stmt, decl),
            // TODO: Handle Probandum and Proba appropriately
            _ => {
                // For now, skip non-item statements at top level
                // They might be test cases or other constructs handled differently
                None
            }
        }
    }

    /// Lower a block body (ergo/reddit/inline return desugaring)
    fn lower_ergo_body(&mut self, body: &crate::syntax::IfBody) -> HirBlock {
        match body {
            crate::syntax::IfBody::Block(block) => self.lower_block(block),
            crate::syntax::IfBody::Ergo(stmt) => {
                let stmts = vec![self.lower_stmt(stmt)];
                HirBlock { stmts, expr: None, span: self.current_span }
            }
            crate::syntax::IfBody::InlineReturn(ret) => {
                let stmt = self.lower_inline_return(ret);
                HirBlock { stmts: vec![stmt], expr: None, span: self.current_span }
            }
        }
    }

    /// Lower inline return to statement
    fn lower_inline_return(&mut self, ret: &crate::syntax::InlineReturn) -> HirStmt {
        let id = self.next_hir_id();
        let (kind, span) = match ret {
            crate::syntax::InlineReturn::Reddit(expr) => {
                let expr_hir = self.lower_expr(expr);
                (HirStmtKind::Redde(Some(expr_hir)), expr.span)
            }
            crate::syntax::InlineReturn::Iacit(expr) => {
                let expr_hir = self.lower_expr(expr);
                (HirStmtKind::Redde(Some(expr_hir)), expr.span)
            }
            crate::syntax::InlineReturn::Moritor(expr) => {
                let expr_hir = self.lower_expr(expr);
                (HirStmtKind::Redde(Some(expr_hir)), expr.span)
            }
            crate::syntax::InlineReturn::Tacet => (HirStmtKind::Redde(None), self.current_span),
        };
        HirStmt { id, kind, span }
    }

    /// Lower a block statement
    fn lower_block(&mut self, block: &crate::syntax::BlockStmt) -> HirBlock {
        self.current_span = block.span;
        let mut stmts = Vec::new();
        self.push_scope();

        for stmt in &block.stmts {
            let hir_stmt = self.lower_stmt(stmt);
            stmts.push(hir_stmt);
        }
        self.pop_scope();

        HirBlock { stmts, expr: None, span: block.span }
    }

    /// Lower a statement (delegates to stmt.rs)
    fn lower_stmt(&mut self, stmt: &Stmt) -> HirStmt {
        stmt::lower_stmt(self, stmt)
    }

    /// Lower an expression (delegates to expr.rs)
    fn lower_expr(&mut self, expr: &crate::syntax::Expr) -> HirExpr {
        expr::lower_expr(self, expr)
    }
}

/// Lower AST to HIR
///
/// This is performed after name resolution, so the resolver
/// is passed in to provide DefId mappings.
pub fn lower(
    program: &Program,
    resolver: &Resolver,
    types: &mut TypeTable,
    interner: &Interner,
) -> (HirProgram, Vec<LowerError>) {
    let mut lowerer = Lowerer::new(resolver, types, interner);
    let hir = lowerer.lower_program(program);
    let errors = lowerer.take_errors();
    (hir, errors)
}
