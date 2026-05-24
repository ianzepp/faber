# Epic 2 Phase 31 Delivery: Rust Cursor Functions

Timestamp: 2026-05-24 16:32:00 EDT

## Objective

Make Rust output for `itera/cursor-iteratio.fab` compile and run by honoring `@ cursor` functions as iterable value producers.

## Baseline

Latest Epic 2 Rust e2e result: 92/100 exempla pass.

`examples/exempla/itera/cursor-iteratio.fab` currently emits invalid Rust because:

- `@ cursor` functions lower with `is_generator: false`.
- `cede i` inside a cursor function emits `i.await` instead of yielding `i`.
- `rangeSync(...)` is emitted as returning `i64`, but callers use it in `itera ex`, which requires an iterable result.

## Implementation Plan

1. Preserve parsed `@ cursor` annotations on top-level functions and genus methods into `HirFunction::is_generator`.
2. Type generator function signatures as `lista<T>`/array results where `T` is the declared source return item type.
3. Emit Rust cursor functions as `Vec<T>` producers.
4. Lower expression-statement `cede value` inside a generator function to `__faber_yielded.push(value);`.
5. Keep async cursor consumption out of scope; unused `@ futura @ cursor` functions may compile as `async fn -> Vec<T>`.
6. Add focused Rust codegen coverage for sync and async cursor function emission.
7. Validate with focused tests, direct `itera/cursor-iteratio.fab` rustc/run, `cargo test -p radix`, and the ignored Rust e2e harness.

## Non-Goals

- Do not implement streaming/lazy generator state machines.
- Do not implement async iteration syntax.
- Do not address `ad` or the dynamic `ignotum` object cluster.

## Validation

- `cargo test -p radix emits_cursor_functions_as_vec_producers --lib`
  - Result: 1 passed.
- `cargo run -p faber -- emit -t rust examples/exempla/itera/cursor-iteratio.fab > /tmp/faber-phase31-cursor.rs`
- `rustc --edition=2021 /tmp/faber-phase31-cursor.rs -o /tmp/faber-phase31-cursor`
- `/tmp/faber-phase31-cursor`
  - Output includes `Sync cursor iteration:` and `Sync collected: [0, 2, 4, 6, 8]`.
- `cargo test -p radix`
  - Result: 454 passed, 0 failed, 3 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
  - Result: `Rust e2e exempla: 93/100 exempla files pass end-to-end`.
  - `itera/cursor-iteratio.fab` no longer appears in the failing exemplar list.

Remaining Rust e2e failures after this phase:

- `ad/ad.fab`
- `destructura/objectum.fab`
- `itera/de.fab`
- `membrum/membrum.fab`
- `objectum/objectum.fab`
- `praefixum/praefixum.fab`
- `si/est.fab`
