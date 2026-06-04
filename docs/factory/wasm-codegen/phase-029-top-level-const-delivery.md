# Phase 029: Top-Level Const Entry Prefix Lowering

## Interpreted Problem

After Phase 028, compile-valid coverage is `80/101`, and every MIR-lowered
exemplar emits compile-valid Wasm. `examples/exempla/intra/intra.fab` remains at
frontend-analyzed because MIR item lowering rejects `HirItemKind::Const` with the
generic diagnostic `top-level Const(...)`.

The continuation plan frames this as a program model question. A full global ABI
would be premature for Wasm and LLVM, but the current examples only need typed,
source-order top-level constants to be visible to entry lowering.

## Normalized Spec

- Keep MIR target-neutral; do not add Wasm globals or imports.
- Lower typed top-level const initializers as immutable entry-prefix locals for
  synthetic `incipit` functions.
- Preserve source order among constants as they appear in HIR items.
- Leave full executable top-level statement semantics outside this phase.
- Add focused tests for MIR and Wasm lowering of an entry that reads a top-level
  const.

## Checkpoints

- `intra/intra.fab` no longer fails on `top-level const` during MIR lowering.
- Compile-valid coverage increases if the remaining `intra` surface is already
  representable.

## Gate Plan

- Focused MIR/Wasm tests for top-level const entry-prefix lowering.
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix -- --test-threads=1`
- `./scripta/lint`

## Result

Implemented top-level const lowering as entry-prefix MIR locals:

- `HirItemKind::Const` is no longer rejected by item lowering.
- The synthetic entry builder materializes typed top-level constants as
  immutable locals before entry body statements.
- Const initializers use ordinary MIR expression lowering and local assignment,
  keeping the representation target-neutral for Wasm and LLVM.

This moved `examples/exempla/intra/intra.fab` from frontend-analyzed to runnable
under the stub host. Compile-valid coverage increased from `80/101` to
`81/101`.

Current limitation: executable top-level statements that appear before an
explicit `incipit` are still a separate HIR/program-model issue. This phase only
fixes top-level const visibility and initialization for MIR entry lowering.

## Validation Log

- `cargo test -p radix top_level_const -- --nocapture`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/intra/intra.fab`: passed.
- `wasm-tools validate /tmp/intra.wat`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix llvm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix -- --test-threads=1`: passed.
- `./scripta/lint`: passed.
