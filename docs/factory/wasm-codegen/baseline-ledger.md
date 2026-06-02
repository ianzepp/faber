# Wasm Codegen Baseline Ledger

**Phase**: 001 baseline harness  
**Date**: 2026-06-02  
**Worktree**: `/Users/ianzepp/work/ianzepp/faber-wasm-codegen`  
**Branch**: `factory/wasm-codegen`  
**Harness**: `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`

## Toolchain

- Compile validator: `wasm-tools validate`
- Validator version: `wasm-tools 1.251.0`
- Instantiator/runtime: unavailable (`wasmtime` not found on PATH)

## Tier Counts

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 0/101
  Wasm emitted: 0/101
  compile-valid: 0/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

## Baseline Finding

Every current exemplar reaches frontend analysis, and every exemplar stops
before validated MIR. The dominant first blocker is target-neutral MIR lowering
for non-empty top-level entry blocks:

```text
unsupported MIR lowering: non-empty entry blocks before primitive expression lowering
```

That blocker appears in all 101 exemplar failures, either alone or alongside a
construct-specific MIR-lowering gap. Because no exemplar reaches Wasm emission,
`wasm-tools` is detected but not yet exercised against generated exemplar WAT.

## Failure Clusters

### Universal MIR Entry Block Gap

- Count: 101/101
- Tier stopped: frontend analyzed
- Root cause evidence: every harness failure includes `unsupported MIR lowering:
  non-empty entry blocks before primitive expression lowering`.
- Suggested next phase: add target-neutral MIR lowering for a conservative
  top-level entry subset, starting with primitive statements and calls that can
  preserve existing MIR validation invariants.

### Iterator MIR Gaps

- Examples include `ceteri/ceteri.fab`, `itera/cursor-iteratio.fab`,
  `itera/in-functione.fab`, `si/ergo-redde.fab`, and syntax stress exemplars.
- Tier stopped: frontend analyzed
- Root cause evidence: `unsupported MIR lowering: itera before iterator MIR
  lowering`.
- Suggested follow-up: defer until entry-block lowering lets simpler exemplars
  progress far enough to separate loop bodies from top-level execution policy.

### Switch/Pattern MIR Gaps

- Examples include `discerne/discerne.fab`, `elige/ergo-redde.fab`,
  `elige/in-functione.fab`, `omnia/omnia.fab`, and
  `syntaxis/discerne-insanum.fab`.
- Tier stopped: frontend analyzed
- Root cause evidence: `unsupported MIR lowering: discerne before switch MIR
  lowering`.
- Suggested follow-up: target after primitive entry/function support and basic
  control-flow emission.

### Assertion Intrinsic MIR Gap

- Example: `adfirma/in-functione.fab`.
- Tier stopped: frontend analyzed
- Root cause evidence: `unsupported MIR lowering: adfirma before assert
  intrinsic MIR lowering`.
- Suggested follow-up: target under runtime intrinsics once basic entry/function
  programs can lower.

### Operator Primitive MIR Gaps

- Examples include `functio/optionalis.fab` and `si/est.fab`.
- Tier stopped: frontend analyzed
- Root cause evidence: `unsupported MIR lowering: binary operator without a MIR
  primitive` and `unsupported MIR lowering: unary operator without a MIR
  primitive`.
- Suggested follow-up: expand primitive MIR operator mapping after entry-block
  lowering reveals which operators block the most compile-valid candidates.

### Top-Level Const MIR Gap

- Example: `intra/intra.fab`.
- Tier stopped: frontend analyzed
- Root cause evidence: `unsupported MIR lowering: top-level const`.
- Suggested follow-up: keep separate from entry execution; top-level declarations
  need source-order and initialization policy.

## Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed, 85 MIR-focused tests.
- `cargo test -p radix`: passed, 487 unit tests, 8 hygiene tests, and radix
  doc tests.
- `./scripta/lint`: passed.

## Next Phase Candidate

Select a MIR-lowering coverage phase for non-empty entry blocks. The first
useful target is not Wasm stack emission; it is getting simple exemplars through
validated MIR without weakening MIR validation or hiding unsupported runtime
surfaces.
