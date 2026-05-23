# Phase 03 Delivery: HIR Model and Lowering

## Scope

Apply the Radix documentation methodology to the Phase 3 HIR files:

- `crates/radix/src/hir/mod.rs`
- `crates/radix/src/hir/nodes.rs`
- `crates/radix/src/hir/visit.rs`
- `crates/radix/src/hir/lower/mod.rs`
- `crates/radix/src/hir/lower/decl.rs`
- `crates/radix/src/hir/lower/expr.rs`
- `crates/radix/src/hir/lower/stmt.rs`
- `crates/radix/src/hir/lower/types.rs`
- `crates/radix/src/hir/lower/pattern.rs`

Test files are excluded.

## Implementation Plan

1. Use bounded agents on disjoint HIR groups:
   - HIR model/traversal: `hir/mod.rs`, `hir/nodes.rs`, `hir/visit.rs`
   - Lowering declarations/statements: `hir/lower/mod.rs`,
     `hir/lower/decl.rs`, `hir/lower/stmt.rs`
   - Lowering expressions/types/patterns: `hir/lower/expr.rs`,
     `hir/lower/types.rs`, `hir/lower/pattern.rs`
2. Review generated documentation from the supervisor context for unsupported
   semantic claims, stale AST/HIR field assumptions, and comments that narrate
   obvious mapping code.
3. Preserve behavior and formatting; this phase is documentation-only.

## Acceptance Criteria

- File headers explain the boundary from syntactic AST to compiler-friendly HIR.
- HIR node, visitor, and lowering docs identify what semantic assumptions begin
  at HIR and what remains unresolved for later passes.
- Public and crate-facing lowering contracts document invariants, error
  behavior, phase context, and recovery/error-node policy where relevant.
- Private helpers stay compact unless they encode lowering policy, AST/HIR
  compatibility, or non-obvious edge cases.
- No test files are modified.
- `cargo fmt --check`, `cargo test -p radix`, and `git diff --check` pass.

## Verification

- `cargo fmt --check` passed.
- `cargo test -p radix` passed: 425 unit tests passed, 3 ignored; binary tests
  passed; 8 hygiene tests passed; doc tests passed with 1 passed and 1 ignored.
- `git diff --check` passed.
- Poker-face completion gate cleared at 91%; the noted gap was sparse
  documentation around private test-item synthesis helpers, now addressed with
  focused comments in `hir/lower/mod.rs`.
