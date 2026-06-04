# Phase 010: Switch, Pattern, And Failable Control Flow Delivery

## Interpreted Problem

LLVM now emits scalar multi-block CFG, direct calls, runtime calls, handles, and
nullable helper calls, but `MirTerminatorKind::Switch` still fails as a whole.
Literal scalar `elige` lowering already reaches MIR as an explicit switch, so
LLVM can lower the narrow integer/boolean subset without choosing a broader
pattern or error ABI.

## Normalized Spec

- Lower literal scalar `Switch` for supported integer and boolean operands.
- Lower `Unreachable` to the native LLVM `unreachable` terminator so closed
  source-level switch arms can compile.
- Keep text, aggregate, dynamic, nullable-handle, and floating switch values
  fail-closed until a runtime/pattern dispatch policy exists.
- Keep `TryCall` and `ReturnError` fail-closed until alternate-exit ABI is
  chosen.
- Preserve MIR and Wasm behavior.
- Update the LLVM baseline ledger with measured counts and residual gaps.

## Repo-Aware Baseline

- LLVM branch and goto terminators already lower to labels and direct branches.
- Wasm lowers switch as dispatch-loop branch chains, including text equality
  through a text runtime helper.
- LLVM currently reports `MIR-to-LLVM unsupported: switch`.
- Phase 009 e2e baseline: 58/102 LLVM emitted, 0/102 verifier-valid, 16
  unsupported LLVM diagnostics.

## Stage Graph

1. Add an LLVM switch terminator path for `i64` and `i1` values.
2. Validate each case constant against the switch operand ABI class.
3. Keep unsupported switch value classes fail-closed with specific diagnostics.
4. Emit native LLVM `unreachable` for MIR unreachable terminators.
5. Add focused tests for source-level literal `elige`, boolean switch MIR, and
   text switch rejection.
6. Add focused fail-closed tests for `ReturnError` and `TryCall`.
7. Update the baseline ledger and record validation evidence.

## ABI Decision

LLVM scalar switch lowering uses native LLVM `switch` syntax:

```llvm
switch i64 %value, label %bb_default [
  i64 200, label %bb_case
]
```

This phase does not define text/pattern matching, runtime comparison dispatch,
or alternate-exit calling conventions.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

No MIR shape or Wasm import naming changes are expected. Wasm validation remains
required because switch terminator semantics are shared by both backend lanes.

## Completion Evidence

Phase 010 adds native LLVM CFG terminators for the scalar switch subset and MIR
unreachable blocks:

- `MirTerminatorKind::Switch` over `numerus` lowers to LLVM `switch i64`.
- `MirTerminatorKind::Switch` over `bivalens` lowers to LLVM `switch i1`.
- Switch case constants must match the switch operand ABI class.
- Text, pointer/handle, floating, aggregate, dynamic, and nullable-handle
  switch values remain fail-closed as `switch value type`.
- `MirTerminatorKind::Unreachable` lowers to LLVM `unreachable`.
- `MirTerminatorKind::ReturnError` and `MirTerminatorKind::TryCall` remain
  fail-closed until an alternate-exit ABI exists.

Focused tests added:

- `llvm_text_target_emits_literal_scalar_switch_cfg`
- `llvm_text_target_emits_boolean_switch_cfg`
- `llvm_text_target_rejects_text_switch_cfg`
- `llvm_text_target_rejects_failable_terminators`

Measured e2e baseline after implementation:

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
frontend analyzed: 102/102
MIR lowered: 74/102
LLVM emitted: 59/102
verifier-valid: 0/102
unsupported diagnostic: 15
result: passed
```

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 27 passed, 0 failed, 1 ignored

cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed

cargo test -p radix mir -- --nocapture
result: 142 passed, 0 failed

cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored

cargo test -p radix
result: 559 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed

rustfmt --edition 2021 --check crates/radix/src/mir/llvm_text.rs crates/radix/src/mir/llvm_text_test.rs crates/radix/src/exempla_e2e/llvm.rs
result: passed

git diff --check
result: passed
```

## Completion Audit

- Literal scalar switch lowering is implemented and tested for integer and
  boolean values.
- Text/pattern switch dispatch remains explicit unsupported behavior.
- Failable control flow remains explicit unsupported behavior.
- Native unreachable support is included because source-level closed switches
  commonly lower to an unreachable continuation block.
- No MIR or Wasm behavior was changed.
