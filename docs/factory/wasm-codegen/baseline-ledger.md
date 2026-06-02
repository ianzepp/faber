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

## Phase 003 Update: Primitive Wasm Emission

**Commit target**: Phase 003  
**Change**: the Wasm text probe now emits a conservative primitive scalar
subset: `numerus` as `i64`, `bivalens` as `i32`, non-param locals, temps,
direct calls, numeric comparisons, boolean `and`/`or`, and numeric/bool
diagnostic calls through explicit `faber_diag` imports.

### Tier Counts After Phase 003

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 2/101
  compile-valid: 2/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Exemplars

- `examples/exempla/aut/aut.fab`
- `examples/exempla/et/et.fab`

Both validate with `wasm-tools validate`. They remain below instantiate/run
tiers because `wasmtime` is unavailable on PATH and the modules use explicit
diagnostic imports that need a host implementation.

### Remaining Wasm-Emission Clusters

- Text/string constants and `textus` values remain unsupported.
- Branch/control-flow terminators remain unsupported.
- Aggregate and enum/struct/array construction remains unsupported.
- Runtime diagnostics with text arguments remain unsupported.
- Runtime-managed types such as `textus`, arrays, structs, enums, nullable
  values, and dynamic values remain unsupported by design in this primitive
  subset.

### Remaining MIR-Lowering Clusters

The Phase 002 MIR-lowering clusters remain broadly unchanged: iterator/range
lowering, switch/pattern lowering, assertion intrinsics, method/runtime gaps,
compound assignment/operator gaps, aggregate/optional validation gaps, and
top-level consts.

### Phase 003 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  in focused Wasm tests when `wasm-tools` is available.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed, 88 MIR-focused tests.
- `cargo test -p radix`: passed, 490 unit tests, 8 hygiene tests, and radix
  doc tests.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select a control-flow Wasm emission phase. Branch terminators now block several
already-MIR-lowered exemplars (`custodi`, `functio/recursio`, and `si/*`).
Keep text diagnostics separate: branch support should improve primitive control
flow without pretending string output is solved.

## Phase 004 Update: Primitive Control-Flow Wasm Emission

**Commit target**: Phase 004  
**Change**: the Wasm text probe now emits primitive multi-block functions with
an explicit dispatch loop over MIR block IDs. `branch`, `goto`, `return`, and
`unreachable` terminators are supported for the primitive scalar subset without
changing MIR or bypassing typed HIR -> MIR.

### Tier Counts After Phase 004

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 3/101
  compile-valid: 3/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Exemplars

- `examples/exempla/aut/aut.fab`
- `examples/exempla/et/et.fab`
- `examples/exempla/functio/recursio.fab`

All three validate with `wasm-tools validate`. Instantiate and run tiers remain
at zero because `wasmtime` is unavailable on PATH; that is reported by the
harness as a skipped host tier, not as a compiler failure.

### Result

The Phase 003 primitive compile-valid exemplars remain valid, and one
branch-heavy numeric exemplar now advances through compile-valid Wasm. This is
still a small measured gain, not a broad success claim.

### Remaining Wasm-Emission Clusters

- Unary primitive values now block nearby numeric examples such as
  `custodi/custodi.fab`.
- Text/string constants and `textus` values remain unsupported.
- Aggregate and enum/struct/array construction remains unsupported.
- Runtime diagnostics with text arguments remain unsupported.
- Switch, try-call, return-error, runtime-managed values, nullable values, and
  dynamic values remain unsupported by design in this primitive subset.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, aggregate/optional validation gaps, and top-level
consts.

### Phase 004 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  in focused Wasm tests when `wasm-tools` is available.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select a unary primitive Wasm emission phase. That should cover numeric
negation and boolean not in the same fail-closed scalar model, and should be
validated by checking whether `custodi/custodi.fab` can advance past the current
`unary value` emission blocker.
