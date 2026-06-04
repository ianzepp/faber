# LLVM Codegen Factory Continuation Plan

**Status**: refreshed after Phase 012 and main-branch integration; continue from remaining gaps only
**Created**: 2026-06-04
**Last refreshed**: 2026-06-04
**Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/llvm-codegen/`
**Historical Ledger**: [`baseline-ledger.md`](baseline-ledger.md)
**Current Focused Gate**: `cargo test -p radix llvm -- --nocapture`
**Current Harness**: `cargo test -p radix --lib exempla_llvm_e2e -- --ignored --nocapture`
**Companion Backend**: MIR-backed Wasm text lowering in
`crates/radix/src/mir/wasm_text.rs`

## Objective

Continue the MIR-backed LLVM text lowering work from the post-merge main branch
baseline. The first LLVM factory sequence, Phases 001-012, is complete and
should remain historical context rather than future work. Those phases created
the ledger, added the ignored e2e harness, wired target selection, expanded
scalar/CFG/call/runtime/text/aggregate/nullable/entrypoint/switch coverage, and
classified the large deferred surfaces.

Do not rerun Phases 001-012 as a continuation plan. Future work should pick one
remaining cluster below, create a new phase delivery spec, update live counts,
and preserve fail-closed diagnostics for unsupported shapes.

LLVM remains an experimental text probe, not a native executable toolchain
promise. Text emission, verifier-valid IR, linked native execution, and runtime
behavior are separate tiers and must not be conflated.

## Live Baseline

Verified on 2026-06-04 from the main worktree:

```text
LLVM focused tests:
  cargo test -p radix llvm -- --nocapture
  current focused gate covers 30 passing tests and 1 ignored execution test

LLVM e2e toolchain:
  verifier: unavailable (llvm-as/opt not found)
  execution/runtime: unavailable

LLVM exempla e2e:
  frontend analyzed: 102/102
  MIR lowered: 87/102
  LLVM emitted: 62/102
  verifier-valid: 0/102
  frontend failed: 0
  MIR lowering failed: 15
  unsupported diagnostic: 25
  emission failed: 0
  output write failed: 0
  verifier failed: 0
```

The e2e harness currently treats missing verifier/runtime tooling as explicit
zero-count tiers. No future plan should claim verifier-valid or runnable LLVM
progress without installing or packaging the required toolchain and runtime.

## Completed And Protected Surface

- Driver target selection for `llvm-text`.
- Ignored exempla e2e harness with frontend, MIR, emission, unsupported,
  verifier, and failure buckets.
- Scalar integer, float, boolean, and vacuum functions.
- Scalar arithmetic, comparisons, boolean operations, casts, and direct returns.
- Multi-block scalar CFG with labels, branches, returns, loops, scalar
  branches, and scalar/boolean literal switches.
- Direct same-program calls with scalar arguments and results.
- LLVM runtime declarations for diagnostics, assert, panic, conversion, format,
  text, aggregate, nullable, and collection helper surfaces already used by
  supported MIR.
- Opaque `ptr` handle policy for text, aggregate-like values, and nullable
  values.
- Aggregate construction and single-step projection helpers when metadata
  proves the type.
- Nullable helper calls for none, some, predicates, unwrap, coalesce, and the
  currently supported option-projection operand shapes.
- Synthetic `@incipit` entry function emission.
- Explicit `MIR-to-LLVM unsupported` diagnostics for deferred provider,
  callable, external, failable, spread, text switch, option-chain, and
  layout-dependent shapes.

## Non-Negotiable Rules

- Keep LLVM lowering MIR-backed; do not bypass MIR from HIR or source.
- Keep MIR target-neutral. LLVM pointer policy, data layout, calling convention,
  symbol linkage, and runtime ABI belong in the LLVM emitter/runtime layer or in
  explicit layout metadata.
- Treat Wasm as an active design client of MIR. MIR changes for LLVM must remain
  lowerable by Wasm or fail closed in Wasm with explicit diagnostics.
- Do not add MIR shapes whose only valid interpretation is "emit this LLVM IR
  instruction." Represent source semantics in MIR and let each backend choose
  target-specific lowering.
- Missing type facts in MIR lowering or validation are upstream bugs.
- Unsupported MIR shapes must fail explicitly and remain visible in focused
  tests and e2e buckets.
- Do not claim LLVM-valid progress without a concrete verifier policy.
- Do not claim runnable/native progress without a concrete execution toolchain,
  entrypoint ABI, runtime linkage, and measured results.
- Keep Rust and Wasm gates green when LLVM changes shared MIR, validation,
  driver, or target-selection behavior.

## Factory Loop

For every future phase:

1. Write `docs/factory/llvm-codegen/phase-0NN-<topic>-delivery.md`.
2. Implement only the selected phase.
3. Run focused tests for touched MIR/LLVM/runtime code.
4. Run `cargo test -p radix llvm -- --nocapture`.
5. Run `cargo test -p radix mir -- --nocapture`.
6. Run `cargo test -p radix wasm -- --nocapture` when the phase changes shared
   MIR shape, validation, types, calls, control flow, projections, or runtime
   abstractions.
7. Run `cargo test -p radix --lib exempla_llvm_e2e -- --ignored --nocapture`.
8. Run `cargo test -p radix` and `./scripta/lint` unless the phase is docs-only.
9. Record live counts, changed failure clusters, and any Wasm implication.
10. Run a completion audit against the phase spec.
11. Commit the phase.

## Remaining Phase Set

### Phase L-013: Verifier Toolchain And IR Validity

Problem: emitted LLVM text is still not verifier-valid because this environment
does not provide `llvm-as` or `opt`. Verifier-valid floors remain zero even when
text emission succeeds.

Scope:

- Decide whether the project should depend on an external LLVM toolchain, a
  repo-owned validation dependency, or a documented skip policy.
- Capture verifier version and command when verifier-valid claims are made.
- Keep text-emission tests independent from verifier availability.
- Fix any IR spelling or type issues exposed by the verifier without weakening
  unsupported diagnostics.

Checkpoint:

- The harness distinguishes emitted text from verifier-valid IR with measured
  nonzero verifier counts, or it documents the skip policy explicitly.

### Phase L-014: Unsupported Operator Matrix

Problem: 25 exempla emit unsupported diagnostics after MIR lowering. Several
are ordinary operator surfaces on handles, option predicates, or less common
scalar operations rather than large runtime designs.

Scope:

- Add type-driven lowering for currently unsupported direct scalar operations,
  including bitwise and shift cases where MIR type facts are sufficient.
- Decide runtime-helper lowering for text/handle equality and concatenation, or
  keep them as precise unsupported diagnostics.
- Lower `IsNil` and `IsNonNil` operand shapes that the nullable helper ABI can
  represent.
- Recheck `vel/vel.fab` and coalesce on primitive nullable values before adding
  any ad hoc LLVM workaround.

Checkpoint:

- Unsupported diagnostics fall below 25 without introducing verifier-invalid
  IR or target-specific MIR assumptions.

### Phase L-015: Text, Pattern, And Dynamic Switch Dispatch

Problem: `elige`/`discerne` examples with text, aggregate, enum, or dynamic
subjects still need runtime comparison and dispatch policy. Literal scalar and
boolean switches are already covered.

Scope:

- Keep scalar literal switch support intact.
- Add runtime comparator-backed dispatch only when value type and comparator
  symbol are known.
- Preserve fail-closed behavior for layout-dependent patterns.
- Coordinate with Wasm if shared MIR pattern lowering changes.

Checkpoint:

- Text or enum-pattern dispatch progresses under a documented runtime policy,
  and unsupported dynamic patterns remain explicit.

### Phase L-016: Aggregate Spread And Nested Projection

Problem: list/object spread, some nested projection bases, and option-chain
field/index shapes still block LLVM emission after MIR succeeds.

Scope:

- Define runtime helper calls or layout policy for aggregate spread.
- Extend projection lowering only when base metadata proves the aggregate type.
- Recheck `destructura/objectum.fab`, `praefixum/praefixum.fab`,
  `lista/lista.fab`, `sparge/sparge.fab`, and
  `syntaxis/destructura-sparsa.fab`.
- Keep rich layout and ownership decisions explicit rather than inferred from
  source spelling.

Checkpoint:

- Supported aggregate/projection examples emit LLVM text, while unsupported
  spread/layout cases name the missing policy.

### Phase L-017: Shared MIR Failure Repair

Problem: 15 exempla still fail before MIR for both Wasm and LLVM. LLVM should
not add backend-local bypasses for those failures.

Scope:

- Coordinate with the Wasm remaining phases for provider, async/cursor,
  closure/callable, iterator/object, optional/default, generic, and aggregate
  validation fixes.
- Add LLVM tests only after a shared MIR shape exists.
- Keep LLVM e2e buckets reporting these as MIR-lowering failures, not LLVM
  emission failures.

Checkpoint:

- MIR-lowered coverage rises above 87/102 for both low-level backends or the
  remaining failures are earlier, clearer source diagnostics.

### Phase L-018: Provider, Async, Callable, And Failable ABI

Problem: provider/HAL calls, async/cursor functions, first-class callable
values, closures, and failable control flow require runtime, calling
convention, and alternate-exit design decisions larger than scalar text
emission.

Scope:

- Split provider, async/cursor, callable/closure, and failable control flow into
  separate delivery specs before implementation.
- Define the runtime symbol, state layout, and alternate-exit ABI for the
  selected surface.
- Preserve explicit unsupported diagnostics for unselected surfaces.
- Keep Wasm lowerability or fail-closed behavior visible for any shared MIR
  shape introduced here.

Checkpoint:

- One large deferred surface reaches a narrow ABI contract with focused tests,
  or remains documented as intentionally unsupported.

### Phase L-019: Native Execution And Runtime Linking

Problem: LLVM has no run tier. A native tier needs verifier tooling, object or
executable generation, runtime symbols, startup ABI, and process policy.

Scope:

- Start only after the verifier policy is settled.
- Decide CLI/runtime integration separately from text-emission coverage.
- Define startup around `incipit`, runtime library linkage, arguments, and
  diagnostics capture.
- Keep execution failures separate from verifier failures and emission
  unsupported diagnostics.

Checkpoint:

- The harness gains a measured native run tier, or the runtime-linking boundary
  is documented for a future factory plan.

## Current Failure Clusters

Re-run the ignored harness before choosing a phase. The 2026-06-04 remaining
clusters are:

- Shared MIR-lowering failures: `ad/ad.fab`, `cede/cede.fab`,
  `clausa/clausa.fab`, `functio/optionalis.fab`, `futura/futura.fab`,
  `generis/generis.fab`, `incipiet/incipiet.fab`, `itera/cursor-iteratio.fab`,
  `itera/de.fab`, `membrum/membrum.fab`, `objectum/objectum.fab`,
  `si/ergo-redde.fab`, `syntaxis/arena-mixta.fab`,
  `syntaxis/fluxus-cede.fab`, and `vocatio/vocatio.fab`.
- Unsupported binary operations or handle operations:
  `adfirma/adfirma.fab`, `assignatio/assignatio.fab`, `binarius/binarius.fab`,
  `discerne/discerne.fab`, `ego/ego.fab`, `omnia/omnia.fab`,
  `ordo/ordo.fab`, `redde/redde.fab`, `sub/sub.fab`, and
  `syntaxis/discerne-insanum.fab`.
- Unsupported switch/value dispatch: `elige/elige.fab`,
  `elige/elige-cum-defalta.fab`, and `elige/elige-sine-defalta.fab`.
- Unsupported nullable predicates and option shapes: `est/est.fab`,
  `si/est.fab`, `unarius/unarius.fab`, `ternarius/ternarius.fab`,
  `optionalis/optionalis.fab`, and `vel/vel.fab`.
- Unsupported spread/projection shapes: `lista/lista.fab`,
  `sparge/sparge.fab`, `syntaxis/destructura-sparsa.fab`,
  `destructura/objectum.fab`, and `praefixum/praefixum.fab`.

## Success Criteria For The Next Continuation

A successful future LLVM factory run should stop at one of these gates:

- LLVM emitted coverage rises materially above 62/102 with no emission failures.
- Unsupported diagnostics fall below 25 while staying precise.
- MIR-lowered coverage rises above 87/102 through shared frontend/MIR fixes.
- Verifier-valid coverage rises above zero under a concrete verifier policy.
- A provider, async, callable, failable, aggregate, or native execution phase
  reaches a clear ABI boundary and leaves explicit follow-up specs instead of
  broad TODOs.

Opus faciendum: make LLVM prove MIR, not distort it.
