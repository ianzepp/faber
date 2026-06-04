# Phase 025: Runtime Collection MIR And Wasm Emission

## Interpreted Problem

Several exemplars stop on `method call before runtime/provider MIR lowering` or on
Wasm emission for collection runtime calls that already exist in MIR. The harness
shows `innatum/innatum.fab` blocked on `primus()`, and collection `appende` /
mutation calls cannot reach compile-valid Wasm because only `longitudo` is emitted.

Closure-taking methods (`filtrata`, `mappata`) remain deferred to callable-value
lowering.

## Normalized Spec

- Extend `MirCollectionOp` with zero-argument and value-argument stdlib array
  methods: `primus`, `ultimus`, `inversa`, `inverte`, `ordinata`, plus existing
  `appende` / `addita` / `accipe` / `continet` / `habet` shapes.
- Keep MIR target-neutral: collection semantics as `MirIntrinsic::Collection`, not
  Wasm imports in MIR.
- Emit honest `faber_runtime` imports for all collection intrinsics used by the
  Wasm text probe, including void mutators.
- Do not lower closure-taking `filtrata` / `mappata` in this phase.
- Preserve Phase 021 expected-tier floors; raise floors only for proven advances.

## Checkpoints

- `innatum/innatum.fab` reaches compile-valid Wasm.
- Existing `71/101` compile-valid exemplars remain valid.
- Closure-method exemplars still fail with explicit diagnostics.

## Gate Plan

- `cargo test -p radix collection -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`