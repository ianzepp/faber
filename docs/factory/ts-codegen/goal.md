# TypeScript Codegen Factory Goal

**Status**: ready for factory assignment  
**Created**: 2026-06-02  
**Target Repo Worktree**: `/Users/ianzepp/work/ianzepp/faber-ts-codegen`  
**Base Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Base Commit**: `32bc7819`  
**Factory Artifact Dir**: `docs/factory/ts-codegen/`  
**Target Backend**: HIR-backed TypeScript codegen in `crates/radix/src/codegen/ts/`  
**Primary Gate**: TypeScript exemplar e2e harness added by this factory run  
**Commit Policy**: commit after each completed phase with passing focused and checkpoint validation

## Worktree Assignment

This factory goal is assigned to exactly this worktree:

```bash
/Users/ianzepp/work/ianzepp/faber-ts-codegen
```

The assigned branch is:

```bash
factory/ts-codegen
```

The worker must run implementation, validation, phase artifacts, ledgers, and
commits from that worktree. Do not run this goal from the main
`/Users/ianzepp/work/ianzepp/faber` checkout or from the Go/Wasm worktrees.

## Objective

Build the HIR-to-TypeScript backend toward practical executable coverage for
the current `examples/exempla/**/*.fab` corpus.

The goal is not theoretical completeness. The factory run should first establish
a truthful TypeScript e2e measurement, then iteratively raise the number of
exemplar files that emit TypeScript, format cleanly when tools are available,
typecheck, and run without unexpected errors. A strong first program target is
70-80% of the current exemplar corpus reaching at least typecheck-valid and, as
runtime support improves, run-valid.

## Non-Negotiable Rules

- Use the existing HIR path. Do not route TypeScript through MIR as part of this
  goal.
- Keep target-specific policy in the TypeScript backend or driver target checks.
- Do not guess missing type information in codegen. Fix parser, lowering,
  semantic analysis, or typecheck when upstream data is missing.
- Unsupported TypeScript target shapes must fail explicitly with useful
  diagnostics.
- Preserve the Rust backend and existing Go/Wasm factory work.
- Keep phase edits scoped and commit only coherent completed slices.
- Do not reshape the exemplar corpus merely to improve pass counts unless a file
  is proven not to be a standalone target-appropriate exemplar.
- Do not treat formatter or TypeScript toolchain absence as backend success.
  Tool absence should be a skipped tier with an explicit reason.

## Tiered Pass Definition

Every exemplar should be classified into the highest tier it reaches:

1. **Frontend analyzed**: lex, parse, semantic analysis, and typed HIR succeed.
2. **TypeScript emitted**: Radix produces `Output::TypeScript`.
3. **Formatted**: generated TypeScript formats with available tooling, or the
   tier is skipped with an explicit missing-tool reason.
4. **Linted**: generated TypeScript passes available lint tooling, or the tier
   is reported separately while the backend is still stabilizing.
5. **Typecheck-valid**: generated TypeScript passes a no-emit typecheck.
6. **Runnable**: generated TypeScript executes under the chosen runtime without
   unexpected runtime errors.
7. **Behavior-checked**: runnable output matches a sibling `.expected` file or
   target-specific expected behavior metadata.

The first numeric goal applies to **typecheck-valid** coverage. Runtime and
behavior tiers should be reported honestly and improved after typecheck coverage
is meaningful.

The pass report should include:

```text
TypeScript e2e exempla:
  frontend analyzed: <n>/<total>
  TypeScript emitted: <n>/<total>
  formatted: <n>/<total> (<tool or skipped reason>)
  linted: <n>/<total> (<tool or advisory>)
  typecheck-valid: <n>/<total>
  runnable: <n>/<total>
  behavior-checked: <n>/<total>
```

## Tooling Policy

The baseline phase must detect and record the local TypeScript toolchain.

Formatter candidates:

```bash
prettier --parser typescript
deno fmt --ext ts -
```

Lint candidates:

```bash
biome check
eslint
```

Typecheck/runtime candidates:

```bash
deno check main.ts
deno run main.ts
tsc --noEmit main.ts
node main.js
```

Prefer the smallest dependable toolchain for the harness. If TypeScript must be
transpiled before Node execution, the harness should make that step explicit.
Do not use Bun for this factory unless a later phase documents a concrete reason
and the user approves it.

## Factory Loop

This goal intentionally allows an open-ended number of phases. After each phase:

1. Save a phase delivery spec under `docs/factory/ts-codegen/`.
2. Run focused TypeScript backend tests for the touched area.
3. Run `cargo test -p radix` unless the phase is docs-only.
4. Run the TypeScript e2e harness when the phase changes TypeScript lowering,
   semantic behavior that affects TypeScript, tooling, or exemplar
   classification.
5. Record new tier counts and remaining failure clusters in a ledger.
6. Commit the completed phase.
7. Choose the next phase from live failure clusters, not from stale docs.

Do not flatten all remaining failures into one mega-phase. Each phase should
target one coherent cluster such as optional values, dynamic values, class
methods, tagged unions, stdlib calls, async/cursor lowering, or runtime support.

## Baseline Phase

The first worker phase must establish the truthful TypeScript harness and
baseline.

Required actions:

- Inspect current `crates/radix/src/codegen/ts/` behavior and existing unit
  tests.
- Inspect `format_generated_code(Target::TypeScript)` and
  `lint_generated_code(Target::TypeScript)` to verify what tools are actually
  used.
- Detect the local formatter, linter, typechecker, and runtime.
- Add or extend an ignored TypeScript exemplar harness.
- Run the harness across `examples/exempla/**/*.fab`.
- Capture every failure path and classify by tier and probable root cause.
- Save the baseline as `docs/factory/ts-codegen/baseline-ledger.md`.
- Keep production behavior changes out of the baseline phase unless a small
  harness-only helper is required.

## Candidate Phase Families

These are starting points, not a fixed schedule. Pick the next phase by current
failure evidence.

### Harness Honesty and Tooling

- Make the TypeScript e2e harness strict once baseline metadata exists.
- Fail on unexpected failures and unexpected passes.
- Keep formatter, linter, typecheck, runtime, and stdout failures separate.
- Preserve useful generated artifacts for failure debugging when practical.

Checkpoint: the harness is repeatable, strict, and useful for future workers.

### Type and Value Shape

- Primitive mappings, nullable `T ∪ nihil`, `sponte`, `ignotum`, ad-hoc unions,
  and dynamic object/value representation.
- Avoid hiding Faber type mismatches behind TypeScript `any` unless a deliberate
  dynamic boundary is documented.

Checkpoint: typecheck-valid tier improves or failure reasons become more
target-specific.

### Collections and Stdlib Methods

- `lista`, `tabula`, `copia`, empty `vacua`, indexing, spread, string methods,
  and norma/stdlib method translations.
- Runtime helper code should be explicit and minimal.

Checkpoint: collection-heavy exemplars typecheck and run where runtime support
is available.

### Declarations and Calls

- Functions, optional/default parameters, classes/genus methods, constructors,
  `creo` hooks, interfaces, type aliases, consts, and enum/tagged-union
  constructors.

Checkpoint: representative `functio`, `genus`, `finge`, and `pactum` exemplars
reach typecheck-valid.

### Control Flow and Pattern Matching

- `si`, `elige`, `discerne`, `dum`, `itera`, `rumpe`, `perge`, expression-valued
  blocks, and match/pattern binding.

Checkpoint: control-flow exemplars typecheck and run or fail with explicit
target diagnostics.

### Async, Cursor, and Runtime Entrypoints

- `@ futura`, `cede`, cursor/yield-like forms, entry IIFEs, and runtime
  execution under the chosen TypeScript runtime.

Checkpoint: async/cursor exemplars move up the runnable tier without weakening
typecheck validation.

### Capability and Host Boundaries

- `ad` and HAL/provider calls should either lower to documented TypeScript
  runtime helpers or fail with explicit target diagnostics.
- Do not invent implicit network/file/process behavior for TypeScript output.

Checkpoint: provider-heavy exemplars are classified honestly and no unresolved
imports are hidden as successes.

## Validation Commands

Use focused commands first, then broaden:

```bash
cargo test -p radix codegen::ts -- --nocapture
cargo test -p radix <focused-ts-test-name> -- --nocapture
cargo test -p radix
cargo test -p radix exempla_ts_e2e -- --ignored --nocapture
./scripta/lint
```

If the final harness name differs, update this document and the baseline ledger
in the same phase that creates the harness.

## Completion Criteria

The factory run may stop successfully when:

- Typecheck-valid TypeScript coverage reaches and sustains a 70-80% pass range
  or better on the current exemplar corpus.
- Runnable coverage is reported honestly with runtime/tooling blockers
  classified.
- Remaining failures are grouped with clear root causes and next-phase
  recommendations.
- No unexpected regression exists in `cargo test -p radix`.
- Every completed phase has a delivery artifact and a commit.

If 70-80% typecheck-valid coverage is reached early, continue only if the next
cluster is small and clearly high-value. Otherwise leave a concise handoff for
the next factory run.

## Handoff Notes

Each long-running worker should begin from the assigned worktree:

```bash
cd /Users/ianzepp/work/ianzepp/faber-ts-codegen
```

Treat this file as the goal document. Keep the live ledger and phase artifacts
inside `docs/factory/ts-codegen/` in this worktree.
