//! Failable-function analysis for Rust `Result` signatures.
//!
//! The Rust backend must know a function's return shape before declaration
//! emission starts, while throw and propagation evidence can appear anywhere in
//! a function body or inside callees. This module performs a small prepass over
//! HIR to collect direct throw sites and call dependencies, then computes the
//! transitive set of functions that need `Result<T, String>` signatures.
//!
//! INVARIANTS
//! ==========
//! - Direct unsuppressed `iace` marks the enclosing function failable.
//! - Calls to failable functions make callers failable, repeated to a fixed
//!   point so dependency chains are covered before codegen writes signatures.
//! - Method-call propagation is name-based because the current backend queries
//!   failable method status by resolved method symbol rather than receiver type.
//! - Handled forms suppress throw propagation for the protected body, then
//!   visit catch/finally blocks normally because those blocks can still throw.
//!
//! LIMITATIONS
//! ===========
//! This is a codegen precomputation, not semantic analysis. It does not prove
//! overload resolution, receiver type identity, or unreachable throw paths; it
//! records the conservative dependency facts the Rust emitter needs to choose
//! between plain returns and `Result` returns consistently.

use super::RustCodegen;
use crate::hir::visit::{walk_expr, HirVisitor};
use crate::hir::{DefId, HirAd, HirExpr, HirExprKind, HirFunction, HirItemKind, HirProgram};
use crate::lexer::Symbol;
use rustc_hash::{FxHashMap, FxHashSet};

/// Per-function evidence used to decide whether a Rust declaration is failable.
#[derive(Default)]
struct FnDeps {
    /// Whether this function body contains an unsuppressed `iace`.
    direct_throw: bool,

    /// Direct function calls by definition id.
    calls: FxHashSet<DefId>,

    /// Method calls by method symbol; resolved later against codegen names.
    method_calls: FxHashSet<Symbol>,
}

/// Return the complete set of functions that must emit `Result` signatures.
pub(super) fn collect_failable_functions(codegen: &RustCodegen<'_>, hir: &HirProgram) -> FxHashSet<DefId> {
    let fn_deps = collect_function_dependencies(hir);
    propagate_failable_functions(codegen, &fn_deps)
}

/// Collect direct throw and call evidence for free functions and struct methods.
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

    // Declarations need a closed set before any source is written, so
    // propagation runs to a fixed point over the call graph instead of asking
    // individual call sites to mutate signatures during expression emission.
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
            // Method calls currently propagate by resolved method name. This is
            // conservative for same-named methods because this pass does not
            // carry receiver type information.
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
    fn visit_ad(&mut self, ad: &HirAd) {
        if ad.catch.is_none() && !self.suppressed {
            self.deps.direct_throw = true;
        }
        crate::hir::visit::walk_ad(self, ad);
    }

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
                // HIR marks a catch-bearing body as locally handled for this
                // prepass, so direct throws and failable calls inside that body
                // should not force the enclosing function to return `Result`.
                self.with_suppressed(self.suppressed || catch.is_some(), |visitor| visitor.visit_block(body));
                if let Some(catch_block) = catch {
                    self.visit_block(catch_block);
                }
                if let Some(finally_block) = finally {
                    self.visit_block(finally_block);
                }
            }
            HirExprKind::Handled { body, catch } => {
                // `Handled` has already established an error boundary around
                // its body; the catch body itself remains ordinary code.
                self.with_suppressed(true, |visitor| visitor.visit_block(body));
                self.visit_block(&catch.body);
            }
            _ => walk_expr(self, expr),
        }
    }
}
