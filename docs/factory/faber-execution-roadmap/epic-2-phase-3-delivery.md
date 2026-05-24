# Epic 2 Phase 3 Delivery: Rust `itera pro` Ranges

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust range-iteration backend slice

## Interpreted Problem

After the string concatenation slice, the Rust e2e harness reports `61/100` exempla passing. Several valid iteration exempla fail because `itera pro` range sources lower to tuple-like Rust expressions such as `(0, 3)`, and tuples are not iterators.

## Normalized Spec

Teach Rust codegen to emit an iterable Rust expression for `itera pro` when the source expression is a Faber `Intervallum`. Preserve the existing tuple-shaped range expression for non-loop contexts that already depend on it, such as range-like expression handling and text slicing.

## Repo-Aware Baseline

- `examples/exempla/itera/intervallum.fab` fails with tuple-not-iterator diagnostics.
- `examples/exempla/itera/intervallum-gradus.fab` fails with tuple-not-iterator diagnostics.
- `examples/exempla/itera/nidificatus.fab` fails on nested tuple range iteration.
- `examples/exempla/itera/cursor-iteratio.fab` includes `itera pro` inside cursor functions and currently surfaces tuple iterator failures before its later cursor/runtime blockers.

## Stage Graph

1. Inspect HIR/codegen handling for `HirExprKind::Itera` and `HirExprKind::Intervallum`.
2. Add a scoped Rust iterable range emitter for `itera pro` only.
3. Add focused tests for exclusive, inclusive, stepped, and descending ranges.
4. Validate focused range exempla and rerun the ignored e2e harness.

## Checkpoints

- `itera pro 0‥3` no longer emits `for i in (0, 3)`.
- Inclusive and stepped intervals preserve their endpoint/step semantics in generated Rust.
- Non-loop `Intervallum` expression emission remains tuple-shaped.
- Remaining failures in cursor or collection examples are recorded rather than hidden.

## Validation

- `cargo test -p radix emits_iterable_rust_ranges_for_itera_pro -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `63/100` exempla files passing end-to-end.
- `itera/intervallum.fab` and `itera/intervallum-gradus.fab` no longer appear in the Rust e2e failure list.
- `itera/cursor-iteratio.fab` now reaches cursor-specific generated Rust failures; `itera/nidificatus.fab` now reaches ownership movement of `cols` in nested `itera ex`.
