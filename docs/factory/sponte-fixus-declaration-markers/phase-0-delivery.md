# Phase 0 Delivery Spec: Design & Planning Confirmation

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 0 - Design & Planning
**Status**: approved
**Created**: 2026-05-21
**Approved By**: user request to implement phases 0-2 (implicit plan acceptance)

## Interpreted Phase Problem

The language change proposal has been written as plan.md. Phase 0 requires explicit confirmation that:
- The interpreted problem statement is accurate.
- The rationale and chosen design (post-name `sponte`/`fixus`, `T ∪ nihil` for value nullability) is locked.
- Scope boundaries (in/out) are accepted for the full effort and for the first three phases.
- Word choice (`sponte`, `fixus`) is final; no renaming expected.
- Open questions that affect Phase 2 implementation are noted with decisions.

No code changes; this phase produces the delivery artifact and (if needed) refinements to plan.

## Normalized Phase Spec

Inputs:
- `docs/factory/sponte-fixus-declaration-markers/plan.md`
- Existing grammar in EBNF.md, parser/types.rs, parser/decl.rs, syntax/ast.rs, lexer/token.rs + scan.rs
- Evidence of current `si` usage from quick inventory pass

Outputs:
- `phase-0-delivery.md` (this file) recording confirmation and Phase 2 tactical decisions
- Any minor plan clarifications (if any)

Decisions recorded here become constraints for Phase 1 (inventory) and Phase 2 (grammar/front-end).

## Confirmation of Core Design

1. **Problem Diagnosis**: Valid. `si` overload (control-flow vs modifier) + visual misalignment in `genus` blocks + ordering ambiguity with `de`/`in` are real UX and diagnostic liabilities. Replacing declaration optionality with dedicated `sponte` (Latin "voluntarily") and introducing `fixus` for post-init immutability cleanly separates concerns.

2. **Syntax Split**:
   - Declaration slots (params, genus fields, similar): `<type> <name> [sponte] [fixus] [default]`
   - Genus field defaults use `:`; parameter defaults use `vel`.
   - Pure type positions (returns, typus aliases, var type annos, casts, expr positions): `T ∪ nihil` (and later general unions)
   - Ownership prefixes (`de`, `in`) remain prefix on type; unchanged.

3. **Word Choice**: `sponte` and `fixus` approved. Consistent with Latin naming (cf. `fixum`/`varia`, `vel`, `ergo`). `sponte` signals "voluntary provision", `fixus` signals "fixed after first assignment".

4. **Migration Boundary**: `si` token remains for `si`/`sin`/`secus` control flow. It is simply no longer consumed by type or param/field parsers for optionality. Old `si T` forms become parse errors (good; negative tests will protect).

## Scope Confirmation for Phases 0-2

**In Scope (0-2)**:
- Full classified inventory of `si`-as-nullable/optional sites (Phase 1 ledger).
- Lexer: `Cup` token for ∪, `Sponte`, `Fixus` keyword tokens + entries in keyword table.
- AST: Extend `Param` (repurpose `optional` → `sponte`), `FieldDecl` (add `sponte`, `fixus`), `TypeExpr` (add `Union` variant; nullable flag will be removed or always-false after parse changes).
- Parser: 
  - `parse_type()`: remove `Si` eat, add support for `type ∪ type` (general enough for `T ∪ nihil`, `A ∪ B ∪ C`).
  - `parse_param_list()`: remove `Si` eat for optional; after name parse `sponte`? `fixus`?, keep `vel` default.
  - `parse_class_member()` (fields): after name, parse optional `sponte`/`fixus`, and retain `:` for field initialization/defaults.
  - Update `try_parse_type`, grammar docs in comments, `is_simple_var_decl` if needed, any other decl sites.
- Var decls: `fixum T name sponte?` or more commonly `fixum (T ∪ nihil) name` for nullable locals; `sponte` on pure var decls is low-value so parser will accept it syntactically if present after name (harmless).
- Ensure `si` as statement starter continues to work (parse_statement still sees TokenKind::Si).
- No semantic, lowering, or codegen changes in Phase 2.

**Out of Scope (for 0-2)**:
- Full migration of all sources (Phase 5).
- Semantic lowering of new forms (Phase 3).
- Codegen for unions (Phase 4).
- Docs, EBNF, explain updates (Phase 6).
- Guardrail tests (Phase 7).
- Making `sponte`/`fixus` purely contextual (we will add as keywords; contextual relaxation left for later or guardrails if user demand appears).
- Changes to `de`/`in` position.
- General union semantics beyond parse (T ∪ nihil may be treated as a union node; lowering decides Option<T> canonicalization).

## Phase 2 Tactical Decisions

- **Token names**: `Sponte`, `Fixus`, `Cup` (for ∪). Add to `TokenKind` near other modifiers (after `Omnia` or in a "Declaration Markers" section).
- **Keyword registration**: Add `"sponte" => TokenKind::Sponte`, `"fixus" => TokenKind::Fixus` in `scan.rs::keyword_or_ident`. They will be reserved identifiers everywhere. (If later we want `let sponte = 1;` we can relax via contextual registry; current precedent for `fixum` etc. is full reservation.)
- **Union parsing**: Extend `parse_type` after the named/func/array base to optionally consume `Cup` and additional types, building `TypeExprKind::Union(vec![left, right, ...])`. Left-associative or flat list. `nihil` remains a valid type name in unions.
- **Nullable flag**: We will remove `nullable: bool` from `TypeExpr` in Phase 2 (or set it to false always). Union syntax replaces it. Downstream code that reads `.nullable` will be updated only where it affects Phase 2 parse paths; full audit in Phase 3.
- **Field init syntax**: Keep `:` in genus fields (`textus email sponte : "foo"`). Parser accepts `sponte`/`fixus` after name, before optional init. Parameter defaults continue to use `vel`.
- **Error messages**: When `si` appears before a type in a decl context, produce a clear diagnostic suggesting `sponte` after name or `∪ nihil` for the type. (Minimal for Phase 2: the eat will simply fail to consume, leading to "expected type" or similar; improved messages are nice-to-have.)
- **Test surface**: Parser tests in `parser/mod_test.rs` and `driver/mod_test.rs` will be updated in Phase 2 only for the new positive forms; bulk replacement of old `si` forms deferred to Phase 5 to keep Phase 2 focused on grammar.

## Open Questions (from plan) — Phase 2 Impact

- General vs narrow union: We implement general `T1 ∪ T2 ∪ ...` (flat Vec in AST) for simplicity and future-proofing. Narrow `T ∪ nihil` is just a usage pattern.
- Canonicalization: Left to Phase 3 semantic.
- Diagnostics distinction: "voluntary slot" vs "nullable value" vs "fixed lifecycle" — message wording in Phase 6+.
- Reversed order `fixus sponte`: For Phase 2 parser we accept only canonical `<name> [sponte] [fixus]` and reject reversed with a specific error (preserves one style).
- Symbolic `?` form: Explicitly deferred (no `textus name?` yet).

## Stage Graph for Phase 0

| Step | Task | Evidence |
| ---- | ---- | -------- |
| 1 | Read and internalize full plan.md | plan.md sections 1-10 |
| 2 | Quick scan of current `si` sites and AST/parser surfaces | grep + read of parser/types.rs, decl.rs, ast.rs, examples, stdlib |
| 3 | Confirm word choice and scope with user intent (via request to proceed) | This delivery |
| 4 | Record tactical choices for Phase 2 | This delivery |
| 5 | Write delivery artifact | phase-0-delivery.md |

## Checkpoint And Gate

Checkpoint for Phase 0:
- Plan scope and words confirmed.
- Tactical decisions for grammar work documented.
- No behavior change yet.

Verification:
- `git status` shows clean tree except new delivery file.
- User proceeds to request Phase 1/2 work.

## Completion Notes

Phase 0 complete. Proceeding to Phase 1 (inventory ledger) and then Phase 2 (implementation) as a single bounded effort limited to front-end parse changes only. All later phases (semantic, migration, docs) explicitly excluded per user instruction "phases 0-2 only".

Autocommit will be performed after Phase 2 verification per AGENTS.md guidance.
