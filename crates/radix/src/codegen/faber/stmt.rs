//! Statement and block emission for the canonical Faber backend.
//!
//! This module reconstructs grammar-valid Faber statements from HIR. The goal
//! is normalized source output that can be parsed again, not recovery of the
//! user's exact source layout, comments, or spelling choices. Helpers here
//! recognize HIR shapes that came from compact Faber syntax and rebuild the
//! canonical form only when the shape is unambiguous.
//!
//! INVARIANTS
//! ==========
//! - Blocks preserve statement order and optional tail expressions.
//! - Compact branch output is used only when a block is equivalent to returning
//!   one expression.
//! - `si`/`sin`/`secus` chains are reconstructed from nested conditional HIR
//!   without changing branch semantics.
//! - Expression-level constructs such as `fac`/`dum` loops and `discerne` arms
//!   can reuse these block helpers without reintroducing source-layout trivia.
//! - Catch blocks, `ad` bodies, and control statements are emitted as canonical
//!   Faber syntax accepted by the parser.

use super::CodeWriter;
use crate::hir::{DefId, HirBlock, HirExpr, HirExprKind, HirStmt, HirStmtKind};
use crate::lexer::{Interner, Symbol};
use crate::semantic::TypeTable;
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    /// Recognize the normalized HIR shape for a `fac ... dum` loop.
    ///
    /// The parser represents this as a body followed by a final conditional
    /// break on the negated loop condition. Re-emission only treats that as
    /// `fac`/`dum` when the final block is exactly the synthetic `rumpe`.
    pub(super) fn as_fac_loop<'a>(&self, block: &'a HirBlock) -> Option<(&'a [HirStmt], &'a HirExpr)> {
        let last = block.stmts.last()?;
        let HirStmtKind::Expr(expr) = &last.kind else {
            return None;
        };
        let HirExprKind::Si { cond, then_block, then_catch: None, else_block: None } = &expr.kind else {
            return None;
        };
        let HirExprKind::Unary(crate::hir::HirUnOp::Not, inner) = &cond.kind else {
            return None;
        };
        if then_block.expr.is_some() || then_block.stmts.len() != 1 {
            return None;
        }
        if !matches!(then_block.stmts[0].kind, HirStmtKind::Rumpe) {
            return None;
        }
        Some((&block.stmts[..block.stmts.len() - 1], inner))
    }

    /// Return the expression represented by a single-expression return block.
    ///
    /// This is the gate for compact `ergo redde` and `sic ... secus ...`
    /// output. Blocks with additional statements or mixed tail forms stay in
    /// brace form to avoid changing control-flow meaning.
    pub(super) fn return_expr<'a>(&self, block: &'a HirBlock) -> Option<&'a HirExpr> {
        if block.stmts.is_empty() {
            return block.expr.as_deref();
        }
        if block.expr.is_some() || block.stmts.len() != 1 {
            return None;
        }
        match &block.stmts[0].kind {
            HirStmtKind::Redde(Some(expr)) => Some(expr),
            _ => None,
        }
    }

    /// Detect an else-block that is semantically a `sin` branch.
    ///
    /// HIR nests `else if` as an else block whose tail expression is another
    /// conditional. The canonical Faber backend flattens that shape back into a
    /// `sin` chain when there are no extra statements to preserve.
    pub(super) fn as_sin_branch<'a>(
        &self,
        block: &'a HirBlock,
    ) -> Option<(&'a HirExpr, &'a HirBlock, Option<&'a HirBlock>)> {
        if !block.stmts.is_empty() {
            return None;
        }

        let expr = block.expr.as_ref()?;
        if let HirExprKind::Si { cond, then_block, then_catch: None, else_block } = &expr.kind {
            Some((cond, then_block, else_block.as_ref()))
        } else {
            None
        }
    }

    /// Write a full `si`/`sin`/`secus` statement chain.
    ///
    /// The chain is syntactic normalization only: every `sin` comes from a
    /// nested conditional else-tail, and any else-block that does not fit that
    /// exact shape remains a terminal `secus` block.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_si_chain(
        &self,
        cond: &HirExpr,
        then_block: &HirBlock,
        else_block: Option<&HirBlock>,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        w.write("si ");
        self.write_expr(cond, types, names, interner, w);
        self.write_si_branch_body(then_block, types, names, interner, w);

        let mut next_else = else_block;
        while let Some(block) = next_else {
            if let Some((sin_cond, sin_then, sin_else)) = self.as_sin_branch(block) {
                w.write(" sin ");
                self.write_expr(sin_cond, types, names, interner, w);
                self.write_si_branch_body(sin_then, types, names, interner, w);
                next_else = sin_else;
            } else {
                w.write(" secus");
                self.write_si_branch_body(block, types, names, interner, w);
                break;
            }
        }
    }

    /// Try to write a conditional expression as compact `sic ... secus ...`.
    ///
    /// Returns `false` when either branch needs block syntax. Callers then fall
    /// back to the ordinary conditional form, which preserves grammar validity
    /// without pretending arbitrary blocks are expressions.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_sic_secus_chain(
        &self,
        cond: &HirExpr,
        then_block: &HirBlock,
        else_block: Option<&HirBlock>,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) -> bool {
        if !self.can_write_sic_secus_chain(then_block, else_block) {
            return false;
        }

        let Some(then_expr) = self.return_expr(then_block) else {
            return false;
        };
        let Some(else_block) = else_block else {
            return false;
        };

        self.write_expr_prec(cond, 2, types, names, interner, w);
        w.write(" sic ");
        self.write_expr_prec(then_expr, 2, types, names, interner, w);
        w.write(" secus ");

        if let Some((sin_cond, sin_then, sin_else)) = self.as_sin_branch(else_block) {
            return self.write_sic_secus_chain(sin_cond, sin_then, sin_else, types, names, interner, w);
        }

        let Some(else_expr) = self.return_expr(else_block) else {
            return false;
        };
        self.write_expr_prec(else_expr, 2, types, names, interner, w);
        true
    }

    /// Validate that a whole conditional chain can be rendered compactly.
    pub(super) fn can_write_sic_secus_chain(&self, then_block: &HirBlock, else_block: Option<&HirBlock>) -> bool {
        if self.return_expr(then_block).is_none() {
            return false;
        }

        let Some(else_block) = else_block else {
            return false;
        };

        if let Some((_, sin_then, sin_else)) = self.as_sin_branch(else_block) {
            return self.can_write_sic_secus_chain(sin_then, sin_else);
        }

        self.return_expr(else_block).is_some()
    }

    /// Write either compact `ergo redde` or a braced branch body.
    pub(super) fn write_si_branch_body(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        if let Some(return_expr) = self.return_expr(block) {
            w.write(" ergo redde ");
            self.write_expr(return_expr, types, names, interner, w);
            return;
        }

        w.writeln(" {");
        w.indented(|w| self.write_block(block, types, names, interner, w));
        w.write("}");
    }

    /// Write the catch side of `cape`.
    ///
    /// A leading local statement carries the caught error binding in HIR. When
    /// no such binding exists, the canonical placeholder `_` keeps the output
    /// grammar-valid while preserving that the catch body intentionally ignores
    /// the error value.
    pub(super) fn write_cape_block(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        if let Some(HirStmt { kind: HirStmtKind::Local(local), .. }) = block.stmts.first() {
            w.write(&self.symbol_to_string(local.name, interner));
            w.writeln(" {");
            w.indented(|w| {
                for stmt in block.stmts.iter().skip(1) {
                    self.write_stmt(stmt, types, names, interner, w);
                }
                if let Some(expr) = &block.expr {
                    self.write_expr(expr, types, names, interner, w);
                    w.newline();
                }
            });
            w.write("}");
            return;
        }

        w.writeln("_ {");
        w.indented(|w| self.write_block(block, types, names, interner, w));
        w.write("}");
    }

    /// Emit an `ad` body after removing synthetic binding setup statements.
    ///
    /// The `ad` header owns the resource binding in source syntax. HIR keeps
    /// that binding in the body as locals as well, so the backend skips those
    /// setup statements to avoid duplicating them in canonical output.
    pub(super) fn write_ad_block(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
        binding: Option<&crate::hir::HirAdBinding>,
    ) {
        let skip = if binding.is_some() {
            1 + usize::from(binding.and_then(|binding| binding.alias).is_some())
        } else {
            0
        };
        for stmt in block.stmts.iter().skip(skip) {
            self.write_stmt(stmt, types, names, interner, w);
        }
        if let Some(expr) = &block.expr {
            self.write_expr(expr, types, names, interner, w);
            w.newline();
        }
    }

    /// Emit a block's statements followed by its tail expression, if present.
    ///
    /// Tail expressions are preserved as expression statements in generated
    /// Faber because this backend emits normalized source text, not lowered
    /// target-language control flow.
    pub(super) fn write_block(
        &self,
        block: &HirBlock,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        for stmt in &block.stmts {
            self.write_stmt(stmt, types, names, interner, w);
        }
        if let Some(expr) = &block.expr {
            self.write_expr(expr, types, names, interner, w);
            w.newline();
        }
    }

    /// Emit one canonical Faber statement.
    ///
    /// The statement surface here is intentionally direct: locals, resource
    /// blocks, returns, loop controls, and expression statements are printed in
    /// parser-valid Faber forms using HIR-owned semantic names and types.
    pub(super) fn write_stmt(
        &self,
        stmt: &HirStmt,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
        w: &mut CodeWriter,
    ) {
        match &stmt.kind {
            HirStmtKind::Local(local) => {
                if local.mutable {
                    w.write("varia ");
                } else {
                    w.write("fixum ");
                }
                if let Some(ty) = local.ty {
                    w.write(&self.type_to_faber(ty, types, names, interner));
                    w.write(" ");
                }
                w.write(&self.symbol_to_string(local.name, interner));
                if let Some(init) = &local.init {
                    w.write(" ← ");
                    self.write_expr(init, types, names, interner, w);
                }
                w.newline();
            }
            HirStmtKind::Expr(expr) => {
                self.write_expr(expr, types, names, interner, w);
                w.newline();
            }
            HirStmtKind::Ad(ad) => {
                w.write("ad ");
                self.write_symbol_literal(ad.path, interner, w);
                w.write(" (");
                for (idx, arg) in ad.args.iter().enumerate() {
                    if idx > 0 {
                        w.write(", ");
                    }
                    self.write_expr(arg, types, names, interner, w);
                }
                w.write(")");
                if let Some(binding) = &ad.binding {
                    let _ = binding.verb;
                    w.write(" → ");
                    w.write(&self.type_to_faber(binding.ty, types, names, interner));
                    w.write(" ");
                    w.write(&self.symbol_to_string(binding.name, interner));
                    if let Some(alias) = binding.alias {
                        w.write(" ut ");
                        w.write(&self.symbol_to_string(alias, interner));
                    }
                }
                if let Some(err_ty) = ad.err_ty {
                    w.write(" ⇥ ");
                    w.write(&self.type_to_faber(err_ty, types, names, interner));
                }
                if let Some(body) = &ad.body {
                    w.writeln(" {");
                    w.indented(|w| self.write_ad_block(body, types, names, interner, w, ad.binding.as_ref()));
                    w.write("}");
                }
                if let Some(catch) = &ad.catch {
                    w.write(" cape ");
                    self.write_cape_block(catch, types, names, interner, w);
                }
                w.newline();
            }
            HirStmtKind::Redde(value) => {
                w.write("redde");
                if let Some(expr) = value {
                    w.write(" ");
                    self.write_expr(expr, types, names, interner, w);
                }
                w.newline();
            }
            HirStmtKind::Rumpe => {
                w.writeln("rumpe");
            }
            HirStmtKind::Perge => {
                w.writeln("perge");
            }
            HirStmtKind::Tacet => {
                w.writeln("tacet");
            }
        }
    }
}
