# Phase 027: Non-Literal Pattern And Dynamic Switch Lowering

## Interpreted Problem

After Phase 026, the live Wasm e2e harness reports `76/101` compile-valid and
`79/101` MIR-lowered exempla. Several remaining examples stop before MIR because
`discerne` lowering only supports literal switch cases:

- `discerne/discerne.fab`
- `omnia/omnia.fab`
- `ordo/ordo.fab`
- `syntaxis/discerne-insanum.fab`

`ordo/ordo.fab` also has unresolved path failures that may be separate from
pattern lowering.

## Normalized Spec

- Extend MIR lowering only for simple non-literal patterns whose semantics are
  already typed and target-neutral.
- Prefer branch-chain lowering with existing equality/predicate MIR operations
  over adding Wasm-specific pattern nodes.
- Keep destructuring, callable guards, provider effects, and broad dynamic
  matching explicit if their semantics are not already represented.
- Preserve existing literal switch lowering and compile-valid floors.
- Keep LLVM readiness in mind: the resulting MIR should be lowerable later as
  branches, `switch`, or runtime comparator calls depending on value type.

## Checkpoints

- At least one of the dynamic-pattern exempla advances from frontend-analyzed to
  validated MIR.
- Compile-valid coverage improves when the lowered shapes are already supported
  by Wasm emission.
- Unsupported pattern shapes continue to fail with specific diagnostics.

## Gate Plan

- Focused MIR tests for simple non-literal `discerne`.
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix -- --test-threads=1`
- `./scripta/lint`
