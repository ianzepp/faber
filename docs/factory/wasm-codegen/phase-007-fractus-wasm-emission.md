# Phase 007: Fractus Wasm Emission

## Interpreted Problem

After Phase 006, the remaining scalar Wasm-emission blocker was `fractus`.
Two MIR-lowered exemplars stopped on `Primitive(Fractus)`.

## Normalized Spec

- Keep HIR -> typed HIR -> MIR -> Wasm intact.
- Represent `fractus` as Wasm `f64`.
- Emit float constants, locals, params, returns, arithmetic, comparisons, unary
  negation, calls, and diagnostics in the existing Wasm text probe.
- Keep host/runtime failures separated through explicit `*_f64` diagnostic
  imports.
- Preserve explicit unsupported diagnostics for aggregates and other
  runtime-managed values.

## Repo-Aware Baseline

The Phase 006 harness reported 24/101 compile-valid exemplars. The visible
MIR-lowered `fractus` blockers were:

- `examples/exempla/functio/typicus.fab`
- `examples/exempla/varia/typicus.fab`

## Stage Graph

1. Add `F64` to the Wasm probe value model.
2. Map `fractus` and float constants to `f64`.
3. Emit `f64` arithmetic, comparisons, and unary negation.
4. Add `*_f64` diagnostic imports.
5. Validate focused WAT tests and the ignored exemplar harness.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when the two visible `fractus` exemplars advance to
compile-valid and no existing text/numeric compile-valid exemplars regress.
