# Phase 010: Diagnostic Fan-Out MIR Lowering

## Scope

Lower multi-argument diagnostic source expressions into valid MIR without
weakening MIR validation.

Source forms such as `nota "x:", x` are variadic, but the MIR diagnostic
intrinsic is intentionally unary. Before this phase, lowering emitted one
runtime call with multiple operands, and validation rejected it with:

```text
invalid MIR: diagnostic runtime call expects 1 MIR arguments
```

## Implementation

- Changed `lower_scribe` to emit one `MirIntrinsic::Diagnostic` runtime call per
  source argument.
- Preserved the MIR validator invariant that each diagnostic runtime call has
  exactly one operand and returns `vacuum`.
- Added a focused MIR test proving multi-argument `nota` lowers into multiple
  unary runtime calls.

## Measured Result

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 42/101
  Wasm emitted: 40/101
  compile-valid: 40/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Compile-valid coverage increased from 32/101 to 40/101. MIR-lowered coverage
increased from 32/101 to 42/101.

New compile-valid exemplars include:

- `examples/exempla/dum/conditio-complexa.fab`
- `examples/exempla/dum/dum.fab`
- `examples/exempla/dum/in-functione.fab`
- `examples/exempla/incipit/functionibus.fab`
- `examples/exempla/nota/gradus.fab`
- `examples/exempla/nota/nota.fab`
- `examples/exempla/perge/perge.fab`
- `examples/exempla/rumpe/rumpe.fab`

New MIR-lowered but not Wasm-emitted exemplars:

- `examples/exempla/conversio/conversio.fab`: value-returning conversion
  runtime calls are not emitted by the Wasm text probe yet.
- `examples/exempla/mori/mori.fab`: panic runtime calls are not emitted by the
  Wasm text probe yet.

The host tiers remain zero because `wasmtime` is unavailable on PATH.

## Remaining Clusters

MIR-lowering clusters still include:

- Iterator/range lowering.
- Switch/pattern lowering.
- Assertion intrinsics.
- Runtime/provider method calls.
- Compound assignment and missing operator MIR primitives.
- Optional/aggregate validation gaps.
- Top-level consts.

Wasm-emission clusters newly visible after this phase:

- Value-returning runtime conversion calls.
- Panic runtime calls.

## Validation

- `cargo test -p radix diagnostic -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

