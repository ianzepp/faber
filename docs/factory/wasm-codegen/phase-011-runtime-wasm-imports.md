# Phase 011: Runtime Wasm Imports

## Scope

Clear the two Wasm-emission failures exposed by Phase 010 without bypassing MIR
or claiming host/runtime execution.

The MIR-lowered exemplars now include runtime conversion, collection length, and
panic calls. This phase maps those MIR runtime intrinsics to explicit Wasm text
imports so the modules can be assembled and validated by `wasm-tools`.

## Implementation

- Added `faber_runtime` imports for value-returning conversion calls.
- Included explicit conversion fallback operands in the import ABI shape.
- Added `faber_runtime` imports for collection length calls.
- Added effect-only `faber_runtime` panic imports; MIR still supplies the
  following `unreachable` terminator.
- Added focused Wasm text tests for conversion imports and panic/length imports.

Conversion hint metadata such as `Hex` remains compiler-side policy data in this
phase. The generated module is compile-valid, but host behavior is not asserted
until a runtime import implementation and instantiate/run tier exist.

## Measured Result

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 42/101
  Wasm emitted: 42/101
  compile-valid: 42/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Compile-valid coverage increased from 40/101 to 42/101.

New compile-valid exemplars:

- `examples/exempla/conversio/conversio.fab`
- `examples/exempla/mori/mori.fab`

The host tiers remain zero because `wasmtime` is unavailable on PATH. The
harness reports this as skipped instantiate/run support, not as a compiler,
codegen, or validation failure.

## Remaining Clusters

There are no current Wasm-emission failures among MIR-lowered exemplars.

Remaining MIR-lowering clusters still include:

- Iterator/range lowering.
- Switch/pattern lowering.
- Assertion intrinsics.
- Runtime/provider method calls.
- Compound assignment and missing operator MIR primitives.
- Optional/aggregate validation gaps.
- Top-level consts.

Host/runtime work remains separate:

- Implement actual `faber_runtime` import semantics.
- Install or provide a local instantiate host before measuring
  instantiate-valid, runnable, or behavior-checked tiers.

## Validation

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.
