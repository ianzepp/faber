# Phase 001 LLVM Readiness Audit And Baseline Ledger

**Status**: implemented  
**Date**: 2026-06-04  
**Scope Source**: `docs/factory/llvm-codegen/continuation-plan.md`

## Objective

Create a durable LLVM factory baseline before adding feature lowering. Phase 001
does not expand LLVM emission except to preserve fail-closed behavior for a
representative unsupported CFG shape. It records current MIR, Wasm, and LLVM
facts so later LLVM phases can choose slices from evidence instead of assumed
backend difficulty.

## Requirements

- Review current MIR nodes, validation rules, Wasm lowering shapes, and LLVM
  probe behavior.
- Create `docs/factory/llvm-codegen/baseline-ledger.md`.
- Classify MIR shapes as directly LLVM-lowerable, runtime-call-backed,
  layout-dependent, verifier-blocked, or intentionally deferred.
- Preserve or add fail-closed tests for representative unsupported LLVM shapes.
- Identify the first implementation slice from current evidence.
- Run `cargo test -p radix llvm -- --nocapture`.
- Run `cargo test -p radix mir -- --nocapture`.
- Run `./scripta/lint`.

## Evidence

Readiness review covered:

- `crates/radix/src/mir/nodes.rs`
- `crates/radix/src/mir/validate.rs`
- `crates/radix/src/mir/wasm_text.rs`
- `crates/radix/src/mir/llvm_text.rs`
- `crates/radix/src/mir/llvm_text_test.rs`

Implementation artifact:

- `docs/factory/llvm-codegen/baseline-ledger.md`

Fail-closed test coverage:

- `llvm_text_target_rejects_unsupported_mir_shapes` keeps text values explicit.
- `llvm_text_target_rejects_multi_block_cfg_until_phase_004` keeps scalar CFG
  rejection explicit until the multi-block phase lands.

## Validation

Baseline before Phase 001 changes:

```text
cargo test -p radix llvm -- --nocapture
result: 2 passed, 0 failed

cargo test -p radix mir -- --nocapture
result: 117 passed, 0 failed
```

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 3 passed, 0 failed

cargo test -p radix mir -- --nocapture
result: 118 passed, 0 failed

./scripta/lint
result: passed

cargo test -p radix
result: 535 passed, 0 failed, 5 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

## Wasm Implications

No MIR shape changed in this phase. Wasm lowering remains a review client only.
The ledger records Wasm support as current evidence for prioritizing later LLVM
work.

## Completion Audit

- Durable phase spec exists: yes, this file.
- Ledger exists: yes, `baseline-ledger.md`.
- Unsupported clusters are documented: yes, in the ledger.
- Representative unsupported test exists: yes, multi-block CFG now has a focused
  LLVM rejection test.
- MIR changes preserve Wasm: no MIR changes were made.
- Required commands pass: yes.
