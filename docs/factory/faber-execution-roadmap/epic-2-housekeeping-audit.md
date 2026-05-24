# Epic 2 Housekeeping Audit

**Date**: 2026-05-24  
**Scope**: Epic 2 exempla boundary and current Rust backend stabilization  
**Baseline HEAD**: `747ca5a1 fix(rust): lower optional map members`

## Validation Snapshot

- `./scripta/lint`: pass after lint cleanup in `crates/radix/src/codegen/rust/decl.rs` and `crates/radix/src/codegen/rust/mod.rs`.
- `cargo test -p radix`: pass, `458` tests, `3` ignored, plus `8` hygiene tests.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`: harness pass, `99/100` exempla pass. The only current failure is `examples/exempla/ad/ad.fab`, intentionally deferred to Epic 3.

## Highest Priority Cleanup And Correctness Risks

1. Harden optional-shape emission behind one helper.
   - Risk: optional locals, optional returns, optional struct fields, optional calls, and optional chains each decide separately whether an expression already produces `Option<T>`.
   - Evidence: `crates/radix/src/codegen/rust/stmt.rs`, `expr/control.rs`, `expr/collection.rs`, `expr/option.rs`, and `expr/call.rs` all contain local variants of this policy.
   - Expected fix: centralize "emit expression into optional target" and "does expression already produce option" logic, then add tests for option-valued locals and option-valued arguments to `sponte` parameters.

2. Harden dynamic `FaberValue` coercion behind one helper.
   - Risk: direct call arguments coerce into `FaberValue`, but method-call and runtime-bridge arguments do not consistently receive target parameter types.
   - Evidence: dynamic detection/coercion is duplicated in `expr/mod.rs`, `expr/collection.rs`, and `expr/verte.rs`.
   - Expected fix: share one target-type emission path for locals, calls, method calls, array/object construction, and `verte` construction.

3. Reject ordinary access on optional receivers unless syntax is explicit.
   - Risk: `struct_def_from_type` and `interface_def_from_type` peel `Option<T>`, so `maybe.field` or `maybe.method()` can typecheck without `?.` or non-null syntax.
   - Evidence: `crates/radix/src/semantic/passes/typecheck/lookup.rs` recurses through `Type::Option`, and access/call typing then treats the receiver as concrete.
   - Expected fix: separate "receiver-like type for nullable-aware operators" from ordinary member/method lookup.

4. Preserve spread and named argument metadata through optional/non-null calls and `finge`.
   - Risk: optional/non-null call HIR stores `Vec<HirExpr>` instead of `Vec<HirCallArg>`, losing `sparge`. `finge` named fields are lowered into positional call args, so reordered or misspelled fields can bind to the wrong variant payload.
   - Evidence: `HirOptionalChainKind::Call(Vec<HirExpr>)`, `HirNonNullKind::Call(Vec<HirExpr>)`, and `hir/lower/expr.rs` variant construction lowering.
   - Expected fix: carry `HirCallArg` or a named argument representation through these forms, then make variant construction check names before codegen.

5. Make the Rust e2e harness truthful as a gate.
   - Risk: the harness reports `99/100` but still returns `ok` because it asserts only `salve-munde.fab`.
   - Evidence: `crates/radix/src/exempla_e2e_test.rs` prints all failures, then only checks `salve-munde.fab`.
   - Expected fix: add expected-failure metadata for `ad/ad.fab` or move `ad` out until Epic 3, then assert the full expected corpus state.

6. Fix generated-code linter plumbing.
   - Risk: `lint_generated_code` removes the temporary Cargo project before reading the fixed `src/main.rs`, so successful Rust fixes cannot be returned. Clippy failures also pollute e2e output even though the harness treats lint as best effort.
   - Evidence: `crates/radix/src/tool.rs` removes the temp dir before reading `main.rs`.
   - Expected fix: read fixed code before cleanup and capture tool stderr so e2e output stays focused on final failures.

## Documentation And Drift

- `docs/factory/exempla-rust-e2e/goal.md` still says `not started` and `71/138`.
- `docs/factory/exempla-rust-e2e/epic-2-boundary-ledger.md` records the historical `59/100` post-relocation state.
- `docs/factory/faber-execution-roadmap/epic-2-phase-34-delivery.md` records the pre-phase `98/100` state.
- Live state is `99/100` with `100` files under `examples/exempla` and `37` relocated `.fab` fixtures under `examples/fixtures/exempla-boundary`.

Follow-up: add a short closeout/status doc or update the focused goal status so future work does not rely on stale historical counts.

## Lower Priority Cleanup

- Split or reorganize `crates/radix/src/codegen/rust/mod_test.rs`; it is now roughly 2,700 lines and mixes many Epic 2 clusters. Natural split: optional/nullability, dynamic values, calls/ownership, collections/iteration, declarations/methods.
- Add more `.expected` files or equivalent runtime assertions. Only one exemplar currently checks stdout; the rest prove compile-and-zero-exit, not behavior.
- Add a guard test around `examples/fixtures/exempla-boundary` so relocated fixtures are not accidentally reintroduced into `examples/exempla`.
- Clean up `ad` terminology before Epic 3. Comments and explain docs still mix endpoint/HTTP wording with capability-call semantics.
- Clean up temp roots from e2e runs after preserving enough failure artifacts for debugging.
