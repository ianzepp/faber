# Epic 2 Phase 1 Delivery: Exempla Boundary Enforcement

**Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`  
**Focused Goal**: `docs/factory/exempla-rust-e2e/goal.md`  
**Date**: 2026-05-24  
**Scope**: Epic 2, corpus boundary only

## Interpreted Problem

The live Rust e2e harness still reports `71/137` exempla passing. The failure set includes real backend bugs, but it also includes files that are not standalone single-file language examples. Backend stabilization is not truthful while package fixtures, HAL/runtime examples, import demos, CLI module examples, test fixtures, stale syntax, and declaration-only examples remain inside `examples/exempla/`.

## Normalized Spec

Move or quarantine non-standalone `.fab` files out of `examples/exempla/` without deleting their source. Preserve reviewable rationale for each relocation. Do not implement Epic 3 `ad` capability behavior, and do not use this phase to paper over valid backend bugs.

## Repo-Aware Baseline

- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` reports `71/137`.
- `docs/factory/exempla-rust-e2e/baseline-ledger.md` classifies non-standalone, test/package, stale, declaration-only, and unsupported/future files.
- `examples/exempla/` is discovered recursively by `crates/radix/src/exempla_e2e_test.rs`, so relocation outside that directory is sufficient for the standalone Rust harness boundary.

## Stage Graph

1. Create a sibling quarantine tree under `examples/fixtures/exempla-boundary/`.
2. Relocate classified non-standalone, test/package fixture, stale, declaration-only, and future-feature files that are not direct Epic 3 inputs.
3. Add a relocation ledger under `docs/factory/exempla-rust-e2e/`.
4. Re-run the Rust e2e harness to confirm the remaining failures are no longer caused by moved corpus-boundary files.

## Relocation Policy

- `runtime-hal/`: HAL and host/runtime-backed files.
- `package-cli/`: CLI module/package-shape files.
- `imports/`: import/helper-library demonstrations.
- `proba/`: test harness and package-selection fixtures.
- `regex/`: external-crate-backed examples.
- `declaration-only/`: examples that emit free Rust declarations without executable bodies.
- `stale-source/`: retired syntax, unknown identifiers, or source that should not be taught to the compiler for pass-count optics.
- `unsupported-rust-target/`: current language surfaces without Rust execution policy, except `ad/ad.fab`, which remains for Epic 3's linked capability-call goal.

## Checkpoints

- No relocated file remains under `examples/exempla/`.
- The relocation ledger records source path, destination path, and reason.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` still runs and reports a smaller corpus.
- Remaining failures are backend/future-feature blockers, not package/module/runtime/dependency boundary errors from relocated files.

## Gate Plan

Do not commit if relocation accidentally deletes source files, if the e2e harness cannot run, or if the move pulls unrelated generated build artifacts into version control.
