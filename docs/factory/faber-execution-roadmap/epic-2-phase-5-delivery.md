# Epic 2 Phase 5 Delivery: Rust `fractus` Literal Spelling

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust primitive literal backend slice

## Interpreted Problem

`examples/exempla/functio/typicus.fab` uses `fractus` literals such as `2.0`, `3.0`, and `7.0`. The Rust backend emits `HirLiteral::Float` with `f64::to_string()`, which renders integral float values as `2`, `3`, and `7`. Rust then treats those generated literals as integers, causing division and function-call type errors.

## Normalized Spec

Emit Rust float literals with an explicit fractional marker when the stored float is finite and has no fractional digits in Rust's default display form. Do not use this slice to change arithmetic promotion for `numerus / numerus`; that remains a separate backend/typechecking issue.

## Checkpoints

- `2.0`-shaped `fractus` values emit as Rust float literals, not integer literals.
- Non-integral float literals keep their normal spelling.
- `functio/typicus.fab` no longer appears in the Rust e2e failure list.

## Validation

- `cargo test -p radix emits_integral_fractus_literals_as_rust_floats -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `65/100` exempla files passing end-to-end.
- `examples/exempla/functio/typicus.fab` no longer appears in the Rust e2e failure list.
