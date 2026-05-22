# Phase 0 Delivery: Baseline and Invariants

## Interpreted Problem

The MIR layer plan needs a factual baseline before introducing compiler code. The baseline must record what the current HIR and Rust surfaces expose, which backend behaviors are semantic lowering rather than plain emission, and what the first MIR subset will include.

## Normalized Spec

- Inspect current worktree state before implementation.
- Capture representative current behavior for primitive arithmetic, calls, `si`, loops, option/null behavior, string formatting, struct construction, `iace` / `tempta`, and `nota`.
- Identify semantic lowering currently owned by the Rust backend.
- Lock a small first MIR subset for the initial vertical slice.
- Create `docs/factory/mir-layer/ledger.md`.
- Do not change compiler behavior in this phase.

## Repo-Aware Baseline

- `radix hir` currently emits item-level JSON only: success state, item count, item IDs, definition IDs, item kind, and spans.
- Full typed HIR is available inside `driver::AnalyzedUnit`, but no developer command prints expression-level typed HIR yet.
- The Rust path still consumes typed HIR directly through `codegen::generate(Target::Rust, ...)`.
- Rust target validation rejects some high-level constructs, including `tempta`, `cape`, and `iace`, before Rust code is emitted.

## Stage Graph

1. Inspect status and relevant compiler surfaces.
2. Run representative HIR and Rust inspection commands.
3. Record known semantic-lowering responsibilities.
4. Define the first MIR subset and explicit deferrals.
5. Save the ledger.

## Checkpoints

- `docs/factory/mir-layer/ledger.md` exists.
- The ledger cites concrete examples and current commands.
- No source behavior changes are made by phase 0.

## Gate Plan

- `git status --short` is inspected before code changes.
- Phase 0 passes if only docs/artifact files change before phase 1 begins.

## Open Questions

- Whether a later phase should deepen `radix hir` or rely entirely on `radix mir` for expression-level inspection.
