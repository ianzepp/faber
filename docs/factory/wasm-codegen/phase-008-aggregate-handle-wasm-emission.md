# Phase 008: Aggregate Handle Wasm Emission

## Scope

Add a compile-valid opaque-handle representation for MIR aggregate
construction in the Wasm text probe.

This phase deliberately does not define aggregate layout, allocation, field
projection, index projection, or runtime host semantics. It only lets validated
MIR aggregate construction produce WAT that `wasm-tools validate` can check.

## Implementation

- Added `WasmValue::AggregateHandle`, represented as Wasm `i32`.
- Mapped arrays, maps, records, sets, structs, enums, and aliases to aggregate
  handles.
- Lowered `MirStmtKind::Construct` to signature-specific
  `faber_aggregate` imports.
- Passed struct and enum variant definition IDs as an explicit leading `i32`
  metadata argument.
- Added aggregate-specific diagnostic imports such as `nota_aggregate`, keeping
  aggregate handles distinct from ordinary numeric `i32` diagnostics.
- Kept place projections fail-closed. Field and index projection examples still
  stop at Wasm emission with `MIR-to-WASM unsupported: place projection`.
- Fixed the Wasm text test temp filename helper so parallel `wasm-tools`
  validation cannot reuse the same temporary WAT path.

## Measured Result

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 28/101
  compile-valid: 28/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Compile-valid coverage increased from 26/101 to 28/101.

New compile-valid exemplars:

- `examples/exempla/finge/finge.fab`
- `examples/exempla/typus/typus.fab`

The host tiers remain zero because `wasmtime` is not available on PATH. The
harness reports that as skipped host/runtime support, not as a compiler or
codegen failure.

## Remaining Wasm-Emission Cluster

The remaining MIR-lowered Wasm blockers are projection-based aggregate examples:

- `examples/exempla/destructura/destructura.fab`
- `examples/exempla/genus/genus.fab`
- `examples/exempla/novum/novum.fab`
- `examples/exempla/varia/destructura.fab`

All stop at:

```text
MIR-to-WASM unsupported: place projection
```

## Validation

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

