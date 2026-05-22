# Phase 6 Delivery Spec: Documentation & Teaching

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 6 - Documentation & Teaching
**Status**: implemented
**Created**: 2026-05-22

## Interpreted Phase Problem

After migration, the language implementation and examples should use the new split:

- `sponte` for voluntary declaration slots.
- `fixus` for recorded post-initialization fixed intent.
- `T ∪ nihil` for nullable value domains in pure type positions.

Phase 6 teaches that shape consistently. It should remove stale docs that describe `si T`, `textus?`, or other historical nullable forms as current syntax, and it should add missing explain-corpus coverage for the new inline union glyph.

## Semantics To Teach

Keep three ideas separate:

```fab
textus email sponte
```

- `sponte` marks declaration obligation: the caller may omit the slot.
- When a `sponte` field has no default, omission is represented as absence (`nihil` / Rust `None`).
- Current Rust codegen stores `sponte` genus fields as `Option<T>`.

```fab
textus nickname sponte : "Anonymous"
```

- The slot may be omitted, but omission uses the field default.
- Current Rust codegen stores this as `Option<String>` and emits `Some("Anonymous".to_string())` for omitted construction.
- Do not teach this as a nullable value-domain declaration; it is voluntary provision plus a default.

```fab
textus ∪ nihil email : "Anonymous"
```

- The value domain remains nullable; the default only supplies the omitted construction value.
- Omitted construction emits the default (`Some("Anonymous".to_string())` in Rust).
- Explicit `nihil` remains valid and emits `None`.
- Explicit text remains valid and emits `Some(value)`.
- This does not collapse to plain `textus`.

Docs should avoid saying "`sponte` means nullable." Operationally, some current backends use nullable storage for omitted voluntary fields, but the source-level concepts are different: `sponte` is about whether a slot must be provided, while `T ∪ nihil` is about what values the slot may hold.

## Required Documentation Updates

- Update `EBNF.md` for declaration markers and inline union type grammar.
- Update `docs/grammatica/` pages that still teach nullable `si T` or `T?`.
- Update `AGENTS.md` syntax guidance if it still mentions old nullable forms.
- Add or update explain corpus entries for:
  - `sponte`
  - `fixus`
  - the union glyph `∪`
  - nullable union form `T ∪ nihil`
- Make the `∪` explain entry clearly distinguish inline value-domain unions from `discretio` tagged unions.
- Ensure `nihil` explain content links to the nullable union form without implying that `sponte` means nullable.
- Document the default interaction explicitly:
  - `textus ∪ nihil email : "Anonymous"` stays nullable and uses the default only when omitted.
  - `textus nickname sponte : "Anonymous"` is voluntary provision with a default; current Rust output stores it as `Option<T>` and fills omitted construction with `Some(default)`.
- Keep negative-test or historical examples clearly labeled as legacy syntax when they remain in docs.

## Explain Corpus Requirement

The explain corpus currently has entries for `nihil`, `nonnihil`, `discretio`, and related terms, but no entry for the inline union glyph. Phase 6 must add an explain entry that teaches:

```fab
functio find() → textus ∪ nihil
typus MaybeText = textus ∪ nihil
```

That entry should cover:

- `∪` reads as a union type operator.
- `T ∪ nihil` is the canonical nullable type spelling.
- `A ∪ B ∪ nihil` canonicalizes internally as optional union semantics.
- `nihil ∪ nihil` is invalid.
- `sponte` belongs on declarations, not pure type positions.

## Checkpoint

Phase 6 is complete when:

- Current docs teach `sponte`, `fixus`, and `T ∪ nihil` consistently.
- The explain corpus includes a discoverable entry for `∪` / inline union types.
- Searches for stale nullable teaching forms are either clean or explicitly classified as legacy/negative examples.
- `cargo test -p faber` passes if explain corpus generation or explain tests are affected.

## Implementation Results

**Created explain entries** (with coverage.toml updates):
- `explain/sponte.md` — voluntary declaration marker for params/fields; distinguishes from value nullability.
- `explain/fixus.md` — post-init fixed intent marker; metadata only (enforcement deferred).
- `explain/∪.md` (unicode filename per test policy) — inline union operator; teaches `T ∪ nihil` canonical nullable form, canonicalization rules, and clear separation from `discretio` tagged unions. Aliases avoid collision with "union" (discretio).

**Updated explain content**:
- `explain/nihil.md` now cross-links to `∪` and notes the `sponte` vs. value-domain distinction.

**Grammar documentation**:
- `EBNF.md` — modernized `parameter`, `fieldDecl`, and `typeAnnotation` productions; removed `si` references; documented post-name markers and `∪` unions with notes on ordering and lowering.

**Teaching docs cleaned and aligned** (all `T?` and `si T` nullable examples removed or labeled; prose updated):
- `docs/grammatica/functiones.md` — Optional Parameters → Voluntary Parameters with `sponte`; defaults section; ownership+optional example; explicit distinction call-out.
- `docs/grammatica/structurae.md` — pactum example and note updated to `numerus ∪ nihil`.
- `docs/grammatica/typi.md` — entire "Nullable Types" section rewritten around `T ∪ nihil`; alias and return examples; type-guard examples; added declaration-vs-value-nullability guidance.
- `docs/grammatica/operatores.md` — vel / est / null-check examples now use `textus ∪ nihil` / `numerus ∪ nihil` local patterns.
- `docs/grammatica/regimen.md` — guard-clause divide example return type updated.

**Project guidance**:
- `AGENTS.md` — revised rule 5 from "use ignotum for nullable params" to current `sponte` + `T ∪ nihil` guidance, with clarification that ignotum remains the unknown escape hatch.

**Verification**:
- `cargo test -p faber -- explain` : all 12 tests green (including coverage manifest and embedded entry validation).
- `cargo run -p faber -- explain ∪` (and sponte/fixus) resolve correctly and render teaching content.
- Residue searches for `si <type>` / `T?` in live docs/ and *.fab (outside si/ control-flow examples and factory/ history) are clean.
- No changes to compiler behavior or examples/stdlib (Phase 5 already migrated).

All checkpoints met. Phase 6 delivers consistent teaching of the sponte/fixus/∪ split. Next (Phase 7) would add guardrail searches and negative parser tests if not already present from earlier phases.
