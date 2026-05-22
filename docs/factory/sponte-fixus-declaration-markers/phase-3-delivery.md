# Phase 3 Delivery Spec: Semantic & Lowering

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 3 - Semantic & Lowering
**Status**: in_progress
**Created**: 2026-05-21

## Interpreted Phase Problem

Phase 2 delivered a working parser for the new syntax and stored `sponte`/`fixus` on syntax AST nodes (`Param`, `FieldDecl`), plus a `TypeExprKind::Union`. However:

- The semantic layer (HIR, `FuncSig`/`ParamType`, `TypeTable`, lowering, resolve, typecheck) still largely routes through the legacy `nullable: bool` flag and the old "optional" concept.
- A Phase 2 "bridge shim" in the parser mutates `ty.nullable = true` when `sponte` is seen, so that downstream code "just works".
- `HirField` has no `sponte`/`fixus` metadata at all.
- `HirParam` only has the legacy `optional` bool.
- Union lowering is a placeholder that only handles the narrow `T ∪ nihil` case in a few places; the full canonicalization rules from the updated plan (`A ∪ B ∪ nihil` → `Option<Union<A, B>>`, duplicate removal, rejection of `nihil ∪ nihil`) are not implemented.
- Declaration-level obligation (`sponte` = voluntary slot) is not yet semantically distinct from value-domain nullability (`T ∪ nihil`).
- `fixus` (post-initialization immutability intent) is parsed but completely ignored in HIR/semantic.

Phase 3 must make `sponte` and source-level unions produce *correct, canonical* semantic representations so that type checking, lowering to HIR, and later Rust codegen see the intended obligation / domain semantics. `fixus` must be preserved as metadata, but deep fixed-field enforcement is deferred to dedicated `fixus` work. The bridge shims must be removed.

## Normalized Phase Spec

**Inputs**:
- Current post-Phase-2 state (syntax nodes carry `sponte`/`fixus`; `TypeExprKind::Union` exists; parser bridge shim active; HIR nodes lag; union lowering is partial).
- Updated plan rules (Design Rules §119–124, Phase 3 checkpoint, Validation bullets for semantic tests and canonicalization).

**Outputs** (what must be true at end of Phase 3):
- `HirParam` and `HirField` carry `sponte: bool` and `fixus: bool`.
- Lowering (`hir/lower/decl.rs`) populates them from syntax without mutating `TypeExpr.nullable`.
- The Phase 2 parser bridge shims are deleted; `sponte` on a declaration no longer forces `nullable` on its type expression.
- `ParamType.optional` (used by `FuncSig` and call checking) continues to reflect voluntary parameters (sourced cleanly from `sponte`).
- A centralized union canonicalization rule exists and is used by both HIR lowering and semantic resolve lowering:
  - Strip `nihil` members (Primitive::Nihil).
  - Canonicalize duplicates.
  - If remaining members empty → diagnostic error (`nihil ∪ nihil` is invalid).
  - If 1 remaining → `Type::Option(that)`.
  - If >1 → `Type::Option( Type::Union(remaining) )`.
- `T ∪ nihil`, `A ∪ B ∪ nihil`, `T ∪ T ∪ nihil` all produce the documented forms.
- `fixus` is recorded on HIR declarations only. Full "no mutation after init" enforcement is out of scope for this plan and belongs to dedicated fixed-field / late-init work.
- New or adapted semantic tests cover the cases listed in the plan's Validation section (fixus, sponte+default+fixus ordering, union canonicalization, degenerate rejection).
- All existing radix unit / driver / semantic tests that were green after Phase 2 remain green (via clean propagation of the sponte → optional flag).

**Out of scope for Phase 3** (per plan boundaries):
- Full Rust codegen emission for non-null unions (Phase 4).
- TypeScript and Go parity; those codegen surfaces are legacy for this effort and should not drive Phase 3.
- Migration of examples/stdlib (Phase 5).
- New diagnostics wording for "voluntary" vs "nullable" vs "fixed" (Phase 6).
- Guardrail negative tests or residue searches (Phase 7).
- Making `sponte`/`fixus` contextual keywords.
- Changing `de`/`in` position or introducing `?` shorthand.

**Tactical Data Model Choices** (to minimize churn):
- Add `sponte: bool, fixus: bool` to `HirParam` and `HirField`.
- Keep the existing `optional: bool` field on `HirParam` (and on `ParamType`) for this phase; lowering sets `optional = sponte` (the voluntary flag). This avoids touching ~30 test construction sites.
- `HirField` gains the two new bools; struct metadata used by `check_struct_literal` will later be extended (basic required-field checking for non-sponte can be added if time permits, but is not required for the Phase 3 checkpoint).
- Union canonicalization lives in a new helper `TypeTable::intern_union_with_nihil` (or similar) called from the two lowering sites.
- `fixus` is stored but its primary use in Phase 3 is "present in HIR" + any simple default-before-fixed ordering in tests.

## Stage Graph

| Step | Task | Evidence / Files |
|------|------|------------------|
| 1 | Reread plan (updated rules) + phase-2-delivery + key HIR/semantic files | This delivery + plan.md |
| 2 | Evolve HIR nodes (`HirParam`, `HirField`) | `hir/nodes.rs` |
| 3 | Update lowering to populate new fields cleanly; delete parser shims | `hir/lower/decl.rs`, `parser/decl.rs` (remove bridge), `parser/types.rs` |
| 4 | Implement canonical union lowering helper + call it from HIR + semantic resolve | `semantic/types.rs`, `hir/lower/types.rs`, `semantic/passes/resolve.rs` |
| 5 | Propagate `sponte` → `optional` (and new fixus) through collect/resolve/typecheck paths that need it | `semantic/passes/typecheck/collect.rs`, `aggregate.rs`, `call.rs`, etc. |
| 6 | Update / add semantic tests for the exact cases in plan Validation | `semantic/passes/*_test.rs`, new tests in driver or resolve_test |
| 7 | Remove any remaining reliance on the nullable bridge in semantic paths | Audit + targeted replaces |
| 8 | `cargo test -p radix` (focus on semantic + driver + codegen paths); targeted `check` of new syntax | Verification |
| 9 | Finalize this delivery + ledger note | phase-3-delivery.md |

## Risks & Mitigations

- Large number of `HirParam { optional: ... }` and `ParamType { optional: ... }` literals in tests — mitigated by keeping the `optional` field and only adding the new flags.
- Struct literal required-field checking for non-sponte fields may be incomplete — acceptable for Phase 3 checkpoint (focus on lowering + union canon + param sponte).
- `fixus` has no deep enforcement here (reassignment after init) — recorded in HIR is sufficient for this phase goal; deeper analysis belongs to dedicated fixed-field / late-init work, not Phase 4 Rust optionality codegen.
- Interaction with existing `Type::Union` and `Type::Option` in typecheck/assignable — the canonical forms we produce will be `Option<T>` or `Option<Union<...>>`, which the existing logic already understands.

## Checkpoint

When this phase is complete:
- Declaration optionality (`sponte`) and value-domain unions (`T ∪ nihil` and general forms) are correctly represented in HIR and the semantic `Type`/`FuncSig`/`ParamType` model. `fixus` metadata is preserved in HIR, without claiming enforcement.
- The exact canonicalization rules from the plan are implemented and tested.
- The parser no longer mutates `TypeExpr.nullable` as a bridge.
- All validation bullets for Phase 3 in the plan (new semantic tests, canonicalization, etc.) are satisfied.
- `./scripta/test` may still fail on unmigrated stdlib/examples (expected until Phase 5); the radix test suite and any direct new-syntax tests must be clean.

Proceed with implementation.
