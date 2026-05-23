# Phase 4 Delivery: Mutable HIR Visitor

## Status

Complete after full `radix` verification.

## Objective

Close the remaining broad HIR traversal gap by adding a mutable visitor and
moving typecheck finalization onto it.

## Scope

- Add `HirVisitorMut` and mutable `walk_*_mut` helpers beside the read-only HIR
  visitor.
- Replace the recursive finalization walker in `semantic/passes/typecheck` with
  finalization policy hooks over `HirVisitorMut`.
- Preserve the previous finalization policy: declarations report unresolved
  inferred types, expression slots only store non-inferred resolutions, and
  `cape` binding types resolve without adding a new diagnostic.

## Validation

- `cargo fmt --all --check` passed.
- `cargo test -p radix semantic::passes::typecheck` passed.
- `cargo test -p radix` passed: 425 unit tests, 8 hygiene tests, and 1 doctest
  passed; 3 unit tests and 1 doctest remain intentionally ignored.
