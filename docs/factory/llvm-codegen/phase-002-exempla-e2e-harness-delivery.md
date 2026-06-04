# Phase 002 LLVM Exempla E2E Harness

**Status**: implemented  
**Date**: 2026-06-04  
**Scope Source**: `docs/factory/llvm-codegen/continuation-plan.md`

## Objective

Add an ignored LLVM exempla e2e harness modeled on the Wasm harness so LLVM
progress across the shared corpus is measurable before expanding scalar
coverage. This phase does not add verifier-valid or native execution tiers.

## Requirements

- Add an ignored `exempla_llvm_e2e` harness.
- Scan the same `examples/exempla` corpus used by Wasm.
- Distinguish frontend analysis, MIR lowering, LLVM text emission, unsupported
  LLVM diagnostics, and unexpected failures.
- Do not claim verifier-valid output without a verifier policy.
- Do not add execution or native run tiers.
- Record measured counts after the harness exists.
- Run focused LLVM tests, the ignored LLVM harness, MIR tests, full radix tests,
  and lint.

## Implementation Notes

- Harness module: `crates/radix/src/exempla_e2e/llvm.rs`
- Module registration: `crates/radix/src/exempla_e2e/mod.rs`
- Temporary LLVM text outputs are written under the existing e2e temp root.
- Measured floors live in `crates/radix/src/exempla_e2e/llvm.rs`:
  frontend analyzed 101, MIR lowered 73, LLVM emitted 0, unsupported
  diagnostics 73.

## Measured Baseline

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed

LLVM e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 73/101
  LLVM emitted: 0/101
  frontend failed: 0
  MIR lowering failed: 28
  unsupported diagnostic: 73
  emission failed: 0
  output write failed: 0
```

## Validation

```text
cargo test -p radix llvm -- --nocapture
result: 3 passed, 0 failed, 1 ignored

cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed

cargo test -p radix mir -- --nocapture
result: 118 passed, 0 failed

cargo test -p radix
result: 535 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed
```

## Wasm Implications

No MIR shape, validation rule, or Wasm lowering behavior is intended to change.

## Completion Audit

- Ignored harness exists: yes.
- Harness scans shared exempla corpus: yes, 101 files measured.
- Harness distinguishes required tiers and unsupported diagnostics: yes.
- Verifier/native tiers are not claimed: yes.
- Counts are recorded: yes.
- Required commands pass: yes.
