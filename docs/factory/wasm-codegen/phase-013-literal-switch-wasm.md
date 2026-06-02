# Phase 013: Literal Switch Wasm Path

## Scope

Lower the simple literal `elige` subset through MIR and emit compile-valid Wasm
dispatch for it.

This phase intentionally covers only single-scrutinee, statement-form
`elige`/`discerne` arms with literal cases and an optional wildcard/default.
Enum variants, destructuring patterns, guards, multi-subject matches, and
value-producing matches remain explicit unsupported MIR-lowering work.

## Implementation

- Added MIR lowering for literal `discerne`/`elige` into
  `MirTerminatorKind::Switch`.
- Kept unsupported switch forms diagnostic instead of guessing or bypassing MIR.
- Added Wasm text dispatch emission for MIR switches inside the existing
  dispatch-loop function shape.
- Used normal scalar equality for numeric, boolean, and float switch cases.
- Used the existing `faber_text eq_text` import path for string switch cases.
- Added focused MIR and Wasm tests for literal numeric and text switch dispatch.

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 48/101
  Wasm emitted: 48/101
  compile-valid: 48/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Compile-Valid Delta

Measured compile-valid coverage increased from 44/101 to 48/101. MIR-lowered
coverage also increased from 44/101 to 48/101.

New compile-valid exemplars:

- `examples/exempla/elige/ceterum.fab`
- `examples/exempla/elige/elige.fab`
- `examples/exempla/elige/ergo-redde.fab`
- `examples/exempla/elige/in-functione.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

## Remaining Failure Clusters

- Iterator/range lowering: `itera before iterator MIR lowering`.
- Non-literal and enum `discerne` lowering.
- Runtime/provider method calls.
- Compound assignment and remaining operator gaps.
- Predicate unary and nullable/optional validation gaps.
- Top-level consts, `ad` provider blocks, and async `cede`.

## Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

## Next Phase Candidate

Iterator/range lowering remains the largest visible cluster. If the next phase
should stay compact, continue with another narrow control-flow subset such as
enum-pattern `discerne`; otherwise, attack iterator MIR lowering to raise the
compile-valid ceiling more substantially.
