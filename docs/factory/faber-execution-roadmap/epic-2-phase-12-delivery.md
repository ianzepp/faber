# Epic 2 Phase 12 Delivery: Non-Consuming `vel` Coalescing

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust ownership lowering for option coalescing

## Interpreted Problem

`examples/exempla/vel/vel.fab` reuses an optional `textus` after a `vel` expression. Rust codegen currently emits `name.unwrap_or(...)`, which consumes the `Option<String>` and causes a later use-of-moved-value failure.

## Normalized Spec

For semantically known `Option<T>` operands, emit Rust coalescing through a clone of the option before `.unwrap_or(...)` or `.or(...)`. Keep non-option `vel` behavior unchanged.

## Checkpoints

- `name vel "fallback"` no longer consumes `name`.
- Option-preserving coalescing chains still work.
- `vel/vel.fab` no longer appears in the Rust e2e failure list.

## Validation

- `cargo test -p radix emits_non_consuming_option_coalesce -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `73/100` exempla files passing end-to-end.
- `examples/exempla/vel/vel.fab` no longer appears in the Rust e2e failure list.
