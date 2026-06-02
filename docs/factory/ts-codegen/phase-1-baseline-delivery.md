# Phase 1 Delivery Spec: TypeScript E2E Baseline

## Interpreted Problem

The TypeScript backend needs a truthful corpus-wide measurement before backend
work can be prioritized. Today the repository has unit coverage for selected
TypeScript emissions, but no ignored exemplar harness that classifies each
`examples/exempla/**/*.fab` file through frontend, emission, formatting,
linting, typecheck, runtime, and behavior tiers.

## Normalized Spec

- Add an ignored `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`
  harness.
- Discover the current exemplar corpus recursively without moving or editing
  examples.
- Detect available local TypeScript formatter, linter, typechecker, and runtime
  tools.
- Compile each exemplar through the existing HIR-backed TypeScript target.
- Preserve tier separation: frontend analyzed, TypeScript emitted, formatted,
  linted, typecheck-valid, runnable, and behavior-checked.
- Treat missing external tools as skipped tiers with explicit reasons.
- Keep production TypeScript backend behavior unchanged in this phase.
- Save the baseline results and failure clusters in
  `docs/factory/ts-codegen/baseline-ledger.md`.

## Repo-Aware Baseline

- TypeScript codegen lives under `crates/radix/src/codegen/ts/`.
- Existing target formatting and linting helpers are owned by the driver/tooling
  layer and must be inspected before recording tool assumptions.
- Existing ignored exemplar harnesses live in `crates/radix/src/exempla_e2e_test.rs`
  and are module-mounted from `crates/radix/src/lib.rs`.
- The current corpus contains 101 `.fab` files under `examples/exempla/`.

## Stage Graph

1. Inspect current TypeScript backend tests and driver formatting/linting
   helpers.
2. Add a tiered ignored TypeScript exemplar e2e harness.
3. Run the harness across the full corpus and capture output.
4. Record tool availability, tier counts, failure clusters, and next-phase
   recommendations.
5. Run focused TypeScript backend tests and `cargo test -p radix`.

## Epic Candidates And Scopable Issues

- Harness honesty and repeatability for future phases.
- Production TypeScript backend fixes are explicitly out of scope for this
  baseline unless the harness needs a small test-only helper.

## Checkpoints

- `cargo test -p radix codegen::ts -- --nocapture`
- `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`
- `cargo test -p radix`

## Companion Skill Plan

- Use factory supervision for phase gating, ledgering, and commit discipline.
- Use delivery only for this persisted phase spec; no subagent is needed for the
  baseline unless tool output becomes too broad to classify directly.

## Gate Plan

The phase can commit when the harness exists, reports tiered counts honestly,
the baseline ledger matches observed output, and `cargo test -p radix` has no
unexpected regression.

## Open Questions

- Which local TypeScript toolchain is available in this worktree environment?
  Answer through detection during implementation.
