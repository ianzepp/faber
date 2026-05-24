# Epic 2 Post-Cleanup Phase 8 Delivery

## Interpreted Problem

The cleanup audit still calls for helper-level coverage around centralized optional-emission decisions. The generated Rust tests prove behavior, but the core predicate deciding whether an expression already produces an optional value is still only indirectly covered.

## Normalized Spec

- Add a focused Rust backend helper test for `expr_may_already_produce_option`.
- Keep the test in an existing optional/nullability companion module.
- Cover nil literals, optional chains, typed optional calls, typed non-optional calls, nullable `verte`, and plain literals.
- Avoid production behavior changes.

## Repo-Aware Baseline

- Optional target emission is centralized through `generate_expr_as_optional_target`.
- The predicate under test lives in `crates/radix/src/codegen/rust/expr/mod.rs`.
- `crates/radix/src/codegen/rust/tests/optional_test.rs` already owns optional/nullability backend behavior.

## Stage Graph

1. Add the helper-level predicate test.
2. Update the audit progress ledger.
3. Run focused optional-helper tests, the full radix suite, lint, and diff checks.
4. Commit the phase if verification passes.

## Checkpoints

- No generated Rust snapshots are required for this helper test.
- The centralized optional predicate has direct coverage for its most important shape decisions.

## Gate Plan

- `cargo test -p radix expr_may_already_produce_option_classifies_wrapping_inputs -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`
- `git diff --check`
