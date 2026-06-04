# Phase 030: Payload Variant `discerne` Lowering

## Interpreted Problem

After Phase 029, compile-valid coverage is `81/101`; reaching 85% requires at
least `86/101`. Two remaining exempla stop before MIR on payload-bearing
`discerne` patterns:

- `examples/exempla/discerne/discerne.fab`
- `examples/exempla/syntaxis/discerne-insanum.fab`

MIR already has target-neutral enum-variant aggregates and variant-field
projections, and the Wasm text probe already has variant-field projection import
support. The gap is control lowering: `HirPattern::Variant(_, fields)` with
binding subpatterns is rejected by the current unit-variant branch-chain path.

## Normalized Spec

- Extend `discerne` lowering for single-subject enum variant patterns with
  payload binding subpatterns.
- Keep the shape target-neutral: branch on variant equality as the existing
  unit-variant path does, then bind payload fields through `MirProjection::VariantField`.
- Support simple binding subpatterns and alias-to-whole-value where they appear
  naturally; keep nested/destructuring subpatterns explicit unsupported shapes.
- Preserve unit-variant behavior and default-arm handling.
- Do not add Wasm-specific MIR nodes or runtime operations.

## Checkpoints

- Payload variant `discerne` examples no longer fail with
  `non-literal discerne pattern before switch MIR lowering`.
- `discerne/discerne.fab` and `syntaxis/discerne-insanum.fab` reach at least
  validated MIR; compile-valid if existing Wasm aggregate/variant-field support
  covers the lowered shape.

## Gate Plan

- Focused MIR tests for payload variant `discerne` branch/bind lowering.
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix -- --test-threads=1`
- `./scripta/lint`

## Result

Payload-bearing enum variant patterns now lower through the existing
target-neutral `discerne` branch-chain path. For each payload arm, lowering
constructs a variant-shaped probe from the subject's variant-field projections,
branches by ordinary MIR equality, then binds simple field names as immutable
locals projected from the matched subject.

This phase also maps `nihil` Wasm carriers to `i32`, which lets functions that
return nullable unions containing `nihil` emit valid WAT instead of failing
during scalar type selection.

Measured compile-valid coverage increased from `81/101` to `83/101`.

New compile-valid exempla:

- `examples/exempla/discerne/discerne.fab`
- `examples/exempla/syntaxis/discerne-insanum.fab`

## Caveat

The lowering intentionally preserves the existing aggregate-equality branch
model. It is sufficient for compile-valid and stub-host runnable Wasm progress,
but it is not the final runtime semantics for payload variant matching. A real
variant tag-test operation or equivalent runtime representation is still needed
before payload `discerne` can be considered semantically complete across
backends.

## Validation Log

- `cargo test -p radix payload_variant_discerne -- --nocapture`: passed.
- `cargo test -p radix nihil_return_carrier -- --nocapture`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/discerne/discerne.fab > /tmp/discerne.wat`: passed.
- `wasm-tools validate /tmp/discerne.wat`: passed.
- `cargo run -p radix --bin radix -- emit -t wasm examples/exempla/syntaxis/discerne-insanum.fab > /tmp/discerne-insanum.wat`: passed.
- `wasm-tools validate /tmp/discerne-insanum.wat`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix llvm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed.
- `cargo test -p radix -- --test-threads=1`: passed.
