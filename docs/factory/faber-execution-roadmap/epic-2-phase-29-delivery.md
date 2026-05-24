# Epic 2 Phase 29 Delivery: Rust Genus Creo Hooks

Timestamp: 2026-05-24 15:24:00 EDT

## Objective

Make Rust output for `genus/creo.fab` compile and run by honoring `creo()` as a post-construction hook for `genus` values.

## Baseline

Latest Epic 2 Rust e2e result: 90/100 exempla pass.

`examples/exempla/genus/creo.fab` currently fails because:

- Rust struct literals are emitted directly, so the generated `creo(&mut self)` methods are never called after defaults and constructor overrides are merged.
- `Circle.area` is declared as `numerus`, but `creo()` assigns the fractional expression `3.14159 * ego.radius * ego.radius`.

## Implementation Plan

1. Track which Rust-emitted structs declare a `creo` method.
2. Wrap struct literal construction for those structs in a temporary block: build the literal, call `creo()`, then return the initialized value.
3. Reuse existing struct-field emission so defaults, optional fields, and field ordering remain unchanged.
4. Correct the exemplar `Circle.area` field type to `fractus`.
5. Suppress Rust naming-style warnings for generated code so preserved Faber method names such as `getValue` do not fail warning-sensitive exemplar runs.
6. Add focused Rust codegen coverage proving constructor hooks are called for typed struct construction.
7. Validate with focused tests, `cargo test -p radix`, direct `genus/creo.fab` rustc/run, and the ignored Rust e2e harness.

## Non-Goals

- Do not add constructor parameters or new source syntax.
- Do not change method receiver inference beyond the existing `ego` mutation detection.
- Do not address the dynamic `ignotum` object cluster.

## Validation

- `cargo test -p radix typed_struct_construction_calls_creo_hook --lib`
  - Result: 1 passed.
- `cargo run -p faber -- emit -t rust examples/exempla/genus/creo.fab > /tmp/faber-phase29-creo.rs`
- `rustc --edition=2021 /tmp/faber-phase29-creo.rs -o /tmp/faber-phase29-creo`
- `/tmp/faber-phase29-creo`
  - Output: `50`, `100`, `5`, `10`, `78.53975`
- `cargo test -p radix`
  - Result: 452 passed, 0 failed, 3 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
  - Result: `Rust e2e exempla: 91/100 exempla files pass end-to-end`.
  - `genus/creo.fab` no longer appears in the failing exemplar list.

Remaining Rust e2e failures after this phase:

- `ad/ad.fab`
- `destructura/objectum.fab`
- `incipiet/incipiet.fab`
- `itera/cursor-iteratio.fab`
- `itera/de.fab`
- `membrum/membrum.fab`
- `objectum/objectum.fab`
- `praefixum/praefixum.fab`
- `si/est.fab`
