# Epic 2 Phase 2 Delivery: Rust String Concatenation

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`
**Date**: 2026-05-24
**Scope**: Epic 2, first core Rust backend stabilization slice

## Interpreted Problem

After corpus-boundary relocation, the Rust e2e harness reports `59/100` exempla passing. Two remaining valid exempla expose the same Rust backend bug: Faber `textus` concatenation lowers to Rust `String + String`, but Rust requires `String + &str` for the `Add` implementation. `assignatio/assignatio.fab` fails on `+=`, and `redde/redde.fab` fails on returning `"Hello, " + name`.

## Normalized Spec

Fix Rust codegen for text concatenation without guessing missing type information. Use semantic type evidence to handle only string/string addition and add-assignment. Leave unrelated numeric, object, enum, method, option, range, and ownership failures for later Epic 2 phases.

## Repo-Aware Baseline

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `59/100`.
- `examples/exempla/assignatio/assignatio.fab` fails with `s += " world".to_string()` expecting `&str`.
- `examples/exempla/redde/redde.fab` fails with `"Hello, ".to_string() + name` expecting `&str`.
- Rust codegen must not synthesize string behavior for untyped or unknown operands.

## Stage Graph

1. Inspect Rust expression/statement codegen for binary `+` and assignment `+=`.
2. Add type-gated string concatenation handling.
3. Add focused Rust codegen tests that cover `String + String` and `String += String`.
4. Validate the two exemplar files and rerun the ignored e2e harness.

## Checkpoints

- Generated Rust for text `+` compiles when both operands are known text.
- Generated Rust for text `+=` compiles when both sides are known text.
- Non-text arithmetic output remains unchanged.
- The two named exemplar failures no longer report Rust `String`/`&str` mismatches.

## Validation

- `cargo test -p radix emits_text_concatenation_without_invalid_string_add -- --nocapture`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `61/100` exempla files passing end-to-end.
- `assignatio/assignatio.fab` and `redde/redde.fab` no longer appear in the Rust e2e failure list.
