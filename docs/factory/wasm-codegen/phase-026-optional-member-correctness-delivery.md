# Phase 026: Optional And Member Shape Correctness

## Interpreted Problem

The live Wasm e2e harness has advanced to `75/101` compile-valid exempla, but
several examples still stop after frontend analysis because MIR validation sees
invalid optional/member/aggregate shapes rather than explicit source diagnostics
or valid target-neutral MIR.

Current representative failures:

- `functio/optionalis.fab`: call argument count mismatches on genus-style calls.
- `optionalis/optionalis.fab`: optional chains on non-nullable bases and named
  aggregate missing fields.
- `vel/vel.fab`: coalesce input is treated as non-nullable.
- `destructura/objectum.fab`, `membrum/membrum.fab`, `praefixum/praefixum.fab`:
  field projection bases lose struct/record shape.
- `generis/generis.fab`: named aggregate references an unknown field.
- `vocatio/vocatio.fab`: call argument count/type mismatches.

## Normalized Spec

- Treat this as correctness mode: fix root causes in type/HIR/MIR lowering or
  validator invariants, not by loosening Wasm emission.
- Preserve MIR target-neutral facts needed by Wasm opaque handles and future
  LLVM aggregate/nullable layout lowering.
- Add focused regression tests for representative named exempla.
- Keep unsupported language surfaces explicit; do not hide provider, async,
  callable, dynamic pattern, top-level const, spread, or cast gaps.
- Raise tier floors only when the ignored harness proves the new counts.

## Checkpoints

- Named invalid-MIR clusters either lower to valid MIR/Wasm where semantics are
  already available, or produce earlier and clearer diagnostics.
- No Wasm emitter guesses around missing type information.
- Current floors remain at least:
  - `compile-valid: 75/101`
  - `instantiate-valid: 75/101`
  - `runnable: 74/101`
  - `behavior-checked: 6/101`

## Gate Plan

- Focused regression tests for optional/member/call lowering.
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
