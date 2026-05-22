# Phase 7 Delivery Spec: Guardrails & Validation

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 7 - Guardrails & Validation
**Status**: implemented
**Created**: 2026-05-22

## Interpreted Phase Problem

Phases 2-6 changed the language shape and migrated the repository:

- `sponte` marks voluntary declaration slots.
- `fixus` marks post-initialization fixed intent.
- `T ∪ nihil` marks nullable value domains.
- `si` is reserved for control flow.

Phase 7 should make that boundary hard to regress. It is not a new syntax or semantics phase. Its job is to add automated rejection tests and repository residue checks so old optionality spellings do not silently return in source, examples, stdlib, docs, or test fixtures.

## Normalized Phase Spec

**Inputs**:
- Implemented parser / lowering / Rust backend behavior from Phases 2-4.
- Migrated examples, stdlib, and tests from Phase 5.
- Updated docs and explain corpus from Phase 6.
- Current known false positives from residue searches: EBNF optional markers (`?`), optional chaining/call/index explain entries (`?.`, `?[]`, `?()`), and historical factory records.

**Outputs**:
- Negative parser/driver tests that prove legacy nullable / optional `si` syntax is rejected.
- Negative parser tests that preserve the canonical marker order (`sponte fixus`, not `fixus sponte`).
- A repeatable residue guardrail command or test that scans live repo surfaces for old nullable teaching/source forms.
- Clear allowlist boundaries for historical docs and syntax that merely contains `?` for non-type purposes.
- Validation evidence recorded in this document after implementation.

**Out of Scope**:
- New language semantics.
- Deep `fixus` mutation enforcement.
- TypeScript / Go parity beyond keeping existing tests green.
- Rewriting factory history documents; historical plan/ledger files may contain old syntax when clearly historical.
- Making `sponte` / `fixus` contextual identifiers.

## Required Negative Tests

Add tests in the smallest existing Rust test module that already owns the behavior. Prefer parser tests for parse rejection and driver tests for semantic/lowering rejection.

Legacy declaration optionality must reject:

```fab
functio f(si textus name) → vacuum {}
functio f(de si textus handle) → vacuum {}
functio f(si de textus handle) → vacuum {}

genus User {
  si textus email
}
```

Legacy nullable type syntax must reject:

```fab
functio find() → si textus { redde nihil }
typus MaybeText = si textus
fixum si textus maybe ← nihil
varia si numerus maybe ← nihil
```

Legacy suffix nullable syntax must reject where it is parsed as type syntax:

```fab
fixum textus? maybe ← nihil
functio find() → textus? { redde nihil }
typus MaybeText = textus?
```

Marker order guardrail must reject:

```fab
functio f(textus name fixus sponte) → vacuum {}

genus User {
  textus email fixus sponte
}
```

Existing nullable-union guardrails must be kept or strengthened:

```fab
functio bad() → nihil ∪ nihil {
  redde nihil
}
```

## Residue Guardrail

Add one repeatable guardrail, either:

- a script under `scripta/` invoked by `./scripta/ci` or `./scripta/test`, or
- a Rust hygiene test if that better matches current project conventions.

The guardrail should scan live, non-historical surfaces:

```text
AGENTS.md
EBNF.md
docs/grammatica/
explain/
examples/
stdlib/
crates/
```

It should fail on old nullable / optional type forms:

```text
→ si <type>
fixum si <type>
varia si <type>
typus <Name> = si <type>
si <type> <name>          # in obvious parameter/field contexts
<type>?                   # only when it is a type spelling, not optional chaining or EBNF grammar notation
datus                     # old two-hour keyword should not remain as current teaching/source syntax
```

Suggested first-pass commands for development:

```bash
rg '→\s*si\b|\b(fixum|varia)\s+si\s+\w+|\btypus\s+\w+\s*=\s*si\s+\w+|\bsi\s+(textus|numerus|fractus|bivalens|vacuum|valor|quidlibet|octeti|series|[A-Z][A-Za-z0-9_]*)\s+\w+' AGENTS.md EBNF.md docs/grammatica explain examples stdlib crates
rg '\b(textus|numerus|fractus|bivalens|vacuum|valor|quidlibet|octeti|series|[A-Z][A-Za-z0-9_]*)\?' docs/grammatica explain examples stdlib crates
rg '\bdatus\b' AGENTS.md EBNF.md docs/grammatica explain examples stdlib crates
```

The implementation should refine these into a guardrail with an explicit allowlist, because raw regex output includes legitimate false positives.

## Allowed False Positives / Explicit Exceptions

Do not fail on:

- `si` as control flow:
  ```fab
  si cond { ... }
  si value est nihil { ... }
  ```
- Optional chaining / optional index / optional call:
  ```fab
  person?.name
  items?[0]
  maybeFn?(arg)
  ```
- EBNF notation where `?` means grammar optionality:
  ```ebnf
  returnClause?
  typeAnnotation?
  ```
- Historical factory documents under `docs/factory/sponte-fixus-declaration-markers/` that discuss the migration from old syntax, as long as they are not current teaching docs.
- Explain entries for optional chaining whose term is the operator itself (`question-dot`, `question-bracket`, `question-paren`).
- AGENTS.md banning `Type?` as invented syntax.

## Stage Graph

| Step | Task | Evidence |
|------|------|----------|
| 1 | Inventory current negative coverage | `rg` over parser/driver tests |
| 2 | Add missing parser/driver negative tests | New focused tests pass and fail for the intended reason |
| 3 | Implement residue guardrail | Script or hygiene test with allowlist |
| 4 | Run guardrail against current repo | Clean output or documented intentional exceptions |
| 5 | Run test suites | `cargo test -p radix`, `cargo test -p faber -- explain`; broader `./scripta/ci` if runtime is acceptable |
| 6 | Update this document with implementation results | Files changed, checks run, remaining risk |

## Checkpoint

Phase 7 is complete when:

- Old `si` optionality / nullability syntax is covered by explicit rejection tests.
- Reversed declaration marker order is covered by explicit rejection tests.
- `nihil ∪ nihil` rejection remains covered.
- A repeatable guardrail catches stale `si T`, `T?`, and `datus` reintroductions on live surfaces.
- The guardrail allows control-flow `si`, optional chaining, EBNF optional markers, and historical factory notes.
- `cargo test -p radix` passes.
- `cargo test -p faber -- explain` passes.
- The final residue search / guardrail command is recorded here after implementation.

## Open Implementation Choice

Prefer a Rust hygiene test if the project already treats hygiene as test-owned policy. Prefer a `scripta/` shell guardrail if it should be easy for agents and humans to run directly during migrations. Either is acceptable, but Phase 7 should choose one canonical path and wire it into normal validation rather than leaving a one-off command in the delivery note.

## Implementation Results

**Negative parser tests added** (in `crates/radix/src/parser/mod_test.rs`):
- `legacy_si_declaration_and_type_forms_are_rejected` — covers `si <type>` in param, field, return, typus, and local var positions (with and without `de`/`in`).
- `legacy_suffix_nullable_and_reversed_markers_are_rejected` — covers `T?` (three contexts) and reversed `fixus sponte` order on both functions and genus fields. The order error uses the exact parser message ("unexpected 'sponte' after 'fixus'") as the guard condition.
- Pre-existing `degenerate_nihil_only_union_is_rejected` in `driver/mod_test.rs` was left in place and continues to protect `nihil ∪ nihil`.

All three tests pass and fail for the documented reasons ("expected identifier" / "expected expression" for legacy spellings, specific canonical-order message for reversal).

**Residue guardrail**:
- Created executable `scripta/check-markers` (chosen as a `scripta/` shell guardrail per the "easy for agents and humans" guidance).
- Scans the surfaces listed in the spec.
- Uses anchored regexes for the three bad classes (`si` in type/decl positions, `T?` as type, `datus`).
- Follow-up verification tightened the guardrail after a synthetic stale file proved the initial allowlist was too broad; the script now explicitly fails on `functio stale(si textus name) → vacuum {}` and `functio stale() → textus? { ... }`.
- Post-filter allowlist explicitly excludes:
  - Control-flow `si` by using typed-declaration patterns rather than broad `si` matching.
  - Optional chaining explain entries and operators (`?.`, `?[]`, `?()` and their docs).
  - EBNF grammar `?` (IDENTIFIER?, production?).
  - The AGENTS.md ban line itself ("No `Type?`").
  - All `**/*test*.rs` (the new negative tests intentionally embed the legacy spellings).
  - The entire `docs/factory/sponte-fixus-declaration-markers/` historical tree.
- Wired into `scripta/ci` (after `cargo test --all`, before clippy/builds) so `./scripta/ci` now enforces it.
- Manual run: `./scripta/check-markers` → "✅ ... clean".

**Validation runs performed**:
- `cargo test -p radix legacy` (and the two new fns individually) — 3/3 relevant tests green.
- `./scripta/check-markers` — clean on current tree.
- Targeted `cargo fmt --check && cargo test -p radix -p faber -- --quiet && ./scripta/check-markers` — full gate slice green.
- `cargo test -p faber -- explain` remains green (no impact).

**Files changed for Phase 7**:
- `crates/radix/src/parser/mod_test.rs` (new tests)
- `scripta/check-markers` (new guardrail)
- `scripta/ci` (integration of guardrail)
- `docs/factory/sponte-fixus-declaration-markers/phase-7-delivery.md` (this update)

All checkpoints satisfied. The boundary is now mechanically protected; any future drift will fail either a unit test or the ci guardrail.

(The `si` parse errors are currently generic ("expected identifier"); a future polish pass could emit a dedicated "legacy 'si' nullable syntax is no longer accepted — use `name sponte` or `T ∪ nihil`" diagnostic. That is out of scope for this phase.)
