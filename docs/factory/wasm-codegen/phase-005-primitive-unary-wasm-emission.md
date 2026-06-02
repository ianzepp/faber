# Phase 005: Primitive Unary Wasm Emission

## Interpreted Problem

Phase 004 moved primitive branch-heavy functions through compile-valid Wasm, but
nearby numeric exemplars still stopped at Wasm emission on `unary value`.

## Normalized Spec

- Keep HIR -> typed HIR -> MIR -> Wasm intact.
- Emit Wasm for scalar MIR unary values in the existing primitive subset.
- Support numeric negation and boolean not.
- Keep nullable predicates and runtime-managed values fail-closed.
- Validate generated WAT in focused tests and rerun the ignored Wasm exemplar
  harness.

## Repo-Aware Baseline

`custodi/custodi.fab` already reached MIR after Phase 004 and failed only at
Wasm emission because negative constants are represented as unary negation.
`unarius/unarius.fab` still contains MIR-lowering gaps for predicate-style
unary forms, so it is not a target for this Wasm-only phase.

## Stage Graph

1. Lower MIR `Neg` on `i64` to `i64.sub 0, operand`.
2. Lower MIR `Not` on `i32` booleans to `i32.eqz`.
3. Lower MIR `BitNot` on `i64` to xor with `-1`.
4. Continue rejecting nil predicates and non-scalar unary values explicitly.
5. Update the baseline ledger with the measured corpus result.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when focused unary WAT validates, `custodi/custodi.fab`
advances to compile-valid, and host/runtime tiers remain separated from
compiler/codegen failures.
