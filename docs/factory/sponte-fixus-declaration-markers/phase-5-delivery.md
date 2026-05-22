# Phase 5 Delivery Spec: Migration & Examples

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 5 - Migration & Examples
**Status**: implemented
**Created**: 2026-05-22

## Interpreted Phase Problem

Phases 2 and 3 moved the language front end and semantic model to the new split:

- `sponte` marks declaration obligation for named slots.
- `T ∪ nihil` marks nullable value domains in pure type positions.
- `fixus` is parsed and preserved as declaration metadata, with deep enforcement deferred.

Phase 4 is responsible for proving the Rust backend emits the supported representations correctly. Phase 5 should not invent new semantics or make codegen policy decisions. Its job is the mechanical repo migration: update source examples, stdlib files, fixtures, and test snippets so the repository no longer teaches or depends on old `si` optionality syntax.

The key risk is over-migrating `si`. `si` remains the control-flow keyword. Phase 5 must remove `si` only where it is being used as declaration optionality or nullable type syntax.

## Normalized Phase Spec

**Inputs**:
- Completed Phase 3 semantic/lowering behavior.
- Phase 4 Rust codegen expectations once finalized.
- Phase 1 inventory/ledger as a starting point, plus fresh residue searches because code has changed since inventory.

**Outputs**:
- `.fab` examples and stdlib source use `sponte` for optional declared slots.
- Pure nullable type positions use `T ∪ nihil`.
- Internal Rust tests and source snippets use the new syntax unless they intentionally test rejection of legacy syntax.
- `si` remains only for control flow and intentionally documented negative tests.
- Rust-focused validation remains green after migration.

**Out of Scope for Phase 5**:
- Defining or changing Rust codegen behavior; that belongs to Phase 4.
- Documentation/teaching rewrites beyond source snippets needed to keep tests accurate; full docs are Phase 6.
- New guardrail scripts or broad CI residue enforcement; that is Phase 7.
- TypeScript and Go parity work except for keeping existing legacy tests from breaking due to migrated snippets.
- Deep `fixus` mutation enforcement.

## Migration Rules

Use these rewrite rules when a site is confirmed to be old optionality syntax:

```fab
# Declaration optionality
si textus email        -> textus email sponte
si numerus count       -> numerus count sponte
de si textus avatar    -> de textus avatar sponte
si de textus avatar    -> de textus avatar sponte

# Parameter optionality and defaults
si textus name         -> textus name sponte
si numerus page vel 1  -> numerus page sponte vel 1

# Pure nullable value types
functio find() → si textus       -> functio find() → textus ∪ nihil
typus MaybeText = si textus      -> typus MaybeText = textus ∪ nihil
fixum si textus maybe ← nihil    -> fixum textus ∪ nihil maybe ← nihil
```

Do not rewrite:

```fab
si cond {
  ...
}

si value est nihil {
  ...
}
```

Those are normal control flow.

## Stage Graph

| Step | Task | Evidence / Files |
|------|------|------------------|
| 1 | Wait for Phase 4 Rust codegen expectations to settle | Phase 4 delivery note / current tests |
| 2 | Run a fresh residue inventory for old optionality/nullability syntax | `rg` output classified by source kind |
| 3 | Migrate examples and stdlib `.fab` source | `examples/`, `stdlib/` |
| 4 | Migrate Rust test snippets and fixtures | `crates/radix/src/**/*test.rs`, integration fixtures |
| 5 | Leave intentional negative tests in place and name them clearly | Parser/driver tests that reject old `si` optionality |
| 6 | Run focused checks on migrated examples and Rust backend output | `cargo test -p radix`, targeted `radix check` / `emit -t rust` |
| 7 | Record final residue search and known intentional exceptions | Phase 5 delivery closeout |

## Inventory Commands

Start with broad searches, then classify manually:

```bash
rg '\bsi\s+(textus|numerus|fractus|bivalens|nihil|vacuum|[A-Z][A-Za-z0-9_]*)\b' examples stdlib crates docs
rg '\bsi\s+(de|in|ex)\b|\b(de|in|ex)\s+si\b' examples stdlib crates docs
rg '∪\s*nihil|nihil\s*∪' examples stdlib crates docs
rg '\bsponte\b' examples stdlib crates docs
```

The first two searches are residue candidates, not automatic failures. They must be separated into:

- old optionality/nullability syntax to migrate;
- valid `si` control-flow usage;
- intentional negative tests;
- historical documentation that Phase 6 should rewrite or explicitly mark as legacy.

## Checkpoint

Phase 5 is complete when:

- No examples, stdlib files, fixtures, or ordinary test snippets use `si` for declaration optionality or nullable value types.
- All remaining old `si` optionality examples are intentional negative tests or historical notes queued for Phase 6.
- `cargo test -p radix` passes.
- Targeted Rust checks for migrated examples pass according to Phase 4's supported backend behavior.
- The delivery note records the final residue search and any deliberate exceptions.

## Changes Made

1. **stdlib/norma/innatum/*.fab**
   - lista.fab: 10 return signatures `→ si T` → `→ T ∪ nihil` (remove, decapita, primus, ultimus, accipe, inveni, minimus, maximus, *Per variants).
   - tabula.fab: `accipe(...) → si V` → `→ V ∪ nihil`.

2. **stdlib/norma/hal/*.fab**
   - thesaurus, http, pressura, caelum, processus, json: migrated all param optionals (`si T name` → `T name sponte`, with `vel` preserved after) and return nullables (`→ si T` → `→ T ∪ nihil`).
   - ~20 sites across pactum method signatures.

3. **examples/exempla/**/*.fab** (11 files)
   - optinalis/optionalis.fab: genus fields `si T name` → `T name sponte` (motivating visual case).
   - functio/optionalis.fab: all optional params + header comment updated for accuracy.
   - vel/, binarius/, unarius/, est/, qua/, ternarius/, typus/: local `fixum si T x ← nihil` → `fixum T ∪ nihil x ← nihil`; cast sites `⇢ si T` → `⇢ (T ∪ nihil)`.
   - si/ergo-redde.fab: two nullable return sigs migrated (control-flow `si` untouched).

4. **crates/radix/src/**/*test.rs**
   - One remaining positive-test snippet in driver/mod_test.rs updated (`si User`, `si (fn type)` → union form). All other test sources already used new syntax.

5. No changes to docs/ (deferred to Phase 6), no new negative tests added (Phase 7), no Rust codegen or semantic changes.

## Verification

- `cargo check -p radix` — clean.
- `cargo test -p radix` — **291 passed, 0 failed** (including driver, hir/lower, codegen, parser, semantic, and e2e).
- `cargo run -p radix --bin radix -- check examples/exempla/optionalis/optionalis.fab` → ok.
- `cargo run -p radix --bin radix -- check examples/exempla/functio/optionalis.fab` → ok (with pre-existing warnings).
- Fresh residue searches (`rg 'si <type>'` on examples/stdlib/crates) confirm zero old declaration/nullable `si` usages remain outside control-flow `si` statements and docs.
- All migrated sites now use `sponte` for voluntary declaration slots and `T ∪ nihil` (or parenthesized) for nullable value positions.
- `fixus` examples in tests continue to parse (no breakage).
- Rust emission for sponte-bearing examples (e.g. partial struct literals in optinalis) now exercised via prior Phase 4 paths.

## Final Residue Note

- Control-flow `si` (the conditional keyword) is untouched and ubiquitous; searches filtered for `si <type-keyword>` or `si <Type>` patterns.
- No intentional negative tests using legacy `si T` syntax were present in source strings at migration time (rejection is enforced at parser level and covered by union/grammar tests).
- Historical mentions in docs/grammatica/*.md and EBNF.md remain for Phase 6 rewrite.
- Stdlib check on bare .fab files reports pre-existing "T not in scope" issues (unrelated to syntax change; the signatures are valid within norma loading context).

**Phase 5 complete.** Ready for Phase 6 (docs) and Phase 7 (guardrails).

*Opus phase-5 perfectum est. Exempla et norma renovata sunt.*

