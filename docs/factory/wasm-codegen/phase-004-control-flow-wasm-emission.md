# Phase 004: Wasm Control-Flow Emission

## Interpreted Problem

Phase 003 made primitive scalar modules compile-valid, but several
already-MIR-lowered numeric exemplars still stop at Wasm emission because the
probe rejects `branch`, `goto`, and multi-block MIR.

## Normalized Spec

- Keep the MIR path unchanged.
- Emit validated Wasm for primitive multi-block functions.
- Support MIR `Branch`, `Goto`, `Return`, and `Unreachable` terminators.
- Use a simple dispatch-loop representation for now so MIR block IDs remain
  explicit and target-neutral MIR does not learn Wasm stack structure.
- Preserve fail-closed behavior for switch, try-call, return-error, aggregates,
  text, and runtime-managed values.
- Add focused Wasm tests that validate branch/recursive generated WAT when
  `wasm-tools` is available.

## Repo-Aware Baseline

The Wasm text probe currently rejects any function with more than one MIR block.
MIR lowering already emits explicit block IDs and terminators for `si`,
`custodi`, and recursive numeric functions.

## Stage Graph

1. Add a dispatch local for multi-block functions.
2. Emit each MIR block guarded by the current dispatch block ID.
3. Emit `branch` and `goto` terminators by updating the dispatch local and
   restarting the dispatch loop.
4. Keep single-block emission unchanged.
5. Validate generated WAT in focused tests and through the exemplar harness.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when at least one branch-heavy exemplar advances to
compile-valid Wasm, the primitive compile-valid exemplars from Phase 003 remain
valid, and unsupported runtime/text/aggregate shapes remain explicitly
classified.

## Open Questions

- The dispatch-loop representation is deliberately simple and not an optimized
  structured Wasm lowering. It is acceptable for the experimental probe while
  coverage is being established.
