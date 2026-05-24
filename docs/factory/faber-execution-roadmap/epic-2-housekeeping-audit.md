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

## Deduplication And Decomposition Targets

These are structural cleanup candidates even when no immediate bug is fixed.
The goal is to make the next Rust backend phases easier to test in small units.

1. Extract Rust type-shape predicates into one module.
   - Current repetition: `resolve_type`, `type_id_is_option`, `type_is_option_or_nihil`, `option_inner_or_self`, and `type_id_is_faber_value` are redefined across `decl.rs`, `mod.rs`, `stmt.rs`, `expr/mod.rs`, `expr/option.rs`, `expr/collection.rs`, and `expr/verte.rs`.
   - Suggested boundary: add a small `codegen/rust/type_shape.rs` or `codegen/rust/expr/shape.rs` with pure helpers over `TypeId` and `TypeTable`.
   - Test payoff: helper tests can cover alias resolution, `Option<T>`, `T ∪ nihil`, `ignotum`, and dynamic unions without snapshotting generated Rust strings.

2. Introduce an expression-emission context object.
   - Current repetition: most expression helpers take `codegen`, `types`, `w`, `in_failable_fn`, `in_entry`, and `suppress_error_propagation`, creating long signatures and making small helpers hard to compose.
   - Suggested boundary: an `ExprEmitter` or `EmitCtx` that owns the invariant context and exposes focused methods such as `expr`, `expr_unwrapped`, `expr_as_type`, and `call_arg`.
   - Test payoff: unit tests can construct one context and test narrow emission helpers without threading eight arguments through every call.

3. Split `expr/call.rs` by responsibility.
   - Current shape: direct calls, method calls, variant constructors, spread expansion, optional-parameter defaults, runtime module bridges, and stdlib collection translations all live in one roughly 1,000-line file.
   - Suggested split: `call/direct.rs`, `call/method.rs`, `call/stdlib.rs`, and `call/args.rs`, or equivalent sibling modules under `expr/call/`.
   - Test payoff: optional/default argument behavior and collection-method translation can be tested independently from runtime bridges and variant constructors.

4. Split `expr/control.rs` by construct family.
   - Current shape: `fac`/`cape`, `si`, `elige`, `discerne`, `itera`, cursor/yield support, and range rendering share one roughly 670-line module.
   - Suggested split: `control/branch.rs`, `control/match.rs`, `control/iteration.rs`, and `control/failable.rs`.
   - Test payoff: range and iteration emitters can get focused tests without pulling in match/failable-control fixtures.

5. Move generated helper prelude emission out of `codegen/rust/mod.rs`.
   - Current shape: the main backend module owns catalog setup, binding/field/variant collection, import detection, entrypoint emission, generated helper types such as `FaberValue`, and final assembly.
   - Suggested split: `prelude.rs` or `helpers.rs` for generated helper types and imports; keep `mod.rs` as orchestration.
   - Test payoff: `FaberValue` helper emission and import detection can be tested directly and reused by future Epic 3 capability-call helper code.

6. Decompose `codegen/rust/mod_test.rs` into companion test modules.
   - Current shape: one roughly 2,700-line test file mixes Epic 2 clusters with older backend tests.
   - Suggested split: `optional_test.rs`, `dynamic_test.rs`, `calls_test.rs`, `collections_test.rs`, `control_test.rs`, and `decl_test.rs` under `codegen/rust/`, wired with the existing `#[cfg(test)] #[path = "..."] mod tests;` convention or a nested test module layout.
   - Test payoff: future changes can run a narrow test name/module and reviewers can see which behavior cluster changed.

7. Replace string-snapshot-heavy tests with helper-level assertions where possible.
   - Current shape: many Epic 2 tests assert exact emitted Rust substrings. That is useful for smoke coverage but makes harmless refactors noisy.
   - Suggested boundary: keep end-to-end generated Rust assertions for public behavior, but add pure helper tests for type-shape, argument planning, optional wrapping decisions, and stdlib method translation selection.
   - Test payoff: decomposition can proceed without every cleanup becoming a brittle output-string migration.

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
