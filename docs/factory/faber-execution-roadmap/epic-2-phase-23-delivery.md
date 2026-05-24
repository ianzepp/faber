# Epic 2 Phase 23 Delivery: Rust Mixed Numeric Arithmetic

Timestamp: 2026-05-24 12:19:50 EDT

## Objective

Fix Rust emission for arithmetic expressions whose semantic result is `fractus` while one or both operands are `numerus`.

## Baseline

Latest Epic 2 Rust e2e result: 83/100 exempla pass.

Representative remaining failures:

- `praefixum/praefixum.fab` emits `2 * 3.14159`, which Rust rejects as integer-by-float arithmetic.
- `genus/creo.fab` emits `3.14159 * self.radius`, where `radius` is `i64` and the expression result is `fractus`.

Typechecking already records numeric widening: arithmetic with a `fractus` operand has `fractus` result, and contextual `fractus` expectations can widen an otherwise integral arithmetic expression. The Rust backend currently emits the operands without using that expression result type, leaving Rust to reject mixed numeric operations.

## Implementation Plan

1. Thread the binary expression result type into Rust operator emission.
2. For arithmetic operators with `fractus` result, emit `as f64` around operands that are semantically `numerus`.
3. Preserve text concatenation, comparisons, coalescing, and range/membership special cases unchanged.
4. Add focused Rust codegen tests for mixed literal/member arithmetic and contextual `fractus` integer division.
5. Validate with focused tests, `cargo test -p radix`, and the ignored Rust e2e harness.

## Non-Goals

- Do not fix heterogeneous object-map `Box<dyn Any>` insertion or dot access in this phase.
- Do not change typechecking policy.
- Do not guess missing types in codegen; only act when semantic types say the arithmetic result is `fractus`.
