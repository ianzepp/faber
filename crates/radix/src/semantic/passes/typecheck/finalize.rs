use crate::hir::visit::{walk_expr_mut, walk_function_mut, walk_local_mut, HirVisitorMut};
use crate::lexer::Span;

use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn finalize_hir(&mut self, hir: &mut HirProgram) {
        self.visit_program_mut(hir);
    }

    fn finalize_decl_type(&mut self, ty: &mut Option<TypeId>, span: Span, message: &'static str) {
        if let Some(current) = *ty {
            let resolved = self.resolve_type(current);
            if self.is_infer(resolved) {
                self.error(SemanticErrorKind::MissingTypeAnnotation, message, span);
            }
            *ty = Some(resolved);
        }
    }

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

    fn resolve_optional_type(&mut self, ty: &mut Option<TypeId>) {
        if let Some(current) = *ty {
            *ty = Some(self.resolve_type(current));
        }
    }
}

impl HirVisitorMut for TypeChecker<'_> {
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

    fn visit_local_mut(&mut self, local: &mut HirLocal) {
        let span = local
            .init
            .as_ref()
            .map(|expr| expr.span)
            .unwrap_or_default();
        self.finalize_decl_type(&mut local.ty, span, "cannot infer variable type");

        walk_local_mut(self, local);
    }

    fn visit_expr_mut(&mut self, expr: &mut HirExpr) {
        self.finalize_expr_type(&mut expr.ty, expr.span);

        walk_expr_mut(self, expr);
    }

    fn visit_cape_mut(&mut self, cape: &mut HirCape) {
        self.resolve_optional_type(&mut cape.binding_ty);

        crate::hir::visit::walk_cape_mut(self, cape);
    }
}
