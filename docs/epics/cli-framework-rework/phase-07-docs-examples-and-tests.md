# Phase 07: Docs, Examples, and Tests

## Goal

Move `docs/grammatica/cli.md` from aspiration to accurate user documentation now that Phases 1-5 are implemented.

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

- Examples should favor small vertical slices that compile today.
- Historical type-first option syntax should be documented as archive-only, not deprecated-supported.
- Generated help snapshots should live in compiler tests when they are needed for behavior locks.

## Acceptance

- The grammar doc accurately describes implemented behavior.
- Every documented CLI feature has a corresponding test or example.
- The aspirational status warning is removed or narrowed to explicitly future-only sections.
- Active CLI examples compile with the current Rust package path.
