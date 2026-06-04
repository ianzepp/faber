# Phase 005 Direct Function Calls Delivery

**Status**: Complete  
**Date**: 2026-06-04

## Objective

Teach the experimental LLVM text probe to lower direct scalar MIR calls between
functions already present in the MIR program. This phase extends the scalar
lane without introducing runtime ABI, callable values, failable calls, or
external symbol policy.

## Scope

Included:

- Lower `MirStmtKind::Call` for `MirCallee::Function`.
- Lower `MirStmtKind::Call` for `MirCallee::Definition` when the definition
  resolves to a MIR function in the current program.
- Emit scalar argument operands in MIR order.
- Store scalar non-vacuum call results into the destination place.
- Emit vacuum calls as side-effecting `call void` statements with no
  destination.
- Add focused tests for helper calls and call chains.

Deferred:

- external definition declarations and linkage policy.
- value callees/callable values.
- `TryCall`, alternate exits, and failable call ABI.
- runtime calls and host boundary symbols.
- verifier integration.

## Validation Plan

Run:

```text
cargo test -p radix llvm -- --nocapture
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
cargo test -p radix mir -- --nocapture
cargo test -p radix wasm -- --nocapture
cargo test -p radix
./scripta/lint
```

## Completion Notes

Implemented in `crates/radix/src/mir/llvm_text.rs` by adding call-statement
lowering, same-program callee resolution for `MirCallee::Function` and
`MirCallee::Definition`, scalar argument rendering, scalar result storage, and
explicit unsupported diagnostics for value callees and external definitions.

Focused coverage in `crates/radix/src/mir/llvm_text_test.rs` now includes
direct helper calls, call chains through locals, external definition rejection,
and value-callee rejection.

Final validation:

```text
cargo test -p radix llvm -- --nocapture
result: 15 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix exempla_llvm_e2e -- --ignored --nocapture
result: 1 passed, 0 failed
counts: 102/102 frontend, 74/102 MIR, 1/102 LLVM, 73 unsupported diagnostics
```

```text
cargo test -p radix mir -- --nocapture
result: 130 passed, 0 failed
```

```text
cargo test -p radix wasm -- --nocapture
result: 29 passed, 0 failed, 1 ignored
```

```text
cargo test -p radix
result: 547 passed, 0 failed, 6 ignored; hygiene 8 passed; doctests 1 passed, 1 ignored
```

```text
./scripta/lint
result: passed
```
