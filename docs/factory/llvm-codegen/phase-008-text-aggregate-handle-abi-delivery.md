# Phase 008: Text And Aggregate Handle ABI Delivery

## Interpreted Problem

Phase 007 can move opaque `ptr` values through runtime calls, but LLVM still
rejects aggregate construction, projections, and ordinary handle returns. Wasm
already treats text and aggregate values as runtime-owned handles. LLVM needs an
equally explicit target-local ABI so it can lower handle-shaped MIR without
weakening MIR semantic types or guessing inline layouts.

## Normalized Spec

- Choose the first LLVM probe ABI for `textus` and aggregate values.
- Keep the representation target-local: LLVM text uses opaque `ptr` runtime
  handles and MIR semantic types remain unchanged.
- Lower aggregate construction to external LLVM helper declarations and calls
  when every operand has a supported scalar-or-handle ABI class.
- Lower simple projection reads and writes through external LLVM helper
  declarations when field/index metadata can prove key and result types.
- Continue to reject spread, nested projection writes, rich enum layout,
  nullable operations, and dynamic/function handles unless the ABI is explicit.
- Wire LLVM through context-preserving MIR lowering where projection metadata is
  needed.
- Preserve Wasm behavior and import naming.

## Repo-Aware Baseline

- LLVM emitter: `crates/radix/src/mir/llvm_text.rs`.
- LLVM driver path currently calls `lower_mir_for_target` and loses validation
  metadata.
- Wasm driver path already uses `lower_mir_with_context_for_target`.
- Wasm handle precedent lives in `crates/radix/src/mir/wasm_text.rs`.
- Phase 007 e2e baseline: 35/102 LLVM emitted, 0/102 verifier-valid, 39
  unsupported LLVM diagnostics.

## Stage Graph

1. Add a context-aware LLVM text entrypoint and wire driver/e2e LLVM lowering
   through `LoweredMirUnit`.
2. Add LLVM aggregate helper declarations for construction and projection
   read/write calls.
3. Lower aggregate construction for non-spread ordered/named/keyed operands to
   runtime-owned `ptr` handles.
4. Lower single-step projection reads and writes where type metadata is
   available.
5. Keep unsupported cases explicit with operation-specific diagnostics.
6. Add focused tests and update the baseline ledger with measured counts.

## ABI Decision

LLVM text uses opaque `ptr` handles for `textus`, arrays, maps, sets, records,
structs, enum values, options, and dynamic value-like runtime objects. The
physical allocation, ownership, and field/index storage are owned by external
runtime helpers. LLVM helper names are target-local symbols with explicit ABI
classes, for example:

- `@__faber_aggregate_array_3_i64_i64_i64() -> ptr`
- `@__faber_aggregate_field_i64_to_ptr(ptr, i64) -> ptr`
- `@__faber_aggregate_set_index_i64_i64(ptr, i64, i64) -> void`

This phase does not define a native runtime implementation or claim executable
LLVM output. It only emits text calls to declared helpers.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

No Wasm import names should change. Wasm tests remain required because the same
MIR aggregate and projection facts are shared.

## Completion Notes

- Added `emit_llvm_text_probe_with_context` so LLVM can use the same validation
  metadata as Wasm for projection result typing.
- Wired the driver and LLVM e2e harness through context-preserving MIR lowering.
- Chose opaque `ptr` handles for ordinary text and aggregate-like values.
- Lifted ordinary handle return support to `ret ptr`.
- Lowered aggregate construction to declared `__faber_aggregate_*` helpers.
- Lowered projection reads and single-step projection writes to declared
  aggregate helper calls.
- Kept aggregate spreads, nested projection writes, `nil`, option operations,
  text operators, switches, unreachable policy, and provider/HAL effects
  explicitly deferred.

Validation completed so far:

```text
cargo test -p radix llvm -- --nocapture
result: 21 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed; 102/102 frontend analyzed; 74/102 MIR lowered; 58/102 LLVM emitted; 0/102 verifier-valid; 28 MIR lowering failures; 16 unsupported LLVM diagnostics; 0 emission failures; 0 output-write failures; 0 verifier failures
```

```text
cargo test -p radix mir -- --nocapture
result: 136 passed, 0 failed
```

```text
cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix
result: 553 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

```text
./scripta/lint
result: passed
```

```text
rustfmt --edition 2021 --check crates/radix/src/mir/llvm_text.rs crates/radix/src/mir/llvm_text_test.rs crates/radix/src/mir/mod.rs crates/radix/src/driver/mod.rs crates/radix/src/exempla_e2e/llvm.rs
result: passed
```

## Completion Audit

- ABI decision is explicit and documented: LLVM handle values use opaque `ptr`
  runtime-owned handles.
- Context-aware LLVM emission is implemented and wired through driver and e2e
  paths.
- Aggregate construction lowers to declared `__faber_aggregate_*` helpers for
  supported non-spread operands.
- Projection reads and single-step writes lower to declared aggregate helpers
  when metadata proves key/result/value types.
- Deferred shapes remain explicit: spread construction, nested projection
  writes, `nil`, option operations, text operators, switches, unreachable
  policy, providers, native runtime/execution, and verifier-valid claims.
- Wasm import ABI was not changed; Wasm-focused tests passed.
- The e2e harness floors match the measured Phase 008 baseline.

Verdict: cleared for commit.
