# Phase 07 Delivery Spec: Codegen Shared and TypeScript Backend

## Target

Document the shared codegen boundary and the TypeScript backend without changing
behavior.

## Files

- `crates/radix/src/codegen/mod.rs`
- `crates/radix/src/codegen/names.rs`
- `crates/radix/src/codegen/writer.rs`
- `crates/radix/src/codegen/ts/mod.rs`
- `crates/radix/src/codegen/ts/decl.rs`
- `crates/radix/src/codegen/ts/expr.rs`
- `crates/radix/src/codegen/ts/stmt.rs`
- `crates/radix/src/codegen/ts/types.rs`

## Acceptance Criteria

- The shared codegen module explains target dispatch, HIR error rejection,
  target-specific output boundaries, and why backends share `CodeWriter` and
  `NameCatalog`.
- Naming documentation explains how `DefId` and interned symbols are recovered
  for backend emission without treating source spellings as semantic identity.
- Writer documentation explains line-start indentation behavior, scoped
  indentation, and backend formatting constraints.
- TypeScript module and file headers explain the backend's role, trade-offs,
  HIR-to-TS mapping policy, async entry wrapping, declaration emission, statement
  and expression lowering, and type translation.
- Documentation does not duplicate every syntax conversion and does not claim
  TypeScript enforces Faber-only semantics that are actually enforced earlier or
  by runtime/codegen conventions.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Rust, Go, and canonical Faber backends; those are later phases.
- Test file changes.
- Runtime behavior, emitted code changes, or target support expansion.

## Validation Plan

- `cargo fmt --check`
- `cargo test -p radix`
- `git diff --check`

## Verification

- `cargo fmt --check` passed.
- `cargo test -p radix` passed.
- `git diff --check` passed.
- Poker-face completion check: 96/100, no blocking gaps.
