# Phase 012: Assert Intrinsic Wasm Path

## Scope

Move assertion-heavy exemplars through the MIR-backed Wasm compile-valid path.

The live harness showed `examples/exempla/adfirma/*.fab` blocked at MIR lowering
with:

```text
unsupported MIR lowering: adfirma before assert intrinsic MIR lowering
```

This phase adds a target-neutral MIR assertion intrinsic and Wasm probe imports
for compile-valid assertion modules. It also adds text equality helper imports
because the current assertion exemplars include `textus` equality checks.

## Implementation

- Added `MirIntrinsic::Assert`.
- Lowered HIR `adfirma` expressions to assertion runtime calls with one boolean
  condition operand and an optional message operand.
- Validated assertion MIR calls as vacuum-returning calls with a `bivalens`
  first operand and one or two arguments.
- Rendered assertion intrinsics in MIR dumps.
- Emitted `faber_runtime` assertion imports in the Wasm text probe.
- Emitted `faber_text` equality/inequality imports for `textus` comparisons.
- Added focused MIR and Wasm tests for assertion lowering and WAT validation.

Assertion behavior remains a host/runtime concern. This phase only proves that
the MIR path emits WAT with explicit imports that `wasm-tools validate` accepts.

## Measured Result

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 44/101
  Wasm emitted: 44/101
  compile-valid: 44/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Compile-valid coverage increased from 42/101 to 44/101. MIR-lowered coverage
also increased from 42/101 to 44/101.

New compile-valid exemplars:

- `examples/exempla/adfirma/adfirma.fab`
- `examples/exempla/adfirma/in-functione.fab`

The host tiers remain zero because `wasmtime` is unavailable on PATH.

## Remaining Clusters

There are no current Wasm-emission failures among MIR-lowered exemplars.

Remaining MIR-lowering clusters still include:

- Iterator/range lowering.
- Switch/pattern lowering.
- Runtime/provider method calls.
- Compound assignment and missing operator MIR primitives.
- Predicate unary gaps.
- Optional/aggregate validation gaps.
- Top-level consts.
- `ad` provider blocks and async `cede`.

Host/runtime work remains separate:

- Implement actual `faber_runtime` assertion behavior.
- Implement `faber_text` comparison behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

## Validation

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.
