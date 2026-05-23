# Phase 2 Delivery: Semantic Read-Only HIR Visitor Adoption

## Status

Complete after focused verification.

## Objective

Remove duplicated recursive HIR walkers from semantic passes that inspect HIR
without mutating it.

## Scope

- Port read-only semantic analyses to `HirVisitor` where traversal state can be
  represented directly in the visitor.
- Preserve pass-specific policy such as loop context, borrow scopes, and match
  exhaustiveness checks.
- Keep mutable typecheck/finalize traversal out of this phase.

## Initial Targets

- `semantic/passes/exhaustive.rs`
- `semantic/passes/lint.rs`
- `semantic/passes/borrow.rs`

## Validation

- `cargo fmt --all --check` passed.
- `cargo test -p radix semantic::passes::exhaustive` passed.
- `cargo test -p radix semantic::passes::lint` passed.
- `cargo test -p radix semantic::passes::borrow` passed.
- `cargo test -p radix semantic::` passed: 46 tests passed.

## Result

The phase removed the broad recursive expression walkers from:

- `crates/radix/src/semantic/passes/exhaustive.rs`
- `crates/radix/src/semantic/passes/lint.rs`
- `crates/radix/src/semantic/passes/borrow.rs`

The remaining expression-kind matches in lint and borrow are policy hooks that
preserve pass-specific state: loop context, unreachable-code scope handling,
borrow lvalues, call argument modes, references, closure scopes, and
`root_def_id` extraction.
