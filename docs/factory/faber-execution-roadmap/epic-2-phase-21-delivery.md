# Epic 2 Phase 21 Delivery: Rust Self-Return Methods

## Interpreted Problem

After real Rust receivers were added, mutating methods that `redde ego` now emit `return self` from a method whose signature still returns the struct by value. Rust correctly rejects `&mut Self` as `Self`. This blocks method-chaining examples such as `calc.setValue(5).double().getResult()`.

## Normalized Spec

- For inherent Rust methods whose declared return type is the enclosing struct and whose body returns `ego`, emit a borrowed self return type: `&mut Struct`.
- Preserve ordinary struct-returning functions and methods that do not return `ego`.
- Preserve `&self` for read-only methods and `&mut self` for mutating methods.
- Do not solve `sparge` call argument lowering in this phase.

## Repo-Aware Baseline

- Phase 19 added current-self lowering, so `ego` already renders as `self` inside methods.
- Phase 19 also infers `&mut self` for methods assigning through `ego`.
- `vocatio/vocatio.fab` now fails on `expected Calculator, found &mut Calculator` for `redde ego`, plus a separate spread-call error.

## Stage Graph

1. Add method-body detection for explicit `redde ego`.
2. Override the generated Rust return type for matching self-return methods.
3. Add focused codegen coverage proving `&mut Struct` return and chainable calls.
4. Run focused validation and the ignored Rust e2e suite.

## Epic Candidates And Scopable Issues

This phase fixes only self-returning methods. Spread calls still need parser/HIR preservation of the `sparge` argument flag and stay out of scope.

## Checkpoints

- `setValue`-style methods emit `fn setValue(&mut self, ...) -> &mut Calculator`.
- `redde ego` inside those methods emits `return self;`.
- `vocatio` has no remaining self-return mismatch; any remaining failure is the spread-call blocker.

## Companion Skill Plan

- `factory`: preserve phase boundary, validation, and commit discipline.
- `delivery`: this saved implementation artifact.

## Gate Plan

- Focused Rust codegen test for chainable `redde ego`.
- Direct Rust emission for `examples/exempla/vocatio/vocatio.fab`.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`.

## Open Questions

- Whether the language should model chainable self returns as references in semantic type information remains open; this phase is a Rust ABI bridge for the current backend.
