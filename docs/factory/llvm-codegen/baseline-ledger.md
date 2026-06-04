# LLVM Codegen Baseline Ledger

**Status**: Phase 004 baseline
**Measured**: 2026-06-04  
**Current Focused Gate**: `cargo test -p radix llvm -- --nocapture`

## Baseline Summary

LLVM is currently an experimental MIR-backed text probe. It is wired through the
driver as `llvm-text`, lowers from validated MIR, and emits a deliberately small
LLVM IR text subset from `crates/radix/src/mir/llvm_text.rs`.

The current emitter supports scalar functions over one or more MIR basic blocks:

- function definitions with direct MIR parameters;
- stack slots for locals and temps so scalar values can cross block edges
  before SSA/phi construction exists;
- `numerus`, `fractus`, `bivalens`, and `vacuum` type spelling;
- integer, float, boolean, and unit constants that fit the current scalar
  policy;
- integer arithmetic for `Add`, `Sub`, `Mul`, `Div`, and `Mod`;
- floating arithmetic for `Add`, `Sub`, `Mul`, `Div`, and `Mod`;
- scalar comparisons for `numerus`, `fractus`, and `bivalens` where MIR
  carries operand type facts;
- boolean `Not`, `And`, `Or`, equality, and inequality;
- local, temp, value, and constant operands within the current scalar policy;
- LLVM labels for MIR basic blocks in MIR storage order;
- direct `return`, `ret void`, unconditional branches, and scalar boolean
  conditional branches.

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
are scalar `numerus`, `fractus`, `bivalens`, or `vacuum`:

- `MirProgram` function order as deterministic emission order.
- `MirFunction` declarations with direct parameters and return type.
- `MirParam` lowered as `%lN` LLVM parameters and stored into `%lN.addr`.
- `MirStmtKind::Assign` without projections.
- `MirValueKind::Operand`.
- `MirValueKind::Binary` for integer and floating `Add`, `Sub`, `Mul`, `Div`,
  and `Mod`.
- `MirValueKind::Binary` for numeric scalar comparisons and boolean
  equality/inequality.
- `MirValueKind::Binary` for boolean `And` and `Or`.
- `MirValueKind::Unary` for numeric negation and boolean `Not`.
- `MirOperand::Place`, `MirOperand::Temp`, `MirOperand::Value`, and scalar
  `MirOperand::Constant`.
- `MirConstant::Int`, `MirConstant::Float`, `MirConstant::Bool`, and
  `MirConstant::Unit`.
- `MirTerminatorKind::Return`.
- `MirTerminatorKind::Goto`.
- `MirTerminatorKind::Branch` with scalar `bivalens` conditions.

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
- `textus`, nullable unions, aggregates, enums, structs, collections, provider
  values, and other non-probe semantic types.
- `MirType::layout_id`, which is reserved but not consumed by the current LLVM
  probe.

### Verifier-Blocked

These shapes may be text-emittable later, but Phase 001 cannot claim LLVM-valid
IR because there is no verifier policy yet:

- Any emitted `.ll` text beyond the current probe examples.
- Multi-block CFG beyond the focused scalar examples until `llvm-as`, `opt`, or
  an equivalent verifier policy is selected.
- Direct calls after Phase 005 until declarations, symbol policy, and result
  typing are verified.
- External definitions and runtime declarations.

### Intentionally Deferred

These shapes are outside the early scalar LLVM lane and should continue to
produce explicit unsupported diagnostics until their named phases:

- `MirStmtKind::Call` and all `MirCallee` variants.
- `MirTerminatorKind::Switch`.
- `MirTerminatorKind::TryCall`.
- `MirTerminatorKind::ReturnError`.
- `MirTerminatorKind::Unreachable`.
- Bitwise operations and shifts until an integer bitwise phase expands the
  supported operation matrix.
- Nullable predicate unary operations such as `IsNil` and `IsNonNil`.
- `MirIntrinsic::Provider`.
- Async, closures, callable values, HAL/provider effects, native entrypoints,
  and executable toolchain behavior.

## Current Failure Clusters

- **E2E Visibility**: Phase 004 measured corpus counts are 102/102 frontend
  analyzed, 74/102 MIR lowered, 1/102 LLVM emitted, 28 MIR lowering failures,
  73 unsupported LLVM diagnostics, 0 unexpected LLVM emission failures, and
  0 output-write failures. Counts are unchanged because the exempla corpus has
  no runtime-free scalar CFG fixture yet.
- **Scalar Type Coverage**: `fractus`, scalar comparisons, boolean unary
  `Not`, and boolean `And`/`Or` are supported for scalar functions.
  Integer bitwise operations and shifts remain unsupported.
- **Control Flow**: branch-return, branch-join, and simple loop scalar MIR now
  emit labels, `br label`, and `br i1`. `switch`, failable control flow,
  alternate exits, and unreachable policy remain unsupported.
- **Calls**: direct MIR calls, definition calls, value callees, and failable
  calls are unsupported.
- **Runtime Boundary**: diagnostics, assertions, panic, conversion, formatting,
  and collection intrinsics have no LLVM ABI.
- **Layout**: text, aggregate, nullable, projection, enum, struct, collection,
  and provider values have no LLVM representation.
- **Verification**: there is no local LLVM verifier integration or skip policy
  recorded.
- **E2E Emission Floor**: the current exempla corpus has one LLVM-emitted
  scalar-only file.

## Fail-Closed Test Inventory

- `llvm_text_target_rejects_unsupported_mir_shapes` verifies unsupported text
  return lowering reports a `MIR-to-LLVM unsupported` diagnostic.
- `llvm_text_target_emits_branch_return_cfg` verifies ordinary scalar `si`
  return CFG emits labels and `br i1`.
- `llvm_text_target_emits_branch_join_cfg` verifies expression-valued scalar
  `si` emits join-block branches and a shared local result.
- `llvm_text_target_emits_simple_loop_cfg` verifies scalar `dum` emits a loop
  backedge.
- `llvm_text_target_still_rejects_switch_cfg` verifies `switch` remains an
  explicit unsupported CFG terminator.
- `exempla_llvm_e2e` is ignored by default and records unsupported LLVM
  diagnostics separately from MIR-lowering and unexpected emission failures.

## Next Implementation Slice

The evidence now points to Phase 005, direct function calls. After scalar CFG,
ordinary examples most often fail LLVM emission on calls, runtime calls, text,
aggregates, or layout-dependent values. Direct calls are the next scalar MIR
surface that can expand lowering without defining the full runtime ABI.

## Wasm Follow-Up Implications

Phase 004 made no MIR changes. No Wasm code changes are required.

Later LLVM phases should continue to compare against Wasm support when the MIR
shape is shared, especially for control flow, runtime intrinsics, aggregate
handles, nullable values, and direct calls.
