# LLVM Codegen Baseline Ledger

**Status**: Phase 007 baseline
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
- direct function calls between MIR functions in the same program when
  arguments and results fit the scalar policy;
- deterministic LLVM `declare` lines and `call` instructions for supported
  runtime intrinsics;
- opaque `ptr` runtime handles for text and aggregate-like runtime values when
  used as runtime-call arguments, destinations, or helper results;
- LLVM labels for MIR basic blocks in MIR storage order;
- direct `return`, `ret void`, unconditional branches, and scalar boolean
  conditional branches.

Everything outside that subset must fail closed with a diagnostic beginning
`MIR-to-LLVM unsupported`.

The ignored exempla harness now distinguishes LLVM text emission from
verifier-valid LLVM IR. Verifier validation is optional and external-tool based:
the harness prefers `llvm-as -o /dev/null <file.ll>`, then
`opt -disable-output <file.ll>`, and reports verifier-valid tiers only when one
of those tools is available. In this measured environment neither `llvm-as` nor
`opt` is on PATH, so verifier-valid floors remain zero.

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
- `MirStmtKind::Call` with `MirCallee::Function`.
- `MirStmtKind::Call` with `MirCallee::Definition` when the definition resolves
  to a MIR function in the current program.
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

### Runtime-Call-Backed Today

These shapes represent source semantics that should remain MIR-level facts but
now lower to LLVM runtime declarations and calls when their operands fit the
current scalar-or-handle ABI:

- `MirStmtKind::RuntimeCall`.
- `MirIntrinsic::Diagnostic`.
- `MirIntrinsic::Assert`.
- `MirIntrinsic::FormatString`.
- `MirIntrinsic::Convert`.
- `MirIntrinsic::Collection`.
- `MirIntrinsic::Panic`.

LLVM runtime symbols use the `__faber_runtime_` prefix as external declarations
such as `declare void @__faber_runtime_diagnostic_nota_1_ptr(ptr)`. This is an
LLVM-side ABI, not a Wasm import ABI. Symbol suffixes record intrinsic kind,
argument count, argument ABI classes, and result ABI class when the call returns
a value. `panic` is declared as a void helper even though MIR records `numquam`;
the following MIR terminator remains responsible for unreachable policy.

Supported runtime ABI classes are `i1`, `i64`, `f64`, `ptr`, and `void`.

### Layout-Dependent

These shapes need physical layout, handle, or ABI decisions before LLVM can
lower them without guessing:

- `MirStmtKind::Construct`.
- `MirAggregate` and all `MirAggregateKind` variants.
- `MirAggregateFields` ordered, named, and keyed forms.
- `MirProjection::Field`, `MirProjection::VariantField`, and
  `MirProjection::Index`.
- `MirValueKind::Option` and all `MirOptionOp` variants.
- `MirConstant::Nil`.
- ordinary function returns of `textus`, nullable unions, aggregates, enums,
  structs, collections, provider values, and other non-probe semantic types.
- text comparison or concatenation implied by `MirBinOp` over text values.
- aggregate projection reads or writes when represented through runtime handles.
- `MirType::layout_id`, which is reserved but not consumed by the current LLVM
  probe.

### Verifier-Blocked

These shapes may be text-emittable later, but the LLVM lane should not claim
LLVM-valid IR unless the optional verifier tier proves it in the current
environment:

- Any emitted `.ll` text beyond the current probe examples unless the optional
  e2e verifier tier reports it as verifier-valid in the current environment.
- Multi-block CFG beyond the focused scalar examples until measured through the
  optional verifier tier.
- Direct calls beyond the focused scalar examples until measured through the
  optional verifier tier and later declaration/linkage policy.
- External definitions and runtime declarations.

### Intentionally Deferred

These shapes are outside the early scalar LLVM lane and should continue to
produce explicit unsupported diagnostics until their named phases:

- `MirStmtKind::Call` with external `MirCallee::Definition`.
- `MirStmtKind::Call` with `MirCallee::Value`.
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

- **E2E Visibility**: Phase 007 measured corpus counts are 102/102 frontend
  analyzed, 74/102 MIR lowered, 35/102 LLVM emitted, 28 MIR lowering failures,
  0/102 verifier-valid, 39 unsupported LLVM diagnostics, 0 unexpected LLVM
  emission failures, 0 output-write failures, and 0 verifier failures.
- **Scalar Type Coverage**: `fractus`, scalar comparisons, boolean unary
  `Not`, and boolean `And`/`Or` are supported for scalar functions.
  Integer bitwise operations and shifts remain unsupported.
- **Control Flow**: branch-return, branch-join, and simple loop scalar MIR now
  emit labels, `br label`, and `br i1`. `switch`, failable control flow,
  alternate exits, and unreachable policy remain unsupported.
- **Calls**: direct scalar MIR calls between same-program functions are
  supported. External definitions, value callees, failable calls, and
  non-scalar call signatures remain unsupported.
- **Runtime Boundary**: diagnostics, assertions, panic, conversion, formatting,
  and collection intrinsics lower to named LLVM runtime declarations and calls.
  Provider/HAL runtime calls remain deferred.
- **Layout**: ordinary text returns, aggregate construction, nullable values,
  projections, enum, struct, collection layout, and provider values have no
  physical LLVM representation beyond opaque runtime handles.
- **Verification**: the e2e harness detects `llvm-as` or `opt` and records
  verifier-valid output only when a verifier is available. Current local
  verifier status is unavailable: `llvm-as` and `opt` were not found on PATH.
- **E2E Emission Floor**: the current exempla corpus has 35 LLVM-emitted files.
  The current verifier-valid floor is zero.

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
- `llvm_text_target_emits_direct_scalar_function_calls` verifies direct scalar
  helper calls store returned values.
- `llvm_text_target_emits_direct_scalar_call_chains` verifies scalar call
  results can flow through locals into later calls.
- `llvm_text_target_rejects_value_callee` verifies callable values remain
  explicitly unsupported.
- `llvm_text_target_rejects_external_definition_call` verifies definitions not
  lowered into the current MIR program remain explicitly unsupported.
- `llvm_text_target_emits_diagnostic_assert_and_panic_runtime_declarations`
  verifies void runtime helper declarations and calls for diagnostics, assert,
  and panic.
- `llvm_text_target_emits_value_returning_runtime_calls` verifies conversion,
  format, and collection runtime helpers declare, call, and store through the
  current scalar-or-handle ABI while `nil` remains layout-blocked.
- `llvm_text_target_rejects_provider_runtime_calls` verifies provider/HAL
  runtime calls remain explicitly deferred.
- `exempla_llvm_e2e` is ignored by default and records unsupported LLVM
  diagnostics, emitted LLVM text, verifier-valid LLVM IR, and verifier failures
  separately from MIR-lowering and unexpected emission failures.

## Next Implementation Slice

The evidence now points to Phase 008, text and aggregate handle ABI. Runtime
helper calls can move opaque handles, but ordinary text returns, aggregate
construction, projections, nullable values, and collection layout still need a
physical ABI before the LLVM lane can claim broader handle support.

## Wasm Follow-Up Implications

Phase 007 made no MIR shape changes and did not alter Wasm import names. Wasm
validation is still required because runtime intrinsics are shared MIR facts.

Later LLVM phases should continue to compare against Wasm support when the MIR
shape is shared, especially for control flow, runtime intrinsics, aggregate
handles, nullable values, and direct calls.
