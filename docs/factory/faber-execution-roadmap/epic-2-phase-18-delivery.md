# Epic 2 Phase 18 Delivery: Rust Itera De Keys And Indices

Timestamp: 2026-05-24 11:43:03 EDT

## Objective

Fix Rust lowering for `itera de` so it matches Faber semantics:

- `itera de tabula<K,V>` binds keys, not `(key, value)` entries.
- `itera de lista<T>` binds numeric indices, not values.
- Map indexing with a key expression borrows the key for Rust `HashMap` indexing.

## Baseline

Latest Epic 2 Rust e2e result: 79/100 exempla pass.

`examples/exempla/si/ergo-redde.fab` currently fails because `itera de obj fixum k` emits `for k in obj`, so `k` is a `(String, i64)` entry tuple instead of a `String` key. `examples/exempla/itera/de.fab` has the same key/index lowering gap, plus separate heterogeneous object-map boxing failures that are not the target of this phase.

## Implementation Plan

1. Add Rust codegen branches for `HirIteraMode::De` over arrays and maps.
2. Lower array `de` iteration to `0..len`.
3. Lower map `de` iteration to `.keys().cloned()`.
4. Borrow map index expressions as `map[&key]`.
5. Add focused regression coverage and run e2e.

## Non-Goals

- Do not fix heterogeneous object-map `Box<dyn Any>` insertion in this phase.
- Do not change `itera ex` borrowed value iteration.
- Do not change parser or HIR shape.
