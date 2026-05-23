# Deduplication and Testability Decomposition Analysis

**Date**: May 2026  
**Context**: Multi-agent exploration pass across the radix compiler  
**Status**: Analysis complete; ready for planning / implementation

## Executive Summary

A coordinated swarm of specialized agents performed a deep deduplication and decomposition analysis of the `crates/radix` compiler. The investigation covered:

- HIR lowering vs. MIR lowering
- All semantic analysis passes
- All four codegen backends (Rust, TypeScript, Go, Faber)
- Large/monolithic functions from a testability perspective

**Core finding**: The compiler suffers from widespread, high-volume structural duplication caused by the absence of a proper HIR visitor/traversal abstraction (while an AST visitor exists in `syntax/visit.rs` but is largely bypassed after lowering).

This duplication manifests as hundreds of near-identical recursive `match &expr.kind` / `&stmt.kind` / `&item.kind` walkers spread across semantic passes, lowering, and every codegen target. It is the primary driver of both maintenance cost and poor unit testability.

Secondary findings include the monolithic nature of `mir/lower.rs`, repeated scope stack implementations, duplicated error-collection boilerplate, and several very large dispatch functions that resist isolated testing.

**Highest-leverage recommendation**: Introduce a `hir::visit` module with a `HirVisitor` trait + default `walk_*` methods. This single piece of infrastructure would eliminate the majority of the duplicated traversal logic and unlock dramatically better testability.

---

## Root Cause

After lowering, the compiler works on a rich HIR (`HirProgram`, `HirItemKind`, `HirStmtKind`, `HirExprKind`, etc.). However, unlike the AST (which has `syntax/visit.rs`), **there is no shared visitor or traversal infrastructure for HIR**.

As a result, every consumer re-implements the same recursive descent pattern:

```rust
match &expr.kind {
    HirExprKind::Binary(_, l, r) => { visit(l); visit(r); }
    HirExprKind::Call(callee, args) => { visit(callee); for a in args { visit(a); } }
    // ... ~30 more arms, repeated with minor variations ...
}
```

This pattern appears in:
- All semantic passes (typecheck, borrow, lint, exhaustive, resolve, finalize)
- MIR lowering
- Codegen (name collection, use counting, error detection, await/ad detection, failable analysis, pretty-printing, etc.)

The duplication is not just stylistic — it creates maintenance drag, risk when adding new `HirExprKind` variants, and makes the large dispatch functions extremely difficult to unit test in isolation.

---

## Detailed Findings by Area

### 1. HIR Traversal Duplication (Highest Volume)

**Semantic Passes Agent** documented near-identical `check_expr` / `finalize_expr` / `resolve_expr` implementations across five+ passes, each with 150–200 LOC matches containing the same recursion skeletons for `Binary`, `Call`, `MethodCall`, `OptionalChain`, `NonNull`, `Ab`, `Si`, `Discerne`, `Verte`, `Conversio`, etc.

**Codegen Agent** found the same pattern in:
- `codegen/names.rs:145` (`collect_expr_names`)
- `go/mod.rs:208` (`collect_expr_use_counts`)
- `go/mod.rs:538` (`expr_contains_ad`)
- `ts/expr.rs:919` (`contains_await_in_expr`)
- `faber/names.rs`
- `rust/failable.rs:117`
- `codegen/mod.rs:230` (error finder)

**Lowering Agent** noted the structural repetition between HIR lower dispatch, MIR lower dispatch, and all four codegen backends.

**Testability Agent** flagged the giant dispatch functions (`generate_expr`, `write_expr_prec`, `resolve_stmt`, `resolve_expr`, `check_expr_*`) as the hardest to test precisely because they embed this duplicated traversal logic.

### 2. MIR Lowering Monolith

`crates/radix/src/mir/lower.rs` (~1990 LOC) contains a single massive `FunctionBuilder` impl with dozens of `lower_*` methods, heavy mutable state (`bindings`, `blocks`, `loops`, `handlers`, temps, errors), and mixed concerns (dispatch + CFG construction + operand management + error paths).

In contrast, `hir/lower/` is already nicely split by syntactic category (`decl.rs`, `expr.rs`, `stmt.rs`, `pattern.rs`, `types.rs`).

The lowering agent explicitly recommended a phased extraction modeled on the successful codegen and HIR lower refactors.

### 3. Codegen Backend Repetition

Each target (Rust, Go, TS, Faber) re-implements very similar logic for:

- `generate_item` / `generate_function` / `generate_struct` etc. dispatch
- `type_to_*` (nearly identical `match Type::*` + primitive mapping + recursion)
- Statement and block emission
- Expression dispatch (the largest surface)

Rust and Go have begun splitting `expr/` into focused submodules (`access.rs`, `call.rs`, `control.rs`, etc.). TypeScript and Faber remain more monolithic.

Name collection, use counting, and various detection walkers are duplicated both within and across targets.

### 4. Scope Stack and Error Boilerplate

Four to five independent scope stack implementations exist (different key types, slightly different lookup rules).

Error reporting follows a repetitive `SemanticError::new(...)` + `.push(...)` pattern with only minor local variations.

### 5. Large Functions / Testability Hotspots (Ranked)

From the dedicated testability agent (most actionable list):

1. `crates/radix/src/codegen/ts/expr.rs:10` — `generate_expr` (~486 LOC match)
2. `crates/radix/src/codegen/faber/expr.rs:8` — `write_expr_prec` (~443 LOC)
3. `crates/radix/src/semantic/passes/resolve.rs` — `resolve_stmt` (~431 LOC) and `resolve_expr` (~182 LOC)
4. `crates/radix/src/codegen/rust/expr/mod.rs:65` — `generate_expr`
5. Driver scanner functions (`scan_expr_for_go_unsupported_errors` etc.)
6. The `check_expr` family across borrow/lint/exhaustive/typecheck

These functions are hard to test because they require heavy shared state (`&mut Resolver`, `TypeTable`, `CodeWriter`, full HIR) and embed both traversal and policy.

---

## Prioritized Recommendations

### Tier 1 — Highest Leverage (Do These First)

1. **Introduce `hir::visit` (or `hir/visit.rs`)**  
   `HirVisitor` trait + default `walk_program` / `walk_item` / `walk_stmt` / `walk_expr` / `walk_block` / `walk_pattern`.  
   Start with read-only, add `&mut` variant later.  
   This collapses the vast majority of duplicated traversal logic.

2. **Split `mir/lower.rs`**  
   Follow the established pattern: move to `mir/lower/{mod.rs, expr.rs, stmt.rs, control.rs, builder.rs, context.rs, ...}` and extract focused lowerer types.

3. **Finish codegen expr factoring**  
   Split `ts/expr.rs` and continue evolving `faber/expr.rs` into an `expr/` subdirectory consistent with Rust/Go.

### Tier 2 — High Value, Lower Risk

4. Generic `ScopeStack<T>` utility.
5. Centralized error/diagnostic collection helper.
6. Common emission helpers in `codegen/` (type formatting, decl helpers, op/literal tables) behind small traits or contexts.

### Tier 3 — Polish

- Harvest remaining large dispatch functions (resolve, driver scanners).
- Add colocated `*_test.rs` (or `#[cfg(test)] mod tests;` with `#[path]`) for the newly extracted units, per project convention.

---

## Suggested Implementation Order

1. Design + implement the HIR Visitor (read-only first).
2. Port one or two semantic passes to use it as a proof of concept.
3. Begin the `mir/lower.rs` split (start with file split + pure helpers).
4. Factor the TS and Faber expr dispatchers.
5. Introduce shared scope + error utilities.
6. Backfill unit tests on the new smaller pieces.

---

## Notes on Testability

All recommendations are explicitly aimed at enabling the project's stated preference for **dedicated `_test.rs` files** and isolated unit tests rather than only coarse integration tests.

The current situation (giant match arms that require full compiler state) makes the AGENTS.md rule difficult to follow in practice for large parts of the compiler. The visitor + factoring work directly addresses this.

---

## References

This document synthesizes findings from a parallel agent swarm (May 2026). Raw subagent outputs are available via the session transcript under the following IDs (for traceability):

- HIR/MIR Lowering: `019e540e-5797-7a70-ac08-5ff4a99e2bf3`
- Semantic Passes & Cross-Cutting: `019e540e-6ce0-73b1-8165-d901f2de7fb2`
- Codegen Backends: `019e540e-4d2b-7843-99f9-9aeeeb6e2b64`
- Long Functions & Testability: `019e540e-793f-7af2-a13b-1cfafe1ade95`

Key files repeatedly referenced:
- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/semantic/passes/{resolve,borrow,lint,exhaustive,typecheck}/**/*.rs`
- `crates/radix/src/codegen/{rust,ts,go,faber}/**/*.rs`
- `crates/radix/src/syntax/visit.rs` (existing precedent)
- `crates/radix/src/hir/nodes.rs` (the shape being walked)

---

*This report is intended as a living document to be picked up for delivery planning.*