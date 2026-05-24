# Epic 2 Phase 9 Delivery: Radix-Aware `conversio`

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, Rust primitive conversion backend slice

## Interpreted Problem

`examples/exempla/conversio/conversio.fab` is a standalone conversion example, but Rust codegen ignores conversion hint parameters such as `Hex`, `Bin`, and `Oct`. The generated Rust tries to parse `"ff"` as base-10 `i64`, panics at runtime, and fails the e2e harness.

## Normalized Spec

Use the existing HIR `Conversio.params` symbols to select radix-aware Rust integer parsing for `textus ⇒ numerus<..., Hex|Bin|Oct>`. Keep ordinary decimal parsing and fallback behavior unchanged.

## Checkpoints

- Decimal text-to-number conversion still emits normal `.parse::<i64>()`.
- Hex, binary, and octal hinted conversions emit `i64::from_str_radix`.
- Fallback values still work for parse conversions.
- Rust e2e no longer reports `conversio/conversio.fab` as a failed exemplar.

## Validation

- `cargo test -p radix emits_radix_parse_for_hinted_numerus_conversio -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `69/100` exempla files passing end-to-end.
- `examples/exempla/conversio/conversio.fab` no longer appears in the Rust e2e failure list.
