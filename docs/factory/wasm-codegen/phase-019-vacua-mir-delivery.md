# Phase 019: `vacua` MIR Lowering

## Interpreted Problem

The Wasm exemplar harness still stops several collection examples at frontend
analysis because `vacua` reaches MIR as an unsupported primitive expression.
That is not a Wasm validation failure and should not be counted as such.

## Normalized Spec

- Lower typed `vacua` expressions through target-neutral MIR.
- Preserve the existing semantic invariant that `vacua` requires collection
  type context.
- Represent empty lists, maps, and sets as ordinary MIR aggregate construction.
- Do not add Wasm-specific collection policy or bypass MIR.
- Re-run the ignored Wasm e2e harness and record the truthful new tier counts.

## Repo-Aware Baseline

Current harness counts after Phase 018:

```text
frontend analyzed: 101/101
MIR lowered: 63/101
Wasm emitted: 62/101
compile-valid: 62/101
```

Direct MIR probes show `examples/exempla/lista/lista.fab` stops on
`unsupported MIR lowering: primitive expression` at an empty collection
declaration, and `examples/exempla/innatum/innatum.fab` has multiple such
failures before later method-call gaps.

## Stage Graph

1. Add an explicit `HirExprKind::Vacua` dispatch path.
2. Lower typed array, map, and set `vacua` to empty MIR aggregate construction.
3. Add MIR dump tests for empty list, map, and set declarations.
4. Run focused MIR validation, the ignored Wasm e2e harness, full radix tests,
   and lint.
5. Update the Wasm baseline ledger with new counts and remaining clusters.

## Checkpoints

- `vacua` no longer reports as a generic primitive-expression MIR gap.
- Empty aggregate MIR validates without requiring codegen to guess element
  types.
- The e2e harness continues to separate compiler/codegen, validation, and host
  tiers.

## Gate Plan

- `cargo test -p radix lowers_vacua -- --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Open Questions

No user decision needed for this phase. Runtime collection operations such as
`primus()` and append/index host ABI support remain explicitly deferred.
