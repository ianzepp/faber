# Phase 003 Scalar Type And Operation Coverage

**Status**: implemented
**Date**: 2026-06-04
**Scope Source**: `docs/factory/llvm-codegen/continuation-plan.md`

## Objective

Expand the experimental MIR-backed LLVM text probe from integer-only arithmetic
to scalar float, comparison, and boolean operations that MIR already represents
with target-neutral facts. Keep runtime calls, text, aggregates, nullable
values, control-flow expansion, calls, verifier-valid claims, and native
execution out of this phase.

## Requirements

- Add `fractus` constants.
- Add floating arithmetic with LLVM spelling.
- Add scalar comparisons for `numerus`, `fractus`, and `bivalens` where MIR has
  enough operand type facts.
- Add boolean unary and binary operations that map directly to scalar LLVM IR.
- Add focused LLVM tests covering integer, float, and boolean scalar functions.
- Raise the ignored LLVM exempla emitted tier if Phase 002 exists.
- Preserve Wasm scalar behavior.
- Run focused LLVM tests, ignored LLVM e2e, MIR tests, Wasm tests, full radix
  tests, and lint.

## Implementation Notes

- LLVM scalar type selection now tracks MIR local, temp, and value types in the
  function context instead of inferring from result text.
- Binary opcode selection uses the left operand MIR type so comparisons can
  return `bivalens` while still choosing `icmp` or `fcmp` from the compared
  scalar type.
- `fractus` lowers to LLVM `double`.
- Integer operations use `add`, `sub`, `mul`, `sdiv`, and `srem`.
- Floating operations use `fadd`, `fsub`, `fmul`, `fdiv`, and `frem`.
- Integer comparisons use signed `icmp`; floating comparisons use ordered
  `fcmp`; boolean equality uses `icmp`.
- Boolean `non`, `et`, and `aut` lower to `xor`, `and`, and `or`.
- `examples/exempla/scalaria/scalaria.fab` gives the ignored LLVM e2e harness a
  target-neutral scalar-only corpus file.

## Measured Baseline

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed

LLVM e2e exempla:
  frontend analyzed: 102/102
  MIR lowered: 74/102
  LLVM emitted: 1/102
  frontend failed: 0
  MIR lowering failed: 28
  unsupported diagnostic: 73
  emission failed: 0
  output write failed: 0
```

## Validation

```text
cargo test -p radix llvm -- --nocapture
result: 6 passed, 0 failed, 1 ignored

cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed

cargo test -p radix mir -- --nocapture
result: 121 passed, 0 failed

cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored

cargo test -p radix
result: 538 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed
```

## Wasm Implications

This phase should not change MIR shapes or Wasm lowering. Wasm tests are still
required because the scalar operation surface overlaps the Wasm backend.

## Completion Audit

- `fractus` constants supported: yes.
- Floating arithmetic supported: yes.
- Scalar comparisons supported: yes.
- Boolean unary/binary scalar operations supported: yes.
- Focused LLVM tests cover integer, float, and boolean scalar functions:
  yes.
- LLVM exempla emitted tier rises: yes, from 0/101 to 1/102.
- Wasm scalar behavior remains unchanged: yes, Wasm filter passed.
- Required commands pass: yes.
