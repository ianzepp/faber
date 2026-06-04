# Wasm Codegen Factory Continuation Plan

**Status**: refreshed after Phases 022-031; continue from remaining gaps only
**Created**: 2026-06-04
**Last refreshed**: 2026-06-04
**Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/wasm-codegen/`
**Historical Ledger**: [`baseline-ledger.md`](baseline-ledger.md)
**Current Harness**: `cargo test -p radix --lib exempla_wasm_e2e -- --ignored --nocapture`
**Companion Backend**: experimental MIR-backed LLVM text probe in
`crates/radix/src/mir/llvm_text.rs`

## Objective

Continue the MIR-backed Wasm text lowering work from the current measured
runtime-capable baseline. The earlier completion plan asked for host tooling,
stub imports, entrypoint policy, runtime method lowering, optional/member
correctness, dynamic switch support, top-level constants, provider planning,
async/closure triage, and LLVM readiness. Those items are either implemented,
protected by focused tests, or split into explicit remaining gaps below.

Do not rerun the old Phase 022-031 sequence as if it were still open. Future
factory work should pick one of the remaining clusters, save a new delivery
spec, run the live harness, and update this plan or the ledger with measured
movement.

Any MIR changes made for Wasm must keep LLVM as a design client. Wasm and LLVM
can choose different physical layout, host ABI, and runtime policies, but they
should consume the same target-neutral MIR facts wherever source semantics are
the same.

## Live Baseline

Verified on 2026-06-04 from the main worktree:

```text
Wasm e2e toolchain:
  compile validator: wasm-tools validate
  instantiator/runtime: wasmtime dev-dependency (in-process linker probe)

Wasm e2e exempla:
  frontend analyzed: 102/102
  MIR lowered: 87/102
  Wasm emitted: 87/102
  compile-valid: 87/102
  instantiate-valid: 87/102
  runnable: 84/102
  behavior-checked: 6/102

Wasm instantiation buckets, compile-valid subset, stubless linker:
  missing-import: 86
  instantiation-trap: 0
  instantiate-valid: 1
  no-runtime: 0

Wasm instantiation buckets, compile-valid subset, stub host:
  instantiate-valid: 87
  missing-import: 0
  instantiation-trap: 0

Wasm run buckets:
  runnable: 84
  no-entry-export: 1
  entry-trap: 2
```

Current target support is no longer just compile-valid WAT. The harness has a
repo-owned `wasmtime` host probe, stub imports, entrypoint execution for
`incipit`, trap buckets, and a small behavior fixture tier.

## Completed And Protected Surface

- WAT validation through `wasm-tools validate`.
- In-process `wasmtime` dev-dependency host probing.
- Stub host imports for diagnostics, text, aggregate handles, assertions,
  panic, conversion, format, and collection/runtime helpers.
- Export and run policy for synthetic `incipit` entrypoints.
- Scalar functions, direct calls, multi-block control flow, loops, branch
  dispatch, scalar switches, and numeric range iteration.
- Opaque text, aggregate, nullable, enum, and collection handle lowering.
- Runtime-backed text/aggregate comparisons and collection operations,
  including current `appende`, `prima`, `mappata`, and `filtrata` coverage.
- Optional helper lowering for nil, some, predicates, unwrap, coalesce, and
  supported chain/projection reads.
- Top-level constants and source-order initialization shapes currently covered
  by focused Wasm tests.
- Payload and unit enum construction, variant tests, payload extraction, and
  `discerne` coverage for the currently supported shapes.

## Non-Negotiable Rules

- Keep Wasm lowering MIR-backed; do not bypass MIR from HIR or source.
- Keep MIR target-neutral. Wasm import names, host ABI details, and stack policy
  belong in the Wasm emitter/runtime layer.
- Treat LLVM lowering as a design client of MIR. New MIR nodes, terminators,
  type facts, projection forms, call forms, and runtime abstractions need a
  backend-neutral meaning that LLVM can also reject or lower deliberately.
- Missing type facts in MIR lowering or validation are upstream bugs.
- Unsupported MIR shapes must fail explicitly and remain visible in the harness.
- Do not raise expected-tier floors unless the new tier is proven by the
  harness and recorded in a ledger or phase result.
- Do not claim behavior progress unless observable output or trap behavior is
  captured by the host and compared to a fixture.

## Factory Loop

For every future phase:

1. Write `docs/factory/wasm-codegen/phase-0NN-<topic>-delivery.md`.
2. Implement only the selected phase.
3. Run focused tests for touched MIR/Wasm/runtime code.
4. Run `cargo test -p radix mir -- --nocapture`.
5. Run `cargo test -p radix wasm -- --nocapture`.
6. Run `cargo test -p radix llvm -- --nocapture` when the phase changes shared
   MIR shape, validation, types, calls, control flow, projections, or runtime
   abstractions.
7. Run `cargo test -p radix --lib exempla_wasm_e2e -- --ignored --nocapture`.
8. Run `cargo test -p radix` and `./scripta/lint` unless the phase is docs-only.
9. Record live counts, changed failure clusters, and any LLVM implication.
10. Run a completion audit against the phase spec.
11. Commit the phase.

## Remaining Phase Set

### Phase W-032: Remaining MIR Validation And Call-Shape Bugs

Problem: 15 exempla still fail before validated MIR. The current cluster mixes
real deferred language surfaces with fixable call/default/generic/aggregate
shape bugs.

Scope:

- Classify each pre-MIR failure as semantic diagnostic, HIR typing issue, MIR
  lowering issue, or intentionally deferred language surface.
- Fix default/optional argument and call arity issues represented by
  `functio/optionalis.fab` and `vocatio/vocatio.fab`.
- Fix generic named aggregate field and map/object aggregate value issues where
  the source semantics are already typed.
- Keep provider, async/cursor, and closure lifecycle work out of this phase
  unless a narrow validation fix is required.

Checkpoint:

- Fixable examples move to validated MIR or earlier, clearer diagnostics.
- The Wasm emitter does not guess around missing type information.

### Phase W-033: Iterator And Collection Surface Completion

Problem: object/map iteration and some collection stress examples still stop
before MIR even though array/range iteration and several runtime collection
methods are already supported.

Scope:

- Implement or explicitly reject `itera de` object/map iteration using typed,
  target-neutral MIR operations.
- Recheck `si/ergo-redde.fab` and related iterator failures after any iterator
  shape changes.
- Preserve current `appende`, `prima`, `mappata`, and `filtrata` behavior.
- Keep captured closures and first-class callable values split unless the
  iterator design actually requires them.

Checkpoint:

- Object/map iteration either reaches MIR/Wasm under a clear runtime contract
  or fails with a source-level diagnostic that explains the unsupported shape.

### Phase W-034: Callable Values And Captured Closures

Problem: simple higher-order collection callback lowering exists, but
first-class callable values and closure/capture examples remain outside the
current execution subset.

Scope:

- Separate compile-time callback lowering from runtime callable values.
- Define capture layout and call dispatch before adding broad closure support.
- Add fail-closed tests for unsupported value callees and capture shapes.
- Coordinate with LLVM before introducing shared MIR callable representation.

Checkpoint:

- `clausa/clausa.fab` and `syntaxis/arena-mixta.fab` either progress under a
  designed callable model or keep precise unsupported diagnostics.

### Phase W-035: Provider And HAL Effect Boundary

Problem: `ad/ad.fab` still blocks before effectful MIR lowering. Provider/HAL
calls are a host boundary, not ordinary runtime methods.

Scope:

- Define target-neutral MIR for provider calls and effectful alternate exits.
- Map only the minimal Wasm import ABI required for the first provider
  exemplar.
- Keep HTTP, filesystem, process, and environment surfaces split unless one is
  required for the selected exemplar.
- Leave LLVM with either matching runtime-call lowering or explicit fail-closed
  diagnostics for the same MIR shapes.

Checkpoint:

- Provider examples move from frontend-analyzed to explicit MIR/provider shapes
  or clearer unsupported host-boundary diagnostics.

### Phase W-036: Async And Cursor Lifecycle

Problem: `cede`, `futura`, `incipiet`, cursor iteration, and flux syntax still
lack a settled suspension, yield, and scheduler/runtime model.

Scope:

- Write a design delivery note before implementation if lifecycle semantics are
  not already settled.
- Split async functions, cursor iteration, and `cede`/yield behavior into
  separately testable slices.
- Preserve current explicit diagnostics until the lifecycle model is real.

Checkpoint:

- No broad async or cursor implementation starts without lifecycle invariants,
  runtime expectations, and focused tests.

### Phase W-037: Run And Behavior Hardening

Problem: 84 examples are runnable, but behavior checking is still only 6/102.
Two examples instantiate then trap, and one lacks an entry export.

Scope:

- Classify `mori/mori.fab` and `ordo/ordo.fab` as expected traps or real runtime
  bugs.
- Classify `scalaria/scalaria.fab` as intentionally no-entry or add the correct
  entry export if source rules require it.
- Expand behavior fixtures for stable diagnostic and simple runtime outputs.
- Keep trap, no-entry, missing-import, and behavior mismatches as separate
  buckets.

Checkpoint:

- Unexpected traps are fixed or explicitly classified.
- Behavior-checked coverage rises above 6 without weakening the run tier.

### Phase W-038: Binary Runtime Packaging

Problem: the current backend validates and runs generated text modules through
the test host. It does not yet promise packaged `.wasm`, component-model, or
deployment-ready runtime artifacts.

Scope:

- Decide whether the next goal is `.wasm` binary output, component-model
  packaging, CLI integration, or real runtime library linkage.
- Keep this phase separate from language semantic gaps.
- Require measured validation and run evidence for any new artifact claim.

Checkpoint:

- The selected packaging/runtime surface has a narrow artifact contract and
  does not blur current WAT, stub-host, and behavior tiers.

## Current Failure Clusters

Re-run the ignored harness before choosing a phase. The 2026-06-04 remaining
clusters are:

- Provider/effect MIR gap: `ad/ad.fab`.
- Async/cursor gaps: `cede/cede.fab`, `futura/futura.fab`,
  `incipiet/incipiet.fab`, `itera/cursor-iteratio.fab`, and
  `syntaxis/fluxus-cede.fab`.
- Callable/closure gaps: `clausa/clausa.fab` and `syntaxis/arena-mixta.fab`.
- Optional/default/call-shape bugs: `functio/optionalis.fab` and
  `vocatio/vocatio.fab`.
- Generic/aggregate bugs: `generis/generis.fab`, `membrum/membrum.fab`, and
  `objectum/objectum.fab`.
- Iterator/object gaps: `itera/de.fab` and `si/ergo-redde.fab`.
- Run-tier follow-up: `mori/mori.fab` and `ordo/ordo.fab` trap on entry;
  `scalaria/scalaria.fab` instantiates but has no `incipit` export.

## Success Criteria For The Next Continuation

A successful future factory run should stop at one of these gates:

- MIR-lowered, Wasm-emitted, compile-valid, and instantiate-valid coverage rises
  materially above 87/102 without weakening validation.
- Runnable coverage rises above 84/102 with no unexplained traps.
- Behavior-checked coverage rises above 6/102 with host-captured expected
  output or expected trap fixtures.
- A provider, async, callable, or packaging phase reaches a clear design
  boundary and leaves explicit follow-up specs instead of broad TODOs.

Opus faciendum: spend the next phases on gaps that still exist.
