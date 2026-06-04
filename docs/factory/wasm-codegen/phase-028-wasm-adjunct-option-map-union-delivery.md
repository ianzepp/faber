# Phase 028: Wasm Adjunct For Option, Map, And Union Shapes

## Interpreted Problem

After Phase 027, compile-valid coverage is `77/101`. Several exempla now have
valid MIR but stop at Wasm emission:

- `optionalis/optionalis.fab`: `option chain value`.
- `praefixum/praefixum.fab`: map-shaped field projection emission.
- `destructura/objectum.fab`: typed union carrier support.

The numbered continuation plan's Phase 028 covers top-level declarations, but
the user target is 85% exempla completion. This interstitial phase converts
already-validated MIR into compile-valid Wasm before returning to top-level
declarations.

## Normalized Spec

- Keep MIR target-neutral; do not add Wasm-specific MIR nodes.
- Add Wasm text emission only for MIR shapes that already validate:
  - map field projection through existing projection import machinery;
  - option-chain values as compile-valid opaque handle operations;
  - union types as opaque aggregate handles when the value is already
    aggregate-like.
- Do not claim behavior correctness for option/map/union runtime semantics
  beyond stub-host instantiation and invocation.
- Preserve existing compile-valid, instantiate-valid, and runnable floors.

## Checkpoints

- `praefixum/praefixum.fab` reaches compile-valid if map field projection is
  enough.
- `optionalis/optionalis.fab` reaches compile-valid if option-chain handle
  emission is enough.
- `destructura/objectum.fab` reaches compile-valid if union-carrier support is
  enough.

## Gate Plan

- Focused Wasm tests for map-field projection and option-chain emission.
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix -- --test-threads=1`
- `./scripta/lint`

## Result

Implemented the Wasm adjunct as scoped emission support:

- map field projections now use the existing aggregate field-import surface;
- option-chain field and index links emit projection calls over handle carriers;
- union values lower to opaque aggregate handles at the Wasm text boundary;
- pass-through scalar operands assigned into aggregate-handle union slots are
  coerced to the destination carrier.

The phase moved compile-valid coverage from `77/101` to `80/101`.

New compile-valid exempla:

- `examples/exempla/destructura/objectum.fab`
- `examples/exempla/optionalis/optionalis.fab`
- `examples/exempla/praefixum/praefixum.fab`

Behavior remains stub-host only for these shapes. Real option, map, and union
runtime semantics still belong behind a concrete aggregate runtime.

## Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix llvm -- --nocapture`: passed.
- `cargo test -p radix -- --test-threads=1`: passed.
- `./scripta/lint`: passed.
