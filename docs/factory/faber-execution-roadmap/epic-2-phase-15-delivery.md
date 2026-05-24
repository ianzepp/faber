# Epic 2 Phase 15 Delivery: Rust Textus Match and Length

Timestamp: 2026-05-24 11:28:19 EDT

## Objective

Fix a narrow Rust backend gap for `textus` operations:

- `elige` over `textus` should emit a Rust string-slice scrutinee so literal string cases typecheck.
- `textus.longitudo()` should lower to native Rust string length.

## Baseline

Latest Epic 2 Rust e2e result: 75/100 exempla pass.

`examples/exempla/elige/ergo-redde.fab` currently emits:

```rust
match name {
    "textus" => { ... }
}
```

That compares a `String` scrutinee to `&str` patterns and fails in rustc. `textus.longitudo()` also leaks through as a direct `String` method in examples such as `incipiet`.

## Implementation Plan

1. Add method-specific semantic typing for the `textus` stdlib surface needed here.
2. Add Rust codegen translation for `textus.longitudo()`.
3. Emit `match <textus>.as_str()` for single-scrutinee text matches.
4. Add focused regression tests and run the Rust exempla e2e harness.

## Non-Goals

- Do not attempt full async `incipiet` support in this phase.
- Do not implement optional parameter/default ABI support.
- Do not expand all string stdlib methods unless directly needed for the targeted examples.
