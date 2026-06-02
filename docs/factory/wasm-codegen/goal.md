# Wasm Codegen Factory Goal

**Status**: ready for factory assignment  
**Created**: 2026-06-02  
**Target Repo Worktree**: `/Users/ianzepp/work/ianzepp/faber-wasm-codegen`  
**Base Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Base Commit**: `5b544fda`  
**Factory Artifact Dir**: `docs/factory/wasm-codegen/`  
**Target Backend**: MIR-backed Wasm codegen in `crates/radix/src/mir/`  
**Primary Gate**: tiered Wasm exemplar harness added by this factory run  
**Commit Policy**: commit after each completed phase with passing focused and checkpoint validation

## Worktree Assignment

This factory goal is assigned to exactly this worktree:

```bash
/Users/ianzepp/work/ianzepp/faber-wasm-codegen
```

The assigned branch is:

```bash
factory/wasm-codegen
```

The worker must run implementation, validation, phase artifacts, ledgers, and
commits from that worktree. Do not run this goal from the main
`/Users/ianzepp/work/ianzepp/faber` checkout or from the Go worktree.

## Objective

Build the MIR-to-Wasm path toward practical coverage for the current
`examples/exempla/**/*.fab` corpus.

The goal is not immediate full execution support for every exemplar. The first
minimum is compile-valid Wasm output from the MIR path. The factory should then
push as many exemplars as practical through validation, instantiation, and
runtime execution. A strong first program target is 70-80% compile-valid
coverage, with lower but steadily improving instantiate/run tiers recorded
separately.

## Non-Negotiable Rules

- Use the MIR path. Do not bypass MIR by emitting Wasm directly from HIR.
- Keep MIR target-neutral. Do not put Wasm import names, stack-machine details,
  or host ABI policy into MIR nodes.
- Missing type information at MIR lowering time is an upstream bug.
- Unsupported MIR or Wasm shapes must fail explicitly.
- Preserve the HIR-to-Rust backend and existing MIR tests.
- Do not reshape the exemplar corpus merely to improve pass counts unless a file
  is proven not to be a standalone target-appropriate exemplar.
- Runtime and host gaps must be classified separately from compiler/codegen
  failures.

## Tiered Pass Definition

Every exemplar should be classified into the highest tier it reaches:

1. **Frontend analyzed**: lex, parse, semantic analysis, and typed HIR succeed.
2. **MIR lowered**: HIR lowers to validated MIR.
3. **Wasm emitted**: MIR emits Wasm text or binary without codegen errors.
4. **Compile-valid**: emitted Wasm validates or assembles with the chosen tool.
5. **Instantiate-valid**: the module can instantiate with available imports.
6. **Runnable**: the module can execute the expected entry path without
   unexpected traps or host errors.
7. **Behavior-checked**: runnable output matches a sibling `.expected` file or
   target-specific expected behavior metadata.

The minimum numeric goal applies to **compile-valid** coverage. Later tiers must
be reported honestly even when they lag.

The pass report should include:

```text
Wasm e2e exempla:
  frontend analyzed: <n>/<total>
  MIR lowered: <n>/<total>
  Wasm emitted: <n>/<total>
  compile-valid: <n>/<total>
  instantiate-valid: <n>/<total>
  runnable: <n>/<total>
  behavior-checked: <n>/<total>
```

## Factory Loop

This goal intentionally allows an open-ended number of phases. After each phase:

1. Save a phase delivery spec under `docs/factory/wasm-codegen/`.
2. Run focused MIR and Wasm tests for the touched area.
3. Run `cargo test -p radix mir` and `cargo test -p radix` unless the phase is
   docs-only.
4. Run the tiered Wasm exemplar harness when the phase changes MIR lowering,
   Wasm emission, validation, host imports, or exemplar classification.
5. Record new tier counts and remaining failure clusters in a ledger.
6. Commit the completed phase.
7. Choose the next phase from live failure clusters, not from stale docs.

Do not flatten all remaining failures into one mega-phase. Each phase should
target one coherent MIR/Wasm cluster such as primitive values, structured
control flow, aggregate layout, calls, runtime intrinsics, host imports, or
validation tooling.

## Baseline Phase

The first worker phase must establish the truthful tiered harness.

Required actions:

- Inspect the existing MIR, Wasm text, LLVM text, and Rust probe tests.
- Decide the local validation toolchain for Wasm text or binary. Prefer a
  tool already available or easy to detect; skip tiers with explicit toolchain
  diagnostics when unavailable.
- Add or extend an ignored tiered Wasm exemplar harness.
- Run the harness across `examples/exempla/**/*.fab`.
- Capture every failure path and classify by tier and probable root cause.
- Save the baseline as `docs/factory/wasm-codegen/baseline-ledger.md`.
- Keep production behavior changes out of the baseline phase unless a tiny
  harness-only helper is required.

## Candidate Phase Families

These are starting points, not a fixed schedule. Pick the next phase by current
failure evidence.

### MIR Lowering Coverage

- Fill target-neutral MIR lowering gaps for high-frequency exemplar constructs.
- Prioritize primitives, locals, direct calls, simple returns, blocks, `si`,
  `dum`, `itera`, `elige`, and `discerne` according to the baseline clusters.

Checkpoint: more exemplars reach validated MIR without weakening validation.

### Wasm Type and Value Model

- Define a conservative MVP mapping for Faber primitives into Wasm value types.
- Decide how `textus`, collections, dynamic values, and nullable values are
  represented at the compile-valid tier.
- Avoid pretending runtime-managed values are solved when only numeric MVP
  support exists.

Checkpoint: primitive and numeric exemplars emit compile-valid Wasm, while
unsupported heap/runtime values fail with clear diagnostics.

### Control Flow Emission

- Emit Wasm for MIR blocks, branches, loops, returns, unreachable paths, and
  switch-like forms.
- Keep Wasm stack discipline explicit and tested.

Checkpoint: representative branch and loop MIR tests validate, and related
exemplars advance at least one tier.

### Calls, Locals, and Function ABI

- Emit direct function calls, parameters, locals, temporaries, and return values.
- Keep entrypoint policy explicit.
- Reject unsupported failable alternate exits until a concrete Wasm ABI exists.

Checkpoint: multi-function primitive exemplars compile-valid and, where
possible, instantiate/run.

### Runtime Intrinsics and Host Boundary

- Define Wasm-side lowering for `nota`, `mone`, `mori`, `adfirma`, and simple
  runtime intrinsics.
- Use explicit imports or stubs only when the ABI is documented and validated.
- Keep host/runtime gaps classified separately from Wasm emitter bugs.

Checkpoint: intrinsic-heavy exemplars move beyond MIR-lowered or fail with
target-specific host-boundary diagnostics.

### Aggregates and Nullable Values

- Add layouts or explicit unsupported diagnostics for arrays, maps, sets,
  structs, enums, options, and dynamic `FaberValue`-like values.
- Prefer small valid subsets over broad invalid emission.

Checkpoint: compile-valid tier improves without hidden runtime lies.

### Harness Honesty and Tooling

- Keep tier counts strict and reproducible.
- Add expected-failure metadata only for target-inherent or runtime-host gaps,
  never for ordinary backend bugs.
- Preserve failure artifacts when useful for debugging generated Wasm.

Checkpoint: the harness is valuable for future long-running workers and does
not require reading raw logs to understand pass counts.

## Validation Commands

Use focused commands first, then broaden:

```bash
cargo test -p radix mir -- --nocapture
cargo test -p radix wasm -- --nocapture
cargo test -p radix <focused-wasm-or-mir-test-name> -- --nocapture
cargo test -p radix
cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture
./scripta/lint
```

If the final harness name differs, update this document and the baseline ledger
in the same phase that creates the harness.

## Completion Criteria

The factory run may stop successfully when:

- Compile-valid Wasm coverage reaches and sustains a 70-80% pass range or
  better on the current exemplar corpus.
- MIR-lowered and Wasm-emitted tier counts are higher than compile-valid count
  or all deltas are explained.
- Instantiate-valid and runnable counts are reported honestly with host/runtime
  blockers classified.
- No unexpected regression exists in `cargo test -p radix`.
- Every completed phase has a delivery artifact and a commit.

If 70-80% compile-valid coverage is reached early, continue only if the next
cluster is small and clearly high-value. Otherwise leave a concise handoff for
the next factory run.

## Handoff Notes

Each long-running worker should begin from the assigned worktree:

```bash
cd /Users/ianzepp/work/ianzepp/faber-wasm-codegen
```

Treat this file as the goal document. Keep the live ledger and phase artifacts
inside `docs/factory/wasm-codegen/` in this worktree.
