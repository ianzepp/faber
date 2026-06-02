# Phase 017 Delivery Spec: Compound Assignment MIR Lowering

## Interpreted Problem

The current Wasm exemplar harness is truthfully measuring 60/101 compile-valid
exemplars. `examples/exempla/assignatio/assignatio.fab` and
`examples/exempla/binarius/binarius.fab` stop at frontend analyzed because
`HirExprKind::AssignOp` has an explicit MIR-lowering diagnostic.

## Normalized Spec

- Lower compound assignment through target-neutral MIR.
- Reuse existing addressable-place lowering for the assignment target.
- Reuse existing binary operator mapping and Wasm emission for the computed
  value.
- Preserve explicit diagnostics for unsupported operators or non-place targets.
- Do not add a compound-assignment MIR node and do not add Wasm-specific policy
  to MIR.

## Repo-Aware Baseline

- Current harness: `cargo test -p radix exempla_wasm_e2e -- --ignored
  --nocapture`.
- Current counts: 61/101 MIR lowered, 60/101 Wasm emitted, 60/101 compile-valid.
- Current blocker text: `compound assignment before assignment-op MIR lowering`.
- Existing simple assignment lowering resolves the left-hand side to
  `MirPlace`, lowers the right-hand side to a destination, and returns the
  assigned place.
- Existing Wasm emission supports numeric binary ops and text concatenation for
  `MirBinOp::Add`.

## Stage Graph

1. Add `AssignOp` handling to expression/statement lowering.
2. Add focused MIR tests for numeric and text compound assignment.
3. Run focused MIR/Wasm validation and the ignored e2e harness.
4. Update the phase artifact and baseline ledger with measured tier deltas.
5. Run full validation and commit the phase.

## Epic Candidates And Scopable Issues

- This phase is one scoped issue: compound assignment lowering.
- Bitwise compound assignment can lower through MIR if it validates, but exemplar
  movement is expected from numeric and text cases.

## Checkpoints

- Focused MIR test proves the LHS remains a place and the compound value is an
  ordinary binary temp assigned back to the target.
- Harness proves whether `assignatio` and `binarius` move tiers.
- Full `cargo test -p radix` and `./scripta/lint` pass before commit.

## Companion Skill Plan

- Factory supervises this phase directly.
- No subagent is needed; the implementation surface is narrow.

## Gate Plan

- Commit only if the harness has no unexpected regressions and full validation
  passes.
- Keep instantiate/run tiers separate; `wasmtime` remains unavailable unless the
  toolchain changes during validation.

## Open Questions

- None for this phase.
