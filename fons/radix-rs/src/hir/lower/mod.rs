//! AST to HIR lowering
//!
//! Transforms the AST into a simplified HIR representation.
//! Uses Latin function names matching Faber keywords.

mod decl;
mod expr;
mod pattern;
mod stmt;
mod types;

use super::{
    HirBlock, HirExpr, HirExprKind, HirId, HirItem, HirItemKind, HirProgram, HirStmt, HirStmtKind,
};
use crate::lexer::{Span, Symbol};
use crate::semantic::Resolver;
use crate::syntax::{Program, Stmt, StmtKind};

/// Lowerer state for AST to HIR transformation
pub struct Lowerer<'a> {
    /// Resolver for name resolution
    resolver: &'a Resolver,
    /// Next HIR ID to assign
    next_id: u32,
    /// Collected errors during lowering
    errors: Vec<LowerError>,
    /// Current span for error reporting
    current_span: Span,
}

/// Lowering error
#[derive(Debug, Clone)]
pub struct LowerError {
    pub message: String,
    pub span: Span,
}

impl<'a> Lowerer<'a> {
    /// Create a new lowerer with the given resolver
    pub fn new(resolver: &'a Resolver) -> Self {
        Self {
            resolver,
            next_id: 0,
            errors: Vec::new(),
            current_span: Span::default(),
        }
    }

    /// Lower a program to HIR
    pub fn lower_program(&mut self, program: &Program) -> HirProgram {
        let mut items = Vec::new();
        let mut entry = None;

        for stmt in &program.stmts {
            self.current_span = stmt.span;

            match &stmt.kind {
                StmtKind::Incipit(entry_stmt) => {
                    // Entry point gets special treatment
                    let block = self.lower_ergo_body(&entry_stmt.body);
                    entry = Some(block);
                }
                _ => {
                    if let Some(item) = self.lower_stmt_item(stmt) {
                        items.push(item);
                    }
                }
            }
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

    /// Record an error
    fn error(&mut self, message: impl Into<String>) {
        self.errors.push(LowerError {
            message: message.into(),
            span: self.current_span,
        });
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
                HirBlock {
                    stmts,
                    expr: None,
                    span: self.current_span,
                }
            }
            crate::syntax::IfBody::InlineReturn(ret) => {
                let stmt = self.lower_inline_return(ret);
                HirBlock {
                    stmts: vec![stmt],
                    expr: None,
                    span: self.current_span,
                }
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

        for stmt in &block.stmts {
            let hir_stmt = self.lower_stmt(stmt);
            stmts.push(hir_stmt);
        }

        HirBlock {
            stmts,
            expr: None,
            span: block.span,
        }
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
pub fn lower(program: &Program, resolver: &Resolver) -> (HirProgram, Vec<LowerError>) {
    let mut lowerer = Lowerer::new(resolver);
    let hir = lowerer.lower_program(program);
    let errors = lowerer.take_errors();
    (hir, errors)
}
