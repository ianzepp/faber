//! AST to HIR lowering
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Transforms the resolved AST into High-Level Intermediate Representation (HIR),
//! desugaring syntax constructs and embedding resolved DefIds. Operates after
//! name resolution (Pass 2) and before type checking (Pass 4).
//!
//! COMPILER PHASE: HIR Lowering (Pass 3)
//! INPUT: AST with resolved names from Resolver
//! OUTPUT: HirProgram with DefIds embedded; lowering errors
//!
//! WHY: HIR eliminates syntactic sugar (ergo, reddit, inline returns) and
//! normalizes constructs (method calls, implicit returns) so later passes
//! (type checking, borrow analysis) work with simpler structures.
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Explicit Returns: ergo/reddit syntax becomes explicit HirStmtKind::Redde
//!   for simpler control-flow analysis
//! - Method Normalization: `obj.method()` becomes HirExprKind::MethodCall,
//!   distinguishing from field access for type checking
//! - Synthetic DefIds: Loop bindings and catch blocks get fresh DefIds for
//!   type checker to attach types
//! - Error Recovery: Invalid constructs become HirExprKind::Error to allow
//!   continued analysis rather than aborting
//!
//! LOWERING SCOPES
//! ===============
//! The lowerer maintains its own scope stack (local_scopes) separate from the
//! Resolver because:
//! - Resolver scopes track global/module-level definitions
//! - Lowerer scopes track function-local bindings (parameters, locals)
//! - Synthetic DefIds for patterns need local tracking
//!
//! WHY: Function parameters and pattern bindings aren't in the global Resolver
//! scope, so the lowerer creates and tracks them locally.
//!
//! ENTRY POINT HANDLING
//! ====================
//! Top-level statements are separated into:
//! - HirProgram::items - Declarations (functio, gens, ordo, etc.)
//! - HirProgram::entry - Executable statements (from incipit or implicit)
//!
//! WHY: Matches target language structure (Rust's items vs main function).
//!
//! NAMING CONVENTION
//! =================
//! Lowering functions use Latin names matching Faber keywords:
//! - lower_functio() for function declarations
//! - lower_gens() for class declarations (gens)
//! - lower_ordo() for enum declarations (ordo)
//!
//! WHY: Makes it clear which AST construct is being lowered by reading the
//! function name.

mod decl;
mod expr;
mod pattern;
mod stmt;
mod types;

use super::{HirBlock, HirExpr, HirExprKind, HirId, HirItem, HirProgram, HirStmt, HirStmtKind};
use crate::lexer::{Interner, Span, Symbol};
use crate::semantic::{Resolver, TypeTable};
use crate::syntax::{PraeparaBlock, PraeparaKind, ProbaCase, ProbaModifier, ProbandumDecl, Program, Stmt, StmtKind};
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
    /// Current self type for lowering `ego` expressions inside methods.
    current_ego_struct: Option<crate::hir::DefId>,
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
            current_ego_struct: None,
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
                                    ty: Some(
                                        self.types
                                            .array(self.types.primitive(crate::semantic::Primitive::Textus)),
                                    ),
                                    init: None,
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
                    let lowered_items = self.lower_stmt_items(stmt);
                    if !lowered_items.is_empty() {
                        items.extend(lowered_items);
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

    /// Lower a statement to item declarations (top-level declarations)
    fn lower_stmt_items(&mut self, stmt: &Stmt) -> Vec<HirItem> {
        match &stmt.kind {
            StmtKind::Var(decl) => self.lower_varia(stmt, decl).into_iter().collect(),
            StmtKind::Func(decl) => self.lower_functio(stmt, decl).into_iter().collect(),
            StmtKind::Class(decl) => self.lower_gens(stmt, decl).into_iter().collect(),
            StmtKind::Enum(decl) => self.lower_ordo(stmt, decl).into_iter().collect(),
            StmtKind::Union(decl) => self.lower_discretio(stmt, decl).into_iter().collect(),
            StmtKind::Interface(decl) => self.lower_pactum(stmt, decl).into_iter().collect(),
            StmtKind::TypeAlias(decl) => self.lower_typus(stmt, decl).into_iter().collect(),
            StmtKind::Import(decl) => self.lower_importa(stmt, decl).into_iter().collect(),
            StmtKind::Proba(case) => vec![self.lower_proba_item(case, &[])],
            StmtKind::Probandum(suite) => {
                let mut items = Vec::new();
                self.lower_probandum_items(suite, &[], &mut items);
                items
            }
            _ => Vec::new(),
        }
    }

    fn lower_probandum_items(
        &mut self,
        suite: &ProbandumDecl,
        inherited_setup: &[&PraeparaBlock],
        out: &mut Vec<HirItem>,
    ) {
        let mut combined_setup = inherited_setup.to_vec();
        for setup in &suite.body.setup {
            if setup.all {
                combined_setup.push(setup);
            }
        }

        for case in &suite.body.tests {
            out.push(self.lower_proba_item(case, &combined_setup));
        }

        for nested in &suite.body.nested {
            self.lower_probandum_items(nested, &combined_setup, out);
        }
    }

    fn lower_proba_item(&mut self, case: &ProbaCase, inherited_setup: &[&PraeparaBlock]) -> HirItem {
        let def_id = self.next_def_id();
        self.push_scope();

        let mut stmts = Vec::new();
        for setup in inherited_setup {
            if matches!(setup.kind, PraeparaKind::Praepara | PraeparaKind::Praeparabit) {
                stmts.extend(self.lower_block(&setup.body).stmts);
            }
        }
        stmts.extend(self.lower_block(&case.body).stmts);
        for setup in inherited_setup {
            if matches!(setup.kind, PraeparaKind::Postpara | PraeparaKind::Postparabit) {
                stmts.extend(self.lower_block(&setup.body).stmts);
            }
        }

        self.pop_scope();

        let ignored = case
            .modifiers
            .iter()
            .any(|modifier| matches!(modifier, ProbaModifier::Omitte(_) | ProbaModifier::Futurum(_)));
        HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: crate::hir::HirItemKind::Function(crate::hir::HirFunction {
                name: case.name,
                type_params: Vec::new(),
                params: Vec::new(),
                ret_ty: Some(self.types.primitive(crate::semantic::Primitive::Vacuum)),
                body: Some(HirBlock { stmts, expr: None, span: case.span }),
                is_async: false,
                // Reused for test metadata on synthetic proba functions.
                is_generator: ignored,
            }),
            span: case.span,
        }
    }

    /// Lower a block body (ergo/reddit/inline return desugaring)
    fn lower_ergo_body(&mut self, body: &crate::syntax::IfBody) -> HirBlock {
        match body {
            crate::syntax::IfBody::Block(block) => self.lower_block(block),
            crate::syntax::IfBody::Ergo(stmt) => {
                let stmts = stmt::lower_stmt_expanded(self, stmt);
                HirBlock { stmts, expr: None, span: self.current_span }
            }
            crate::syntax::IfBody::InlineReturn(ret) => {
                let stmts = self.lower_inline_return(ret).into_iter().collect();
                HirBlock { stmts, expr: None, span: self.current_span }
            }
        }
    }

    /// Lower inline control-flow to an optional statement.
    ///
    /// WHY: `reddit`, `iacit`, and `moritor` are distinct exits, while `tacet`
    /// is an explicit no-op and should not be rewritten into an implicit return.
    fn lower_inline_return(&mut self, ret: &crate::syntax::InlineReturn) -> Option<HirStmt> {
        let id = self.next_hir_id();
        let (kind, span) = match ret {
            crate::syntax::InlineReturn::Reddit(expr) => {
                let expr_hir = self.lower_expr(expr);
                (HirStmtKind::Redde(Some(expr_hir)), expr.span)
            }
            crate::syntax::InlineReturn::Iacit(expr) => {
                let value = self.lower_expr(expr);
                (
                    HirStmtKind::Expr(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Throw(Box::new(value)),
                        ty: None,
                        span: expr.span,
                    }),
                    expr.span,
                )
            }
            crate::syntax::InlineReturn::Moritor(expr) => {
                let value = self.lower_expr(expr);
                (
                    HirStmtKind::Expr(HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Panic(Box::new(value)),
                        ty: None,
                        span: expr.span,
                    }),
                    expr.span,
                )
            }
            crate::syntax::InlineReturn::Tacet => return None,
        };
        Some(HirStmt { id, kind, span })
    }

    /// Lower a block statement
    fn lower_block(&mut self, block: &crate::syntax::BlockStmt) -> HirBlock {
        self.current_span = block.span;
        let mut stmts = Vec::new();
        self.push_scope();

        for stmt in &block.stmts {
            stmts.extend(stmt::lower_stmt_expanded(self, stmt));
        }
        self.pop_scope();

        HirBlock { stmts, expr: None, span: block.span }
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
