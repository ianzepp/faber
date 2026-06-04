# Phase 009: Nullable And Optional Operations Delivery

## Interpreted Problem

LLVM now has an opaque handle ABI for text and aggregate-like values, but nullable
values are still rejected. MIR already represents nullable behavior explicitly
through `MirOptionOp` and nullable semantic types. LLVM needs a type-driven
representation that can lower supported nullable operations without pretending
that source-level nullability is a target-neutral pointer convention.

## Normalized Spec

- Choose a first LLVM nullable representation for scalar and handle payloads.
- Lower `None`, `Some`, `IsNil`, `IsNonNil`, `Unwrap`, and `Coalesce` only when
  the payload has a supported scalar-or-handle ABI class.
- Keep optional chains through fields, indexes, and calls deferred.
- Keep invalid or unsupported nullable shapes fail-closed with
  `MIR-to-LLVM unsupported` diagnostics.
- Preserve MIR semantic types and Wasm behavior.

## Repo-Aware Baseline

- LLVM option operations currently fail in `crates/radix/src/mir/llvm_text.rs`
  with `MIR-to-LLVM unsupported: option value`.
- `MirConstant::Nil` currently fails with `MIR-to-LLVM unsupported: nil constant`.
- Wasm has limited handle coalesce support but still rejects most explicit
  option operations.
- Phase 008 e2e baseline: 58/102 LLVM emitted, 0/102 verifier-valid, 16
  unsupported LLVM diagnostics.

## Stage Graph

1. Add an LLVM option helper ABI for nullable values.
2. Lower `MirConstant::Nil` as the null handle literal for handle-shaped
   contexts.
3. Lower `MirOptionOp::{None, Some, IsNil, IsNonNil, Unwrap, Coalesce}` through
   declared helper calls when the result and payload ABI classes are known.
4. Keep `MirOptionOp::Chain` deferred.
5. Add focused tests for scalar and handle nullable operations plus chain
   rejection.
6. Update the LLVM baseline ledger with measured counts and residual gaps.

## ABI Decision

LLVM nullable values are runtime-owned opaque handles (`ptr`) regardless of
payload class in this probe phase. Payload class is still encoded in helper
names so the runtime ABI remains type-directed:

- `@__faber_option_none_i64() -> ptr`
- `@__faber_option_some_i64(i64) -> ptr`
- `@__faber_option_is_nil(ptr) -> i1`
- `@__faber_option_unwrap_i64(ptr) -> i64`
- `@__faber_option_coalesce_i64(ptr, i64) -> i64`

This avoids guessing inline struct layout for scalar nullable values before an
LLVM verifier/runtime story exists. It also matches the Phase 008 handle model:
LLVM text can declare and call helpers, but this phase does not claim native
execution.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

No MIR shape or Wasm import naming changes are expected. Wasm validation remains
required because option MIR facts are shared by both backend lanes.

## Completion Evidence

Phase 009 adds the first LLVM nullable ABI without introducing inline nullable
layout. Supported nullable values are opaque runtime handles, and payload ABI
classes are encoded in helper symbols. `nil` now lowers to the LLVM `null`
handle literal.

Implemented support:

- `MirConstant::Nil` as `ptr null`.
- `MirOptionOp::None` through `__faber_option_none_{payload}`.
- `MirOptionOp::Some` through `__faber_option_some_{payload}`.
- `MirOptionOp::IsNil` through `__faber_option_is_nil`.
- `MirOptionOp::IsNonNil` through `__faber_option_is_non_nil`.
- `MirOptionOp::Unwrap` through `__faber_option_unwrap_{payload}`.
- `MirOptionOp::Coalesce` through `__faber_option_coalesce_{payload}`.

Still deferred:

- `MirOptionOp::Chain`.
- Optional field, index, and call chaining.
- Native execution/runtime implementation of the declared helpers.
- LLVM verifier claims when `llvm-as` or `opt` is unavailable.

Focused tests added:

- `llvm_text_target_emits_nil_as_null_handle`
- `llvm_text_target_emits_scalar_option_helpers`
- `llvm_text_target_rejects_option_chain`

Measured e2e baseline after implementation:

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
frontend analyzed: 102/102
MIR lowered: 74/102
LLVM emitted: 58/102
verifier-valid: 0/102
unsupported diagnostic: 16
result: passed
```

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 24 passed, 0 failed, 1 ignored

cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed

cargo test -p radix mir -- --nocapture
result: 139 passed, 0 failed

cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored

cargo test -p radix
result: 556 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored

./scripta/lint
result: passed

rustfmt --edition 2021 --check crates/radix/src/mir/llvm_text.rs crates/radix/src/mir/llvm_text_test.rs
result: passed

git diff --check
result: passed
```

## Completion Audit

- Nullable representation chosen: runtime-owned opaque `ptr` handles with
  payload-specific helper names.
- `None`, `Some`, `IsNil`, `IsNonNil`, `Unwrap`, and `Coalesce` lower when
  payload and fallback values fit the scalar-or-handle ABI.
- Optional chains remain fail-closed with `MIR-to-LLVM unsupported: option chain
  value`.
- Invalid option payload typing remains fail-closed before helper emission.
- MIR and Wasm surfaces are unchanged; validation covers shared nullable facts.
