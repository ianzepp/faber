# LLVM Codegen Baseline Ledger

**Status**: Phase 001 baseline  
**Measured**: 2026-06-04  
**Current Focused Gate**: `cargo test -p radix llvm -- --nocapture`

## Baseline Summary

LLVM is currently an experimental MIR-backed text probe. It is wired through the
driver as `llvm-text`, lowers from validated MIR, and emits a deliberately small
LLVM IR text subset from `crates/radix/src/mir/llvm_text.rs`.

The current emitter supports single-block scalar functions only:

- function definitions with direct MIR parameters;
- `numerus`, `bivalens`, and `vacuum` type spelling;
- integer and boolean constants that fit the current scalar policy;
- integer arithmetic for `Add`, `Sub`, `Mul`, `Div`, and `Mod`;
- local, temp, value, and constant operands when defined in the same block;
- direct `return` and `ret void`.

Everything outside that subset must fail closed with a diagnostic beginning
`MIR-to-LLVM unsupported`.

## Command Evidence

Baseline before Phase 001 changes:

```text
cargo test -p radix llvm -- --nocapture
result: 2 passed, 0 failed
```

```text
cargo test -p radix mir -- --nocapture
result: 117 passed, 0 failed
```

Final Phase 001 validation:

```text
cargo test -p radix llvm -- --nocapture
result: 3 passed, 0 failed
```

```text
cargo test -p radix mir -- --nocapture
result: 118 passed, 0 failed
```

```text
./scripta/lint
result: passed
```

```text
cargo test -p radix
result: 535 passed, 0 failed, 5 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

## MIR Shape Classification

### Directly LLVM-Lowerable Today

These shapes have current LLVM text support when all operands and return values
are scalar `numerus`, `bivalens`, or `vacuum` and the function has exactly one
basic block:

- `MirProgram` function order as deterministic emission order.
- `MirFunction` declarations with direct parameters and return type.
- `MirParam` lowered as `%lN` LLVM parameters.
- `MirStmtKind::Assign` without projections.
- `MirValueKind::Operand`.
- `MirValueKind::Binary` for `Add`, `Sub`, `Mul`, `Div`, and `Mod`.
- `MirOperand::Place`, `MirOperand::Temp`, `MirOperand::Value`, and scalar
  `MirOperand::Constant` when definitions are already in the same block.
- `MirConstant::Int`, `MirConstant::Bool`, and `MirConstant::Unit`.
- `MirTerminatorKind::Return`.

### Runtime-Call-Backed Later

These shapes represent source semantics that should remain MIR-level facts but
need an LLVM runtime or host-boundary policy before text emission can be honest:

- `MirStmtKind::RuntimeCall`.
- `MirIntrinsic::Diagnostic`.
- `MirIntrinsic::Assert`.
- `MirIntrinsic::FormatString`.
- `MirIntrinsic::Convert`.
- `MirIntrinsic::Collection`.
- `MirIntrinsic::Panic`.
- Text comparison or concatenation implied by `MirBinOp` over text values.
- Aggregate projection reads or writes when represented through runtime handles.

Wasm already lowers many of these through explicit imports in
`crates/radix/src/mir/wasm_text.rs`. LLVM should define its own ABI instead of
copying Wasm import names into MIR.

### Layout-Dependent

These shapes need physical layout, handle, or ABI decisions before LLVM can
lower them without guessing:

- `MirStmtKind::Construct`.
- `MirAggregate` and all `MirAggregateKind` variants.
- `MirAggregateFields` ordered, named, and keyed forms.
- `MirProjection::Field`, `MirProjection::VariantField`, and
  `MirProjection::Index`.
- `MirValueKind::Option` and all `MirOptionOp` variants.
- `MirConstant::String` and `MirConstant::Nil`.
- `MirConstant::Float` until the scalar float phase defines exact LLVM spelling.
- `Type::Primitive(Fractus)`, `textus`, nullable unions, aggregates, enums,
  structs, collections, provider values, and other non-probe semantic types.
- `MirType::layout_id`, which is reserved but not consumed by the current LLVM
  probe.

### Verifier-Blocked

These shapes may be text-emittable later, but Phase 001 cannot claim LLVM-valid
IR because there is no verifier policy yet:

- Any emitted `.ll` text beyond the current probe examples.
- Multi-block CFG after Phase 004 until `llvm-as`, `opt`, or an equivalent
  verifier policy is selected.
- Direct calls after Phase 005 until declarations, symbol policy, and result
  typing are verified.
- External definitions and runtime declarations.

### Intentionally Deferred

These shapes are outside the early scalar LLVM lane and should continue to
produce explicit unsupported diagnostics until their named phases:

- `MirStmtKind::Call` and all `MirCallee` variants.
- `MirTerminatorKind::Goto`.
- `MirTerminatorKind::Branch`.
- `MirTerminatorKind::Switch`.
- `MirTerminatorKind::TryCall`.
- `MirTerminatorKind::ReturnError`.
- `MirTerminatorKind::Unreachable`.
- `MirValueKind::Unary`.
- Boolean logic, scalar comparisons, bitwise operations, and shifts until the
  scalar operation phase expands the supported operation matrix.
- `MirIntrinsic::Provider`.
- Async, closures, callable values, HAL/provider effects, native entrypoints,
  and executable toolchain behavior.

## Current Failure Clusters

- **Scalar Type Coverage**: no `fractus`, scalar comparisons, boolean unary
  operations, or boolean binary operations.
- **Control Flow**: ordinary branch-shaped scalar MIR fails with
  `MIR-to-LLVM unsupported: branch`; other multi-block shapes may fail later
  with `MIR-to-LLVM unsupported: multiple basic blocks`.
- **Calls**: direct MIR calls, definition calls, value callees, and failable
  calls are unsupported.
- **Runtime Boundary**: diagnostics, assertions, panic, conversion, formatting,
  and collection intrinsics have no LLVM ABI.
- **Layout**: text, aggregate, nullable, projection, enum, struct, collection,
  and provider values have no LLVM representation.
- **Verification**: there is no local LLVM verifier integration or skip policy
  recorded.
- **E2E Visibility**: there is no LLVM exempla harness, so corpus progress is
  not yet measured.

## Fail-Closed Test Inventory

- `llvm_text_target_rejects_unsupported_mir_shapes` verifies unsupported text
  return lowering reports a `MIR-to-LLVM unsupported` diagnostic.
- `llvm_text_target_rejects_multi_block_cfg_until_phase_004` verifies ordinary
  scalar branch-shaped MIR is rejected explicitly as `branch` before
  multi-block CFG support lands.

## Next Implementation Slice

The evidence points to Phase 002, the LLVM exempla e2e harness, before scalar
coverage expansion. LLVM currently has only focused unit tests, so adding more
lowering first would still leave corpus progress and unsupported clusters
invisible. The harness should classify at least:

- frontend analyzed;
- MIR lowered;
- LLVM emitted;
- LLVM unsupported diagnostic;
- verifier-valid only if a verifier policy is available.

Execution/native tiers remain out of scope.

## Wasm Follow-Up Implications

Phase 001 made no MIR changes. No Wasm code changes are required.

Later LLVM phases should continue to compare against Wasm support when the MIR
shape is shared, especially for control flow, runtime intrinsics, aggregate
handles, nullable values, and direct calls.
