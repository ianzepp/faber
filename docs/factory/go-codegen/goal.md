# Go Codegen Factory Goal

**Status**: ready for factory assignment  
**Created**: 2026-06-02  
**Target Repo Worktree**: `/Users/ianzepp/work/ianzepp/faber-go-codegen`  
**Base Repo**: `/Users/ianzepp/work/ianzepp/faber`  
**Base Commit**: `5b544fda`  
**Factory Artifact Dir**: `docs/factory/go-codegen/`  
**Target Backend**: HIR-backed Go codegen in `crates/radix/src/codegen/go/`  
**Primary Gate**: `cargo test -p radix exempla_go_e2e -- --ignored --nocapture`  
**Commit Policy**: commit after each completed phase with passing focused and checkpoint validation

## Worktree Assignment

This factory goal is assigned to exactly this worktree:

```bash
/Users/ianzepp/work/ianzepp/faber-go-codegen
```

The assigned branch is:

```bash
factory/go-codegen
```

The worker must run implementation, validation, phase artifacts, ledgers, and
commits from that worktree. Do not run this goal from the main
`/Users/ianzepp/work/ianzepp/faber` checkout or from the Wasm worktree.

## Objective

Build the HIR-to-Go backend toward practical executable coverage for the current
`examples/exempla/**/*.fab` corpus.

The goal is not theoretical completeness. The factory run should iteratively
raise the number of exemplar files that compile to Go, compile through the Go
toolchain, and run without unexpected errors. A strong first program target is
70-80% of the current exemplar corpus passing end-to-end.

## Non-Negotiable Rules

- Use the existing HIR path. Do not route Go through MIR as part of this goal.
- Keep target-specific policy in the Go backend or driver target checks.
- Do not guess missing type information in codegen. Fix parser, lowering,
  semantic analysis, or typecheck when upstream data is missing.
- Unsupported Go target shapes must fail explicitly with useful diagnostics.
- Preserve the Rust backend. Do not trade Rust correctness for Go progress.
- Keep phase edits scoped and commit only coherent completed slices.
- Do not reshape the exemplar corpus merely to improve pass counts unless a file
  is proven not to be a standalone target-appropriate exemplar.

## Pass Definition

For each `.fab` exemplar:

1. Radix must produce Go output for `Target::Go`.
2. Generated Go must pass the e2e harness compile/run path.
3. Runtime failure is a pass only when recorded as explicit expected-failure
   metadata with a target-specific reason.
4. Stdout is checked only where the exemplar has a sibling `.expected` file.

The pass count must be reported as:

```text
Go e2e exempla: <passed>/<total> exempla files pass end-to-end
Expected-output checks enabled for <n> exempla files
```

## Factory Loop

This goal intentionally allows an open-ended number of phases. After each phase:

1. Save a phase delivery spec under `docs/factory/go-codegen/`.
2. Run the focused tests for the touched area.
3. Run `cargo test -p radix` unless the phase is docs-only.
4. Run the Go e2e harness when the phase changes Go lowering, semantic
   behavior that affects Go, or exemplar classification.
5. Record the new pass count and the remaining failure clusters in a ledger.
6. Commit the completed phase.
7. Choose the next phase from the live failure clusters, not from stale docs.

Do not flatten all remaining failures into one mega-phase. Each phase should
target one coherent cluster such as optional values, struct methods, pattern
matching, dynamic values, stdlib methods, or package/runtime imports.

## Baseline Phase

The first worker phase must be a read-only baseline and clustering phase.

Required actions:

- Run `cargo test -p radix exempla_go_e2e -- --ignored --nocapture`.
- Capture the pass count, total count, and expected-output count.
- Capture every failure path and classify by failure kind:
  - frontend compile diagnostic,
  - Go codegen diagnostic,
  - `gofmt` or lint failure,
  - `go run` compile failure,
  - runtime failure,
  - stdout mismatch.
- Group failures by probable root cause and backend module.
- Save the baseline as `docs/factory/go-codegen/baseline-ledger.md`.
- Do not modify production code in the baseline phase.

## Candidate Phase Families

These are starting points, not a fixed schedule. Pick the next phase by current
failure evidence.

### Type and Value Shape

- Primitive conversions and numeric promotion.
- `textus`, `numerus`, `fractus`, `bivalens`, `nihil`, and `vacuum` mapping.
- Nullable `T ∪ nihil` and `sponte` storage.
- Dynamic `ignotum` and ad-hoc union lowering.

Checkpoint: a focused Go backend test plus improved or unchanged e2e pass count.

### Collections and Iteration

- `lista`, `tabula`, `copia` literals and `vacua`.
- Indexing, key iteration, value iteration, ranges, spread, and stdlib methods.
- Runtime shapes for object literals and map member access.

Checkpoint: targeted collection tests and e2e failures in the collection cluster
decrease.

### Declarations and Calls

- Functions, methods, receivers, constructors, defaults, optional parameters,
  spread calls, and ownership-like clone policy where needed for Go values.
- Struct fields, `creo` hooks, enum variants, and trait/interface surfaces.

Checkpoint: generated Go compiles for representative `functio`, `genus`,
`finge`, and `pactum` exempla.

### Control Flow and Pattern Matching

- `si`, `elige`, `discerne`, `dum`, `itera`, `rumpe`, `perge`, and expression
  valued control flow.
- Exhaustive and wildcard match behavior.

Checkpoint: control-flow exempla compile and run or fail with explicit target
diagnostics.

### Runtime and Stdlib Bridges

- `nota`, `mone`, `mori`, `adfirma`, string templates, conversions, and norma
  calls that can be represented in Go without package-mode support.
- Reject or isolate runtime/provider calls that need unresolved external
  support.

Checkpoint: runtime helper output is deterministic and e2e failures caused by
missing helper code decrease.

### Test Harness Honesty

- Improve Go e2e metadata when failures are target-inherent rather than backend
  bugs.
- Add `.expected` files only when behavior is stable and valuable to assert.
- Keep the harness strict about unexpected failures and unexpected passes.

Checkpoint: the e2e harness remains truthful and useful as a gate.

## Validation Commands

Use focused commands first, then broaden:

```bash
cargo test -p radix codegen::go -- --nocapture
cargo test -p radix <focused-go-test-name> -- --nocapture
cargo test -p radix
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
./scripta/lint
```

Run `go version` before diagnosing toolchain failures as compiler failures.

## Completion Criteria

The factory run may stop successfully when:

- Go e2e reaches and sustains a 70-80% pass range or better on the current
  exemplar corpus.
- Remaining failures are classified with clear root causes and next-phase
  recommendations.
- No unexpected regression exists in `cargo test -p radix`.
- Every completed phase has a delivery artifact and a commit.

If 70-80% is reached early, continue only if the next cluster is small and
clearly high-value. Otherwise leave a concise handoff for the next factory run.

## Handoff Notes

Each long-running worker should begin from the assigned worktree:

```bash
cd /Users/ianzepp/work/ianzepp/faber-go-codegen
```

Treat this file as the goal document. Keep the live ledger and phase artifacts
inside `docs/factory/go-codegen/` in this worktree.
