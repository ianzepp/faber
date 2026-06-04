# Phase 007: Runtime Calls And Host Boundary Delivery

## Interpreted Problem

LLVM text lowering rejects every `MirStmtKind::RuntimeCall`, so ordinary MIR that
uses diagnostics, assertions, panic, conversion, string formatting, or collection
operations cannot cross the backend boundary. Wasm already lowers these through a
Wasm import ABI, but LLVM needs its own target policy instead of inheriting Wasm
module/import names into MIR.

## Normalized Spec

- Classify `MirIntrinsic` shapes into LLVM runtime declarations, layout-sensitive
  handle calls, and deferred host/provider calls.
- Emit named LLVM runtime declarations for supported runtime-call-backed MIR.
- Lower supported `RuntimeCall` statements to `call` instructions using those
  declarations.
- Store value-returning runtime calls into their MIR destination.
- Reject unsupported runtime shapes with operation-specific
  `MIR-to-LLVM unsupported` diagnostics.
- Keep MIR target-neutral and leave Wasm import ABI unchanged.

## Repo-Aware Baseline

- Current LLVM emitter: `crates/radix/src/mir/llvm_text.rs`.
- Current focused tests: `crates/radix/src/mir/llvm_text_test.rs`.
- Runtime MIR definitions: `crates/radix/src/mir/nodes.rs`.
- Wasm precedent: `crates/radix/src/mir/wasm_text.rs`.
- Phase 006 e2e baseline: 1/102 LLVM emitted, 0/102 verifier-valid in this
  environment because `llvm-as`/`opt` are unavailable.

## Stage Graph

1. Add an LLVM runtime ABI helper that maps supported `MirIntrinsic` calls to
   stable `@__faber_runtime_*` names and LLVM signatures.
2. Scan the MIR program before definitions and emit deterministic `declare`
   lines for runtime symbols used by the program.
3. Lower `RuntimeCall` statements through that helper.
4. Add focused tests for diagnostic, assert, panic, conversion, format, and
   collection runtime calls plus provider rejection.
5. Update the LLVM baseline ledger with the new runtime boundary policy and
   measured counts.

## Runtime Boundary Policy

LLVM runtime symbols use the same `__faber_runtime_` prefix as the Wasm local
import symbols, but they are LLVM external declarations, not Wasm imports. The
symbol suffix records intrinsic kind, argument count, argument ABI classes, and
result ABI class where relevant.

Supported ABI classes in this phase:

- `i1` for `bivalens`;
- `i64` for `numerus`;
- `f64` for `fractus`;
- `ptr` handle for `textus`, aggregate, nullable, provider, and other
  non-scalar semantic values that still need a physical layout phase;
- `void` for `vacuum` calls.

Provider/HAL calls remain deferred because the host boundary has provider module
identity and effect semantics beyond a plain runtime helper declaration.

## Checkpoints

- `cargo test -p radix llvm -- --nocapture`
- `cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- Completion audit against this spec before commit.

## Wasm Follow-Up

This phase should not change MIR shape or Wasm import naming. Wasm validation is
still required because runtime intrinsics are shared MIR facts and ABI drift
would be easy to introduce accidentally.

## Completion Notes

- Added deterministic LLVM runtime declaration emission before function
  definitions.
- Lowered supported `MirStmtKind::RuntimeCall` statements to LLVM `call`
  instructions.
- Added the LLVM runtime ABI naming policy for diagnostics, assert, format,
  conversion, collection operations, and panic.
- Kept provider/HAL runtime calls explicitly unsupported.
- Kept ordinary handle returns explicitly unsupported with
  `MIR-to-LLVM unsupported: handle return ABI`.
- Left `nil` constants unsupported until nullable/layout policy exists.
- Updated the ignored LLVM exempla harness floors to the measured Phase 007
  baseline: 35/102 LLVM emitted, 0/102 verifier-valid, 39 unsupported LLVM
  diagnostics.

Validation completed:

```text
cargo test -p radix llvm -- --nocapture
result: 18 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed; 102/102 frontend analyzed; 74/102 MIR lowered; 35/102 LLVM emitted; 0/102 verifier-valid; 28 MIR lowering failures; 39 unsupported LLVM diagnostics; 0 emission failures; 0 output-write failures; 0 verifier failures
```

```text
cargo test -p radix mir -- --nocapture
result: 133 passed, 0 failed
```

```text
cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix
result: 550 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

```text
./scripta/lint
result: passed
```

```text
rustfmt --edition 2021 --check crates/radix/src/mir/llvm_text.rs crates/radix/src/mir/llvm_text_test.rs crates/radix/src/exempla_e2e/llvm.rs
result: passed
```

## Completion Audit

- Runtime intrinsic classification is implemented in the LLVM emitter:
  diagnostics, assert, format, conversion, collection operations, and panic map
  to LLVM runtime declarations; provider runtime calls fail closed.
- Runtime declarations are deterministic and emitted before function
  definitions.
- Runtime call statements emit LLVM `call` instructions and value-returning
  calls store into MIR destinations.
- Layout-sensitive gaps remain explicit: ordinary handle returns, `nil`, place
  projections, constructs, and unreachable terminators still report
  `MIR-to-LLVM unsupported`.
- Wasm import ABI was not changed; Wasm-focused tests passed.
- The e2e harness floors match the measured Phase 007 baseline.

Verdict: cleared for commit.
