# Epic 2 Phase 17 Delivery: Rust Optional Return Values

Timestamp: 2026-05-24 11:36:04 EDT

## Objective

Fix Rust emission for nullable (`T ∪ nihil` / `Option<T>` in Rust) value boundaries:

- explicit `redde` statements inside nullable-returning functions;
- typed `nihil ∷ T ∪ nihil` local initializers.

## Baseline

Latest Epic 2 Rust e2e result: 79/100 exempla pass.

`examples/exempla/si/ergo-redde.fab` currently emits raw values from functions returning `Option<T>`:

```rust
fn divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 { return None; }
    return a / b;
}
```

Rust requires non-null success values to be wrapped as `Some(...)`.

## Implementation Plan

1. Track the current Rust function return type during body emission.
2. In `redde`, wrap a non-optional, non-`nihil` value in `Some(...)` when the current function returns `Option<T>`.
3. Avoid double-wrapping typed `nihil` as `Some(None)` for optional locals.
4. Keep these codegen rules target-shape driven; do not guess missing semantic types.
5. Add focused regression coverage.
6. Validate with focused tests, direct example emission, and the Rust exempla e2e harness.

## Non-Goals

- Do not implement optional parameter/default ABI support in this phase.
- Do not fix `itera de` key iteration, even though it remains in `si/ergo-redde.fab`.
- Do not rewrite examples to avoid nullable returns.
