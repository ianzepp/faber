# Phase 05 Delivery Spec: Semantic Typecheck

## Target

Document the semantic typecheck subsystem as the high-semantic-weight hot path
that assigns, checks, resolves, and finalizes HIR types.

## Files

- `crates/radix/src/semantic/passes/typecheck/mod.rs`
- `crates/radix/src/semantic/passes/typecheck/collect.rs`
- `crates/radix/src/semantic/passes/typecheck/infer.rs`
- `crates/radix/src/semantic/passes/typecheck/lookup.rs`
- `crates/radix/src/semantic/passes/typecheck/ops.rs`
- `crates/radix/src/semantic/passes/typecheck/call.rs`
- `crates/radix/src/semantic/passes/typecheck/item.rs`
- `crates/radix/src/semantic/passes/typecheck/expr.rs`
- `crates/radix/src/semantic/passes/typecheck/stmt.rs`
- `crates/radix/src/semantic/passes/typecheck/control.rs`
- `crates/radix/src/semantic/passes/typecheck/aggregate.rs`
- `crates/radix/src/semantic/passes/typecheck/access.rs`
- `crates/radix/src/semantic/passes/typecheck/convert.rs`
- `crates/radix/src/semantic/passes/typecheck/pattern.rs`
- `crates/radix/src/semantic/passes/typecheck/finalize.rs`

## Acceptance Criteria

- The typecheck subsystem's role in the semantic pipeline is clear from module
  and file headers.
- Type inference, unification, substitution resolution, recovery, and final HIR
  type replacement are documented at the contracts future maintainers need.
- Call, access, aggregate, conversion, pattern, item, statement, and
  control-flow checks describe their dependencies and error behavior without
  restating obvious Rust mechanics.
- Unknown/nullability policy, `ignotum`, `nihil`, optional/union handling, and
  failable call/error-channel behavior are documented where relevant.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Non-typecheck semantic passes; those were Phase 4.
- Test file changes.
- Runtime behavior, diagnostics wording, or compiler logic changes.

## Validation Plan

- `cargo fmt --check`
- `cargo test -p radix`
- `git diff --check`

## Verification

- `cargo fmt --check` passed.
- `cargo test -p radix` passed: 425 passed, 0 failed, 3 ignored; hygiene and
  doctest targets passed.
- `git diff --check` passed.
- Poker-face completion estimate: 93%. Largest residual issue was this
  verification section being stale before final update; no material unsupported
  typecheck documentation claims were found.
