# Phase 015: Numeric Range Iteration Wasm Path

## Scope

Lower `itera pro` over numeric range expressions through MIR and emit
compile-valid Wasm for that control-flow subset.

This phase covers:

- `itera pro start‥end fixum i { ... }`
- `itera pro start ante end fixum i { ... }`
- `itera pro start…end fixum i { ... }`
- explicit `per step` values, including negative steps
- default step selection based on range direction
- `perge` and `rumpe` inside the lowered loop

This phase does not cover collection iteration (`itera ex`/`itera de`) or
cursor/generator iteration. Those still require iterator/runtime ABI decisions
and remain explicit MIR-lowering failures.

## Implementation

- Added `HirExprKind::Itera` dispatch in MIR expression lowering.
- Lowered `HirIteraMode::Pro` with `Intervallum` sources into target-neutral
  MIR locals, comparisons, branches, gotos, and ordinary assignments.
- Routed `perge` to the loop increment block and `rumpe` to the after block.
- Preserved explicit unsupported diagnostics for collection iteration.
- Added focused MIR and Wasm tests for numeric range iteration.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 56/101
  Wasm emitted: 55/101
  compile-valid: 55/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 50/101 to 55/101. MIR-lowered
coverage increased from 51/101 to 56/101.

New compile-valid exemplars:

- `examples/exempla/ante/ante.fab`
- `examples/exempla/itera/intervallum-gradus.fab`
- `examples/exempla/itera/intervallum.fab`
- `examples/exempla/per/per.fab`
- `examples/exempla/usque/usque.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission
because `ignotum` has no Wasm value model yet.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Collection/cursor iteration: `itera collection before iterator MIR lowering`.
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

The largest remaining route to the 70-80% compile-valid target is collection
iteration or method/runtime lowering. A smaller next phase is compound
assignment lowering, which should move `assignatio`/`binarius` if Wasm emission
for the resulting scalar operations is already present.
