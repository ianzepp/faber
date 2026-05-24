# Epic 2 Phase 24 Delivery: Rust Enum Variant Qualification

Timestamp: 2026-05-24 12:24:31 EDT

## Objective

Fix Rust emission for enum/discretio variant values, constructors, and patterns so valid enum exempla compile as standalone Rust.

## Baseline

Latest Epic 2 Rust e2e result: 83/100 exempla pass.

Representative failures:

- `discerne/discerne.fab` emits match patterns such as `Click { x, y }` instead of `Event::Click { x, y }`.
- `finge/finge.fab` emits constructors such as `Active()` and tuple-style `Click(100, 200)` even though Rust enum variants are unit or struct-like.
- `ordo/ordo.fab` emits enum values and match patterns such as `rubrum` instead of `Color::rubrum`.

## Implementation Plan

1. Collect Rust backend metadata from each enum variant to its parent enum and payload field names.
2. Emit variant paths as `Enum::Variant`.
3. Emit direct variant calls as Rust enum constructors:
   - unit variants: `Enum::Variant`
   - payload variants: `Enum::Variant { field: value, ... }`
4. Emit variant patterns with the same enum qualification.
5. Add focused Rust codegen tests and run the full validation gate.

## Non-Goals

- Do not change enum/discretio syntax, typechecking, or exhaustiveness policy.
- Do not implement dynamic object-map support in this phase.
- Do not change the current enum declaration shape.
