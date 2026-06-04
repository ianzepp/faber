# Phase 004 Multi-Block Scalar CFG Delivery

**Status**: Complete  
**Date**: 2026-06-04

## Objective

Teach the experimental LLVM text probe to emit MIR scalar control-flow graphs
instead of rejecting ordinary multi-block functions. This phase covers LLVM
labels, unconditional `goto`, scalar boolean `branch`, and scalar `return`
terminators while preserving MIR block order.

## Scope

Included:

- Emit one LLVM label per MIR basic block.
- Emit an explicit `entry` block for allocas, parameter stores, and the initial
  branch to `%bb0`.
- Store locals and temps in stack slots so values remain available across block
  edges without claiming SSA construction.
- Lower `MirTerminatorKind::Goto`.
- Lower `MirTerminatorKind::Branch` for `i1` scalar conditions.
- Continue lowering `MirTerminatorKind::Return`.
- Add focused coverage for branch-return, branch-join, and simple loop scalar
  functions.

Deferred:

- `switch` lowering.
- direct and failable calls.
- alternate exits, error returns, and unreachable policy.
- phi construction and LLVM verification.
- runtime, aggregate, nullable, text, and layout-dependent values.

## Validation Plan

Run:

```text
cargo test -p radix llvm -- --nocapture
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
cargo test -p radix mir -- --nocapture
cargo test -p radix wasm -- --nocapture
cargo test -p radix
./scripta/lint
```

## Completion Notes

Implemented in `crates/radix/src/mir/llvm_text.rs` by switching scalar locals
and temps to stack slots, emitting an explicit `entry` block, preserving MIR
block storage order as `bbN:` labels, and lowering `return`, `goto`, and scalar
boolean `branch` terminators.

Focused coverage now includes scalar branch-return, branch-join, simple loop,
and explicit unsupported `switch` tests in
`crates/radix/src/mir/llvm_text_test.rs`.

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 9 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed
counts: 102/102 frontend, 74/102 MIR, 1/102 LLVM, 73 unsupported diagnostics
```

```text
cargo test -p radix mir -- --nocapture
result: 124 passed, 0 failed
```

```text
cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix
result: 541 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

```text
./scripta/lint
result: passed
```
