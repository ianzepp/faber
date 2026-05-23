# Phase 10 Delivery Spec: Faber Canonical Backend

## Target

Document the canonical Faber backend's source-preserving purpose, grammar-valid
output policy, and known preservation limits without changing behavior.

## Files

- `crates/radix/src/codegen/faber/mod.rs`
- `crates/radix/src/codegen/faber/names.rs`
- `crates/radix/src/codegen/faber/ops.rs`
- `crates/radix/src/codegen/faber/decl.rs`
- `crates/radix/src/codegen/faber/expr.rs`
- `crates/radix/src/codegen/faber/stmt.rs`
- `crates/radix/src/codegen/faber/types.rs`
- `crates/radix/src/codegen/faber/literal.rs`
- `crates/radix/src/codegen/faber/pattern.rs`

## Acceptance Criteria

- The Faber backend module explains why canonical Faber generation exists, how
  it differs from target codegen, HIR error rejection, and what source details
  cannot be preserved after lowering.
- Name and operator docs explain `DefId` recovery, fallback naming, precedence,
  associativity/parentheses policy, and canonical operator spelling.
- Declaration and statement docs explain grammar-valid output for functions,
  tests, imports, structs, enums, interfaces, aliases, constants, entry blocks,
  `si/sin/secus`, `fac/dum`, `discerne`, `ad`, and block/tail handling.
- Expression, type, literal, and pattern docs explain canonical spelling,
  nullable/type syntax, object fields, string/regex quoting limits, match arms,
  pattern aliases, and unsupported or lossy surfaces.
- Documentation does not claim byte-for-byte source preservation or preservation
  of comments/original formatting that HIR no longer contains.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Rust, TypeScript, and Go target backends.
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
- Poker-face completion check: 95/100, no blocking gaps.
