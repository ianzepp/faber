//! Pass 4: Borrow checking
//!
//! Validates ownership and borrowing rules for Rust target.
//! Only runs when targeting Rust.

use crate::hir::{
    DefId, HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirProgram, HirStmt,
    HirStmtKind,
};
use crate::semantic::{ParamMode, Resolver, SemanticError, SemanticErrorKind, Type, TypeTable};
use rustc_hash::FxHashMap;

/// Analyze borrowing and ownership
pub fn analyze(
    hir: &HirProgram,
    _resolver: &Resolver,
    types: &TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut checker = BorrowChecker::new(types);
    checker.check_program(hir);

    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

#[derive(Clone, Copy, Default)]
struct BorrowState {
    moved: bool,
    shared: u32,
    mutable: bool,
}

#[derive(Clone, Copy)]
enum BorrowKind {
    Shared,
    Mutable,
}

struct BorrowScope {
    borrows: Vec<(DefId, BorrowKind)>,
}

struct BorrowChecker<'a> {
    types: &'a TypeTable,
    states: FxHashMap<DefId, BorrowState>,
    scopes: Vec<BorrowScope>,
    errors: Vec<SemanticError>,
}

impl<'a> BorrowChecker<'a> {
    fn new(types: &'a TypeTable) -> Self {
        Self {
            types,
            states: FxHashMap::default(),
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn check_program(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            self.check_item(item);
        }

        if let Some(entry) = &hir.entry {
            self.reset();
            self.check_block(entry);
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
            HirItemKind::Const(const_item) => {
                self.reset();
                self.check_expr(&const_item.value);
            }
            _ => {}
        }
    }

    fn check_function(&mut self, func: &HirFunction) {
        self.reset();
        for param in &func.params {
            self.ensure_state(param.def_id);
        }
        if let Some(body) = &func.body {
            self.check_block(body);
        }
    }

    fn reset(&mut self) {
        self.states.clear();
        self.scopes.clear();
    }

    fn check_block(&mut self, block: &HirBlock) {
        self.push_scope();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(expr) = &block.expr {
            self.check_expr(expr);
        }
        self.pop_scope();
    }

    fn check_stmt(&mut self, stmt: &HirStmt) {
        match &stmt.kind {
            HirStmtKind::Local(local) => {
                self.ensure_state(local.def_id);
                if let Some(init) = &local.init {
                    self.check_expr(init);
                }
            }
            HirStmtKind::Expr(expr) => self.check_expr(expr),
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.check_expr(expr);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    fn check_expr(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Path(def_id) => self.read_use(*def_id, expr.span),
            HirExprKind::Literal(_) => {}
            HirExprKind::Binary(_, lhs, rhs) => {
                self.check_expr(lhs);
                self.check_expr(rhs);
            }
            HirExprKind::Unary(_, operand) => self.check_expr(operand),
            HirExprKind::Call(callee, args) => {
                self.check_expr(callee);
                self.check_call_args(callee, args);
            }
            HirExprKind::MethodCall(receiver, _name, args) => {
                self.check_expr(receiver);
                for arg in args {
                    self.check_expr(arg);
                }
            }
            HirExprKind::Field(object, _) => self.check_expr(object),
            HirExprKind::Index(object, index) => {
                self.check_expr(object);
                self.check_expr(index);
            }
            HirExprKind::Block(block) => self.check_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_expr(cond);
                self.check_block(then_block);
                if let Some(block) = else_block {
                    self.check_block(block);
                }
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.check_expr(scrutinee);
                for arm in arms {
                    self.push_scope();
                    if let Some(guard) = &arm.guard {
                        self.check_expr(guard);
                    }
                    self.check_expr(&arm.body);
                    self.pop_scope();
                }
            }
            HirExprKind::Loop(block) => self.check_block(block),
            HirExprKind::Dum(cond, block) => {
                self.check_expr(cond);
                self.check_block(block);
            }
            HirExprKind::Itera(binding, iter, block) => {
                self.check_expr(iter);
                self.ensure_state(*binding);
                self.check_block(block);
            }
            HirExprKind::Assign(target, value) => {
                self.check_lvalue(target);
                self.check_move_expr(value);
            }
            HirExprKind::AssignOp(_, target, value) => {
                self.check_lvalue(target);
                self.check_expr(value);
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    self.check_expr(element);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.check_expr(value);
                }
            }
            HirExprKind::Tuple(elements) => {
                for element in elements {
                    self.check_expr(element);
                }
            }
            HirExprKind::Clausura(params, _, body) => {
                self.push_scope();
                for param in params {
                    self.ensure_state(param.def_id);
                }
                self.check_expr(body);
                self.pop_scope();
            }
            HirExprKind::Cede(expr) => self.check_expr(expr),
            HirExprKind::Qua(expr, _) => self.check_expr(expr),
            HirExprKind::Ref(kind, inner) => match self.root_def_id(inner) {
                Some(def_id) => match kind {
                    crate::hir::HirRefKind::Shared => self.borrow_shared(def_id, expr.span),
                    crate::hir::HirRefKind::Mutable => self.borrow_mut(def_id, expr.span),
                },
                None => self.check_expr(inner),
            },
            HirExprKind::Deref(expr) => self.check_expr(expr),
            HirExprKind::Error => {}
        }
    }

    fn check_call_args(&mut self, callee: &HirExpr, args: &[HirExpr]) {
        let Some(callee_ty) = callee.ty else {
            for arg in args {
                self.check_expr(arg);
            }
            return;
        };

        let sig = match self.types.get(callee_ty) {
            Type::Func(sig) => Some(sig),
            Type::Alias(_, resolved) => match self.types.get(*resolved) {
                Type::Func(sig) => Some(sig),
                _ => None,
            },
            _ => None,
        };

        match sig {
            Some(sig) => {
                for (arg, param) in args.iter().zip(sig.params.iter()) {
                    match param.mode {
                        ParamMode::Ref => self.borrow_from_expr(arg, BorrowKind::Shared),
                        ParamMode::MutRef => self.borrow_from_expr(arg, BorrowKind::Mutable),
                        ParamMode::Move | ParamMode::Owned => self.move_from_expr(arg),
                    }
                }
            }
            None => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
        }
    }

    fn check_lvalue(&mut self, target: &HirExpr) {
        if let Some(def_id) = self.root_def_id(target) {
            self.write_use(def_id, target.span);
        } else {
            self.check_expr(target);
        }
    }

    fn check_move_expr(&mut self, expr: &HirExpr) {
        if let Some(def_id) = self.root_def_id(expr) {
            self.move_use(def_id, expr.span);
        } else {
            self.check_expr(expr);
        }
    }

    fn borrow_from_expr(&mut self, expr: &HirExpr, kind: BorrowKind) {
        if let Some(def_id) = self.root_def_id(expr) {
            match kind {
                BorrowKind::Shared => self.borrow_shared(def_id, expr.span),
                BorrowKind::Mutable => self.borrow_mut(def_id, expr.span),
            }
        } else {
            self.check_expr(expr);
        }
    }

    fn move_from_expr(&mut self, expr: &HirExpr) {
        if let Some(def_id) = self.root_def_id(expr) {
            self.move_use(def_id, expr.span);
        } else {
            self.check_expr(expr);
        }
    }

    fn root_def_id(&self, expr: &HirExpr) -> Option<DefId> {
        match &expr.kind {
            HirExprKind::Path(def_id) => Some(*def_id),
            HirExprKind::Field(object, _) => self.root_def_id(object),
            HirExprKind::Index(object, _) => self.root_def_id(object),
            HirExprKind::Deref(inner) => self.root_def_id(inner),
            _ => None,
        }
    }

    fn ensure_state(&mut self, def_id: DefId) {
        self.states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
    }

    fn read_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self
            .states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable {
            self.error(
                SemanticErrorKind::MutableBorrowConflict,
                "use while mutably borrowed",
                span,
            );
        }
    }

    fn write_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self
            .states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable || state.shared > 0 {
            self.error(
                SemanticErrorKind::MutableBorrowConflict,
                "write while borrowed",
                span,
            );
        }
    }

    fn move_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self
            .states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable || state.shared > 0 {
            self.error(
                SemanticErrorKind::CannotMoveOut,
                "cannot move out while borrowed",
                span,
            );
            return;
        }
        state.moved = true;
    }

    fn borrow_shared(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self
            .states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
        if state.moved {
            self.error(
                SemanticErrorKind::BorrowOfMoved,
                "borrow of moved value",
                span,
            );
            return;
        }
        if state.mutable {
            self.error(
                SemanticErrorKind::MutableBorrowConflict,
                "shared borrow conflicts with mutable borrow",
                span,
            );
            return;
        }
        state.shared = state.shared.saturating_add(1);
        if let Some(scope) = self.scopes.last_mut() {
            scope.borrows.push((def_id, BorrowKind::Shared));
        }
    }

    fn borrow_mut(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self
            .states
            .entry(def_id)
            .or_insert_with(BorrowState::default);
        if state.moved {
            self.error(
                SemanticErrorKind::BorrowOfMoved,
                "borrow of moved value",
                span,
            );
            return;
        }
        if state.mutable || state.shared > 0 {
            self.error(
                SemanticErrorKind::MutableBorrowConflict,
                "mutable borrow conflicts with existing borrow",
                span,
            );
            return;
        }
        state.mutable = true;
        if let Some(scope) = self.scopes.last_mut() {
            scope.borrows.push((def_id, BorrowKind::Mutable));
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BorrowScope {
            borrows: Vec::new(),
        });
    }

    fn pop_scope(&mut self) {
        let Some(scope) = self.scopes.pop() else {
            return;
        };
        for (def_id, kind) in scope.borrows {
            if let Some(state) = self.states.get_mut(&def_id) {
                match kind {
                    BorrowKind::Shared => {
                        state.shared = state.shared.saturating_sub(1);
                    }
                    BorrowKind::Mutable => {
                        state.mutable = false;
                    }
                }
            }
        }
    }

    fn error(&mut self, kind: SemanticErrorKind, message: &str, span: crate::lexer::Span) {
        self.errors
            .push(SemanticError::new(kind, message.to_owned(), span));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{
        HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirParam,
        HirParamMode, HirProgram, HirStmt, HirStmtKind,
    };
    use crate::lexer::Span;
    use crate::semantic::{FuncSig, ParamType, Primitive};

    fn span() -> Span {
        Span::default()
    }

    fn lit_expr(id: u32) -> HirExpr {
        HirExpr {
            id: crate::hir::HirId(id),
            kind: HirExprKind::Literal(HirLiteral::Int(1)),
            ty: None,
            span: span(),
        }
    }

    #[test]
    fn reports_use_after_move() {
        let mut types = TypeTable::new();
        let numerus = types.primitive(Primitive::Numerus);
        let func_ty = types.function(FuncSig {
            params: vec![ParamType {
                ty: numerus,
                mode: ParamMode::Move,
                optional: false,
            }],
            ret: numerus,
            is_async: false,
            is_generator: false,
        });

        let call = HirExpr {
            id: crate::hir::HirId(3),
            kind: HirExprKind::Call(
                Box::new(HirExpr {
                    id: crate::hir::HirId(2),
                    kind: HirExprKind::Path(DefId(20)),
                    ty: Some(func_ty),
                    span: span(),
                }),
                vec![HirExpr {
                    id: crate::hir::HirId(4),
                    kind: HirExprKind::Path(DefId(1)),
                    ty: None,
                    span: span(),
                }],
            ),
            ty: None,
            span: span(),
        };

        let program = HirProgram {
            items: vec![HirItem {
                id: crate::hir::HirId(0),
                def_id: DefId(0),
                kind: HirItemKind::Function(HirFunction {
                    name: crate::lexer::Symbol(1),
                    type_params: Vec::new(),
                    params: vec![HirParam {
                        def_id: DefId(1),
                        name: crate::lexer::Symbol(2),
                        ty: numerus,
                        mode: HirParamMode::Owned,
                        span: span(),
                    }],
                    ret_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![
                            HirStmt {
                                id: crate::hir::HirId(1),
                                kind: HirStmtKind::Expr(call),
                                span: span(),
                            },
                            HirStmt {
                                id: crate::hir::HirId(5),
                                kind: HirStmtKind::Expr(HirExpr {
                                    id: crate::hir::HirId(6),
                                    kind: HirExprKind::Path(DefId(1)),
                                    ty: None,
                                    span: span(),
                                }),
                                span: span(),
                            },
                        ],
                        expr: None,
                        span: span(),
                    }),
                    is_async: false,
                    is_generator: false,
                }),
                span: span(),
            }],
            entry: None,
        };

        let result = analyze(&program, &Resolver::new(), &types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::UseAfterMove));
    }

    #[test]
    fn reports_mutable_borrow_conflict() {
        let mut types = TypeTable::new();
        let numerus = types.primitive(Primitive::Numerus);
        let shared = HirExpr {
            id: crate::hir::HirId(1),
            kind: HirExprKind::Ref(
                crate::hir::HirRefKind::Shared,
                Box::new(HirExpr {
                    id: crate::hir::HirId(2),
                    kind: HirExprKind::Path(DefId(1)),
                    ty: None,
                    span: span(),
                }),
            ),
            ty: None,
            span: span(),
        };
        let mutable = HirExpr {
            id: crate::hir::HirId(3),
            kind: HirExprKind::Ref(
                crate::hir::HirRefKind::Mutable,
                Box::new(HirExpr {
                    id: crate::hir::HirId(4),
                    kind: HirExprKind::Path(DefId(1)),
                    ty: None,
                    span: span(),
                }),
            ),
            ty: None,
            span: span(),
        };

        let program = HirProgram {
            items: vec![HirItem {
                id: crate::hir::HirId(0),
                def_id: DefId(0),
                kind: HirItemKind::Function(HirFunction {
                    name: crate::lexer::Symbol(1),
                    type_params: Vec::new(),
                    params: vec![HirParam {
                        def_id: DefId(1),
                        name: crate::lexer::Symbol(2),
                        ty: numerus,
                        mode: HirParamMode::Owned,
                        span: span(),
                    }],
                    ret_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![
                            HirStmt {
                                id: crate::hir::HirId(5),
                                kind: HirStmtKind::Expr(shared),
                                span: span(),
                            },
                            HirStmt {
                                id: crate::hir::HirId(6),
                                kind: HirStmtKind::Expr(mutable),
                                span: span(),
                            },
                        ],
                        expr: None,
                        span: span(),
                    }),
                    is_async: false,
                    is_generator: false,
                }),
                span: span(),
            }],
            entry: None,
        };

        let result = analyze(&program, &Resolver::new(), &types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::MutableBorrowConflict));
    }
}
