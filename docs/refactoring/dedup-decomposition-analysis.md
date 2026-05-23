# Deduplication and Testability Decomposition Analysis

**Date**: May 2026  
**Updated**: 2026-05-23  
**Context**: Refactoring follow-up after HIR/MIR visitor and MIR lower decomposition work  
**Status**: Remaining decomposition opportunities; completed items removed

## Executive Summary

The original swarm analysis correctly identified traversal duplication and MIR
lowering size as major sources of maintenance drag. The highest-leverage items
from that report have since been implemented:

- `crates/radix/src/hir/visit.rs` now provides read-only `HirVisitor` and
  mutable `HirVisitorMut` traversal contracts.
- `crates/radix/src/mir/visit.rs` now provides read-only and fallible MIR
  visitors plus CFG successor helpers.
- MIR lowering has been split across `crates/radix/src/mir/lower.rs` and
  focused modules under `crates/radix/src/mir/lower/`.

The remaining work is narrower: finish applying those traversal and
decomposition patterns to the largest compiler surfaces that still mix dispatch,
policy, formatting, and test setup in large units.

## Current Source of Truth

Implemented traversal infrastructure:

- `crates/radix/src/hir/visit.rs`
- `crates/radix/src/mir/visit.rs`
- Existing precedent: `crates/radix/src/syntax/visit.rs`

Current large files worth tracking:

| File | Current shape | Why it still matters |
| --- | --- | --- |
| `crates/radix/src/codegen/ts/expr.rs` | Large expression emitter | TypeScript expression policy, formatting, and helper detection still sit in one broad module. |
| `crates/radix/src/codegen/faber/expr.rs` | Large expression pretty-printer | Faber round-trip formatting is central, but expression cases are still concentrated in one file. |
| `crates/radix/src/semantic/passes/resolve.rs` | Large resolver pass | Name-resolution policy and recursive control flow remain dense and hard to isolate. |
| `crates/radix/src/mir/lower.rs` | Split driver/context holder | Much smaller than before, but still owns core builder state and may deserve more internal shaping as MIR grows. |

## Remaining Findings

### 1. Visitor Adoption Is Incomplete By Design

The completed HIR visitor work shifts the useful follow-up to two cases:

- Mechanical scans should use `HirVisitor` or `HirVisitorMut` instead of
  hand-rolled recursive walks.
- Policy-heavy passes may still need explicit `match` statements. Those should
  stay local when the match is deciding semantics, formatting, or target policy
  rather than merely traversing children.

Current examples of successful visitor adoption include name collection,
semantic lint/borrow/exhaustive scans, type finalization, Go use/ad detection,
Rust failable dependency analysis, MIR lowering context collection, MIR dump,
MIR validation, and the MIR Rust probe.

The remaining question is not "add a visitor"; it is "when touching a large
pass, separate traversal from policy if traversal is still being duplicated."

### 2. TypeScript And Faber Codegen Remain The Clearest Decomposition Targets

Rust and Go expression codegen already use focused expression submodules. TS and
Faber still keep broad expression dispatch in single files:

- `crates/radix/src/codegen/ts/expr.rs`
- `crates/radix/src/codegen/faber/expr.rs`

The next useful split should follow the existing Rust/Go pattern:

- access and member operations,
- calls and methods,
- control-flow expressions,
- literals and formatting,
- operators,
- option/null operations,
- conversion/ascription forms,
- collection and aggregate construction.

The goal is not cosmetic file count. The goal is to make target policy testable
in focused units without constructing every surrounding codegen concern.

### 3. Resolver Size Is Still A Testability Hotspot

`crates/radix/src/semantic/passes/resolve.rs` remains dense. Some of that density
is legitimate because resolver logic owns lexical scopes, definitions, imports,
patterns, type references, and use-site binding.

Still, future work should look for separable seams around:

- item/type declaration registration,
- alias and type-reference resolution,
- statement/block scope handling,
- pattern binding,
- import/provider binding,
- expression path resolution.

This should be done only when a concrete change needs it. The resolver is
central enough that blind refactoring would be riskier than extracting a tested
unit during related feature work.

### 4. Shared Scope And Diagnostic Helpers May Still Pay Off

The original report noted several independent scope stacks and repeated
diagnostic push patterns. That still appears plausible, but it should be proved
with a focused inventory before introducing a generic abstraction.

Useful acceptance criteria for any shared helper:

- It removes repeated logic from at least two active passes.
- It does not hide phase-specific rules behind vague generic names.
- It keeps diagnostics attached to the right span and phase.
- It has dedicated unit tests.

## Prioritized Recommendations

### Tier 1

1. Split TypeScript expression codegen into focused modules, using the Rust/Go
   expression layouts as precedent.
2. Split Faber expression printing only where the extracted module has a clear
   formatting contract and focused tests.
3. When modifying any recursive analysis, migrate mechanical traversal to
   `HirVisitor`, `HirVisitorMut`, `MirVisitor`, or `FallibleMirVisitor` where
   that reduces duplicated child walking.

### Tier 2

4. Extract resolver subunits opportunistically during resolver feature work.
5. Inventory scope-stack implementations and introduce a shared helper only if
   the duplication is concrete and the resulting API is phase-specific enough.
6. Inventory diagnostic construction boilerplate before adding a helper.

### Tier 3

7. Continue moving large dispatch tests into dedicated `_test.rs` files as
   modules become small enough to test directly.
8. Periodically rerun line-count and visitor-adoption checks after large syntax
   or HIR changes.

## Suggested Implementation Order

1. Start with `codegen/ts/expr.rs`; it is the largest remaining expression
   emitter and has clear precedent from Rust/Go.
2. Apply the same pattern selectively to `codegen/faber/expr.rs`, preserving
   round-trip formatting behavior.
3. During future resolver work, extract one resolver concern at a time with
   dedicated tests.
4. Only after those splits, consider shared scope or diagnostic helpers.

## Notes On Testability

The main testability problem is now less about missing traversal
infrastructure and more about large modules that combine traversal, policy,
formatting, and state setup. The visitor work makes lightweight scans easier,
but target emitters and resolver logic still need decomposition around behavior
boundaries.

Future refactors should keep the repository convention of dedicated `_test.rs`
files for focused behavior where practical.

## References

Original raw subagent outputs are available via the session transcript under
these IDs:

- HIR/MIR Lowering: `019e540e-5797-7a70-ac08-5ff4a99e2bf3`
- Semantic Passes & Cross-Cutting: `019e540e-6ce0-73b1-8165-d901f2de7fb2`
- Codegen Backends: `019e540e-4d2b-7843-99f9-9aeeeb6e2b64`
- Long Functions & Testability: `019e540e-793f-7af2-a13b-1cfafe1ade95`

Key current files:

- `crates/radix/src/hir/visit.rs`
- `crates/radix/src/mir/visit.rs`
- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/lower/*.rs`
- `crates/radix/src/codegen/{rust,ts,go,faber}/**/*.rs`
- `crates/radix/src/semantic/passes/resolve.rs`

---

*This document now tracks remaining decomposition opportunities after the
visitor and MIR-lowering work, not the already-completed initial refactor
recommendations.*
