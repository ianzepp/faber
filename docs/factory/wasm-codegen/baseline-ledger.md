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

## Phase 002 Update: Non-Empty Entry MIR Lowering

**Commit target**: Phase 002  
**Change**: top-level entry blocks now lower through the ordinary
`FunctionBuilder` body path as synthetic vacuum-returning MIR functions.

### Tier Counts After Phase 002

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 0/101
  compile-valid: 0/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Result

The universal entry-block failure is removed. Thirty-two exemplars now reach
validated MIR. No exemplar reaches Wasm emission yet because the currently
lowered modules still contain MIR shapes outside the Wasm text probe subset.

### Remaining MIR-Lowering Clusters

- Iterator/range lowering: `itera before iterator MIR lowering`.
- Switch/pattern lowering: `discerne before switch MIR lowering`.
- Runtime assertions: `adfirma before assert intrinsic MIR lowering`.
- Method/runtime gaps: `method call before runtime/provider MIR lowering`.
- Compound assignment/operator gaps: `compound assignment before assignment-op
  MIR lowering`, `binary operator without a MIR primitive`, and `unary operator
  without a MIR primitive`.
- Aggregate/optional validation gaps surfaced by broader entry lowering:
  field projection, optional-chain, map aggregate, and named aggregate
  validation errors.
- Top-level declarations remain separate: `top-level const`.

### Wasm-Emission Clusters Now Visible

- Runtime calls are not emitted by the Wasm text probe.
- Text/string values are not represented in the Wasm type/value model.
- Aggregate construction is not emitted.
- Comparison and boolean binary ops such as `Eq`, `Gt`, `Lt`, `And`, and `Or`
  are not emitted.
- Branch/control-flow terminators are not emitted.

### Phase 002 Validation Log

- `cargo test -p radix entry -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed, 86 MIR-focused tests.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed, 488 unit tests, 8 hygiene tests, and radix
  doc tests.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select a Wasm type/value model and primitive emission phase. The highest-value
next step is to emit compile-valid Wasm for primitive entry/function modules by
adding Wasm support for local declarations, direct calls, boolean/comparison
operators, and conservative unsupported diagnostics for runtime/text/aggregate
values.
