# Epic 2 Phase 27 Delivery: Rust Optional If Expressions

Timestamp: 2026-05-24 14:16:00 EDT

## Objective

Make Rust output for option-shaped `si`/ternary expressions compile and run for the standalone `ternarius/ternarius.fab` exemplar.

## Baseline

Latest Epic 2 Rust e2e result: 88/100 exempla pass.

`examples/exempla/ternarius/ternarius.fab` currently emits invalid Rust for:

```fab
fixum _ result ← nonnihil maybe sic maybe secus "default"
```

The semantic result type is `Option<String>`, but Rust codegen emits:

```rust
Some(if maybe != None { maybe } else { "default" })
```

That mixes `Option<String>` and `&str` inside the Rust `if` and wraps the whole expression rather than each non-option branch.

## Implementation Plan

1. Thread the typed `si` expression result into Rust `if` expression emission.
2. When the result type is `Option<T>`, emit branch bodies that return `Option<T>`:
   - preserve option-producing branch expressions;
   - wrap concrete branch expressions in `Some(...)`;
   - preserve `None` for `nihil`.
3. Teach optional local initialization that option-shaped `si` expressions already produce an option.
4. Add focused Rust codegen coverage for an option-shaped ternary.
5. Run focused validation, full radix tests, and ignored Rust e2e.

## Non-Goals

- Do not implement general flow-sensitive non-null narrowing.
- Do not solve dynamic `ignotum` comparisons.
- Do not touch Epic 3 `ad` capability calls.

## Validation

- `cargo test -p radix option_shaped_if_expression_branches --lib`
- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`

Result: Rust e2e exempla moved from 88/100 to 89/100. `examples/exempla/ternarius/ternarius.fab` now passes end-to-end; the remaining failures are outside this phase's option-shaped `si` expression scope.
