# Phase 003: Primitive Wasm Emission

## Interpreted Problem

Phase 002 moved 32 exemplars to validated MIR, but none reach Wasm emission.
The visible blockers are all in the experimental Wasm text probe: runtime
calls, text/string values, aggregates, comparison and boolean binary ops, direct
calls, locals/temps, and branch terminators.

## Normalized Spec

- Keep emission MIR-backed only.
- Add a conservative primitive Wasm subset:
  - numeric and boolean locals/temps
  - direct calls
  - numeric comparisons and boolean `and`/`or`
  - primitive diagnostic runtime calls through explicit Wasm imports
- Keep unsupported text, aggregate, nullable, dynamic, and heap/runtime-managed
  values fail-closed.
- Keep host/runtime gaps separate: imported diagnostic functions can validate,
  but instantiation/running still requires a host.
- Add focused Wasm tests that validate generated WAT with `wasm-tools` when
  available.

## Repo-Aware Baseline

`crates/radix/src/mir/wasm_text.rs` currently emits only expression-tree WAT
for a single-block assignment/return subset. It does not declare non-param
locals, emit direct calls, emit runtime calls, distinguish binary operator
operand/result types, or handle branches.

## Stage Graph

1. Add local/temp declaration support for primitive Wasm value types.
2. Emit assignments through `local.set` and operand loads through `local.get`.
3. Emit direct call statements and value-producing direct calls.
4. Emit primitive numeric comparison and boolean binary operators.
5. Emit numeric/bool diagnostic runtime calls as explicit Wasm imports.
6. Run the tiered harness and update the ledger with new compile-valid counts.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when at least one exemplar reaches compile-valid Wasm,
unsupported heap/runtime shapes still fail explicitly, and imported diagnostic
host requirements are classified as host/runtime gaps rather than compiler
success beyond validation.

## Open Questions

- Branch emission may be too large for this phase if local/call/runtime support
  already exposes a better next cluster.
- Text diagnostics require a string representation and should not be faked as
  primitive diagnostics.
