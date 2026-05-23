use super::RustCodegen;
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{DefId, HirExpr, HirExprKind, HirFunction, HirItemKind, HirProgram};
use crate::lexer::Symbol;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Default)]
struct FnDeps {
    direct_throw: bool,
    calls: FxHashSet<DefId>,
    method_calls: FxHashSet<Symbol>,
}

pub(super) fn collect_failable_functions(codegen: &RustCodegen<'_>, hir: &HirProgram) -> FxHashSet<DefId> {
    let fn_deps = collect_function_dependencies(hir);
    propagate_failable_functions(codegen, &fn_deps)
}

fn collect_function_dependencies(hir: &HirProgram) -> FxHashMap<DefId, FnDeps> {
    let mut fn_deps = FxHashMap::default();
    let mut register_fn = |def_id: DefId, func: &HirFunction| {
        let mut visitor = DependencyVisitor::default();
        if let Some(body) = &func.body {
            visitor.visit_block(body);
        }
        fn_deps.insert(def_id, visitor.deps);
    };

    for item in &hir.items {
        match &item.kind {
            HirItemKind::Function(func) => register_fn(item.def_id, func),
            HirItemKind::Struct(strukt) => {
                for method in &strukt.methods {
                    register_fn(method.def_id, &method.func);
                }
            }
            _ => {}
        }
    }

    fn_deps
}

fn propagate_failable_functions(codegen: &RustCodegen<'_>, fn_deps: &FxHashMap<DefId, FnDeps>) -> FxHashSet<DefId> {
    let mut failable = FxHashSet::default();
    for (def_id, deps) in fn_deps {
        if deps.direct_throw {
            failable.insert(*def_id);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (def_id, deps) in fn_deps {
            if failable.contains(def_id) {
                continue;
            }

            if depends_on_failable(codegen, deps, &failable) {
                failable.insert(*def_id);
                changed = true;
            }
        }
    }

    failable
}

fn depends_on_failable(codegen: &RustCodegen<'_>, deps: &FnDeps, failable: &FxHashSet<DefId>) -> bool {
    deps.calls.iter().any(|callee| failable.contains(callee))
        || deps.method_calls.iter().any(|method| {
            codegen
                .names
                .iter()
                .any(|(callee_def, name)| *name == *method && failable.contains(callee_def))
        })
}

#[derive(Default)]
struct DependencyVisitor {
    deps: FnDeps,
    suppressed: bool,
}

impl DependencyVisitor {
    fn with_suppressed(&mut self, suppressed: bool, f: impl FnOnce(&mut Self)) {
        let previous = self.suppressed;
        self.suppressed = suppressed;
        f(self);
        self.suppressed = previous;
    }
}

impl HirVisitor for DependencyVisitor {
    fn visit_expr(&mut self, expr: &HirExpr) {
        match &expr.kind {
            HirExprKind::Throw(_) => {
                if !self.suppressed {
                    self.deps.direct_throw = true;
                }
                walk_expr(self, expr);
            }
            HirExprKind::Call(callee, _) => {
                if !self.suppressed {
                    if let HirExprKind::Path(def_id) = &callee.kind {
                        self.deps.calls.insert(*def_id);
                    }
                }
                walk_expr(self, expr);
            }
            HirExprKind::MethodCall(_, method, _) => {
                if !self.suppressed {
                    self.deps.method_calls.insert(*method);
                }
                walk_expr(self, expr);
            }
            HirExprKind::Tempta { body, catch, finally } => {
                self.with_suppressed(self.suppressed || catch.is_some(), |visitor| visitor.visit_block(body));
                if let Some(catch_block) = catch {
                    self.visit_block(catch_block);
                }
                if let Some(finally_block) = finally {
                    self.visit_block(finally_block);
                }
            }
            HirExprKind::Handled { body, catch } => {
                self.with_suppressed(true, |visitor| visitor.visit_block(body));
                self.visit_block(&catch.body);
            }
            _ => walk_expr(self, expr),
        }
    }
}
