# Phase 8 Execution Spec

## Target

Implement `docs/factory/mir-layer/phase-8-delivery.md`: validate finished MIR before a successful `lower_analyzed_unit` result can reach any backend consumer.

## Repo Baseline

- Phase 7 leaves `MirProgram` expressive enough to carry control flow, alternate exits, aggregates, options, runtime intrinsics, collection operations, and provider calls.
- `MirBlock` already requires one terminator structurally; validation should inspect the finished block graph, not replace lowerer builder discipline.
- Parameters are currently represented both as `MirParam` entries and matching local declarations; validation must treat matching duplicates as the existing contract.
- Field projection type checks need HIR-derived struct and variant field metadata in addition to `MirProgram` and `TypeTable`.
- `MirOperand::Value` has no global value table; validation must accept only value IDs defined earlier in the current function traversal.

## Implementation Shape

- Add `crates/radix/src/mir/validate.rs`.
- Define `MirValidationError`, `MirFunctionSignature`, and explicit `MirValidationContext`.
- Validate function, block, local, temp, and earlier-value references.
- Validate terminator block targets and function exit contracts.
- Validate assignment, return, return-error, branch, `try_call`, construct, runtime-call, aggregate, option, and provider/runtime operation contracts.
- Use `TypeTable::assignable` for semantic type compatibility where available.
- Build validation context from the analyzed unit during MIR lowering so struct fields, variant fields, and function signatures are available.
- Run validation inside `lower_analyzed_unit` before returning successful MIR.
- Keep validation target-neutral and leave backend policy, ABI/layout, optimization, and deferred source features out of scope.

## Validation Gates

- Direct invalid-MIR tests for bad block targets.
- Direct invalid-MIR tests for bad local/temp/value references.
- Direct invalid-MIR tests for return and return-error type contracts.
- Direct invalid-MIR tests for non-`bivalens` branch conditions.
- Direct invalid-MIR tests for malformed `try_call` edges.
- Direct invalid-MIR tests for runtime-call destination mismatch.
- Direct invalid-MIR tests for malformed aggregate payload shape.
- Positive tests for representative real lowered Phase 3-7 MIR fixtures.
- `cargo test -p radix mir`
- `cargo test -p radix`
- `./scripta/ci`
