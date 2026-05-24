# Epic 2 Phase 6 Delivery: Explicit Empty `lista` Exemplar

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, standalone corpus source correction

## Interpreted Problem

`examples/exempla/lista/lista.fab` demonstrates array literals, but its empty-list example uses `fixum _ empty ← []`. The project grammar guidance requires empty collections to carry an explicit declared type with `vacua`, so this source is not a valid standalone exemplar obligation for type inference.

After correcting the empty collection source, the same exemplar exposes a Rust backend ownership bug: array spread lowers to `Vec::extend(first)`, which moves the source vector and prevents a later spread from reusing it.

## Normalized Spec

Rewrite the empty-list example to use canonical type-first syntax and `vacua`. Do not teach Rust codegen to infer an element type for untyped empty array literals, because that would violate the explicit empty-collection invariant.

Emit Rust array spread from cloned elements so spread reads from the source collection instead of consuming it.

## Checkpoints

- `lista/lista.fab` uses `fixum lista<T> name ← vacua` for the empty collection.
- The example still demonstrates non-empty arrays, nested arrays, and spread arrays.
- Rust array spread does not move a source vector that can be reused later.
- Rust e2e no longer reports `lista/lista.fab` as a failed exemplar.

## Validation

- `cargo run -p faber -- check examples/exempla/lista/lista.fab`
- `cargo test -p radix emits_array_spread_without_moving_source_vector -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `66/100` exempla files passing end-to-end.
- `examples/exempla/lista/lista.fab` no longer appears in the Rust e2e failure list.
