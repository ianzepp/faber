# Phase 031: Enum Member Paths and Scalar `verte` Ascription

## Interpreted Problem

After Phase 030, compile-valid coverage is `83/101`; reaching the explicit
85% target requires at least three more exempla. Two remaining failures are
small target-neutral lowering gaps rather than backend-specific Wasm work:

- `examples/exempla/ordo/ordo.fab` stops on enum member paths such as
  `Color.rubrum` that semantic analysis has typed but MIR lowering does not
  turn into enum variant values.
- `examples/exempla/ternarius/ternarius.fab` stops on simple `verte` type
  ascription (`nihil ∷ textus ∪ nihil`) because aggregate lowering only accepts
  construction-like `verte` targets.
- `examples/exempla/inter/inter.fab` stops on `inter` membership because the
  typed operator reaches MIR as `HirBinOp::Between`, but MIR lowering has not
  connected it to the existing collection `Contains` intrinsic.

## Normalized Spec

- Lower enum member values to enum-variant MIR aggregates when HIR exposes them
  as resolved variant paths or enum namespace field access.
- Lower non-construction `verte` as a typed value passthrough when the target
  type is scalar/union/option-like and no object-entry construction metadata is
  present.
- Lower `lhs inter rhs` membership to the existing target-neutral collection
  `Contains(rhs, lhs)` intrinsic.
- Keep struct/map/array/set/enum construction `verte` behavior unchanged.
- Do not add Wasm-specific MIR nodes.

## Checkpoints

- `ordo/ordo.fab` reaches compile-valid Wasm if the lowered enum member shapes
  are already supported downstream.
- `ternarius/ternarius.fab` reaches compile-valid Wasm if existing ternary and
  union-carrier support covers the resulting MIR.
- `inter/inter.fab` reaches compile-valid Wasm if existing collection intrinsic
  imports cover array membership checks.

## Gate Plan

- Focused MIR tests for unit enum member value lowering and scalar `verte`
  passthrough.
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix -- --test-threads=1`
- `./scripta/lint`

## Result

Three small target-neutral expression lowering gaps are closed:

- Unit enum variant values now lower from variant paths to zero-payload enum
  variant aggregate construction.
- Scalar/non-construction `verte` now lowers as a target-typed passthrough temp,
  preserving aggregate construction behavior for struct/map/array/set/enum
  targets.
- `inter` membership now lowers to `MirCollectionOp::Contains` with the
  collection operand first and candidate value second.

Measured compile-valid coverage increased from `83/101` to `86/101`, satisfying
the 85% target (`86/101`).

New compile-valid exempla:

- `examples/exempla/inter/inter.fab`
- `examples/exempla/ordo/ordo.fab`
- `examples/exempla/ternarius/ternarius.fab`

## Caveats

`ordo/ordo.fab` is compile-valid and instantiate-valid under the stub host, but
its `incipit` export still traps at runtime because enum aggregate equality is
still implemented through the existing opaque aggregate handle model. This phase
improves compile coverage, not final enum runtime semantics.

Map object spread remains intentionally unsupported in this phase. MIR keyed
map aggregates do not yet have a spread entry shape, so `objectum/objectum.fab`
needs a separate MIR node/validation change rather than a small lowering tweak.

## Validation Log

- `cargo test -p radix unit_enum_variant_path_values -- --nocapture`: passed.
- `cargo test -p radix scalar_verte_as_target_typed_passthrough -- --nocapture`: passed.
- `cargo test -p radix inter_membership -- --nocapture`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/ordo/ordo.fab > /tmp/ordo.wat`: passed.
- `wasm-tools validate /tmp/ordo.wat`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/ternarius/ternarius.fab > /tmp/ternarius.wat`: passed.
- `wasm-tools validate /tmp/ternarius.wat`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/inter/inter.fab > /tmp/inter.wat`: passed.
- `wasm-tools validate /tmp/inter.wat`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix llvm -- --nocapture`: passed.
- `cargo test -p radix -- --test-threads=1`: passed.
- `./scripta/lint`: passed.
