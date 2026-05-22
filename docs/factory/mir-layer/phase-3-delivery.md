# Phase 3 Delivery: Primitive Expression Lowering

## Interpreted Problem

After Phase 2, MIR can be inspected from source and can represent function shells plus trivial bodies. Phase 3 should make MIR useful for ordinary straight-line primitive computation without crossing into control-flow normalization, runtime intrinsics, aggregates, or backend consumption.

The purpose is to prove the first real lowering path:

```text
typed HIR function body -> typed MIR statements/terminators
```

for simple code that uses primitive values, locals, calls, and returns.

## Normalized Spec

- Lower primitive literals into MIR constants:
  - `numerus`,
  - `fractus`,
  - `textus`,
  - `bivalens`,
  - `nihil`,
  - unit/vacuum where needed.
- Lower function parameters as MIR locals.
- Lower local declarations with simple initializers.
- Lower reads of parameters and locals into `MirOperand::Place`.
- Lower assignment to locals.
- Lower primitive unary operations.
- Lower primitive binary operations.
- Lower direct function calls where the callee resolves to a known semantic definition.
- Lower explicit `redde` statements.
- Preserve `MirType` on lowered values, locals, temporaries, and return operands.
- Keep `radix mir` deterministic for the supported subset.
- Keep unsupported constructs as explicit MIR-lowering diagnostics.

## Expression Lowering Contract

Phase 3 should introduce a small expression-lowering convention:

- Constants lower directly to `MirOperand::Constant`.
- Parameter and local references lower to `MirOperand::Place`.
- Compound expressions lower by materializing into a fresh temporary.
- Calls lower through `MirStmtKind::Call` and materialize into a destination when they produce a value.
- Expression lowering returns an operand that later statements or terminators can consume.

Representative shape:

```text
temps:
  %0: ty#1
bb0:
  %0 = _0 + _1
  return %0
```

This is intentionally not SSA. Fresh temporaries are enough for the first primitive lowering slice.

## Direct Call Contract

Direct calls should preserve semantic identity first:

```text
call def#N(...)
```

Use `MirCallee::Definition(DefId)` for Phase 3 direct calls instead of requiring every callee to be mapped to a local `MirFunctionId`.

Rationale:

- Local functions, imported functions, stdlib functions, and future package/library functions may not all live in the same `MirProgram`.
- `DefId` is the semantic anchor already produced by resolution.
- A later validation or linking phase can map local definitions to `MirFunctionId` where that is useful.

## Local Binding Contract

- Function parameters are locals created before body lowering.
- Local declarations create new `MirLocal` entries.
- Initialized locals emit an assignment from the lowered initializer operand/value.
- Reassignment emits an assignment to the existing local place.
- Uninitialized local declarations are out of scope unless they are already trivial and do not require definite-assignment reasoning.

Phase 3 should not introduce new definite-assignment semantics. Existing semantic analysis remains responsible for accepting or rejecting source programs before MIR lowering.

## Supported Phase 3 Lowering Subset

Supported:

- Function bodies with straight-line statements.
- Primitive constants.
- Parameter/local reads.
- Local declarations with simple primitive initializers.
- Local reassignment.
- Primitive unary and binary expressions.
- Direct calls to semantically resolved functions.
- Explicit `redde expr`.
- No-value return for `vacuum` functions where already trivial.

Unsupported and expected to diagnose:

- `si`, `sin`, `secus`, and expression-valued branches.
- `dum`, `fac`, `itera`, `rumpe`, and `perge`.
- `elige` and `discerne`.
- Runtime diagnostic calls such as `nota`, `mone`, `vide`, and `scribe`.
- String-template application / `scriptum` formatting.
- Method calls.
- Struct, enum, tuple, array, map, and set construction.
- Field access, index access, optional chain, and non-null assertion.
- `iace`, `tempta`, `cape`, `mori`, and recoverable failure flow.
- Closures, `cede`, async/generator shapes, and collection transforms.

## Stage Graph

1. Extend `mir::lower` with an expression lowering helper that returns `MirOperand`.
2. Add temporary allocation for compound primitive expressions.
3. Lower primitive constants and parameter/local paths.
4. Lower primitive unary and binary expressions.
5. Lower simple local declarations and assignment.
6. Lower direct function calls with `MirCallee::Definition`.
7. Lower explicit returns.
8. Add deterministic MIR dump tests for each supported shape.
9. Add negative tests for unsupported control-flow and runtime constructs.

## Checkpoints

- `radix mir` prints deterministic MIR for primitive straight-line functions.
- Tests cover constants, parameters, locals, assignment, unary/binary operations, direct calls, and explicit returns.
- Unsupported control flow fails with an explicit unsupported-MIR diagnostic.
- Unsupported runtime calls such as `nota` fail with an explicit unsupported-MIR diagnostic.
- No target backend consumes MIR.
- Existing HIR-to-codegen behavior remains unchanged.

## Fixture Candidates

Primitive arithmetic:

```fab
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}
```

Straight-line locals:

```fab
functio computa() → numerus {
    varia numerus x ← 1
    x ← x + 2
    redde x
}
```

Direct call:

```fab
functio duplex(numerus n) → numerus {
    redde n * 2
}

functio usa() → numerus {
    redde duplex(4)
}
```

Unsupported control flow:

```fab
functio signum(numerus n) → numerus {
    si n > 0 ergo redde n
    redde 0
}
```

Unsupported runtime call:

```fab
incipit {
    nota "salve"
}
```

## Out Of Scope

- Control-flow normalization.
- Runtime intrinsic boundary.
- Aggregate and option representation.
- Failure-flow lowering.
- MIR validation beyond fail-closed lowering checks.
- Rust backend consumption.
- WASM or native output.
- SSA.

## Validation

- Focused unit tests for primitive expression lowering.
- CLI/tool test proving `radix mir` emits deterministic text for a straight-line primitive function.
- Negative tests for unsupported `si` and `nota`.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 3 complete.

## Completion Gate

Phase 3 is complete when straight-line primitive Faber functions lower into deterministic typed MIR, direct calls preserve semantic callee identity, unsupported control-flow/runtime constructs fail clearly, and no backend behavior changes.
