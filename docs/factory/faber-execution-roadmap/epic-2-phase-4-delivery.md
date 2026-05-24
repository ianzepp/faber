# Epic 2 Phase 4 Delivery: Standalone `inter` Example

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, exempla standalone corpus cleanup

## Interpreted Problem

`examples/exempla/inter/inter.fab` is meant to demonstrate the `inter` membership operator, but its executable checks live at top level beside inferred `fixum` declarations. The Rust backend emits the string declaration as a global `pub const String`, which is invalid Rust because `"active".to_string()` is not const.

## Normalized Spec

Keep the example as a standalone single-file executable by moving the declarations and membership checks into `incipit`. Do not change Rust global constant semantics for this narrow corpus-shape issue.

## Checkpoints

- The example still demonstrates string and numeric `inter` membership.
- The source no longer requires top-level dynamic `String` construction.
- Rust e2e no longer reports `inter/inter.fab` as a failed exemplar.

## Validation

- `cargo run -p faber -- check examples/exempla/inter/inter.fab`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `64/100` exempla files passing end-to-end.
- `examples/exempla/inter/inter.fab` no longer appears in the Rust e2e failure list.
