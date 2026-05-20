use super::RustCodegen;
use crate::hir::{
    DefId, HirBlock, HirCollectionFilterKind, HirExpr, HirExprKind, HirFunction, HirItemKind, HirOptionalChainKind,
    HirProgram, HirStmtKind,
};
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
        let mut deps = FnDeps::default();
        if let Some(body) = &func.body {
            visit_block(body, false, &mut deps);
        }
        fn_deps.insert(def_id, deps);
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

fn visit_block(block: &HirBlock, suppressed: bool, deps: &mut FnDeps) {
    for stmt in &block.stmts {
        match &stmt.kind {
            HirStmtKind::Local(local) => {
                if let Some(init) = &local.init {
                    visit_expr(init, suppressed, deps);
                }
            }
            HirStmtKind::Ad(ad) => {
                for arg in &ad.args {
                    visit_expr(arg, suppressed, deps);
                }
                if let Some(body) = &ad.body {
                    visit_block(body, suppressed, deps);
                }
                if let Some(catch) = &ad.catch {
                    visit_block(catch, suppressed, deps);
                }
            }
            HirStmtKind::Expr(expr) => visit_expr(expr, suppressed, deps),
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    visit_expr(expr, suppressed, deps);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    if let Some(expr) = &block.expr {
        visit_expr(expr, suppressed, deps);
    }
}

fn visit_expr(expr: &HirExpr, suppressed: bool, deps: &mut FnDeps) {
    match &expr.kind {
        HirExprKind::Throw(value) => {
            if !suppressed {
                deps.direct_throw = true;
            }
            visit_expr(value, suppressed, deps);
        }
        HirExprKind::Call(callee, args) => {
            if !suppressed {
                if let HirExprKind::Path(def_id) = &callee.kind {
                    deps.calls.insert(*def_id);
                }
            }
            visit_expr(callee, suppressed, deps);
            for arg in args {
                visit_expr(arg, suppressed, deps);
            }
        }
        HirExprKind::MethodCall(receiver, method, args) => {
            if !suppressed {
                deps.method_calls.insert(*method);
            }
            visit_expr(receiver, suppressed, deps);
            for arg in args {
                visit_expr(arg, suppressed, deps);
            }
        }
        HirExprKind::Tempta { body, catch, finally } => {
            let body_suppressed = suppressed || catch.is_some();
            visit_block(body, body_suppressed, deps);
            if let Some(catch_block) = catch {
                visit_block(catch_block, suppressed, deps);
            }
            if let Some(finally_block) = finally {
                visit_block(finally_block, suppressed, deps);
            }
        }
        HirExprKind::Binary(_, lhs, rhs) | HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
            visit_expr(lhs, suppressed, deps);
            visit_expr(rhs, suppressed, deps);
        }
        HirExprKind::Unary(_, operand)
        | HirExprKind::Cede(operand)
        | HirExprKind::Ref(_, operand)
        | HirExprKind::Deref(operand)
        | HirExprKind::Panic(operand) => visit_expr(operand, suppressed, deps),
        HirExprKind::Verte { source, entries, .. } => {
            visit_expr(source, suppressed, deps);
            if let Some(entries) = entries {
                for field in entries {
                    match &field.key {
                        crate::hir::HirObjectKey::Computed(expr) | crate::hir::HirObjectKey::Spread(expr) => {
                            visit_expr(expr, suppressed, deps)
                        }
                        crate::hir::HirObjectKey::Ident(_) | crate::hir::HirObjectKey::String(_) => {}
                    }
                    if let Some(value) = &field.value {
                        visit_expr(value, suppressed, deps);
                    }
                }
            }
        }
        HirExprKind::Conversio { source, fallback, .. } => {
            visit_expr(source, suppressed, deps);
            if let Some(fallback) = fallback {
                visit_expr(fallback, suppressed, deps);
            }
        }
        HirExprKind::Field(object, _) | HirExprKind::Index(object, _) => {
            visit_expr(object, suppressed, deps);
        }
        HirExprKind::OptionalChain(object, chain) => {
            visit_expr(object, suppressed, deps);
            match chain {
                HirOptionalChainKind::Member(_) => {}
                HirOptionalChainKind::Index(index) => visit_expr(index, suppressed, deps),
                HirOptionalChainKind::Call(args) => {
                    for arg in args {
                        visit_expr(arg, suppressed, deps);
                    }
                }
            }
        }
        HirExprKind::NonNull(object, chain) => {
            visit_expr(object, suppressed, deps);
            match chain {
                crate::hir::HirNonNullKind::Member(_) => {}
                crate::hir::HirNonNullKind::Index(index) => visit_expr(index, suppressed, deps),
                crate::hir::HirNonNullKind::Call(args) => {
                    for arg in args {
                        visit_expr(arg, suppressed, deps);
                    }
                }
            }
        }
        HirExprKind::Ab { source, filter, transforms } => {
            visit_expr(source, suppressed, deps);
            if let Some(filter) = filter {
                if let HirCollectionFilterKind::Condition(cond) = &filter.kind {
                    visit_expr(cond, suppressed, deps);
                }
            }
            for transform in transforms {
                if let Some(arg) = &transform.arg {
                    visit_expr(arg, suppressed, deps);
                }
            }
        }
        HirExprKind::Block(block) | HirExprKind::Loop(block) => {
            visit_block(block, suppressed, deps);
        }
        HirExprKind::Si(cond, then_block, else_block) => {
            visit_expr(cond, suppressed, deps);
            visit_block(then_block, suppressed, deps);
            if let Some(else_block) = else_block {
                visit_block(else_block, suppressed, deps);
            }
        }
        HirExprKind::Discerne(scrutinees, arms) => {
            for scrutinee in scrutinees {
                visit_expr(scrutinee, suppressed, deps);
            }
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    visit_expr(guard, suppressed, deps);
                }
                visit_expr(&arm.body, suppressed, deps);
            }
        }
        HirExprKind::Dum(cond, block) => {
            visit_expr(cond, suppressed, deps);
            visit_block(block, suppressed, deps);
        }
        HirExprKind::Itera(_, _, _, iter, block) => {
            visit_expr(iter, suppressed, deps);
            visit_block(block, suppressed, deps);
        }
        HirExprKind::Intervallum { start, end, step, .. } => {
            visit_expr(start, suppressed, deps);
            visit_expr(end, suppressed, deps);
            if let Some(step) = step {
                visit_expr(step, suppressed, deps);
            }
        }
        HirExprKind::Array(elements) => {
            for element in elements {
                match element {
                    crate::hir::HirArrayElement::Expr(expr) | crate::hir::HirArrayElement::Spread(expr) => {
                        visit_expr(expr, suppressed, deps)
                    }
                }
            }
        }
        HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
            for element in elements {
                visit_expr(element, suppressed, deps);
            }
        }
        HirExprKind::Scriptum(_, args) => {
            for arg in args {
                visit_expr(arg, suppressed, deps);
            }
        }
        HirExprKind::Adfirma(cond, message) => {
            visit_expr(cond, suppressed, deps);
            if let Some(message) = message {
                visit_expr(message, suppressed, deps);
            }
        }
        HirExprKind::Struct(_, fields) => {
            for (_, value) in fields {
                visit_expr(value, suppressed, deps);
            }
        }
        HirExprKind::Clausura(_, _, body) => visit_expr(body, suppressed, deps),
        HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
    }
}
