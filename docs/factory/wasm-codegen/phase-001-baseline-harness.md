# Phase 001: Truthful Wasm Exemplar Baseline Harness

## Interpreted Problem

The factory goal requires a tiered Wasm exemplar harness before production
Wasm/MIR behavior changes. The harness must classify every
`examples/exempla/**/*.fab` file by the highest tier reached, keep toolchain
absence separate from compiler failure, and give later phases live failure
clusters to choose from.

## Normalized Spec

- Add an ignored `exempla_wasm_e2e` harness under the existing radix exemplar
  e2e tests.
- Reuse existing exemplar discovery and temporary artifact helpers.
- Run the driver frontend, MIR lowering, MIR Wasm text emission, external WAT
  validation, and instantiation as separate tiers.
- Prefer `wasm-tools validate` when present; fall back to `wat2wasm`.
- Treat missing validation or runtime tools as explicit skipped tiers.
- Avoid production compiler behavior changes in this baseline phase.

## Repo-Aware Baseline

Existing MIR-backed Wasm support is an experimental WAT probe exposed through
`Target::WasmText`. It emits from validated MIR, not HIR, and currently supports
only a small subset of primitive single-block functions. Existing exemplar e2e
coverage lived in `crates/radix/src/exempla_e2e_test.rs` for Rust, Go, and
Faber round-trip targets.

## Stage Graph

1. Inspect MIR/Wasm probe and existing e2e harness patterns.
2. Add ignored Wasm exemplar harness with tier classification.
3. Run focused Wasm tests and the ignored harness.
4. Record baseline counts and failure clusters in the factory ledger.
5. Run required radix validation gates and commit the phase.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix`

## Gate Plan

This phase is complete when the ignored harness compiles, enumerates the full
current exemplar corpus, prints the required tier count block, records toolchain
availability, and leaves production compiler behavior unchanged.

## Open Questions

- `wasmtime` is not installed on the current PATH, so instantiate/run tiers are
  present but not yet exercisable.
- The next implementation phase should start from the live MIR-lowering cluster,
  not from Wasm emission or validation.
