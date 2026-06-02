# Phase 021: Wasm E2E Regression Gate And Handoff

## Interpreted Problem

Phase 020 reached the goal's first successful compile-valid range at 71/101
exemplars, but the ignored Wasm harness still only printed the tier ledger. It
needed an expected-tier floor so future workers cannot accidentally lower the
baseline without a failing gate.

## Normalized Spec

- Preserve the current tiered Wasm classification report.
- Establish expected tier floors for the current exemplar corpus.
- Fail the ignored harness when an exemplar regresses below its expected tier.
- Keep improvements non-failing so future phases can raise floors after ledger
  updates.
- Leave instantiate, run, and behavior tiers unclaimed while `wasmtime` and a
  host/runtime policy are unavailable.
- Record a concise handoff for the next factory run instead of starting a broad
  ABI phase.

## Baseline Floors

- Default floor for every current and future `.fab` exemplar:
  `FrontendAnalyzed`.
- Explicit compile-valid floor for the current 71 compile-valid exemplars.
- Explicit MIR-lowered floor for `examples/exempla/si/est.fab`, which currently
  reaches validated MIR and fails only during Wasm emission on dynamic boxing.

## Gate Plan

- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Handoff

The factory run can stop at this point. Compile-valid coverage is inside the
requested 70-80% first target and is now protected by expected-tier floors.

The next useful work is not a tiny pass-count cleanup. It should be planned as
one or more ABI-bearing phases:

- local Wasm host/runtime support for instantiate and run tiers;
- runtime/provider and collection method MIR lowering;
- collection and aggregate host imports for mutation, lookup, and iteration;
- dynamic `ignotum` boxing/coercion ABI;
- remaining MIR validation clusters for optional/object/member shapes.

Do not claim instantiate, runnable, or behavior-checked progress until the
host imports and entrypoint/run policy are implemented and measured.
