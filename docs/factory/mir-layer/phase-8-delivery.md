# Phase 8 Delivery: MIR Validation

## Interpreted Problem

Phases 1 through 7 make MIR inspectable and expressive enough for a meaningful Rust backend vertical slice. MIR now represents primitive computation, explicit control flow, alternate exits, structured `cape` handling, aggregate and option/null operations, runtime intrinsics, collection operations, and provider calls.

Before any backend consumes MIR, the compiler needs a validation pass that proves the MIR program is internally coherent. Without that gate, every backend must defensively rediscover basic invariants, and lowering bugs can turn into confusing backend failures.

Phase 8 should turn MIR from a debug/inspection artifact into a backend-trustable compiler contract for the supported subset.

## Normalized Spec

- Add a MIR validation module.
- Validate the MIR shapes that exist after Phase 7.
- Run validation before `lower_analyzed_unit` returns successful MIR.
- Keep validation target-neutral and independent of Rust backend requirements.
- Report all validation failures that can be found in one pass.
- Attach source spans to validation diagnostics where MIR nodes already carry spans.
- Keep unsupported or unresolved MIR shapes from reaching any backend.
- Add focused tests that construct invalid MIR directly.
- Add success tests proving representative Phase 3-7 MIR fixtures validate.

## Validation Entry Point

Phase 8 should introduce a public validation API such as:

```rust
validate_program(program, context) -> Result<(), Vec<MirValidationError>>
```

The exact context shape can be chosen during implementation, but it should be explicit. Some checks need more than `MirProgram` alone:

- semantic type information for type compatibility,
- function signature/error information for direct calls and `try_call`,
- provider/import identity information where available.

If a check cannot be made without future metadata, Phase 8 should fail only on the invariant it can prove, and document the deferred check.

## Structural Contract

The validator should check:

- function IDs are valid within the program,
- block IDs are valid within each function,
- local and temp IDs are valid within each function,
- block IDs referenced by terminators exist,
- every block has exactly one terminator,
- function parameter locals exist and have matching types,
- local/temp declarations do not create impossible duplicate IDs,
- statements appear only inside blocks and before the block terminator by construction.

The validator is allowed to assume finished MIR uses the `MirBlock` structure, not the lowerer's internal open-block builder.

## Reference Contract

The validator should check references inside:

- `MirOperand`,
- `MirPlace` bases and projections,
- assignment destinations,
- call and runtime-call destinations,
- `try_call` destinations and error places,
- branch, goto, switch, and `try_call` block targets,
- aggregate fields,
- option operations,
- runtime-call arguments,
- callee references where the referenced function is local to the current `MirProgram`.

`MirOperand::Value(MirValueId)` needs an explicit rule. If Phase 8 does not establish a real value-definition table, the validator should reject value operands that cannot be proven to reference a defined earlier `MirValue`.

## Type Contract

The validator should check the basic type facts that backends should be able to trust:

- assigned value type is compatible with destination place type,
- `return` operand type matches the function return type,
- no-value `return` appears only where the function return type is `vacuum`,
- `return_error` appears only when the function has an alternate-exit type,
- `return_error` operand type matches the function alternate-exit type,
- branch conditions are `bivalens`,
- `try_call` success destination matches the callee success type when known,
- `try_call` error place matches the callee alternate-exit type when known,
- construct destination type matches aggregate type,
- runtime-call destination type matches runtime-call return type,
- option operation result types match the operation contract,
- projection base/index operands have enough type information to validate or fail clearly.

Phase 8 should use existing semantic assignability rules if there is an appropriate local API. If not, it should start with exact semantic `TypeId` equality plus explicitly documented primitive/vacuum exceptions rather than guessing.

## Operation Contracts

The validator should add operation-specific checks for current MIR nodes:

- `Return` and `ReturnError` follow the function exit contract.
- `TryCall` is a terminator with valid success and error edges.
- `TryCall` is used only for callees that are known failable when that knowledge is available.
- `Goto`, `Branch`, and `Switch` target valid blocks.
- `Switch` cases are well-formed and have valid targets.
- aggregate payload shape matches aggregate kind where Phase 8 can prove it.
- struct/variant named fields are non-empty and stable where present.
- map keyed fields have both key and value operands.
- option operations use valid operands and produce the declared result type.
- runtime intrinsics use valid operands and keep target-neutral identity.
- provider calls preserve provider identity without target linkage strings.
- panic runtime calls may return `numquam` and normally flow to `unreachable`.

The validator should not interpret target translations, Cargo metadata, WASM imports, native symbols, or Rust module paths.

## Backend-Readiness Contract

Validated MIR should contain no known unresolved backend blockers for the supported subset:

- no invalid references,
- no missing required types,
- no unsupported placeholder nodes,
- no target-specific lowering fragments,
- no malformed local handler edge,
- no malformed alternate-exit edge.

This does not mean all Faber source constructs are supported by MIR. Deferred source constructs should continue to fail during MIR lowering before validation.

## Repo-Aware Baseline

- `crates/radix/src/mir/nodes.rs` defines the MIR model after Phases 1-7.
- `crates/radix/src/mir/lower.rs` already constructs explicit blocks and terminators through an internal open-block builder.
- `finish_blocks` currently seals leftover open blocks with `unreachable`; validation should check finished MIR, not replace the builder discipline.
- `MirTerminatorKind::TryCall` represents local `cape` handling edges for failable calls.
- `MirTerminatorKind::ReturnError` represents function-level recoverable alternate exits.
- `MirStmtKind::RuntimeCall` now carries structured target-neutral intrinsics.
- `MirStmtKind::Construct` now carries aggregate payloads with ordered, named, or keyed fields.
- `MirValueKind::Option` carries option/null operations from Phase 6A/6B.
- No backend currently consumes MIR.

## Stage Graph

1. Add `crates/radix/src/mir/validate.rs`.
2. Define `MirValidationError` with message and span.
3. Define the validation context needed for type and callee checks.
4. Validate function, block, local, temp, and value reference integrity.
5. Validate terminator target integrity.
6. Validate assignment, return, alternate-exit, branch, and `try_call` type contracts.
7. Validate aggregate, projection, option, runtime, collection, and provider operation contracts.
8. Integrate validation into successful MIR lowering.
9. Add direct invalid-MIR unit tests.
10. Add success tests for representative real lowered MIR from Phases 3-7.
11. Keep target backends unchanged.

## Checkpoints

- Hand-built MIR with an invalid block target fails validation.
- Hand-built MIR with an invalid local/temp reference fails validation.
- Hand-built MIR with a mismatched return type fails validation.
- Hand-built MIR with `return_error` in a non-failable function fails validation.
- Hand-built MIR with a non-`bivalens` branch condition fails validation.
- Hand-built MIR with malformed `try_call` edges fails validation.
- Hand-built MIR with runtime-call destination/return type mismatch fails validation.
- Representative valid MIR fixtures from Phases 3-7 pass validation.
- `radix mir` still emits deterministic output for supported source programs.
- No target backend consumes MIR in this phase.

## Out Of Scope

- Full source semantic analysis.
- Definite-assignment analysis.
- Borrow checking or ownership validation independent of Rust.
- Lifetime, drop, retain, release, or free semantics.
- ABI/layout validation.
- SSA, phi nodes, optimization, or data-flow optimization passes.
- Exhaustiveness checking beyond existing semantic passes.
- Backend-specific legality for Rust, WASM, native, Go, TypeScript, or Python.
- Completing deferred source features such as `itera`, `elige`, `discerne`, closures, async/cursor behavior, collection pipelines, propagation syntax, or `demum`.

## Validation

- Focused unit tests for structural validation failures.
- Focused unit tests for reference validation failures.
- Focused unit tests for type-contract validation failures.
- Focused unit tests for operation-specific validation failures.
- Positive validation tests for representative lowered MIR fixtures.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 8 complete.

## Completion Gate

Phase 8 is complete when successful MIR lowering runs through validation, invalid MIR is rejected before any backend can consume it, diagnostics identify the violated invariant with source spans where available, representative Phase 3-7 MIR fixtures still validate and dump deterministically, and validation does not expand into future source features, optimization, ABI/layout, or backend-specific policy.
