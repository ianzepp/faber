//! Statement and block typing for local control surfaces inside HIR bodies.
//!
//! This module owns the places where typechecking mutates lexical binding
//! state: local declarations, block scopes, and `redde` statements. Expression
//! checking handles most value forms, but statements decide when a synthesized
//! value is intentionally ignored, when a local declaration needs a fresh
//! inference variable, and how return statements contribute to the enclosing
//! function's normal result type.
//!
//! INVARIANTS
//! ==========
//! - A local binding is inserted only after its declared or inferred type has
//!   been established.
//! - `redde` contributes only to the normal return channel. Alternate exits are
//!   checked through `current_error` in expression/control-flow code.
//! - Blocks always introduce a lexical scope and type to their trailing
//!   expression when present, otherwise to `vacuum`.
//! - Ignored expression statements must not leave unresolved inference variables
//!   dangling; finalization should only see genuinely unresolved declarations or
//!   expressions.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Validate a `redde` statement against the current normal return contract.
    ///
    /// Annotated functions use their declared return type as an expected type so
    /// nested expressions can benefit from top-down information. Unannotated
    /// functions collect the first observed return type and unify later returns
    /// against it; `check_function` writes the final type back to the function
    /// signature after the body has been checked.
    pub(super) fn check_return(&mut self, value: Option<&mut HirExpr>, span: crate::lexer::Span) {
        let value_ty = match value {
            Some(expr) => {
                if let Some(expected) = self.current_return {
                    self.check_expr_with_expected(expr, Some(expected))
                } else {
                    self.check_expr(expr)
                }
            }
            None => self.vacuum_type(),
        };

        if let Some(expected) = self.current_return {
            self.unify(value_ty, expected, span, "return type does not match function signature");
            return;
        }

        match self.inferred_return {
            None => self.inferred_return = Some(value_ty),
            Some(existing) => {
                self.unify(value_ty, existing, span, "incompatible return types");
            }
        }
    }

    /// Establish a local declaration's type and install its binding.
    ///
    /// The four local shapes deliberately map to different inference policies:
    /// annotated initializers are checked bidirectionally, bare annotations are
    /// accepted as-is, initializer-only locals synthesize from the expression,
    /// and declarations without either side receive a fresh inference variable
    /// that must be resolved before finalization.
    pub(super) fn check_local(&mut self, local: &mut HirLocal) {
        let inferred = match (&local.ty, &mut local.init) {
            (Some(ty), Some(init)) => {
                let init_ty = self.check_expr_with_expected(init, Some(*ty));
                self.unify(init_ty, *ty, init.span, "initializer does not match annotation");
                *ty
            }
            (Some(ty), None) => *ty,
            (None, Some(init)) => self.check_expr(init),
            (None, None) => self.fresh_infer(),
        };

        if local.ty.is_none() {
            local.ty = Some(inferred);
        }

        self.insert_binding(local.def_id, inferred, local.mutable);
    }

    /// Check one statement for side effects on type state and lexical scope.
    ///
    /// Most expression-like constructs delegate to expression checking. The
    /// statement layer mainly enforces that discarded values are resolved and
    /// that `redde` is routed through the function-return accumulator instead of
    /// being treated as an ordinary expression result.
    pub(super) fn check_stmt(&mut self, stmt: &mut HirStmt) {
        match &mut stmt.kind {
            HirStmtKind::Local(local) => self.check_local(local),
            HirStmtKind::Expr(expr) => {
                let expr_ty = self.check_expr(expr);
                if self.is_infer(self.resolve_type(expr_ty)) {
                    let vacuum = self.vacuum_type();
                    self.unify(expr_ty, vacuum, expr.span, "ignored expression result must resolve");
                }
            }
            HirStmtKind::Ad(ad) => {
                for arg in &mut ad.args {
                    self.check_expr(arg);
                }
                if let Some(body) = &mut ad.body {
                    self.check_block(body, None);
                }
                if let Some(catch) = &mut ad.catch {
                    self.check_block(catch, None);
                }
            }
            HirStmtKind::Redde(value) => self.check_return(value.as_mut(), stmt.span),
            HirStmtKind::Rumpe | HirStmtKind::Perge | HirStmtKind::Tacet => {}
        }
    }

    /// Check a lexical block and return its value type.
    ///
    /// Blocks are expression-compatible in Faber, so callers can pass an
    /// expected type that guides the trailing expression. Non-expression blocks
    /// produce `vacuum`; this keeps statement-only bodies explicit without
    /// inventing a bottom type for control-flow statements.
    pub(super) fn check_block(&mut self, block: &mut HirBlock, expected: Option<TypeId>) -> TypeId {
        self.push_scope();
        for stmt in &mut block.stmts {
            self.check_stmt(stmt);
        }
        let ty = if let Some(expr) = &mut block.expr {
            self.check_expr_with_expected(expr, expected)
        } else {
            self.vacuum_type()
        };
        self.pop_scope();
        ty
    }
}
