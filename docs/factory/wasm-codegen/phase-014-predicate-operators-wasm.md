# Phase 014: Predicate Operators Wasm Path

## Scope

Lower scalar predicate operators through MIR and emit compile-valid Wasm for the
supported handle/scalar subset.

This phase covers:

- `positivum x` and `negativum x` as numeric comparisons against zero.
- `verum x` and `falsum x` as boolean equality checks.
- `x est nihil` and `x non est nihil` as MIR nil/non-nil tests.
- Nullable values in the Wasm text probe as opaque `i32` handles for
  compile-valid nil testing.

This phase does not claim a complete nullable runtime ABI or an `ignotum` Wasm
value model. Those remain runtime/host and dynamic-value work.

## Implementation

- Added predicate-specific MIR lowering for sign and boolean checks using
  existing target-neutral `MirBinOp` comparisons.
- Added `est nihil`/`non est nihil` lowering to existing target-neutral
  `MirUnOp::IsNil` and `MirUnOp::IsNonNil`.
- Mapped non-nil `est`/`non est` to MIR equality/inequality for already
  type-compatible operands.
- Added Wasm text support for nil constants, nullable handle locals, and
  nil/non-nil unary tests.
- Added focused MIR and Wasm tests for the predicate subset.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 51/101
  Wasm emitted: 50/101
  compile-valid: 50/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 48/101 to 50/101. MIR-lowered
coverage increased from 48/101 to 51/101.

New compile-valid exemplars:

- `examples/exempla/est/est.fab`
- `examples/exempla/unarius/unarius.fab`

New MIR-lowered but not Wasm-emitted exemplar:

- `examples/exempla/si/est.fab`: stops at Wasm emission with
  `MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Iterator/range lowering: `itera before iterator MIR lowering`.
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

Iterator/range lowering remains the largest cluster and is now the most
important route toward the 70-80% compile-valid target. A smaller alternative
is compound assignment lowering, but it is less likely to move as many
exemplars as iterator support.
