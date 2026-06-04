# Phase 006 LLVM Verifier Policy Delivery

**Status**: Complete  
**Date**: 2026-06-04

## Objective

Make the LLVM factory distinguish emitted LLVM text from verifier-valid LLVM IR.
This phase must not require a local LLVM installation for ordinary emission
tests, and it must not raise verifier-valid floors unless a verifier is present
and measured.

## Scope

Included:

- Detect an external LLVM verifier tool from `llvm-as` or `opt`.
- Prefer verifier classification in the ignored `exempla_llvm_e2e` harness when
  a verifier is available.
- Keep emission tiers independent from verifier availability.
- Report the exact verifier command and version in the e2e output when
  available.
- Record verifier-valid counts separately from LLVM-emitted counts.
- Keep verifier-valid expected floors at zero unless measured in the current
  environment.

Deferred:

- adding an LLVM toolchain dependency.
- requiring LLVM verifier availability in default tests.
- native execution/runtime tiers.
- changing LLVM IR generation to satisfy a verifier if the local verifier is
  unavailable.

## Validation Plan

Run:

```text
cargo test -p radix llvm -- --nocapture
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
cargo test -p radix mir -- --nocapture
cargo test -p radix
./scripta/lint
```

Wasm validation is not required for this phase because no MIR, Wasm, shared
control-flow, type-representation, projection, call, or runtime abstraction is
changed.

## Completion Notes

Implemented in `crates/radix/src/exempla_e2e/llvm.rs` by adding optional LLVM
verifier detection, a verifier-valid tier, verifier-valid and verifier-failed
buckets, and a zero verifier-valid floor.

Local tool status:

```text
llvm-as: unavailable
opt: unavailable
selected verifier: unavailable (llvm-as/opt not found)
```

Because no verifier is available in this environment, the harness preserves
LLVM-emitted counts and reports `verifier-valid: 0/102` without claiming
verifier-valid IR.

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 15 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed
counts: 102/102 frontend, 74/102 MIR, 1/102 LLVM emitted, 0/102 verifier-valid, 73 unsupported diagnostics
```

```text
cargo test -p radix mir -- --nocapture
result: 130 passed, 0 failed
```

```text
cargo test -p radix
result: 547 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

```text
./scripta/lint
result: passed
```

```text
rustfmt --edition 2021 --check crates/radix/src/exempla_e2e/llvm.rs
result: passed
```
