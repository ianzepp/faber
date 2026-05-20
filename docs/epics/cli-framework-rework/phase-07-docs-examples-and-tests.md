# Phase 07: Docs, Examples, and Tests

## Goal

Move `docs/grammatica/cli.md` from aspiration to accurate user documentation once implementation has caught up.

## Scope

- Update `docs/grammatica/cli.md` to distinguish shipped behavior from remaining plans.
- Promote working examples under `examples/exempla/cli`.
- Add compiler tests for parsing, validation, diagnostics, and generated output.
- Add end-to-end tests for executable CLI behavior.
- Preserve historical notes only where they help explain current design.
- Add migration notes for any syntax accepted historically but not carried forward.

## Out Of Scope

- Recreating the archived historical corpus in the active repository.
- General language documentation cleanup unrelated to CLI behavior.

## Design Questions

- Should examples favor tiny vertical slices or realistic tools such as `vivi`-shaped command trees?
- Where should generated help snapshots live?
- Should historical syntax forms be documented as supported, deprecated, or archive-only?

## Acceptance

- The grammar doc accurately describes implemented behavior.
- Every documented CLI feature has a corresponding test or example.
- The aspirational status warning is removed or narrowed to explicitly future-only sections.

