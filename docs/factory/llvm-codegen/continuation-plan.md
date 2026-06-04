# LLVM Codegen Factory Continuation Plan

**Status**: ready for first factory assignment  
**Created**: 2026-06-04  
**Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Factory Artifact Dir**: `docs/factory/llvm-codegen/`  
**Template Source**: `docs/factory/wasm-codegen/continuation-plan.md`  
**Current Focused Gate**: `cargo test -p radix llvm -- --nocapture`  
**Companion Backend**: MIR-backed Wasm text lowering in
`crates/radix/src/mir/wasm_text.rs`

## Objective

Begin the MIR-backed LLVM text lowering work as a separate backend lane while
the Wasm factory continues processing the broader MIR/Wasm plan. The LLVM lane
should use the Wasm factory document as its operating template: each phase gets
a durable delivery spec, live baseline evidence, focused and broad validation,
explicit unsupported-shape handling, and a coherent commit.

LLVM should start as an experimental text probe, not as a native executable
toolchain promise. The early goal is to prove that current MIR is a strong
enough shared contract for another low-level target and to feed useful pressure
back into MIR without importing LLVM-specific ABI, layout, or runtime shortcuts
into source-shaped or target-neutral compiler layers.

Any MIR changes made for LLVM must preserve the Wasm path. Wasm and LLVM can
choose different physical layouts, import/runtime ABIs, and validation tools,
but they should consume the same source-semantic MIR facts wherever the language
operation is the same.

## Live Baseline

Verified on 2026-06-04 from this worktree:

```text
LLVM focused tests:
  cargo test -p radix llvm -- --nocapture
  result: 2 passed, 0 failed

Current LLVM target shape:
  frontend/driver target: wired as `llvm-text`
  MIR lowering: `lower_analyzed_unit`
  emitter: `crates/radix/src/mir/llvm_text.rs`
  validation tool: none yet
  exempla e2e harness: none yet
  run/native execution tier: none
```

Current probe support is intentionally narrow:

- emits single-block integer scalar functions;
- supports `numerus`, `bivalens`, and `vacuum` type spelling;
- supports integer arithmetic `Add`, `Sub`, `Mul`, `Div`, and `Mod`;
- supports local/parameter operands and direct returns;
- rejects unsupported MIR shapes with `MIR-to-LLVM unsupported` diagnostics.

Current known gaps include:

- no LLVM IR verifier integration;
- no exempla e2e ledger or expected-tier floors;
- no multi-block CFG emission;
- no `fractus` constants or floating arithmetic;
- no direct call emission;
- no runtime-call, aggregate, nullable, projection, switch, provider, async, or
  closure lowering;
- no ABI or layout model beyond scalar probe values.

## Non-Negotiable Rules

- Keep LLVM lowering MIR-backed; do not bypass MIR from HIR or source.
- Keep MIR target-neutral. LLVM pointer policy, data layout, calling convention,
  symbol linkage, and runtime ABI belong in the LLVM emitter/runtime layer or in
  explicit layout metadata, not in source-shaped compiler nodes.
- Treat Wasm as an active design client of MIR. MIR changes for LLVM must remain
  lowerable by Wasm or fail closed in Wasm with explicit diagnostics.
- Do not add MIR shapes whose only valid interpretation is "emit this LLVM IR
  instruction." Represent source semantics in MIR and let LLVM choose the
  target-specific lowering.
- Missing type facts in MIR lowering or validation are upstream bugs.
- Unsupported MIR shapes must fail explicitly and remain visible in focused
  tests and any future e2e harness.
- Do not claim LLVM-valid progress without a concrete verifier policy. Text
  emission alone is not the same as verifier-valid IR.
- Do not claim runnable/native progress without a concrete execution toolchain,
  entrypoint ABI, and measured results.
- Keep the current Rust backend and Wasm factory gates green when LLVM changes
  shared MIR, validation, driver, or target-selection behavior.

## Factory Loop

For every phase:

1. Write `docs/factory/llvm-codegen/phase-0NN-<topic>-delivery.md`.
2. Implement only the selected phase.
3. Run focused tests for touched MIR/LLVM/runtime code.
4. Run `cargo test -p radix llvm -- --nocapture`.
5. Run `cargo test -p radix mir -- --nocapture`.
6. Run `cargo test -p radix wasm -- --nocapture` when the phase changes MIR
   shape, validation, type representation, calls, control flow, projections, or
   runtime abstractions.
7. Run any LLVM exempla e2e harness once it exists.
8. Run `cargo test -p radix` and `./scripta/lint` unless the phase is docs-only.
9. Update a ledger or phase result section with live counts and remaining
   clusters.
10. Record whether any MIR change has a Wasm follow-up implication.
11. Run a completion audit against the phase spec.
12. Commit the phase.

## Recommended Phase Set

### Phase 001: LLVM Readiness Audit And Baseline Ledger

Problem: LLVM has a target surface and two focused probe tests, but no durable
factory baseline. Starting feature lowering without classifying current MIR
would make LLVM chase Wasm changes blindly.

Scope:

- Review current MIR nodes, validation rules, Wasm lowering shapes, and the
  current LLVM probe.
- Classify each MIR shape as directly LLVM-lowerable, runtime-call-backed,
  layout-dependent, verifier-blocked, or intentionally deferred.
- Create `docs/factory/llvm-codegen/baseline-ledger.md` with current support
  facts, command output summaries, and explicit unsupported clusters.
- Add or preserve fail-closed tests for representative unsupported shapes.
- Identify the first implementation slice based on live evidence rather than
  assumed backend difficulty.

Checkpoint:

- A durable LLVM ledger exists.
- Unsupported MIR shapes are intentionally documented rather than accidental.
- `cargo test -p radix llvm -- --nocapture`, `cargo test -p radix mir -- --nocapture`,
  and `./scripta/lint` pass.

### Phase 002: LLVM Exempla E2E Harness

Problem: Wasm progress is measurable because it has a corpus harness. LLVM
currently only has focused unit tests, so pass counts and regressions are not
visible.

Scope:

- Add an ignored `exempla_llvm_e2e` harness modeled on the Wasm harness.
- Classify tiers honestly, starting with frontend analyzed, MIR lowered,
  LLVM emitted, and unsupported diagnostic.
- Add verifier-valid only if a local verifier tool is available or a
  repo-owned validation dependency is selected.
- Do not add execution/run tiers yet.
- Record expected floors only after a measured baseline exists.

Checkpoint:

- The LLVM harness can scan the same exempla corpus and report stable counts.
- The harness distinguishes MIR-lowering failures from LLVM-emission failures.
- No unsupported LLVM shape is counted as a compiler crash.

### Phase 003: Scalar Type And Operation Coverage

Problem: the current LLVM probe handles only integer scalar arithmetic and a few
primitive type spellings.

Scope:

- Add `fractus` constants and floating arithmetic with correct LLVM op spelling.
- Add scalar comparisons for `numerus`, `fractus`, and `bivalens` where MIR
  already carries enough type facts.
- Add boolean unary/binary operations that map directly to scalar LLVM IR.
- Keep text, aggregates, nullable values, and runtime calls out of this phase.

Checkpoint:

- Focused LLVM tests cover integer, float, and boolean scalar functions.
- The e2e LLVM emitted tier rises if Phase 002 exists.
- Wasm scalar behavior remains unchanged.

### Phase 004: Multi-Block Scalar CFG

Problem: LLVM currently rejects every MIR function with more than one basic
block. That blocks `si`, `dum`, short-circuiting, and many ordinary programs
that MIR already represents cleanly.

Scope:

- Emit LLVM labels for MIR basic blocks.
- Lower `Goto`, scalar `Branch`, and scalar `Return`.
- Preserve MIR block order for deterministic output while respecting
  terminator edges.
- Add focused tests for `si`, simple loops if already represented by MIR, and
  branch-return functions.
- Keep `Switch`, `TryCall`, and alternate-exit control flow separate.

Checkpoint:

- Multi-block scalar MIR no longer fails with `multiple basic blocks`.
- Unsupported terminators still fail with specific diagnostics.

### Phase 005: Direct Function Calls

Problem: useful scalar LLVM programs need direct calls between MIR functions,
but the current probe only returns local arithmetic.

Scope:

- Lower `MirStmtKind::Call` and direct `MirCallee::Function` calls for scalar
  argument and result types.
- Add declarations or fail-closed diagnostics for known external definitions.
- Preserve destination typing and reject value callees until callable-value MIR
  is ready.
- Add focused tests for scalar helper functions and call chains.

Checkpoint:

- Direct scalar function calls emit verifier-shaped LLVM text.
- External/user-value callees remain explicit unsupported cases.

### Phase 006: LLVM Verifier Policy

Problem: emitted LLVM text is not yet checked by an LLVM verifier, so the
factory cannot honestly claim LLVM-valid output.

Scope:

- Decide between an external `llvm-as`/`opt` prerequisite, a Rust crate, or a
  documented skip policy.
- Add verifier detection and tier classification if tooling is present.
- Keep emission tests independent from verifier availability.
- Record exact tool version and command in the ledger when verifier-valid tiers
  are claimed.

Checkpoint:

- The factory can distinguish emitted LLVM text from verifier-valid LLVM IR.
- Expected floors are raised only for measured verifier-valid output.

### Phase 007: Runtime Calls And Host Boundary

Problem: Wasm uses imports for diagnostics, text, aggregate, conversion, panic,
assert, and collection runtime operations. LLVM needs an equivalent target
policy without pushing Wasm import names into MIR.

Scope:

- Classify `MirIntrinsic` variants into direct lowering, runtime declaration,
  layout-dependent, and deferred groups.
- Lower diagnostics, assert, panic, conversion, and collection operations as
  calls to an LLVM-side Faber runtime ABI where the signature is known.
- Keep provider/HAL effects separate unless this phase explicitly defines the
  host boundary.
- Document every runtime symbol introduced by the LLVM emitter.

Checkpoint:

- Runtime-call-backed MIR either lowers to named LLVM runtime calls or fails
  with operation-specific diagnostics.
- Wasm import ABI remains unchanged.

### Phase 008: Text And Aggregate Handle ABI

Problem: text, arrays, maps, structs, and enum variants need a physical ABI
before LLVM can lower many MIR shapes. Wasm currently uses opaque handles and
runtime imports; LLVM needs an equally explicit policy.

Scope:

- Decide the first LLVM probe ABI for `textus` and aggregate values: opaque
  pointers, integer handles, or explicit runtime-owned structs.
- Attach any required physical representation through target-local lowering or
  explicit layout metadata, not by weakening semantic MIR types.
- Lower aggregate construction/projection only where layout and ownership are
  defined.
- Keep spread, dynamic values, and rich enum payload layout deferred unless the
  ABI is clear.

Checkpoint:

- The ABI decision is documented before broad aggregate lowering begins.
- Any implemented aggregate/text lowering has focused tests and verifier policy.

### Phase 009: Nullable And Optional Operations

Problem: MIR has explicit nullable operations, but LLVM needs a representation
for payload presence, nil, unwrap, coalesce, and optional chaining.

Scope:

- Choose representation separately for scalar nullable values and
  runtime-backed nullable handles if needed.
- Lower `None`, `Some`, `IsNil`, `IsNonNil`, `Unwrap`, and `Coalesce` only where
  the payload layout is known.
- Keep optional chains through fields, indexes, and calls dependent on the
  aggregate/call support phases.
- Add verifier-focused tests for scalar nullable operations before aggregate
  nullable operations.

Checkpoint:

- Nullable lowering is explicit and type-driven.
- Unsupported nullable shapes fail before emitting invalid LLVM.

### Phase 010: Switch, Pattern, And Failable Control Flow

Problem: `Switch`, pattern-driven branching, and failable calls need explicit
LLVM CFG lowering and runtime/error ABI decisions.

Scope:

- Lower literal scalar `Switch` to LLVM `switch` or branch chains.
- Keep text/aggregate/dynamic matching runtime-backed.
- Define `ReturnError` and `TryCall` lowering only after alternate-exit ABI is
  chosen.
- Add fail-closed tests for unsupported non-literal and layout-dependent
  patterns.

Checkpoint:

- Literal scalar switch lowering is tested.
- Failable control flow remains explicit, not silently erased.

### Phase 011: Top-Level Initialization And Entrypoint ABI

Problem: LLVM will eventually need globals, source-order initialization, and an
entrypoint shape, but the current probe only emits standalone functions.

Scope:

- Decide how top-level constants and synthetic initialization enter MIR or LLVM.
- Define the text target's export/entrypoint naming policy for `incipit`.
- Keep process arguments, environment, filesystem, and HTTP out of this phase.
- Coordinate with Wasm Phase 024/028 decisions where source semantics overlap.

Checkpoint:

- Top-level initialization and entrypoint policy are documented before run tiers
  are attempted.

### Phase 012: Provider, Async, Closures, And Deferred Surfaces

Problem: provider calls, `ad` blocks, async/cursor functions, and closures
cross runtime and calling-convention boundaries that are larger than scalar LLVM
lowering.

Scope:

- Prefer design notes and explicit unsupported diagnostics before implementation
  unless the runtime model is already settled.
- Split provider, async, and closures into separate future factory plans if
  implementation becomes substantial.
- Keep the LLVM probe useful by failing clearly on these surfaces.

Checkpoint:

- Deferred language surfaces are named and classified, not left as accidental
  unsupported errors.

## Current Failure Clusters To Recheck Before Each Phase

The next worker should rerun focused tests and, after Phase 002, the ignored
LLVM e2e harness. Current known clusters are:

- No e2e harness or ledger for LLVM target counts.
- No verifier policy for emitted LLVM text.
- Multi-block MIR rejected as `multiple basic blocks`.
- Float constants and `fractus` arithmetic rejected.
- Non-arithmetic scalar operations mostly rejected.
- Calls, runtime calls, constructs, projections, options, switches, and
  non-return terminators are unsupported.
- Text, aggregate, nullable, provider, async, closure, and entrypoint ABI
  decisions are not made.

## Success Criteria For This Continuation

A successful LLVM factory run should stop at one of these gates:

- A durable LLVM baseline ledger exists and classifies current MIR support.
- An ignored LLVM exempla e2e harness reports honest tiers and protects expected
  floors.
- Scalar LLVM emitted coverage rises materially without weakening MIR
  validation.
- A verifier-valid tier exists and is protected by expected floors.
- A runtime or layout phase reaches a clear ABI boundary and leaves explicit
  follow-up delivery specs rather than ambiguous TODOs.

Opus faciendum: make LLVM prove MIR, not distort it.
