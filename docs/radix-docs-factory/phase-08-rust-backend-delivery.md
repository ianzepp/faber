# Phase 08 Delivery Spec: Rust Backend

## Target

Document the Rust backend's contracts and high-weight expression/type
translation decisions without changing behavior.

## Files

- `crates/radix/src/codegen/rust/mod.rs`
- `crates/radix/src/codegen/rust/cli.rs`
- `crates/radix/src/codegen/rust/decl.rs`
- `crates/radix/src/codegen/rust/stmt.rs`
- `crates/radix/src/codegen/rust/types.rs`
- `crates/radix/src/codegen/rust/failable.rs`
- `crates/radix/src/codegen/rust/expr/mod.rs`
- `crates/radix/src/codegen/rust/expr/access.rs`
- `crates/radix/src/codegen/rust/expr/block.rs`
- `crates/radix/src/codegen/rust/expr/call.rs`
- `crates/radix/src/codegen/rust/expr/collection.rs`
- `crates/radix/src/codegen/rust/expr/control.rs`
- `crates/radix/src/codegen/rust/expr/convert.rs`
- `crates/radix/src/codegen/rust/expr/format.rs`
- `crates/radix/src/codegen/rust/expr/literal.rs`
- `crates/radix/src/codegen/rust/expr/ops.rs`
- `crates/radix/src/codegen/rust/expr/option.rs`
- `crates/radix/src/codegen/rust/expr/pattern.rs`
- `crates/radix/src/codegen/rust/expr/verte.rs`

## Acceptance Criteria

- The Rust backend module explains target orchestration, precomputed name and
  failable-function state, import collection, entry emission, and target-specific
  Result/ownership trade-offs.
- CLI generation docs explain generated parser/support code, single-command vs
  subcommand behavior, exit handling, defaults, and why this code lives in the
  Rust backend.
- Declaration, statement, type, and failable-analysis docs explain failable
  propagation, `Ok` wrapping, reference/ownership mapping, type translation,
  test metadata emission, and unsupported statement surfaces.
- Expression module docs explain propagation context, entry/catch suppression,
  optional/null handling, collection/std-lib translation, conversion/formatting
  policy, pattern emission, and `verte` construction trade-offs.
- Documentation does not claim Rust codegen redoes semantic checking or supports
  HIR surfaces that currently fail closed.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Go, TypeScript, and canonical Faber backends.
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
