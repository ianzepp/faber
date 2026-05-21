use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn check_function(&mut self, def_id: DefId, func: &mut HirFunction) {
        self.push_scope();
        for param in &func.params {
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            self.insert_binding(param.def_id, param.ty, mutable);
        }
        if let Some(param) = &func.cli_args {
            self.insert_binding(param.def_id, param.ty, false);
        }

        let prev_return = self.current_return;
        let prev_inferred = self.inferred_return;
        self.current_return = func.ret_ty;
        self.inferred_return = None;

        if let Some(body) = &mut func.body {
            self.check_block(body, None);
        }

        let inferred = self.inferred_return.take();
        if func.ret_ty.is_none() {
            let ret_ty = inferred.unwrap_or(self.vacuum_type());
            func.ret_ty = Some(ret_ty);
            if let Some(sig) = self.functions.get_mut(&def_id) {
                sig.ret = ret_ty;
            }
        }

        self.current_return = prev_return;
        self.inferred_return = prev_inferred;
        self.pop_scope();
    }
    pub(super) fn check_const(&mut self, def_id: DefId, const_item: &mut crate::hir::HirConst) {
        let value_ty = self.check_expr(&mut const_item.value);

        let ty = if let Some(annotated) = const_item.ty {
            self.unify(
                value_ty,
                annotated,
                const_item.value.span,
                "constant value does not match annotation",
            );
            annotated
        } else {
            value_ty
        };

        const_item.ty = Some(ty);
        self.consts.insert(def_id, ty);
    }
    pub(super) fn check_item(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(func) => self.check_function(item.def_id, func),
            HirItemKind::Const(const_item) => self.check_const(item.def_id, const_item),
            HirItemKind::Struct(struct_item) => {
                for method in &mut struct_item.methods {
                    self.check_function(method.def_id, &mut method.func);
                }
            }
            _ => {}
        }
    }
}
