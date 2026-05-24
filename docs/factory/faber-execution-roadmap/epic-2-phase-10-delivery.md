# Epic 2 Phase 10 Delivery: Borrowed Array Iteration

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust ownership lowering for `itera ex` over arrays

## Interpreted Problem

Several valid standalone exempla reuse an array after `itera ex`. The Rust backend currently emits `for item in array`, which consumes the `Vec<T>` and causes move errors when the source is used again or when nested loops reuse an inner collection.

## Normalized Spec

For `itera ex` over a semantically known Faber array, emit Rust iteration by shared reference and clone each element into the Faber loop binding. Leave `itera pro` range lowering and non-array iteration unchanged.

## Checkpoints

- Array `itera ex` no longer consumes the source `Vec`.
- Loop bodies still receive value-shaped bindings, not Rust references.
- `itera/nidificatus.fab` no longer fails on moved nested array sources.
- Rust e2e pass count improves or newly exposed blockers are documented.

## Validation

- `cargo test -p radix emits_borrowed_iteration_for_lista_itera_ex -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `70/100` exempla files passing end-to-end.
- `examples/exempla/itera/nidificatus.fab` no longer appears in the Rust e2e failure list.
- `examples/exempla/itera/in-functione.fab` still fails, but now at repeated by-value function calls with `numbers`; that is a separate function-argument ownership phase.
