# Phase 017: Compound Assignment MIR Lowering

## Scope

Lower compound assignment expressions through target-neutral MIR so Wasm can
reuse existing binary operation emission.

This phase covers:

- Numeric compound assignment such as `x ⊕ 5`, `x ⊖ 10`, `x ⊛ 2`, and
  `x ⊘ 3`.
- Text concatenating assignment through `s ⊕ "..."`.
- Statement-position and value-position `HirExprKind::AssignOp`.

This phase does not add a compound-assignment MIR node. It also does not expand
the nullable/option Wasm value model exposed by `binarius.fab`.

## Implementation

- Added place-aware `AssignOp` lowering in the MIR function builder.
- Reused existing assignment target resolution for paths, fields, and indexes.
- Reused existing HIR-to-MIR binary operator mapping.
- Emitted the compound operation as an ordinary binary temp assigned back to the
  original `MirPlace`.
- Updated statement lowering to route `AssignOp` directly through the
  assignment-op lowering path.
- Added focused MIR coverage for numeric and text compound assignment.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 63/101
  Wasm emitted: 61/101
  compile-valid: 61/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 60/101 to 61/101. MIR-lowered
coverage increased from 61/101 to 63/101.

New compile-valid exemplar:

- `examples/exempla/assignatio/assignatio.fab`

Newly exposed Wasm-emission blocker:

- `examples/exempla/binarius/binarius.fab` now reaches validated MIR but stops
  at Wasm emission with `MIR-to-WASM unsupported: option value`.

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission
because `ignotum` has no Wasm value model yet.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Option/nullable value emission, now surfaced by `binarius.fab`.
- Dynamic `ignotum` Wasm value model, surfaced by `si/est.fab`.
- Map/cursor/provider iteration.
- Runtime/provider method calls.
- Remaining operator gaps such as `inter`/`intra`.
- Non-literal and enum `discerne` lowering.
- Aggregate/optional validation gaps.
- Top-level consts, `ad` provider blocks, closures, and async `cede`.

## Validation Log

- `cargo test -p radix lowers_compound_assignment_to_binary_and_assign -- --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

## Next Phase Candidate

The harness now has two Wasm-emission blockers: option value emission in
`binarius.fab` and dynamic `ignotum` in `si/est.fab`. A focused option-value MVP
could move `binarius` if it can stay compile-valid without claiming runtime
nullable semantics. Runtime/provider method calls remain the larger coverage
cluster but need ABI policy.
