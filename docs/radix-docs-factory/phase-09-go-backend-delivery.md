# Phase 09 Delivery Spec: Go Backend

## Target

Document the Go backend's contracts, target compromises, and fail-closed
surfaces without changing behavior.

## Files

- `crates/radix/src/codegen/go/mod.rs`
- `crates/radix/src/codegen/go/decl.rs`
- `crates/radix/src/codegen/go/stmt.rs`
- `crates/radix/src/codegen/go/types.rs`
- `crates/radix/src/codegen/go/expr/mod.rs`
- `crates/radix/src/codegen/go/expr/access.rs`
- `crates/radix/src/codegen/go/expr/call.rs`
- `crates/radix/src/codegen/go/expr/collection.rs`
- `crates/radix/src/codegen/go/expr/control.rs`
- `crates/radix/src/codegen/go/expr/convert.rs`
- `crates/radix/src/codegen/go/expr/literal.rs`
- `crates/radix/src/codegen/go/expr/ops.rs`
- `crates/radix/src/codegen/go/expr/option.rs`
- `crates/radix/src/codegen/go/expr/variants.rs`

## Acceptance Criteria

- The Go backend module explains target orchestration, name/use-count metadata,
  variant/struct field catalogs, package/import emission, entry behavior, and
  Go-specific trade-offs.
- Declaration, statement, and type docs explain exported field naming, receiver
  method emission, interface/variant representation, pointer optionals, map/set
  choices, and no-op borrow-mode lowering.
- Expression docs explain Go statement-expression gaps, function-wrapper
  lowering, recover-based `tempta`, optional/null handling, collection/std-lib
  translations, conversion/format/literal/operator policy, and variant
  constructors.
- Documentation clearly states target compromises such as lack of Rust-style
  borrow semantics, pointer optionals, interface-based variants, and unsupported
  HIR surfaces that fail closed.
- Documentation does not claim Go codegen redoes semantic checking or implements
  stronger semantics than the generated Go actually enforces.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Rust, TypeScript, and canonical Faber backends.
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
