# Epic 2 Phase 11 Delivery: Owned Destructuring From Array Indexes

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust ownership lowering for array destructuring

## Interpreted Problem

Array destructuring lowers into local bindings initialized from indexed source expressions. For nested arrays, generated Rust tries to move a `Vec<T>` out of `matrix[index]`, which Rust rejects.

## Normalized Spec

When a local initializer is an indexed expression and the local's semantic type is an owned array or text value, clone the indexed value at the local boundary. Do not change index assignment or map/dynamic object behavior in this phase.

## Checkpoints

- Nested array destructuring emits `.clone()` for owned vector elements.
- Scalar destructuring remains valid.
- `destructura/destructura.fab` and `varia/destructura.fab` no longer fail on moving nested arrays out of indexes.
- Rust e2e pass count improves or newly exposed blockers are documented.

## Validation

- `cargo test -p radix clones_owned_array_values_from_indexed_locals -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `72/100` exempla files passing end-to-end.
- `examples/exempla/destructura/destructura.fab` and `examples/exempla/varia/destructura.fab` no longer appear in the Rust e2e failure list.
