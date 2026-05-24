# Epic 2 Phase 7 Delivery: `mori` Fractus And Numeric Indexing

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, `mori` exemplar plus Rust array-index lowering

## Interpreted Problem

`examples/exempla/mori/mori.fab` is a valid standalone panic example, but it currently mixes two issues. Its `divide` function promises `fractus` while returning `numerus / numerus`, and generated Rust indexes `Vec<T>` with Faber `numerus` (`i64`) directly.

## Normalized Spec

Make the exemplar explicit about floating-point division, and teach Rust codegen to cast numeric array indexes to `usize` when indexing Rust arrays/vectors. Do not broaden this slice into ownership fixes for destructuring or general numeric promotion.

## Checkpoints

- `mori/mori.fab` keeps demonstrating `mori` for fatal invariant violations.
- Generated Rust for `items[index]` uses a Rust-compatible `usize` index.
- Focused regression coverage proves numeric list indexing no longer emits a raw `i64` index.
- Rust e2e no longer reports `mori/mori.fab` as a failed exemplar.

## Validation

- `cargo test -p radix emits_usize_cast_for_lista_indexing -- --nocapture`
- `cargo run -p faber -- check examples/exempla/mori/mori.fab`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `67/100` exempla files passing end-to-end.
- `examples/exempla/mori/mori.fab` no longer appears in the Rust e2e failure list.
