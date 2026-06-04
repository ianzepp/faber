# Wasm Codegen Factory Continuation Plan

**Status**: ready for next factory assignment  
**Created**: 2026-06-04  
**Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Factory Artifact Dir**: `docs/factory/wasm-codegen/`  
**Baseline Gate**: `phase-021-regression-gate-handoff.md`  
**Current Harness**: `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`  
**Companion Backend**: experimental MIR-backed LLVM text probe in
`crates/radix/src/mir/llvm_text.rs`

## Objective

Continue the MIR-backed Wasm text lowering work after the first successful
compile-valid coverage gate. The prior run reached and protected the initial
70-80% compile-valid goal. The next factory run should make the Wasm path more
real by moving past compile-valid WAT into measured instantiation and execution,
while continuing to close high-value MIR-lowering and Wasm-emission gaps.

Any MIR changes made for this work should also preserve a clean path for the
LLVM text lowering implementation that follows. Wasm and LLVM can make different
layout, ABI, and runtime choices, but they should consume the same target-neutral
MIR facts wherever the source semantics are the same.

The plan is intentionally phase-oriented. Each phase should save its own
delivery spec, update the ledger with live harness counts, run focused and broad
validation, and commit only a coherent slice.

## Live Baseline

Verified on 2026-06-04 from this worktree:

```text
Wasm e2e toolchain:
  compile validator: wasm-tools validate
  instantiator/runtime: unavailable
Wasm e2e exempla:
  frontend analyzed: 101/101
  MIR lowered: 72/101
  Wasm emitted: 71/101
  compile-valid: 71/101
  instantiate-valid: 0/101
  runnable: 0/101
  behavior-checked: 0/101
```

Tooling:

- `wasm-tools 1.251.0` is available.
- `wasmtime` is not currently on `PATH`.
- The ignored e2e harness has expected-tier floors, so regressions below the
  Phase 021 baseline fail.

## Non-Negotiable Rules

- Keep Wasm lowering MIR-backed; do not bypass MIR from HIR or source.
- Keep MIR target-neutral. Wasm import names, host ABI details, and stack policy
  belong in the Wasm emitter/runtime layer.
- Treat LLVM lowering as a design client of MIR. New MIR nodes, terminators,
  type facts, projection forms, call forms, and runtime abstractions need a
  backend-neutral meaning that could be lowered to LLVM later.
- Do not add MIR shapes whose only valid interpretation is "call this Wasm
  import" or "use this Wasm stack trick." Represent source semantics in MIR and
  let Wasm/LLVM choose target-specific ABI lowering.
- Missing type facts in MIR lowering or validation are upstream bugs.
- Unsupported MIR shapes must fail explicitly and remain visible in the harness.
- Do not raise expected-tier floors unless the new tier is proven by the
  harness and recorded in the ledger.
- Do not claim instantiation, runnable, or behavior-checked progress without a
  concrete host/import/entrypoint policy and measured results.

## Factory Loop

For every phase:

1. Write `docs/factory/wasm-codegen/phase-0NN-<topic>-delivery.md`.
2. Implement only the selected phase.
3. Run focused tests for touched MIR/Wasm/runtime code.
4. Run `cargo test -p radix mir -- --nocapture`.
5. Run `cargo test -p radix wasm -- --nocapture`.
6. Run `cargo test -p radix llvm -- --nocapture` when the phase changes MIR
   shape, validation, type representation, calls, control flow, projections, or
   runtime abstractions.
7. Run `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`.
8. Run `cargo test -p radix` and `./scripta/lint` unless the phase is docs-only.
9. Update `baseline-ledger.md` or add a short phase result section with live
   counts and remaining clusters.
10. Record whether any MIR change has an LLVM follow-up implication.
11. Run a completion audit against the phase spec.
12. Commit the phase.

## Recommended Phase Set

### Phase 022: Host Tooling And Instantiation Harness

Problem: compile-valid coverage is protected, but every compile-valid exemplar
stops below instantiate-valid because no Wasm runtime is available locally.

Scope:

- Decide whether the factory should depend on a developer-installed `wasmtime`
  binary or a Rust dev-dependency host such as the `wasmtime` crate.
- Prefer repo-reproducible validation if adding the dependency is acceptable;
  otherwise document the external tool prerequisite and keep skips explicit.
- Extend the e2e harness so compile-valid modules with unresolved imports are
  classified into host/import buckets, not a single runtime-unavailable bucket.
- Do not implement Faber runtime imports yet except tiny no-op stubs needed to
  prove module instantiation honestly.

Checkpoint:

- Harness distinguishes no-runtime, missing-import, instantiation trap, and
  successful instantiation.
- Existing `71/101` compile-valid floor still passes.

### Phase 023: Minimal Import Stub Host

Problem: most compile-valid modules import `faber_diag`, `faber_text`,
`faber_aggregate`, or `faber_runtime`, so instantiation cannot advance without a
host surface.

Scope:

- Define a minimal host/import contract for current compile-valid WAT imports.
- Stub diagnostics, text handles, aggregate handles, assertions, panic, and
  conversions enough to instantiate modules without pretending full behavior is
  implemented.
- Keep handle values opaque and deterministic for tests.
- Record every import provided by the host in a small reference document or
  test fixture.

Checkpoint:

- A meaningful subset of the 71 compile-valid exemplars reaches
  instantiate-valid.
- Runnable and behavior-checked tiers remain unclaimed unless an entrypoint is
  actually invoked.

### Phase 024: Entrypoint And Run Policy

Problem: current modules do not have a user-facing execution policy. The harness
intentionally probes a missing export, so it cannot distinguish runnable
programs from merely instantiable modules.

Scope:

- Define the Wasm export shape for synthetic `incipit` functions and ordinary
  top-level functions.
- Export enough names for the harness to invoke simple entry programs.
- Keep CLI arguments, environment, filesystem, and HTTP out of this phase.
- Add a small behavior fixture mechanism only for examples that have stable
  observable output under the stub host.

Checkpoint:

- At least `salve-munde.fab`, primitive diagnostic examples, and simple function
  examples reach runnable.
- Behavior-checked is introduced only where expected output can be captured
  through the host.

### Phase 025: Runtime Method And Collection MIR Lowering

Problem: several remaining exemplars stop before MIR on collection or provider
method calls, especially `innatum`, `morphologia`, and syntax collection cases.

Scope:

- Extend target-neutral MIR lowering for known norma collection/runtime methods
  where analysis can resolve provenance and receiver type.
- Preserve genus-method direct calls from Phase 020.
- Keep provider/HAL `ad` blocks separate unless this phase explicitly handles
  their effect model.
- Do not lower by spelling alone; use library/provenance metadata.
- Model collection operations as semantic runtime/intrinsic operations, not
  Wasm imports. The same MIR should be lowerable later to LLVM calls, inline
  runtime helpers, or native data-structure operations.

Checkpoint:

- More exemplars move from frontend-analyzed to validated MIR.
- Unsupported provider/effectful calls still fail with specific diagnostics.
- The LLVM probe either keeps failing closed on the new runtime shapes or gains
  a narrow, target-appropriate lowering without changing the MIR contract.

### Phase 026: Optional And Member Shape Correctness

Problem: several examples fail MIR validation rather than Wasm emission:
optional chains on non-nullable bases, call argument mismatches, field
projection base mismatches, named aggregate field errors, and malformed
coalesce inputs.

Scope:

- Treat this as correctness mode, not pass-count grinding.
- Audit whether each failure is a real source diagnostic, a HIR typing bug, a
  MIR lowering bug, or a validator invariant that is too narrow.
- Fix root causes in the earliest responsible phase.
- Preserve enough MIR type/projection metadata for both Wasm opaque-handle
  lowering and future LLVM aggregate/nullable layout lowering.
- Add regression tests for representative failures:
  `functio/optionalis.fab`, `optionalis/optionalis.fab`, `vel/vel.fab`,
  `destructura/objectum.fab`, `membrum/membrum.fab`, and `vocatio/vocatio.fab`.

Checkpoint:

- Invalid MIR clusters either become valid MIR or earlier, clearer source/semantic
  diagnostics.
- No Wasm emitter guesses around missing type information.

### Phase 027: Non-Literal Pattern And Dynamic Switch Lowering

Problem: `discerne`, `omnia`, `ordo`, and syntax stress examples still stop on
non-literal patterns or unresolved switch subjects.

Scope:

- Extend MIR switch/pattern lowering only where semantics are already typed and
  target-neutral.
- Keep dynamic matching and destructuring explicit; do not encode Wasm-specific
  pattern policy in MIR.
- Reuse existing text/aggregate comparison imports where the Wasm emitter can
  honestly support the lowered shape.
- Keep switch and pattern MIR low enough that LLVM can eventually lower it to
  `switch`, branch chains, or runtime comparator calls depending on value type.

Checkpoint:

- Literal and simple non-literal pattern examples move at least to validated
  MIR, and compile-valid where imports already exist.

### Phase 028: Top-Level Declarations And Source-Order Initialization

Problem: `intra/intra.fab` stops on top-level consts; this is a program model
question, not a Wasm text formatting problem.

Scope:

- Define how top-level constants enter MIR: immutable globals, synthetic
  initialization, or explicit rejection depending on current language rules.
- Keep source-order initialization and side-effect rules explicit.
- Add MIR validation for the chosen shape before Wasm emission.
- Choose a MIR representation that can map to both Wasm globals/init functions
  and LLVM globals/init blocks without target-specific leakage.

Checkpoint:

- Top-level const examples no longer fail with a generic unsupported MIR
  diagnostic.

### Phase 029: Provider Blocks And HAL Boundary

Problem: `ad/ad.fab` and related future service/HTTP work require an effectful
host boundary. This is larger than ordinary runtime-method lowering.

Scope:

- Define the MIR representation for provider calls and effectful alternate
  exits needed by `ad` blocks.
- Map the minimal provider ABI into Wasm imports without committing to the full
  Component Model or Cloudflare Worker deployment story.
- Keep the MIR operation at the provider/effect boundary, not at the Wasm import
  boundary, so LLVM can lower the same operation to host runtime calls later.
- Keep HTTP, filesystem, process, and environment surfaces split into follow-up
  phases unless one is needed for a narrow exemplar.

Checkpoint:

- `ad` examples move from frontend-analyzed to explicit MIR/provider shapes or
  clearer unsupported host-boundary diagnostics.

### Phase 030: Async, Closures, And Deferred Language Surfaces

Problem: `cede`, `futura`, `incipiet`, cursor iteration, and closure examples
remain outside the current MIR/Wasm execution subset.

Scope:

- Do not start here unless earlier host/runtime phases are stable.
- Split async and closures into separate delivery specs.
- Prefer a design note plus explicit diagnostics before implementation if the
  runtime model is not settled.

Checkpoint:

- No broad implementation starts without a concrete lifecycle model and tests.

### Phase 031: LLVM Readiness Audit

Problem: the Wasm phases are likely to expand MIR faster than the LLVM text
probe. Before starting serious LLVM lowering, the repo needs a short audit that
separates good MIR growth from Wasm-shaped shortcuts.

Scope:

- Review MIR nodes, validation rules, and Wasm lowering added since Phase 021.
- Classify each new MIR shape as directly LLVM-lowerable, runtime-call-backed,
  layout-dependent, or intentionally deferred.
- Add fail-closed LLVM tests for important MIR shapes that should not silently
  miscompile yet.
- Identify the smallest LLVM text expansion phase, likely multi-block control
  flow, scalar calls, or `fractus` support.

Checkpoint:

- A durable LLVM follow-up note exists before LLVM implementation begins.
- The Wasm path keeps its measured tiers, and LLVM unsupported diagnostics
  remain explicit rather than accidental.

## Current Failure Clusters To Recheck Before Each Phase

The next worker should rerun the ignored harness and select from live data, but
the 2026-06-04 clusters are:

- Host/runtime unavailable: all `71/101` compile-valid exemplars stop below
  instantiate-valid.
- Wasm emission gap: `si/est.fab` reaches MIR and stops on numeric coercion from
  `I32` to `AggregateHandle`.
- Provider/effect MIR gaps: `ad` blocks.
- Async/callable MIR gaps: `cede`, `futura`, `incipiet`, cursor iteration, and
  closures.
- Runtime/provider method MIR gaps: `innatum`, `morphologia`, and syntax
  collection examples.
- Collection iteration MIR gaps: `itera/de.fab` and `si/ergo-redde.fab`.
- Optional/member/aggregate validation bugs: optional chains, field projection
  base mismatches, named aggregate fields, map aggregate values, and coalesce
  nullability.
- Pattern/switch gaps: non-literal `discerne`, `omnia`, `ordo`, and syntax
  stress cases.
- Top-level const: `intra/intra.fab`.
- Aggregate spread/cast gaps: `objectum/objectum.fab` and
  `ternarius/ternarius.fab`.

## Success Criteria For This Continuation

A successful next factory run should stop at one of these gates:

- A real instantiate-valid tier exists and is protected by expected floors.
- A runnable tier exists for simple `incipit` programs with host-captured
  diagnostics.
- MIR-lowered coverage rises materially above `72/101` without weakening
  validation.
- Compile-valid coverage rises materially above `71/101` and the new floors are
  recorded.
- A larger ABI phase reaches a clear design boundary and leaves explicit
  follow-up delivery specs rather than ambiguous TODOs.

Opus faciendum: make each tier mean what it says.
