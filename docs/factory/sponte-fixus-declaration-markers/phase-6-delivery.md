# Phase 6 Delivery Spec: Documentation & Teaching

**Parent Plan**: `docs/factory/sponte-fixus-declaration-markers/plan.md`
**Phase**: 6 - Documentation & Teaching
**Status**: planned
**Created**: 2026-05-22

## Interpreted Phase Problem

After migration, the language implementation and examples should use the new split:

- `sponte` for voluntary declaration slots.
- `fixus` for recorded post-initialization fixed intent.
- `T ∪ nihil` for nullable value domains in pure type positions.

Phase 6 teaches that shape consistently. It should remove stale docs that describe `si T`, `textus?`, or other historical nullable forms as current syntax, and it should add missing explain-corpus coverage for the new inline union glyph.

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

