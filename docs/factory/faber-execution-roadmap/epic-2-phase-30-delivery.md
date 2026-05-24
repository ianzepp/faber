# Epic 2 Phase 30 Delivery: Rust Async Entry And Futura Functions

Timestamp: 2026-05-24 16:04:00 EDT

## Objective

Make Rust output for `incipiet/incipiet.fab` compile and run as a standalone executable by preserving `@ futura` function metadata and giving generated entry code a minimal async execution boundary.

## Baseline

Latest Epic 2 Rust e2e result: 91/100 exempla pass.

`examples/exempla/incipiet/incipiet.fab` currently emits invalid Rust:

- `@ futura` functions lower with `is_async: false`, so Rust codegen emits ordinary `fn fetchData() -> String`.
- `cede fetchData()` still emits `fetchData().await`, producing `no field await on type String`.
- The standalone Rust e2e harness invokes `rustc` directly, so the fix cannot depend on Tokio or another external runtime.

## Implementation Plan

1. Preserve parsed `@ futura` annotations on top-level functions and genus methods into `HirFunction::is_async`.
2. Keep existing Rust function emission policy that maps `HirFunction::is_async` to `async fn`.
3. Detect entry blocks containing `cede` and wrap generated `main` body in a small std-only `__faber_block_on(async { ... })` boundary.
4. Emit the block-on helper only when needed.
5. Add focused Rust codegen coverage for `@ futura`, `.await`, and the generated block-on boundary.
6. Compile standalone Rust e2e outputs with Rust edition 2021 so generated `async`/`.await` syntax is validated under the intended Rust language edition.
7. Validate with focused tests, direct `incipiet/incipiet.fab` rustc/run, `cargo test -p radix`, and the ignored Rust e2e harness.

## Non-Goals

- Do not implement `@ cursor` generator lowering in this phase.
- Do not introduce an external async runtime dependency.
- Do not alter `ad` capability behavior or the dynamic `ignotum` object cluster.

## Validation

- `cargo test -p radix emits_async_futura_functions_and_entry_block_on --lib`
  - Result: 1 passed.
- `cargo run -p faber -- emit -t rust examples/exempla/incipiet/incipiet.fab > /tmp/faber-phase30-incipiet.rs`
- `rustc --edition=2021 /tmp/faber-phase30-incipiet.rs -o /tmp/faber-phase30-incipiet`
- `/tmp/faber-phase30-incipiet`
  - Output: `Starting async program...`, `Received: data loaded`, `Length: 11`, `Program complete`
- `cargo test -p radix`
  - Result: 453 passed, 0 failed, 3 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
  - Result: `Rust e2e exempla: 92/100 exempla files pass end-to-end`.
  - `incipiet/incipiet.fab` no longer appears in the failing exemplar list.

Remaining Rust e2e failures after this phase:

- `ad/ad.fab`
- `destructura/objectum.fab`
- `itera/cursor-iteratio.fab`
- `itera/de.fab`
- `membrum/membrum.fab`
- `objectum/objectum.fab`
- `praefixum/praefixum.fab`
- `si/est.fab`
