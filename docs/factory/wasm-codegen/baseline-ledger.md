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

## Phase 005 Update: Primitive Unary Wasm Emission

**Commit target**: Phase 005  
**Change**: the Wasm text probe now emits scalar MIR unary values for numeric
negation, boolean not, and numeric bitwise not. Nullable predicates and
runtime-managed unary values remain explicitly unsupported.

### Tier Counts After Phase 005

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 4/101
  compile-valid: 4/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Exemplars

- `examples/exempla/aut/aut.fab`
- `examples/exempla/custodi/custodi.fab`
- `examples/exempla/et/et.fab`
- `examples/exempla/functio/recursio.fab`

All four validate with `wasm-tools validate`. Instantiate and run tiers remain
at zero because `wasmtime` is unavailable on PATH; this remains a skipped host
tier rather than a compiler or codegen failure.

### Result

`custodi/custodi.fab` advanced from Wasm emission failure to compile-valid
after numeric negation support. This raises measured compile-valid coverage from
3/101 to 4/101. `unarius/unarius.fab` still stops before MIR because several
predicate-style unary forms do not yet lower to MIR primitives, so it is not
counted as a Wasm-codegen miss.

### Remaining Wasm-Emission Clusters

- Text/string constants and `textus` values remain unsupported.
- Aggregate and enum/struct/array construction remains unsupported.
- Runtime diagnostics with text arguments remain unsupported.
- Switch, try-call, return-error, runtime-managed values, nullable values, and
  dynamic values remain unsupported by design in this primitive subset.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, and top-level consts.

### Phase 005 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  in focused Wasm tests when `wasm-tools` is available.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select the next measured blocker from the harness rather than jumping to a
percentage claim. The largest visible Wasm-emission cluster is still
text/string representation and diagnostics, while several other exemplars are
blocked earlier in MIR lowering.

## Phase 006 Update: Text Handle Wasm Emission

**Commit target**: Phase 006  
**Change**: the Wasm text probe now treats `textus` as an opaque `i32` handle
for the compile-valid tier. String constants emit handle constants, text
diagnostics use distinct `*_text` imports, text concatenation calls
`faber_text.concat`, and format-string runtime calls use signature-specific
`faber_text.format_*` imports.

### Tier Counts After Phase 006

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 24/101
  compile-valid: 24/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Exemplars

- `examples/exempla/aut/aut.fab`
- `examples/exempla/cura/cura.fab`
- `examples/exempla/cura/nidificatus.fab`
- `examples/exempla/custodi/custodi.fab`
- `examples/exempla/discretio/discretio.fab`
- `examples/exempla/et/et.fab`
- `examples/exempla/fixum/fixum.fab`
- `examples/exempla/functio/functio.fab`
- `examples/exempla/functio/recursio.fab`
- `examples/exempla/incipit/incipit.fab`
- `examples/exempla/mone/mone.fab`
- `examples/exempla/privata/privata.fab`
- `examples/exempla/protecta/protecta.fab`
- `examples/exempla/publica/publica.fab`
- `examples/exempla/redde/redde.fab`
- `examples/exempla/salve-munde.fab`
- `examples/exempla/scriptum/scriptum.fab`
- `examples/exempla/si/ergo.fab`
- `examples/exempla/si/nidificatus.fab`
- `examples/exempla/si/secus.fab`
- `examples/exempla/si/si.fab`
- `examples/exempla/si/sin.fab`
- `examples/exempla/varia/varia.fab`
- `examples/exempla/vide/vide.fab`

All listed modules validate with `wasm-tools validate`. Instantiate and run
tiers remain at zero because `wasmtime` is unavailable on PATH and the modules
now also depend on explicit `faber_text` host imports.

### Result

Measured compile-valid coverage increased from 4/101 to 24/101. The phase also
removed the emitted-but-invalid `redde/redde.fab` failure that appeared during
development by routing text concatenation through an explicit text import
instead of numeric Wasm arithmetic.

### Remaining Wasm-Emission Clusters

- Aggregate and enum/struct/array construction remains unsupported:
  `destructura/destructura.fab`, `finge/finge.fab`, `genus/genus.fab`,
  `novum/novum.fab`, `typus/typus.fab`, and `varia/destructura.fab`.
- `fractus` values remain unsupported by the Wasm scalar model:
  `functio/typicus.fab` and `varia/typicus.fab`.
- Runtime-managed values beyond text handles remain unsupported, including
  structs, enums, arrays, nullable values, dynamic values, switch, try-call, and
  return-error paths.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, and several diagnostic runtime arity validation gaps.

### Phase 006 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  for text handles, text diagnostics, text concat, and format-string imports.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select a small scalar expansion phase for `fractus` support. Only two currently
MIR-lowered exemplars visibly stop on `Primitive(Fractus)`, but this is a clean
compile-valid Wasm type/value-model extension and should be cheaper than
starting aggregate layout.

## Phase 007 Update: Fractus Wasm Emission

**Commit target**: Phase 007  
**Change**: the Wasm text probe now represents `fractus` as Wasm `f64`, emits
float constants, `f64` arithmetic/comparisons/unary negation, calls, returns,
locals, parameters, and `*_f64` diagnostic imports.

### Tier Counts After Phase 007

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 26/101
  compile-valid: 26/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

The following exemplars advanced from MIR-lowered Wasm-emission failure to
compile-valid:

- `examples/exempla/functio/typicus.fab`
- `examples/exempla/varia/typicus.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH and the modules depend on explicit host imports for diagnostics/text.

### Result

Measured compile-valid coverage increased from 24/101 to 26/101. No
emitted-invalid modules were reported by the harness.

### Remaining Wasm-Emission Clusters

- Aggregate and enum/struct/array construction remains unsupported:
  `destructura/destructura.fab`, `finge/finge.fab`, `genus/genus.fab`,
  `novum/novum.fab`, `typus/typus.fab`, and `varia/destructura.fab`.
- Runtime-managed values beyond text handles remain unsupported, including
  structs, enums, arrays, nullable values, dynamic values, switch, try-call, and
  return-error paths.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, and several diagnostic runtime arity validation gaps.

### Phase 007 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  for `fractus` values and `nota_f64` diagnostics.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

The remaining MIR-lowered Wasm blockers are now mostly aggregate/runtime-managed
values. The next small compile-valid phase should either add an opaque handle
subset for arrays/structs/enums or shift earlier to MIR-lowering clusters if the
goal is to increase the `MIR lowered` tier first.

## Phase 008 Update: Aggregate Handle Wasm Emission

**Commit target**: Phase 008
**Change**: the Wasm text probe now represents constructed arrays, maps,
records, sets, structs, and enums as opaque `i32` aggregate handles for the
compile-valid tier. Aggregate construction lowers to signature-specific
`faber_aggregate` imports, struct and variant constructors pass a leading
definition-id metadata argument, and whole-aggregate diagnostics use distinct
`*_aggregate` imports.

### Tier Counts After Phase 008

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 28/101
  compile-valid: 28/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

The following exemplars advanced from MIR-lowered Wasm-emission failure to
compile-valid:

- `examples/exempla/finge/finge.fab`
- `examples/exempla/typus/typus.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

Measured compile-valid coverage increased from 26/101 to 28/101. No
emitted-invalid modules were reported by the harness.

This phase intentionally does not define aggregate runtime layout. It only
establishes a validateable Wasm ABI surface for MIR aggregate construction and
whole-aggregate diagnostics.

### Remaining Wasm-Emission Clusters

- Place projections remain unsupported and now account for all remaining
  MIR-lowered Wasm emission failures:
  `destructura/destructura.fab`, `genus/genus.fab`, `novum/novum.fab`, and
  `varia/destructura.fab`.
- Runtime-managed values beyond opaque handles remain unsupported, including
  projection helpers, nullable values, dynamic values, switch, try-call, and
  return-error paths.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, and several diagnostic runtime arity validation gaps.

### Phase 008 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  for opaque aggregate handles and aggregate diagnostics.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

The next compile-valid Wasm phase should target type-directed aggregate
projections. The current blockers need field and index projection helper imports
chosen from MIR place projection result types; this should stay separate from
aggregate construction so runtime layout decisions remain explicit.

## Phase 009 Update: Aggregate Projection Wasm Emission

**Commit target**: Phase 009
**Change**: the Wasm text probe now emits compile-valid reads from aggregate
field and index projections. Wasm emission receives the same
`MirValidationContext` used to validate the MIR, so projection result types come
from target-neutral semantic metadata rather than Wasm-specific MIR nodes.

### Tier Counts After Phase 009

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 32/101
  Wasm emitted: 32/101
  compile-valid: 32/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

The following exemplars advanced from MIR-lowered Wasm-emission failure to
compile-valid:

- `examples/exempla/destructura/destructura.fab`
- `examples/exempla/genus/genus.fab`
- `examples/exempla/novum/novum.fab`
- `examples/exempla/varia/destructura.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

Measured compile-valid coverage increased from 28/101 to 32/101. No
emitted-invalid modules were reported by the harness.

All currently MIR-lowered exemplars now emit compile-valid Wasm. The next
coverage ceiling is the `MIR lowered` tier, not Wasm emission for validated MIR.

### Remaining Wasm-Emission Clusters

- No remaining failures at `MirLowered` in the current harness result.
- Projected assignment destinations remain intentionally unsupported in the
  Wasm probe, but no current MIR-lowered exemplar reaches that shape.
- Runtime-managed values beyond opaque helper imports remain host/runtime work,
  including actual aggregate layout and behavior.

### Remaining MIR-Lowering Clusters

The established MIR-lowering clusters remain: iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, and several diagnostic runtime arity validation gaps.

### Phase 009 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed, including WAT validation
  for aggregate field and index projection reads.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select the next phase from MIR-lowering clusters. The most direct way to keep
compile-valid coverage moving is to raise `MIR lowered` beyond 32/101, then let
the now-complete Wasm-emission subset classify the newly lowered programs.

## Phase 010 Update: Diagnostic Fan-Out MIR Lowering

**Commit target**: Phase 010
**Change**: multi-argument diagnostic source expressions now lower into one
unary `MirIntrinsic::Diagnostic` runtime call per source argument. MIR
validation still requires diagnostic runtime calls to have exactly one operand;
the lowering now satisfies that invariant instead of emitting invalid MIR.

### Tier Counts After Phase 010

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 42/101
  Wasm emitted: 40/101
  compile-valid: 40/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 32/101 to 40/101. MIR-lowered
coverage increased from 32/101 to 42/101.

New compile-valid exemplars include:

- `examples/exempla/dum/conditio-complexa.fab`
- `examples/exempla/dum/dum.fab`
- `examples/exempla/dum/in-functione.fab`
- `examples/exempla/incipit/functionibus.fab`
- `examples/exempla/nota/gradus.fab`
- `examples/exempla/nota/nota.fab`
- `examples/exempla/perge/perge.fab`
- `examples/exempla/rumpe/rumpe.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

The previous diagnostic runtime arity validation cluster is removed for the
files above. Two files now advance to validated MIR but stop at Wasm emission:

- `examples/exempla/conversio/conversio.fab`: value-returning conversion
  runtime calls remain unsupported by the Wasm text probe.
- `examples/exempla/mori/mori.fab`: panic runtime calls remain unsupported by
  the Wasm text probe.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, and top-level consts.

### Remaining Wasm-Emission Clusters

- Value-returning runtime conversion calls.
- Panic runtime calls.

### Phase 010 Validation Log

- `cargo test -p radix diagnostic -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Either add Wasm probe support for runtime conversion/panic calls to clear the
two newly visible Wasm-emission failures, or continue raising the `MIR lowered`
tier through another MIR-lowering cluster such as assertions or iterator/range
lowering.

## Phase 011 Update: Runtime Wasm Imports

**Commit target**: Phase 011
**Change**: value-returning conversion runtime calls, collection length runtime
calls, and panic runtime calls now emit explicit `faber_runtime` Wasm imports
from MIR. These imports make the generated WAT compile-valid while keeping
runtime/host execution separate from compiler/codegen success.

### Tier Counts After Phase 011

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 42/101
  Wasm emitted: 42/101
  compile-valid: 42/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 40/101 to 42/101. MIR-lowered
coverage remains 42/101.

New compile-valid exemplars:

- `examples/exempla/conversio/conversio.fab`
- `examples/exempla/mori/mori.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This is still a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

There are no remaining Wasm-emission failures for the current MIR-lowered
exemplar set. The compile-valid ceiling is again the `MIR lowered` tier.

Conversion hint semantics and actual aggregate/text runtime behavior are not
claimed by this phase. The generated modules validate because their host import
ABI is declared; behavior remains runtime-host work.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are iterator/range lowering,
switch/pattern lowering, assertion intrinsics, method/runtime gaps, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, and top-level consts.

### Remaining Host/Runtime Clusters

- Provide real `faber_runtime` import implementations for conversion, panic, and
  collection length.
- Add a local instantiate/run host before measuring instantiate-valid, runnable,
  or behavior-checked tiers.

### Phase 011 Validation Log

- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select the next phase from MIR-lowering clusters. Assertion intrinsics or
iterator/range lowering are likely direct candidates for raising the
`MIR lowered` ceiling.

## Phase 012 Update: Assert Intrinsic Wasm Path

**Commit target**: Phase 012
**Change**: `adfirma` now lowers to a target-neutral MIR assertion runtime
intrinsic. The Wasm text probe emits explicit assertion imports, and textus
equality/inequality lowers to explicit `faber_text` comparison imports so the
assertion exemplars validate as WAT.

### Tier Counts After Phase 012

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 44/101
  Wasm emitted: 44/101
  compile-valid: 44/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 42/101 to 44/101. MIR-lowered
coverage also increased from 42/101 to 44/101.

New compile-valid exemplars:

- `examples/exempla/adfirma/adfirma.fab`
- `examples/exempla/adfirma/in-functione.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

The assertion MIR-lowering cluster is removed from the current harness result.
There are no remaining Wasm-emission failures for the current MIR-lowered
exemplar set. The compile-valid ceiling remains the `MIR lowered` tier.

Assertion behavior and text comparison behavior are not claimed by this phase.
The generated modules validate because their host import ABI is declared;
behavior remains runtime-host work.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are iterator/range lowering,
switch/pattern lowering, runtime/provider method calls, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, and async `cede`.

### Remaining Host/Runtime Clusters

- Provide real `faber_runtime` import implementations for assertion, conversion,
  panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Add a local instantiate/run host before measuring instantiate-valid, runnable,
  or behavior-checked tiers.

### Phase 012 Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Select the next phase from MIR-lowering clusters. Iterator/range lowering has
the largest visible cluster, while switch/pattern lowering is another compact
candidate if the next phase should stay control-flow focused.

## Phase 013 Update: Literal Switch Wasm Path

**Commit target**: Phase 013
**Change**: single-scrutinee literal `elige`/`discerne` arms now lower through
target-neutral MIR `Switch` terminators and emit compile-valid Wasm dispatch.
String switch cases use the explicit `faber_text eq_text` import path.

### Tier Counts After Phase 013

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 48/101
  Wasm emitted: 48/101
  compile-valid: 48/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 44/101 to 48/101. MIR-lowered
coverage also increased from 44/101 to 48/101.

New compile-valid exemplars:

- `examples/exempla/elige/ceterum.fab`
- `examples/exempla/elige/elige.fab`
- `examples/exempla/elige/ergo-redde.fab`
- `examples/exempla/elige/in-functione.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

The simple literal switch subset is no longer part of the generic
`discerne before switch MIR lowering` cluster. Remaining pattern-matching
forms still fail explicitly instead of falling through an incomplete lowering:
enum variants, destructuring patterns, guarded cases, multi-subject matches,
and value-producing matches.

There are no remaining Wasm-emission failures for the current MIR-lowered
exemplar set. The compile-valid ceiling remains the `MIR lowered` tier.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are iterator/range lowering,
non-literal and enum `discerne`, runtime/provider method calls, compound
assignment/operator gaps, predicate unary gaps, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, and async `cede`.

### Remaining Host/Runtime Clusters

- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 013 Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Iterator/range lowering remains the largest visible cluster. If the next phase
should stay compact, continue with another narrow control-flow subset such as
enum-pattern `discerne`; otherwise, attack iterator MIR lowering to raise the
compile-valid ceiling more substantially.

## Phase 014 Update: Predicate Operators Wasm Path

**Commit target**: Phase 014
**Change**: scalar predicate operators now lower through target-neutral MIR and
emit compile-valid Wasm for the supported scalar/handle subset. This includes
sign checks, boolean truth checks, and nil/non-nil tests.

### Tier Counts After Phase 014

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 51/101
  Wasm emitted: 50/101
  compile-valid: 50/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 48/101 to 50/101. MIR-lowered
coverage increased from 48/101 to 51/101.

New compile-valid exemplars:

- `examples/exempla/est/est.fab`
- `examples/exempla/unarius/unarius.fab`

New MIR-lowered but not Wasm-emitted exemplar:

- `examples/exempla/si/est.fab`: stops at Wasm emission with
  `MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

The predicate unary cluster is removed for the scalar and nullable-handle
subset. The phase also makes `est nihil` lower to explicit nil-test MIR instead
of blocking as a missing binary primitive.

The nullable Wasm representation added here is compile-valid only: nullable
slots are represented as opaque `i32` handles for the text probe, with zero used
for `nihil`. This is not a complete runtime ABI and does not claim behavior at
instantiate/run tiers.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are iterator/range lowering,
runtime/provider method calls, compound assignment/operator gaps such as
`inter`/`intra`, non-literal and enum `discerne`, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, closures, and async `cede`.

### Remaining Wasm-Emission Clusters

- Dynamic `ignotum` has no Wasm value model yet, surfaced by
  `examples/exempla/si/est.fab`.

### Remaining Host/Runtime Clusters

- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Define and implement nullable and dynamic value ABIs before claiming
  instantiate/run behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 014 Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Iterator/range lowering remains the largest cluster and is now the most
important route toward the 70-80% compile-valid target. A smaller alternative
is compound assignment lowering, but it is less likely to move as many
exemplars as iterator support.

## Phase 015 Update: Numeric Range Iteration Wasm Path

**Commit target**: Phase 015
**Change**: `itera pro` over numeric `Intervallum` sources now lowers through
target-neutral MIR CFG and emits compile-valid Wasm. Collection iteration is
still separated behind explicit iterator/runtime ABI diagnostics.

### Tier Counts After Phase 015

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 56/101
  Wasm emitted: 55/101
  compile-valid: 55/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 50/101 to 55/101. MIR-lowered
coverage increased from 51/101 to 56/101.

New compile-valid exemplars:

- `examples/exempla/ante/ante.fab`
- `examples/exempla/itera/intervallum-gradus.fab`
- `examples/exempla/itera/intervallum.fab`
- `examples/exempla/per/per.fab`
- `examples/exempla/usque/usque.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission with
`MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

The numeric range subset of the iterator cluster is now compile-valid through
MIR-backed Wasm. The lowering uses ordinary MIR locals, comparisons, branches,
gotos, and assignments; it does not introduce Wasm-specific MIR nodes.

`perge` targets the synthetic increment block and `rumpe` targets the loop exit
block. Default range steps are selected with a MIR branch based on the start/end
direction. Inclusive and exclusive ranges use the corresponding comparison
operators instead of precomputing a target-specific range object.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are collection/cursor iteration,
runtime/provider method calls, compound assignment/operator gaps such as
`inter`/`intra`, non-literal and enum `discerne`, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, closures, and async `cede`.

### Remaining Wasm-Emission Clusters

- Dynamic `ignotum` has no Wasm value model yet, surfaced by
  `examples/exempla/si/est.fab`.

### Remaining Host/Runtime Clusters

- Define collection/cursor iterator ABI for `itera ex` and `itera de`.
- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Define and implement nullable and dynamic value ABIs before claiming
  instantiate/run behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 015 Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

The largest remaining route to the 70-80% compile-valid target is collection
iteration or method/runtime lowering. A smaller next phase is compound
assignment lowering, which should move `assignatio`/`binarius` if Wasm emission
for the resulting scalar operations is already present.

## Phase 016 Update: Array Iteration Wasm Path

**Commit target**: Phase 016
**Change**: array-backed `itera ex` and `itera de` now lower through
target-neutral MIR CFG and emit compile-valid Wasm. Map, set, text, cursor, and
provider-backed iteration remain explicit iterator/runtime ABI gaps.

### Tier Counts After Phase 016

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 61/101
  Wasm emitted: 60/101
  compile-valid: 60/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 55/101 to 60/101. MIR-lowered
coverage increased from 56/101 to 61/101.

New compile-valid exemplars:

- `examples/exempla/ceteri/ceteri.fab`
- `examples/exempla/itera/ex.fab`
- `examples/exempla/itera/in-functione.fab`
- `examples/exempla/itera/nidificatus.fab`
- `examples/exempla/sparge/sparge.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission with
`MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

Array collection loops now use existing target-neutral MIR building blocks:
mutable index locals, collection length runtime intrinsics, numeric comparison,
branch/goto control flow, and aggregate index projections. `itera ex` binds the
indexed element; `itera de` binds the numeric index.

No Wasm-specific loop or iterator nodes were added to MIR. Non-array collection
iteration still fails before codegen with the existing explicit unsupported
diagnostic.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are map/cursor/provider
iteration, runtime/provider method calls, compound assignment/operator gaps such
as `inter`/`intra`, non-literal and enum `discerne`, aggregate/optional
validation gaps, top-level consts, `ad` provider blocks, closures, and async
`cede`.

### Remaining Wasm-Emission Clusters

- Dynamic `ignotum` has no Wasm value model yet, surfaced by
  `examples/exempla/si/est.fab`.

### Remaining Host/Runtime Clusters

- Define map, set, text, cursor, and provider iterator ABI for `itera ex` and
  `itera de`.
- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Define and implement nullable and dynamic value ABIs before claiming
  instantiate/run behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 016 Validation Log

- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

Compound assignment is a contained next target and should move
`assignatio/assignatio.fab` and `binarius/binarius.fab` if the resulting scalar
operations are already covered by Wasm emission. Runtime/provider method
lowering is larger and likely unlocks more exemplars, but it needs tighter ABI
policy before implementation.

## Phase 017 Update: Compound Assignment MIR Lowering

**Commit target**: Phase 017
**Change**: compound assignment expressions now lower through target-neutral MIR
as an ordinary binary value assigned back to the original place. No
compound-assignment MIR node or Wasm-specific MIR policy was added.

### Tier Counts After Phase 017

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 63/101
  Wasm emitted: 61/101
  compile-valid: 61/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 60/101 to 61/101. MIR-lowered
coverage increased from 61/101 to 63/101.

New compile-valid exemplar:

- `examples/exempla/assignatio/assignatio.fab`

Newly exposed Wasm-emission blocker:

- `examples/exempla/binarius/binarius.fab` now reaches validated MIR but stops
  at Wasm emission with `MIR-to-WASM unsupported: option value`.

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission with
`MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

Compound assignment now reuses the existing assignment-place resolver for paths,
fields, and indexes. The lowerer reads the target place as the binary left-hand
operand, lowers the right-hand side normally, assigns the binary temp back to
the same place, and returns that place when the expression value is needed.

This moved `assignatio.fab` to compile-valid and separated `binarius.fab` from
the compound-assignment cluster; `binarius` is now blocked by option/nullable
Wasm value emission instead.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are map/cursor/provider
iteration, runtime/provider method calls, remaining operator gaps such as
`inter`/`intra`, non-literal and enum `discerne`, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, closures, and async `cede`.

### Remaining Wasm-Emission Clusters

- Option/nullable values have no Wasm value model yet, surfaced by
  `examples/exempla/binarius/binarius.fab`.
- Dynamic `ignotum` has no Wasm value model yet, surfaced by
  `examples/exempla/si/est.fab`.

### Remaining Host/Runtime Clusters

- Define map, set, text, cursor, and provider iterator ABI for `itera ex` and
  `itera de`.
- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Define and implement nullable and dynamic value ABIs before claiming
  instantiate/run behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 017 Validation Log

- `cargo test -p radix lowers_compound_assignment_to_binary_and_assign -- --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

### Next Phase Candidate

A focused option-value compile-valid MVP could move `binarius.fab`, but it must
not claim runtime nullable semantics. Runtime/provider method lowering remains
the larger coverage cluster and likely unlocks more exemplars, but it requires
clear ABI policy before implementation.

## Phase 018 Update: Option Coalesce And Bitwise Wasm Emission

**Commit target**: Phase 018
**Change**: handle-level option coalesce and integer bitwise MIR binary
operations now emit compile-valid Wasm. This closes the `binarius.fab`
Wasm-emission path without claiming full nullable runtime semantics.

### Tier Counts After Phase 018

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 63/101
  Wasm emitted: 62/101
  compile-valid: 62/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 61/101 to 62/101. Wasm-emitted
coverage increased from 61/101 to 62/101. MIR-lowered coverage stayed at
63/101.

New compile-valid exemplar:

- `examples/exempla/binarius/binarius.fab`

`examples/exempla/si/est.fab` still reaches MIR but stops at Wasm emission with
`MIR-to-WASM unsupported: type Primitive(Ignotum)`.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This remains a skipped host/runtime tier, not a compiler or codegen
failure.

### Result

Nullable text coalesce now emits a raw-handle `select`, using `0` as the nil
handle and only when nullable value, fallback, and result share the same Wasm
carrier. Other option operations remain explicit unsupported shapes.

Integer bitwise operations now map to `i64.and`, `i64.or`, `i64.xor`,
`i64.shl`, and signed `i64.shr_s`.

### Remaining MIR-Lowering Clusters

The remaining high-level MIR-lowering clusters are map/cursor/provider
iteration, runtime/provider method calls, remaining operator gaps such as
`inter`/`intra`, non-literal and enum `discerne`, aggregate/optional validation
gaps, top-level consts, `ad` provider blocks, closures, and async `cede`.

### Remaining Wasm-Emission Clusters

- Dynamic `ignotum` has no Wasm value model yet, surfaced by
  `examples/exempla/si/est.fab`.
- Full nullable ABI is still incomplete for `Some`, unwrap, optional chains, and
  mixed-carrier nullable primitive values.

### Remaining Host/Runtime Clusters

- Define map, set, text, cursor, and provider iterator ABI for `itera ex` and
  `itera de`.
- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, and collection length.
- Provide real `faber_text` comparison behavior.
- Define and implement nullable and dynamic value ABIs before claiming
  instantiate/run behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 018 Validation Log

- `cargo test -p radix wasm_target_emits_integer_bitwise_ops -- --nocapture`: passed.
- `cargo test -p radix wasm_target_emits_option_coalesce_for_nullable_handles -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

An initial combined focused-test command used an invalid Cargo filter form and
failed before running tests. The individual focused filters and full
Wasm-focused suite passed.

### Next Phase Candidate

The only remaining Wasm-emission blocker in the current harness is dynamic
`ignotum` in `si/est.fab`. Larger compile-valid gains are more likely from MIR
lowering clusters such as runtime/provider method calls, map/cursor iteration,
and non-literal `discerne`, but those need more policy than the small scalar
emission phases.

## Phase 019 Update: `vacua` MIR Lowering

**Commit target**: Phase 019
**Change**: typed `vacua` expressions now lower through target-neutral MIR as
empty array, map, or set aggregate construction. This removes a generic
primitive-expression MIR stop without adding Wasm-only collection shortcuts.

### Tier Counts After Phase 019

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 64/101
  Wasm emitted: 63/101
  compile-valid: 63/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 62/101 to 63/101. MIR-lowered
coverage increased from 63/101 to 64/101, and Wasm-emitted coverage increased
from 62/101 to 63/101.

New compile-valid exemplar:

- `examples/exempla/lista/lista.fab`

`examples/exempla/innatum/innatum.fab` no longer reports `vacua` as a primitive
MIR expression, but still stops at runtime/provider method-call MIR lowering.

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This is still a skipped host/runtime tier, not a compiler, codegen, or
validator failure.

### Result

The MIR lowering path now preserves the semantic type of `vacua` and emits an
ordinary `Construct` statement:

- `lista<T>` -> empty `MirAggregateKind::Array` with ordered fields.
- `tabula<K, V>` -> empty `MirAggregateKind::Map` with keyed fields.
- `copia<T>` -> empty `MirAggregateKind::Set` with ordered fields.

Alias normalization follows the existing MIR type helper. Unsupported
non-collection `vacua` shapes remain fail-closed with a specific aggregate MIR
diagnostic.

### Remaining MIR-Lowering Clusters

- Runtime/provider and user-defined method calls.
- `ad` provider blocks.
- Async `cede` and cursor iteration.
- Closures/callable values.
- Collection iteration for maps/text/cursors.
- Non-literal and enum `discerne`.
- Aggregate and optional validation gaps around object/member projections.
- Remaining operator/top-level declaration gaps such as `inter` and top-level
  const.

### Remaining Wasm-Emission Clusters

- Dynamic `ignotum` still blocks `examples/exempla/si/est.fab` after MIR.
- Collection mutation/index/contains intrinsics have MIR shapes but no Wasm host
  ABI emission yet.
- Full nullable ABI remains incomplete beyond the narrow handle coalesce subset.

### Remaining Host/Runtime Clusters

- Define map, set, text, cursor, and provider iterator ABI for `itera ex` and
  `itera de`.
- Provide real `faber_runtime` import implementations for assertion,
  conversion, panic, collection length, and future collection operations.
- Provide real `faber_text` comparison behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 019 Validation Log

- `cargo test -p radix lowers_vacua -- --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed.
- `./scripta/lint`: passed.

`cargo fmt --check` still reports pre-existing formatting drift in
`crates/radix/src/mir/lower.rs`; the Phase 019 edits were manually kept in the
same formatted style without accepting unrelated churn.

### Next Phase Candidate

The highest-value next compile-valid candidates are runtime/provider method
calls and collection method Wasm host ABI emission. Treat those as separate
policy phases: method lowering must remain MIR-based, and collection Wasm
support must define imports honestly before claiming instantiate or runnable
tiers.

## Phase 020 Update: Genus Method MIR Lowering

**Commit target**: Phase 020
**Change**: bodyful genus methods now lower through target-neutral MIR as
ordinary definition functions with an explicit receiver parameter. Calls such
as `receiver.method(args...)` resolve to direct MIR definition calls before the
runtime/provider fallback path. Wasm emission also handles the compile-valid
shapes newly exposed by lowering those method bodies: duplicate same-spelled
method names, aggregate projection assignments, narrow opaque `ignotum` handle
carriers, handle equality, and explicit `i64` to `f64` numeric coercion.

### Tier Counts After Phase 020

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 72/101
  Wasm emitted: 71/101
  compile-valid: 71/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Compile-Valid Delta

Measured compile-valid coverage increased from 63/101 to 71/101. MIR-lowered
coverage increased from 64/101 to 72/101, and Wasm-emitted coverage increased
from 63/101 to 71/101.

New compile-valid exemplars:

- `examples/exempla/abstractus/abstractus.fab`
- `examples/exempla/ego/ego.fab`
- `examples/exempla/genus/creo.fab`
- `examples/exempla/genus/methodi.fab`
- `examples/exempla/implet/implet.fab`
- `examples/exempla/nexum/nexum.fab`
- `examples/exempla/pactum/pactum.fab`
- `examples/exempla/sub/sub.fab`

Instantiate and run tiers remain at zero because `wasmtime` is unavailable on
PATH. This is still a skipped host/runtime tier, not a compiler, codegen, or
validator failure.

### Result

The MIR lowering context now records genus method targets by receiver struct
and method symbol. Method signatures are registered with the receiver as
parameter zero, and method bodies bind `ego` to that receiver local. Wasm sees
the result as ordinary functions and direct calls; no method-specific Wasm
shortcut or HIR bypass was added.

The Wasm text probe now disambiguates duplicate function names when multiple
definitions share a source spelling. It emits host-imported aggregate
projection setters for non-nested field, variant-field, and index assignments.
It also treats `ignotum` as an opaque aggregate handle for carrier/equality
purposes and supports the narrow `i64` to `f64` numeric coercion needed by
mixed numerus/fractus arithmetic.

`examples/exempla/si/est.fab` now reaches MIR but still stops at Wasm emission
with `MIR-to-WASM unsupported: numeric coercion I32 to AggregateHandle`. That
is a remaining dynamic-boxing/ABI gap, not a validation-tool failure.

### Remaining MIR-Lowering Clusters

- Runtime/provider and collection method calls, including `innatum`,
  `morphologia`, and syntax stress exemplars.
- `ad` provider blocks.
- Async `cede` and cursor/provider iteration.
- Closures/callable values.
- Collection iteration for maps/text/cursors.
- Non-literal and enum `discerne`.
- Aggregate and optional validation gaps around object/member projections,
  optional chains, named aggregates, and coalesce.
- Remaining operator/top-level declaration gaps such as `inter`, `intra`, map
  spread, and `verte` cast.

### Remaining Wasm-Emission Clusters

- Dynamic boxing/coercion for mixed primitive values flowing into `ignotum`
  handles.
- Nested aggregate projection assignments remain fail-closed.
- Full nullable ABI remains incomplete beyond the narrow handle coalesce subset.

### Remaining Host/Runtime Clusters

- Define map, set, text, cursor, provider iterator, and collection mutation ABI.
- Provide real `faber_runtime`, `faber_text`, and `faber_aggregate` host
  implementations before claiming instantiate or runnable behavior.
- Add a local instantiate/run host before measuring instantiate-valid,
  runnable, or behavior-checked tiers.

### Phase 020 Validation Log

- `cargo test -p radix genus_method -- --nocapture`: passed.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix mir -- --nocapture`: passed, 114 MIR-focused tests.
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above.
- `cargo test -p radix`: passed, 516 unit tests, 8 hygiene tests, and radix
  doc tests.
- `./scripta/lint`: passed.

### Next Phase Candidate

The current compile-valid coverage is 71/101, crossing the lower end of the
70-80% target while keeping instantiate/run tiers unclaimed. Further coverage
should come from honest ABI-bearing phases, especially runtime/provider method
lowering and collection/aggregate host imports, or from resolving the
remaining MIR validation clusters without weakening MIR validation.

## Phase 021 Update: Expected-Tier Regression Gate And Handoff

**Commit target**: Phase 021
**Change**: the ignored Wasm e2e harness now enforces expected tier floors for
the current exemplar corpus. All files default to at least
`FrontendAnalyzed`; the current 71 compile-valid exemplars must remain
`CompileValid` or better; `examples/exempla/si/est.fab` must remain
`MirLowered` or better.

### Tier Counts After Phase 021

```text
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 72/101
  Wasm emitted: 71/101
  compile-valid: 71/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

### Result

This phase does not chase new pass-count movement. It makes the Phase 020
baseline a real regression gate: if any expected compile-valid exemplar drops
to Wasm-emitted, MIR-lowered, frontend-only, or source-readable, the ignored
harness fails and prints the file, expected tier, reached tier, and failure
reason.

Improvements remain non-failing. A future phase that intentionally improves a
file should update the ledger and raise the expected floor in the harness.

### Completion Audit

- Compile-valid coverage is 71/101, inside the requested 70-80% first target.
- MIR-lowered and Wasm-emitted counts are higher than or equal to
  compile-valid in the expected shape: 72 MIR-lowered, 71 Wasm-emitted, 71
  compile-valid. The one MIR/Wasm delta is `si/est.fab`, blocked by dynamic
  boxing/coercion into an `ignotum` aggregate handle.
- Instantiate-valid, runnable, and behavior-checked tiers remain at 0/101 and
  are reported as host/runtime skipped because `wasmtime` is unavailable and
  no entrypoint/run policy exists.
- The Wasm path remains HIR -> typed HIR -> validated MIR -> Wasm text; the
  harness calls `lower_analyzed_unit_with_context` before
  `emit_wasm_text_probe_with_context`.
- Every completed factory phase has a delivery artifact and commit through
  Phase 021.
- The next remaining clusters are broad ABI/runtime or MIR validation efforts,
  not a small clearly high-value cleanup suitable for extending this run.

### Phase 021 Validation Log

- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`: passed and
  produced the tier counts above with expected-tier floors enforced.
- `cargo test -p radix mir -- --nocapture`: passed, 114 MIR-focused tests.
- `cargo test -p radix wasm -- --nocapture`: passed.
- `cargo test -p radix`: passed, 516 unit tests, 8 hygiene tests, and radix
  doc tests.
- `./scripta/lint`: passed.

### Handoff For Next Factory Run

Plan the next run around one coherent ABI-bearing surface rather than pass
count arithmetic:

- local Wasm host/runtime support for instantiate and run tiers;
- runtime/provider and collection method MIR lowering;
- collection and aggregate host imports for mutation, lookup, and iteration;
- dynamic `ignotum` boxing/coercion ABI;
- remaining MIR validation clusters for optional/object/member shapes.

Do not claim instantiate, runnable, or behavior-checked progress until the host
imports and entrypoint/run policy are implemented and measured.
