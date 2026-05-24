# Epic 2 Phase 14 Delivery: Rust Lista Morphology Methods

Timestamp: 2026-05-24 11:19:19 EDT

## Objective

Make `examples/exempla/morphologia/morphologia.fab` compile and run through the Rust e2e harness by implementing the `lista` stdlib method contracts it exercises:

- `filtrata((T) -> bivalens) -> lista<T>`
- `mappata((T) -> U) -> lista<U>`
- `addita(T) -> lista<T>`
- `inversa() -> lista<T>`
- `inverte() -> vacuum`
- `ordinata() -> lista<T>`

## Baseline

Latest Epic 2 Rust e2e result: 74/100 exempla pass.

Current `morphologia` Rust emission is invalid because it leaves direct `Vec` method calls such as `items.filtrata(...)`, `items.addita(...)`, and `items.inversa()`. The semantic pass also assigns wrong return types for some array methods because its temporary fallback treats all zero-arg array methods as `numerus` and non-function single-arg array methods as `vacuum`.

## Implementation Plan

1. Replace the broad array-method fallback in semantic typechecking with method-specific `lista` contracts for the methods above.
2. Add Rust codegen translations guarded by receiver array type and arity.
3. Add focused regression coverage for Rust emission and semantic return shape.
4. Validate with the focused tests, direct `morphologia` emission, and the ignored Rust exempla e2e harness.

## Non-Goals

- Do not implement every method in `stdlib/norma/innatum/lista.fab`.
- Do not invent new syntax or change the example source unless needed after the compiler fix.
- Do not paper over missing semantic type information in codegen.
