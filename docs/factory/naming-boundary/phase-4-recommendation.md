# Phase 4: Declaration Taxonomy Recommendation

**Status**: decision complete — defer implementation  
**Date**: 2026-06-04

## Question

Should syntax AST declaration variants use Faber taxonomy (`GenusDecl`,
`PactumDecl`, `OrdoDecl`, `DiscretioDecl`) or compiler taxonomy (`ClassDecl`,
`InterfaceDecl`, `EnumDecl`, `UnionDecl`)?

## Recommendation

**Defer renaming.** Keep `ClassDecl`, `InterfaceDecl`, `EnumDecl`, and
`UnionDecl` for this repository cycle.

## Reasons

1. **Churn vs payoff**: Renaming touches parser dispatch, collector, HIR lower,
   explain corpus, exempla, and hundreds of tests. Mode-level boundary work
   already removed the highest-confidence inconsistencies.

2. **StmtKind is already Latin**: `StmtKind::Class` wraps `ClassDecl`; the
   statement taxonomy matches source keywords while struct names read as
   semantic categories. That split is intentional, not accidental.

3. **HIR/semantic layers use English product types**: `HirStruct`, `Type::Struct`
   model compiler concepts after lowering. Forcing `GenusDecl` in syntax without
   matching HIR would recreate a two-name system with little bug-fix value.

4. **Risk**: Large mechanical renames obscure real regressions in a pass whose
   goal is naming clarity only.

## If revisited later

Split into separate delivery specs per family:

- `genus` / `ClassDecl` → only if genus-specific bugs or explain drift appear
- `pactum` / `InterfaceDecl`
- `ordo` / `EnumDecl`
- `discretio` / `UnionDecl`

Each spec should include parser, AST, collector, HIR lower, and round-trip tests.

## Completion

The naming-boundary factory pass can stop here: boundary documented, mode and
visibility enums aligned, declaration taxonomy explicitly deferred with rationale.