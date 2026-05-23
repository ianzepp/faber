# Phase 04 Delivery Spec: Semantic Core and Non-Typecheck Passes

## Target

Document semantic pipeline orchestration, shared semantic state, diagnostics, and
the non-typecheck semantic passes in `crates/radix`.

## Files

- `crates/radix/src/semantic/mod.rs`
- `crates/radix/src/semantic/error.rs`
- `crates/radix/src/semantic/scope.rs`
- `crates/radix/src/semantic/types.rs`
- `crates/radix/src/semantic/passes/mod.rs`
- `crates/radix/src/semantic/passes/collect.rs`
- `crates/radix/src/semantic/passes/resolve.rs`
- `crates/radix/src/semantic/passes/borrow.rs`
- `crates/radix/src/semantic/passes/exhaustive.rs`
- `crates/radix/src/semantic/passes/lint.rs`

## Acceptance Criteria

- Pass ordering and phase boundaries are clear, including the AST collection and
  resolution passes, HIR lowering boundary, typecheck handoff, and later
  analysis passes.
- Error and warning behavior is documented without implying that all diagnostics
  are fatal or that warnings block code generation.
- The type table's role is clear as an interning and assignability policy
  utility, including `ignotum`, unions, options, and primitive widening.
- Scope and resolver invariants are documented at the symbol table boundary.
- Borrow, exhaustiveness, and lint documentation states when the passes run and
  what kind of diagnostics they produce.
- Documentation is behavior-preserving and excludes test files.

## Out Of Scope

- Semantic typecheck subsystem internals; those are Phase 5.
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
- Poker-face completion estimate: 94%. Largest residual issue was this
  verification section being stale before final update; no material unsupported
  semantic documentation claims were found.
