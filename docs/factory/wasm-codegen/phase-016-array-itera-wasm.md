# Phase 016: Array Iteration Wasm Path

## Scope

Lower array-backed `itera ex` and `itera de` loops through target-neutral MIR and
emit compile-valid Wasm for that subset.

This phase covers:

- `itera ex` over array/lista-typed values as index-based element iteration.
- `itera de` over array/lista-typed values as index iteration.
- `perge` and `rumpe` inside the lowered array loop.

This phase does not cover map, set, text, cursor, or provider-backed iteration.
Those still require iterator/runtime ABI policy and remain explicit
MIR-lowering failures.

## Implementation

- Extended `HirExprKind::Itera` MIR lowering for `HirIteraMode::Ex` and
  `HirIteraMode::De` when the analyzed source type normalizes to an array.
- Lowered array loops with ordinary MIR locals, branches, gotos, runtime
  collection-length calls, and existing index projections.
- Kept map/cursor iteration behind the existing `itera collection before
  iterator MIR lowering` diagnostic instead of inventing a Wasm-specific
  iterator ABI.
- Reused the runtime-call value helper from control-flow lowering so collection
  length calls share the same MIR shape as other runtime intrinsics.
- Added a focused MIR test for array `ex` and `de` loops.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 61/101
  Wasm emitted: 60/101
  compile-valid: 60/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 55/101 to 60/101. MIR-lowered
coverage increased from 56/101 to 61/101.

New compile-valid exemplars:

- `examples/exempla/ceteri/ceteri.fab`
- `examples/exempla/itera/ex.fab`
- `examples/exempla/itera/in-functione.fab`
- `examples/exempla/itera/nidificatus.fab`
- `examples/exempla/sparge/sparge.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission
because `ignotum` has no Wasm value model yet.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Map/cursor/provider iteration: `itera collection before iterator MIR
  lowering` and async `cede` in cursor examples.
- Runtime/provider method calls.
- Compound assignment and remaining operator gaps such as `inter`/`intra`.
- Dynamic `ignotum` Wasm value model.
- Non-literal and enum `discerne` lowering.
- Aggregate/optional validation gaps.
- Top-level consts, `ad` provider blocks, closures, and async `cede`.

## Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

## Next Phase Candidate

Compound assignment is a contained next target and should move
`assignatio/assignatio.fab` and `binarius/binarius.fab` if the resulting scalar
operations are already covered by Wasm emission. Runtime/provider method
lowering is larger and likely unlocks more exemplars, but it needs tighter ABI
policy before implementation.
