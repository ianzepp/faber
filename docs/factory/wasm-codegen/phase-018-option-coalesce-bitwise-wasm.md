# Phase 018: Option Coalesce And Bitwise Wasm Emission

## Scope

Emit compile-valid Wasm for the remaining MIR emission gaps in the current
`binarius.fab` path.

This phase covers:

- `MirOptionOp::Coalesce` when the nullable carrier, fallback, and result share
  the same raw Wasm storage class.
- `numerus` integer bitwise operations: `BitAnd`, `BitOr`, `BitXor`, `Shl`, and
  `Shr`.

This phase does not implement a complete nullable runtime ABI. `Some`, unwrap,
optional chains, and mixed-carrier numeric nullable values remain explicitly
unsupported until they have a real representation.

## Implementation

- Added handle-level coalesce emission using `select value fallback
  (value != 0)`, preserving the existing zero-handle representation for `nil`.
- Required coalesce operands/results to share the same raw Wasm carrier before
  emitting WAT.
- Kept unsupported diagnostics for other `MirOptionOp` variants.
- Added `i64.and`, `i64.or`, `i64.xor`, `i64.shl`, and `i64.shr_s` emission for
  integer bitwise MIR binary operations.
- Added focused Wasm tests for nullable text coalesce and integer bitwise WAT.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 63/101
  Wasm emitted: 62/101
  compile-valid: 62/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 61/101 to 62/101. Wasm-emitted
coverage increased from 61/101 to 62/101. MIR-lowered coverage stayed at
63/101.

New compile-valid exemplar:

- `examples/exempla/binarius/binarius.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission
because `ignotum` has no Wasm value model yet.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Dynamic `ignotum` Wasm value model, surfaced by `si/est.fab`.
- Map/cursor/provider iteration.
- Runtime/provider method calls.
- Remaining operator gaps such as `inter`/`intra`.
- Non-literal and enum `discerne` lowering.
- Aggregate/optional validation gaps.
- Top-level consts, `ad` provider blocks, closures, and async `cede`.
- Full nullable ABI: `Some`, unwrap, optional chains, and mixed-carrier nullable
  primitive values.

## Validation Log

- `cargo test -p radix wasm_target_emits_integer_bitwise_ops -- --nocapture`: passed.
- `cargo test -p radix wasm_target_emits_option_coalesce_for_nullable_handles -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

Note: an initial combined focused-test command used an invalid Cargo filter form
and failed before running tests. The individual focused filters and the full
Wasm-focused suite passed.

## Next Phase Candidate

The only remaining Wasm-emission blocker in the current harness is dynamic
`ignotum` in `si/est.fab`. Larger compile-valid gains are more likely from MIR
lowering clusters such as runtime/provider method calls, map/cursor iteration,
and non-literal `discerne`, but those need more policy than the small scalar
emission phases.
