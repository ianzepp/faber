use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn finalize_hir(&mut self, hir: &mut HirProgram) {
        for item in &mut hir.items {
            self.finalize_item(item);
        }

        if let Some(entry) = &mut hir.entry {
            self.finalize_block(entry);
        }
    }

    pub(super) fn finalize_item(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(func) => self.finalize_function(func),
            HirItemKind::Const(const_item) => {
                if let Some(ty) = const_item.ty {
                    let resolved = self.resolve_type(ty);
                    if self.is_infer(resolved) {
                        self.error(
                            SemanticErrorKind::MissingTypeAnnotation,
                            "cannot infer constant type",
                            const_item.value.span,
                        );
                    }
                    const_item.ty = Some(resolved);
                }
                self.finalize_expr(&mut const_item.value);
            }
            HirItemKind::Struct(struct_item) => {
                for method in &mut struct_item.methods {
                    self.finalize_function(&mut method.func);
                }
            }
            _ => {}
        }
    }

    pub(super) fn finalize_function(&mut self, func: &mut HirFunction) {
        if let Some(ret) = func.ret_ty {
            let resolved = self.resolve_type(ret);
            if self.is_infer(resolved) {
                let span = func.body.as_ref().map(|body| body.span).unwrap_or_default();
                self.error(SemanticErrorKind::MissingTypeAnnotation, "cannot infer return type", span);
            }
            func.ret_ty = Some(resolved);
        }

        if let Some(body) = &mut func.body {
            self.finalize_block(body);
        }
    }

    pub(super) fn finalize_block(&mut self, block: &mut HirBlock) {
        for stmt in &mut block.stmts {
            self.finalize_stmt(stmt);
        }
        if let Some(expr) = &mut block.expr {
            self.finalize_expr(expr);
        }
    }

    pub(super) fn finalize_stmt(&mut self, stmt: &mut HirStmt) {
        match &mut stmt.kind {
            HirStmtKind::Local(local) => {
                if let Some(ty) = local.ty {
                    let resolved = self.resolve_type(ty);
                    if self.is_infer(resolved) {
                        self.error(
                            SemanticErrorKind::MissingTypeAnnotation,
                            "cannot infer variable type",
                            local
                                .init
                                .as_ref()
                                .map(|expr| expr.span)
                                .unwrap_or_default(),
                        );
                    }
                    local.ty = Some(resolved);
                }
                if let Some(init) = &mut local.init {
                    self.finalize_expr(init);
                }
            }
            HirStmtKind::Expr(expr) => self.finalize_expr(expr),
            HirStmtKind::Ad(ad) => {
                for arg in &mut ad.args {
                    self.finalize_expr(arg);
                }
                if let Some(body) = &mut ad.body {
                    self.finalize_block(body);
                }
                if let Some(catch) = &mut ad.catch {
                    self.finalize_block(catch);
                }
            }
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.finalize_expr(expr);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    pub(super) fn finalize_expr(&mut self, expr: &mut HirExpr) {
        let resolved = expr.ty.map(|ty| self.resolve_type(ty));
        if let Some(ty) = resolved {
            if self.is_infer(ty) {
                self.error(
                    SemanticErrorKind::MissingTypeAnnotation,
                    "cannot infer expression type",
                    expr.span,
                );
            } else {
                expr.ty = Some(ty);
            }
        }

        match &mut expr.kind {
            HirExprKind::Binary(_, lhs, rhs) => {
                self.finalize_expr(lhs);
                self.finalize_expr(rhs);
            }
            HirExprKind::Unary(_, operand) => self.finalize_expr(operand),
            HirExprKind::Call(callee, args) => {
                self.finalize_expr(callee);
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.finalize_expr(receiver);
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::Field(object, _) => self.finalize_expr(object),
            HirExprKind::Index(object, index) => {
                self.finalize_expr(object);
                self.finalize_expr(index);
            }
            HirExprKind::OptionalChain(object, chain) => {
                self.finalize_expr(object);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(_) => {}
                    crate::hir::HirOptionalChainKind::Index(index) => self.finalize_expr(index),
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.finalize_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.finalize_expr(object);
                match chain {
                    crate::hir::HirNonNullKind::Member(_) => {}
                    crate::hir::HirNonNullKind::Index(index) => self.finalize_expr(index),
                    crate::hir::HirNonNullKind::Call(args) => {
                        for arg in args {
                            self.finalize_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.finalize_expr(source);
                if let Some(filter) = filter {
                    if let crate::hir::HirCollectionFilterKind::Condition(cond) = &mut filter.kind {
                        self.finalize_expr(cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &mut transform.arg {
                        self.finalize_expr(arg);
                    }
                }
            }
            HirExprKind::Block(block) => self.finalize_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.finalize_expr(cond);
                self.finalize_block(then_block);
                if let Some(block) = else_block {
                    self.finalize_block(block);
                }
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                for scrutinee in scrutinees {
                    self.finalize_expr(scrutinee);
                }
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.finalize_expr(guard);
                    }
                    self.finalize_expr(&mut arm.body);
                }
            }
            HirExprKind::Loop(block) => self.finalize_block(block),
            HirExprKind::Dum(cond, block) => {
                self.finalize_expr(cond);
                self.finalize_block(block);
            }
            HirExprKind::Itera(_, _, _, iter, block) => {
                self.finalize_expr(iter);
                self.finalize_block(block);
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                self.finalize_expr(start);
                self.finalize_expr(end);
                if let Some(step) = step {
                    self.finalize_expr(step);
                }
            }
            HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.finalize_expr(lhs);
                self.finalize_expr(rhs);
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    match element {
                        HirArrayElement::Expr(expr) | HirArrayElement::Spread(expr) => self.finalize_expr(expr),
                    }
                }
            }
            HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.finalize_expr(element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.finalize_expr(cond);
                if let Some(message) = message {
                    self.finalize_expr(message);
                }
            }
            HirExprKind::Panic(value) | HirExprKind::Throw(value) => self.finalize_expr(value),
            HirExprKind::Tempta { body, catch, finally } => {
                self.finalize_block(body);
                if let Some(catch) = catch {
                    self.finalize_block(catch);
                }
                if let Some(finally) = finally {
                    self.finalize_block(finally);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.finalize_expr(value);
                }
            }
            HirExprKind::Clausura(_, _, body) => self.finalize_expr(body),
            HirExprKind::Verte { source, entries, .. } => {
                self.finalize_expr(source);
                if let Some(entries) = entries {
                    for field in entries {
                        self.finalize_object_field(field);
                    }
                }
            }
            HirExprKind::Conversio { source, fallback, .. } => {
                self.finalize_expr(source);
                if let Some(fallback) = fallback {
                    self.finalize_expr(fallback);
                }
            }
            HirExprKind::Cede(expr) | HirExprKind::Ref(_, expr) | HirExprKind::Deref(expr) => self.finalize_expr(expr),
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    pub(super) fn finalize_object_field(&mut self, field: &mut HirObjectField) {
        match &mut field.key {
            HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => self.finalize_expr(expr),
            HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
        }
        if let Some(value) = &mut field.value {
            self.finalize_expr(value);
        }
    }
}
