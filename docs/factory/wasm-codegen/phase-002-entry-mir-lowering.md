# Phase 002: Non-Empty Entry MIR Lowering

## Interpreted Problem

The baseline harness shows all 101 exemplars reach frontend analysis and then
stop before validated MIR. The universal blocker is the current MIR lowerer
rejecting every non-empty top-level entry block.

## Normalized Spec

- Add target-neutral MIR lowering for non-empty top-level entry blocks.
- Reuse the existing `FunctionBuilder` body lowering path instead of inventing a
  Wasm-specific entry representation.
- Synthesize a normal vacuum-returning MIR function for the top-level entry.
- Preserve explicit fail-closed diagnostics for unsupported statements and
  expressions inside entry blocks.
- Preserve existing MIR validation invariants.
- Do not change Wasm emission policy in this phase.

## Repo-Aware Baseline

`MirLowerer::lower_entry` currently accepts only empty entry blocks and emits an
empty synthetic function. Function bodies already lower through
`FunctionBuilder::lower_body`, which supports locals, expression statements,
returns, control flow, runtime calls, aggregates, and explicit unsupported
diagnostics for deferred constructs.

## Stage Graph

1. Replace the explicit non-empty entry rejection with `FunctionBuilder`
   lowering for the entry body.
2. Keep empty entry output stable.
3. Add focused MIR tests for simple non-empty entries and unsupported entry
   contents.
4. Run the Wasm exemplar harness to reveal the next live tier/failure clusters.
5. Update the baseline ledger with Phase 002 counts and commit.

## Checkpoints

- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when at least some exemplars move beyond the frontend
analyzed tier, no MIR validation regressions occur, unsupported entry contents
remain explicit MIR-lowering failures, and the ledger records the new live
failure clusters.

## Open Questions

- Wasm emission may still reject most lowered MIR shapes after this phase. That
  is expected and should be classified separately by the harness.
- Runtime and host tiers remain unavailable until emitted WAT reaches validation
  and a runtime/entry policy exists.
