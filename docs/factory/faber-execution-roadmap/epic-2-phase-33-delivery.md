# Epic 2 Phase 33 Delivery: Rust Dynamic Context Coercions

## Interpreted Problem

Phase 32 introduced generated `FaberValue` for Rust dynamic object values, raising the Rust e2e corpus to `95/100`. The remaining Epic 2 failures now expose places where Rust codegen emits concrete values into contexts that are already typed as `FaberValue`, such as assignments, function arguments, boolean/type checks, and map spread.

## Normalized Spec

- Emit `FaberValue` coercions whenever a Rust expression is written into a dynamic target context.
- Cover mutable assignment into dynamic locals.
- Cover direct function-call arguments whose parameter type is `ignotum`/anonymous union.
- Cover equality/type-check predicates between `FaberValue` and concrete/nil values.
- Cover map index assignment, empty object literals, and object/map spread when dynamic values participate.
- Keep the phase scoped to Rust codegen; do not alter Faber syntax or semantic inference rules.
- Do not solve `ad/ad.fab`; that remains Epic 3.

## Repo-Aware Baseline

- Dynamic type rendering and helper emission live in `crates/radix/src/codegen/rust/types.rs` and `crates/radix/src/codegen/rust/mod.rs`.
- Dynamic construction logic lives in `crates/radix/src/codegen/rust/expr/verte.rs` and `collection.rs`.
- Assignment lowering lives in `crates/radix/src/codegen/rust/expr/ops.rs`.
- Direct call lowering lives in `crates/radix/src/codegen/rust/expr/call.rs`.
- Predicate lowering lives in `crates/radix/src/codegen/rust/expr/ops.rs`.

## Stage Graph

1. Add a shared Rust expression helper for emitting values as `FaberValue`.
2. Use that helper for assignment and map index assignment dynamic targets.
3. Add parameter type metadata to Rust direct-call argument emission and coerce dynamic parameter arguments.
4. Coerce predicate operands when one side is dynamic and the other is concrete or nil.
5. Recheck direct exempla and the ignored Rust e2e harness.

## Checkpoints

- Focused tests prove direct calls, assignments, and comparisons use `FaberValue` coercions.
- `destructura/objectum.fab` and `si/est.fab` should advance if no unrelated blocker remains.
- Full Rust e2e count is recorded after the phase.

## Gate Plan

- `cargo test -p radix rust_dynamic_context_coercions`
- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
- Poker-face audit before commit.

## Open Questions

- Optional chaining over map-backed object values remains separate; it is the `membrum/membrum.fab` blocker.
