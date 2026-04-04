//! Pass 5: Borrow checking
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Validates ownership and borrowing rules for safe memory management when
//! targeting Rust. Tracks move semantics, shared/mutable borrows, and parameter
//! passing modes (de/in/ex) to ensure generated Rust code compiles without
//! lifetime errors.
//!
//! COMPILER PHASE: Semantic (Pass 5, Rust target only)
//! INPUT: Typed HIR with DefIds and TypeIds
//! OUTPUT: Borrowing errors (use-after-move, borrow conflicts); mode lints
//!
//! WHY: Faber's de/in/ex parameter modes map directly to Rust's &/&mut/move,
//! but users may write invalid borrow patterns. This pass catches errors early
//! rather than emitting invalid Rust code that fails at rustc compile time.
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Simplified Borrow Model: Tracks basic ownership (moved/borrowed) without
//!   full lifetime analysis, relying on Rust's borrow checker for complex cases
//! - Scope-Based Borrows: Borrows are tracked per lexical scope and released
//!   on scope exit, matching Rust's non-lexical-lifetimes behavior
//! - Mode Validation: de → in/ex promotions are errors (immutable → mutable);
//!   unused in/ex parameters generate warnings (should be de)
//! - Optimistic Checking: Allows borderline cases that rustc would accept,
//!   avoiding false positives on complex lifetime patterns
//!
//! BORROW STATES
//! =============
//! Each variable tracks:
//! - moved: bool - Value has been moved out (cannot use again)
//! - shared: u32 - Count of active shared borrows
//! - mutable: bool - Active mutable borrow exists
//!
//! INVARIANTS
//! ==========
//! INV-1: Cannot move value that is borrowed (shared > 0 || mutable)
//! INV-2: Cannot mutably borrow if any borrows exist (shared > 0 || mutable)
//! INV-3: Cannot use value after move (moved == true)
//! INV-4: Cannot assign to de parameter (immutable borrow)
//!
//! PARAMETER MODE LINTS
//! ====================
//! - `in` param never mutated → suggest `de` (shared borrow sufficient)
//! - `ex` param never consumed → suggest `de` (move not required)
//!
//! EDGE CASES
//! ==========
//! - Field/index projections: root_def_id() extracts the base variable from
//!   field access chains (e.g., `x.y.z` → DefId for `x`)
//! - Method calls: Borrows receiver based on method signature if available

use crate::hir::{
    DefId, HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirParamMode, HirProgram, HirStmt,
    HirStmtKind,
};
use crate::semantic::{ParamMode, Resolver, SemanticError, SemanticErrorKind, Type, TypeTable, WarningKind};
use rustc_hash::FxHashMap;

/// Analyze borrowing and ownership
pub fn analyze(hir: &HirProgram, _resolver: &Resolver, types: &TypeTable) -> Result<(), Vec<SemanticError>> {
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
    param_usage: FxHashMap<DefId, ParamUsage>,
    errors: Vec<SemanticError>,
}

#[derive(Clone, Copy)]
struct ParamUsage {
    mode: HirParamMode,
    span: crate::lexer::Span,
    mutated: bool,
    passed_in_or_ex: bool,
    passed_ex: bool,
    returned: bool,
    moved: bool,
}

impl<'a> BorrowChecker<'a> {
    fn new(types: &'a TypeTable) -> Self {
        Self {
            types,
            states: FxHashMap::default(),
            scopes: Vec::new(),
            param_usage: FxHashMap::default(),
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
            self.param_usage.insert(
                param.def_id,
                ParamUsage {
                    mode: param.mode,
                    span: param.span,
                    mutated: false,
                    passed_in_or_ex: false,
                    passed_ex: false,
                    returned: false,
                    moved: false,
                },
            );
        }
        if let Some(body) = &func.body {
            self.check_block(body);
        }
        self.emit_mode_lints();
    }

    fn reset(&mut self) {
        self.states.clear();
        self.scopes.clear();
        self.param_usage.clear();
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
            HirStmtKind::Ad(ad) => {
                for arg in &ad.args {
                    self.check_expr(arg);
                }
                if let Some(body) = &ad.body {
                    self.check_block(body);
                }
                if let Some(catch) = &ad.catch {
                    self.check_block(catch);
                }
            }
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    if let Some(def_id) = self.root_def_id(expr) {
                        if let Some(usage) = self.param_usage.get_mut(&def_id) {
                            usage.returned = true;
                        }
                    }
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
            HirExprKind::OptionalChain(object, chain) => {
                self.check_expr(object);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(_) => {}
                    crate::hir::HirOptionalChainKind::Index(index) => self.check_expr(index),
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.check_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.check_expr(object);
                match chain {
                    crate::hir::HirNonNullKind::Member(_) => {}
                    crate::hir::HirNonNullKind::Index(index) => self.check_expr(index),
                    crate::hir::HirNonNullKind::Call(args) => {
                        for arg in args {
                            self.check_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.check_expr(source);
                if let Some(filter) = filter {
                    if let crate::hir::HirCollectionFilterKind::Condition(cond) = &filter.kind {
                        self.check_expr(cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &transform.arg {
                        self.check_expr(arg);
                    }
                }
            }
            HirExprKind::Block(block) => self.check_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_expr(cond);
                self.check_block(then_block);
                if let Some(block) = else_block {
                    self.check_block(block);
                }
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                for scrutinee in scrutinees {
                    self.check_expr(scrutinee);
                }
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
            HirExprKind::Itera(_, binding, iter, block) => {
                self.check_expr(iter);
                self.ensure_state(*binding);
                self.check_block(block);
            }
            HirExprKind::Assign(target, value) => {
                self.check_lvalue(target);
                self.check_expr(value);
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
            HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.check_expr(element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.check_expr(arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.check_expr(cond);
                if let Some(message) = message {
                    self.check_expr(message);
                }
            }
            HirExprKind::Panic(value) | HirExprKind::Throw(value) => self.check_expr(value),
            HirExprKind::Tempta { body, catch, finally } => {
                self.check_block(body);
                if let Some(catch) = catch {
                    self.check_block(catch);
                }
                if let Some(finally) = finally {
                    self.check_block(finally);
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
            HirExprKind::Verte { source, entries, .. } => {
                self.check_expr(source);
                if let Some(entries) = entries {
                    for (_, value) in entries {
                        self.check_expr(value);
                    }
                }
            }
            HirExprKind::Conversio { source, fallback, .. } => {
                self.check_expr(source);
                if let Some(fallback) = fallback {
                    self.check_expr(fallback);
                }
            }
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
                    if let Some(arg_def_id) = self.root_def_id(arg) {
                        if let Some(arg_usage) = self.param_usage.get_mut(&arg_def_id) {
                            if matches!(param.mode, ParamMode::MutRef | ParamMode::Move) {
                                arg_usage.passed_in_or_ex = true;
                            }
                            if matches!(param.mode, ParamMode::Move) {
                                arg_usage.passed_ex = true;
                            }
                            if matches!(arg_usage.mode, HirParamMode::Ref)
                                && matches!(param.mode, ParamMode::MutRef | ParamMode::Move)
                            {
                                self.error(
                                    SemanticErrorKind::ModeMismatch,
                                    "cannot pass `de` parameter to `in` or `ex` position",
                                    arg.span,
                                );
                            }
                        }
                    }
                    match param.mode {
                        ParamMode::Ref => self.borrow_from_expr(arg, BorrowKind::Shared),
                        ParamMode::MutRef => self.borrow_from_expr(arg, BorrowKind::Mutable),
                        ParamMode::Move => self.move_from_expr(arg),
                        ParamMode::Owned => self.check_expr(arg),
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

    #[allow(clippy::only_used_in_recursion)]
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
        self.states.entry(def_id).or_default();
    }

    fn read_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self.states.entry(def_id).or_default();
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable {
            self.error(SemanticErrorKind::MutableBorrowConflict, "use while mutably borrowed", span);
        }
    }

    fn write_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        if let Some(usage) = self.param_usage.get_mut(&def_id) {
            if matches!(usage.mode, HirParamMode::Ref) {
                self.error(
                    SemanticErrorKind::AssignToImmutableBorrow,
                    "cannot assign to `de` parameter",
                    span,
                );
            } else if matches!(usage.mode, HirParamMode::MutRef) {
                usage.mutated = true;
            }
        }
        let state = self.states.entry(def_id).or_default();
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable || state.shared > 0 {
            self.error(SemanticErrorKind::MutableBorrowConflict, "write while borrowed", span);
        }
    }

    fn move_use(&mut self, def_id: DefId, span: crate::lexer::Span) {
        if let Some(usage) = self.param_usage.get_mut(&def_id) {
            usage.moved = true;
        }
        let state = self.states.entry(def_id).or_default();
        if state.moved {
            self.error(SemanticErrorKind::UseAfterMove, "use after move", span);
            return;
        }
        if state.mutable || state.shared > 0 {
            self.error(SemanticErrorKind::CannotMoveOut, "cannot move out while borrowed", span);
            return;
        }
        state.moved = true;
    }

    fn borrow_shared(&mut self, def_id: DefId, span: crate::lexer::Span) {
        let state = self.states.entry(def_id).or_default();
        if state.moved {
            self.error(SemanticErrorKind::BorrowOfMoved, "borrow of moved value", span);
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
        let state = self.states.entry(def_id).or_default();
        if state.moved {
            self.error(SemanticErrorKind::BorrowOfMoved, "borrow of moved value", span);
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
        self.scopes.push(BorrowScope { borrows: Vec::new() });
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

    fn emit_mode_lints(&mut self) {
        let mut warnings = Vec::new();
        for usage in self.param_usage.values() {
            match usage.mode {
                HirParamMode::MutRef => {
                    if !usage.mutated && !usage.passed_in_or_ex {
                        warnings.push(SemanticError::new(
                            SemanticErrorKind::Warning(WarningKind::UnusedMutRefParam),
                            "`in` parameter is never mutated; consider `de`",
                            usage.span,
                        ));
                    }
                }
                HirParamMode::Move => {
                    if !usage.passed_ex && !usage.returned && !usage.moved {
                        warnings.push(SemanticError::new(
                            SemanticErrorKind::Warning(WarningKind::UnusedMoveParam),
                            "`ex` parameter is never consumed; consider `de`",
                            usage.span,
                        ));
                    }
                }
                HirParamMode::Owned | HirParamMode::Ref => {}
            }
        }
        self.errors.extend(warnings);
    }
}

#[cfg(test)]
#[path = "borrow_test.rs"]
mod tests;
