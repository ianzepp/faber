# Phase 009: Aggregate Projection Wasm Emission

## Scope

Add compile-valid Wasm text emission for MIR aggregate projection reads.

This phase targets the four remaining MIR-lowered Wasm emission failures after
Phase 008. It does not add aggregate layout, in-module memory, projected writes,
or host runtime behavior.

## Implementation

- Added `lower_analyzed_unit_with_context`, returning validated MIR plus the
  `MirValidationContext` used to prove it.
- Kept the existing `lower_analyzed_unit` API as the program-only wrapper.
- Added `emit_wasm_text_probe_with_context` so the Wasm probe can type
  projections from validation metadata without putting Wasm details in MIR.
- Updated the Wasm driver path and Wasm e2e harness to pass validation context.
- Lowered field and index projection reads to explicit `faber_aggregate`
  imports:
  - `field_i32_to_i64`
  - `field_i32_to_text`
  - `field_i32_to_i32`
  - `index_i64_to_i64`
  - `index_i64_to_aggregate`
- Preserved fail-closed behavior for projected assignment destinations.

## Measured Result

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 32/101
  compile-valid: 32/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Compile-valid coverage increased from 28/101 to 32/101.

New compile-valid exemplars:

- `examples/exempla/destructura/destructura.fab`
- `examples/exempla/genus/genus.fab`
- `examples/exempla/novum/novum.fab`
- `examples/exempla/varia/destructura.fab`

The host tiers remain zero because `wasmtime` is unavailable on PATH. The
harness reports that as skipped host/runtime support, not as a compiler or
codegen failure.

## Remaining Clusters

All currently MIR-lowered exemplars now emit compile-valid Wasm. The next
coverage limit is MIR-lowering reach:

- Iterator/range lowering.
- Switch/pattern lowering.
- Assertion intrinsics.
- Runtime/provider method calls.
- Compound assignment and missing operator MIR primitives.
- Optional/aggregate validation gaps.
- Top-level consts.
- Diagnostic runtime arity validation gaps.

## Validation

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

