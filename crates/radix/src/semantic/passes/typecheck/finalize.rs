//! Final HIR type annotation pass for typechecking.
//!
//! Checking can leave HIR nodes with aliases or inference variables while
//! constraints are still being gathered. Finalization is the cleanup pass that
//! walks the already-checked HIR, resolves checker-local substitutions and
//! semantic aliases, and writes the stable table-local `TypeId`s back onto nodes.
//!
//! RESPONSIBILITY BOUNDARY
//! =======================
//! Finalization does not introduce language behavior. It does not make
//! `ignotum` concrete, infer missing public contracts by policy, or reinterpret
//! `nihil` as optional. If an inference variable cannot be resolved by the
//! constraints already discovered, finalization reports a missing annotation and
//! leaves the erroneous state visible to the caller through diagnostics.

use crate::hir::visit::{walk_expr_mut, walk_function_mut, walk_local_mut, HirVisitorMut};
use crate::lexer::Span;

use super::*;

impl<'a> TypeChecker<'a> {
    /// Resolve all type annotations on the checked HIR.
    pub(super) fn finalize_hir(&mut self, hir: &mut HirProgram) {
        self.visit_program_mut(hir);
    }

    /// Finalize a declaration type that may be part of a public contract.
    ///
    /// Declarations cannot silently keep unresolved inference variables because
    /// later passes and generated code treat their `TypeId`s as stable contract
    /// facts. Missing constraints become missing-annotation diagnostics.
    fn finalize_decl_type(&mut self, ty: &mut Option<TypeId>, span: Span, message: &'static str) {
        if let Some(current) = *ty {
            let resolved = self.resolve_type(current);
            if self.is_infer(resolved) {
                self.error(SemanticErrorKind::MissingTypeAnnotation, message, span);
            }
            *ty = Some(resolved);
        }
    }

    /// Finalize an expression annotation after all surrounding constraints ran.
    fn finalize_expr_type(&mut self, ty: &mut Option<TypeId>, span: Span) {
        if let Some(current) = *ty {
            let resolved = self.resolve_type(current);
            if self.is_infer(resolved) {
                self.error(SemanticErrorKind::MissingTypeAnnotation, "cannot infer expression type", span);
            } else {
                *ty = Some(resolved);
            }
        }
    }

    /// Resolve optional metadata types that do not require an annotation error.
    ///
    /// Handler binding annotations are derived from local control-flow analysis;
    /// if they exist, they should be canonicalized for downstream passes.
    fn resolve_optional_type(&mut self, ty: &mut Option<TypeId>) {
        if let Some(current) = *ty {
            *ty = Some(self.resolve_type(current));
        }
    }
}

impl HirVisitorMut for TypeChecker<'_> {
    /// Finalize item-level type contracts and then descend into bodies.
    fn visit_item_mut(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(function) => self.visit_function_mut(function),
            HirItemKind::Const(const_item) => {
                self.finalize_decl_type(&mut const_item.ty, const_item.value.span, "cannot infer constant type");
                self.visit_expr_mut(&mut const_item.value);
            }
            HirItemKind::Struct(struct_item) => {
                for method in &mut struct_item.methods {
                    self.visit_function_mut(&mut method.func);
                }
            }
            _ => {}
        }
    }

    /// Finalize function return and alternate-exit contracts before body nodes.
    fn visit_function_mut(&mut self, function: &mut HirFunction) {
        let span = function
            .body
            .as_ref()
            .map(|body| body.span)
            .unwrap_or_default();
        self.finalize_decl_type(&mut function.ret_ty, span, "cannot infer return type");
        self.finalize_decl_type(&mut function.err_ty, span, "cannot infer alternate-exit type");

        walk_function_mut(self, function);
    }

    /// Finalize local declaration types before visiting their initializer.
    fn visit_local_mut(&mut self, local: &mut HirLocal) {
        let span = local
            .init
            .as_ref()
            .map(|expr| expr.span)
            .unwrap_or_default();
        self.finalize_decl_type(&mut local.ty, span, "cannot infer variable type");

        walk_local_mut(self, local);
    }

    /// Finalize expression type annotations after inference has converged.
    fn visit_expr_mut(&mut self, expr: &mut HirExpr) {
        self.finalize_expr_type(&mut expr.ty, expr.span);

        walk_expr_mut(self, expr);
    }

    /// Canonicalize handler binding types derived from failable control flow.
    fn visit_cape_mut(&mut self, cape: &mut HirCape) {
        self.resolve_optional_type(&mut cape.binding_ty);

        crate::hir::visit::walk_cape_mut(self, cape);
    }
}
