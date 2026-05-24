# Epic 2 Phase 25 Delivery: Rust Optional Chain Access

Timestamp: 2026-05-24 12:32:38 EDT

## Objective

Fix Rust optional-chain emission so `?.` and `?[...]` work for both optional and non-optional receivers.

## Baseline

Latest Epic 2 Rust e2e result: 86/100 exempla pass.

`examples/exempla/optionalis/optionalis.fab` currently emits invalid Rust such as:

- `(alice).as_ref()` where `alice` is a plain `User`, not an `Option<User>`.
- `(items).as_ref().map(...)` where `items` is a plain `Vec<String>`.
- nested optional field access that can produce `Option<Option<T>>` instead of flattening to the source-level nullable result.

## Implementation Plan

1. Thread the optional-chain expression result type into Rust optional access emission if needed.
2. Detect whether the receiver itself is `Option<T>`.
3. Detect whether the selected member/index value is already optional.
4. Emit:
   - non-optional receiver + non-optional value as `Some(value)`;
   - non-optional receiver + optional value as `value`;
   - optional receiver + non-optional value as `.map(...)`;
   - optional receiver + optional value as `.and_then(...)`.
5. Lower optional array/map indexing through `.get(...).cloned()` so out-of-bounds access returns `None`.
6. Add focused Rust codegen coverage and run the full validation gate.

## Non-Goals

- Do not change typechecking or source grammar.
- Do not implement dynamic `ignotum`/object-map support.
- Do not redesign optional call semantics beyond preserving the existing call shape.

## Validation

- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`

Result: Rust e2e exempla moved from 86/100 to 87/100. `examples/exempla/optionalis/optionalis.fab` now passes end-to-end; the remaining failures are outside this phase's optional-chain scope.
