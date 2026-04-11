# Go Emitter Delivery Plan

Internal planning artifact for continuing `radix-rs` Go codegen work after verifier cleanup.

Last updated: 2026-04-10

---

## 1. Interpreted Problem

### Claimed problem

Continue the `radix-rs` Go emitter process and make progress against the real gate:

- build Go output from `examples/exempla/`
- verify emitted Go with the exempla build/verify scripts
- use Go codegen behavior, not just unit tests, as the source of truth

### Inferred actual problem

The main blocker is no longer build-script noise. The verifier now reports a much cleaner set of real emitter defects. The next phase is a semantic correctness push on the Go backend, not package-system infrastructure work.

### Evidence

- `bun run build:exempla:radix-go` is the operative gate.
- `scripta/build-exempla.ts` now uses package-aware verification for Go.
- Current gate result:
  - `4/131` emitted Go verification failures
  - no remaining compile-time codegen failures across `examples/exempla/`

### Confidence

High.

### Open questions

- Whether some JS/Bun-oriented exempla should be target-gated for Go rather than translated.
- Whether `proba` examples should emit runnable Go test functions or be considered unsupported for Go output.

---

## 2. Normalized Spec

### Project Frame

Improve `compilers/radix-rs` Go codegen until `bun run build:exempla:radix-go` fails only on intentionally unsupported or explicitly gated cases.

### Problem Statement

The Go backend exists and has unit coverage, but exempla verification still shows correctness gaps across casts, nullability, receiver/member lowering, variant construction, collection methods, and a small set of target-specific surfaces.

### Functional Requirements

1. Emit Go that compiles for self-contained exempla outputs.
2. Preserve current verifier behavior:
   - build self-contained single-file outputs
   - syntax-check package-shaped or import-dependent outputs
3. Reduce remaining real failures in `bun run build:exempla:radix-go`.
4. Keep `cargo test --manifest-path compilers/radix-rs/Cargo.toml` green.
5. Add or extend focused Go backend unit tests as each bug family is fixed.

### Non-Functional / Technical Constraints

1. Do not add full Go package/module support to the compiler in this wave.
2. Do not silently weaken the verification gate again.
3. Prefer emitter correctness over speculative support for every exotic target surface.
4. Preserve existing Rust/Faber/TS behavior.
5. Fix root causes in codegen or lowering, not emitted exempla by hand.

### Required Languages

- Rust for compiler work
- Go for output validation semantics
- Bun/TypeScript only for build/verify script support

### Open Questions

1. Which exempla should be treated as intentionally unsupported on Go?
2. Should Go `proba` lower into `_test.go`-style artifacts in a later wave?

---

## 3. Repo-Aware Baseline

### Project Frame

Active implementation lives in:

- `compilers/radix-rs/src/codegen/go/*.rs`
- `compilers/radix-rs/src/driver/*`
- `scripta/build-exempla.ts`

Operational gate:

- `bun run build:exempla:radix-go`

Supporting gate:

- `cargo test --manifest-path compilers/radix-rs/Cargo.toml`

### Hard Gates

1. `cargo test --manifest-path compilers/radix-rs/Cargo.toml`
2. `bun run build:exempla:radix-go`

### Constraint Decisions

#### Decision
Treat `build:exempla:radix-go` as the delivery truth for this wave.

#### Why
The remaining failures now reflect actual backend defects, not verifier mismatch.

#### Tradeoff
Work is driven by corpus behavior, so some fixes may feel less elegant than purely unit-test-driven work.

#### Decision
Defer full package support in the compiler.

#### Why
The user explicitly called that a bigger lift than is warranted right now.

#### Tradeoff
Some package-shaped examples can only be syntax-checked for now.

#### Decision
Prioritize semantic correctness over breadth.

#### Why
The remaining failures cluster around wrong Go, not missing optional polish.

#### Tradeoff
Some target-specific surfaces may remain gated rather than translated immediately.

### Architecture Discovery

Current likely ownership boundaries:

- `compilers/radix-rs/src/codegen/go/expr.rs`
  - casts
  - conversio
  - nullability/coalesce
  - member access
  - translated methods
- `compilers/radix-rs/src/codegen/go/stmt.rs`
  - iterator lowering
  - pattern/match execution
  - local/return statement behavior
- `compilers/radix-rs/src/codegen/go/decl.rs`
  - receivers
  - methods
  - import emission
  - declarations
- upstream HIR / semantic passes if codegen inputs are malformed

### Tradeoffs Accepted

1. Package-shaped Go output is currently syntax-validated instead of fully built.
2. Some exempla surfaces may be intentionally unsupported for Go in this wave.
3. Delivery will proceed in bug-family waves rather than by individual example files.

### Scope Boundaries

In scope:

- Go codegen correctness
- targeted verifier support
- unit test expansion
- exempla-driven bug retirement

Out of scope for this wave:

- full Go package/module graph support in compiler output
- broad runtime ecosystem design for Go
- large refactors outside Go backend needs

---

## 4. Current Failure Taxonomy

### A. Compile-time codegen failure

Resolved in the current wave.

### B. Real emitted Go verification failures

1. Interop stubs that are not valid Go
   - examples: `externa/externa.go`, `ad/ad.go`
   - current shape: JS/Bun-ish field or method access on `any`

2. Nullability / optional ternary result typing
   - examples: `ternarius/ternarius.go`
   - current shape: optional ternary lowering still mixes untyped `nil`, `string`, and `*string`

3. Map/member/index typing issues
   - examples: `membrum/membrum.go`
   - current shape: invalid chained indexing through `any`, mismatched collection element types

Resolved in the current wave:

- cast semantics (`qua`)
- conversio type-hint compile failure
- general coalesce / pointer-vs-value fixes (`vel`, `unarius`, `binarius`, `mori`)
- receiver/member lowering (`genus/*`, `vocatio`, `pactum`)
- variant construction / enum value semantics (`finge`, `ordo`)
- collection helper translation (`clausa`, `morphologia`, `innatum`)
- `proba` function-name sanitation
- fixed-arity spread-call recovery in `vocatio`

---

## 5. Stage Graph

### Stage A: Stabilize Type-Directed Expression Semantics

Goal:

- Fix expression forms that directly produce wrong Go types or illegal Go syntax.

Targets:

- `qua`
- `conversio`
- `vel`
- nil/nonnil checks
- pointer-vs-value returns
- `innatum` / collection conversion

Primary files:

- `compilers/radix-rs/src/codegen/go/expr.rs`
- `compilers/radix-rs/src/codegen/go/types.rs`
- possibly semantic/HIR support files if inputs are wrong

Exit criteria:

- `conversio/conversio.fab` compiles
- `qua`, `vel`, `unarius`, `ternarius`, `binarius`, `innatum`, `mori` stop failing in Go verify

Status:

- largely complete for this wave
- remaining Stage A work is `ternarius`

### Stage B: Stabilize Receiver / Member / Construction Semantics

Goal:

- Make methods, struct fields, and enum/variant construction map correctly to Go values.

Targets:

- method bodies using `self`
- instance field access
- `finge`
- `ordo`
- member access through typed structs/maps

Primary files:

- `compilers/radix-rs/src/codegen/go/decl.rs`
- `compilers/radix-rs/src/codegen/go/expr.rs`

Exit criteria:

- `genus/*`, `vocatio`, `pactum`, `finge`, `ordo`, `membrum` stop failing in Go verify

Status:

- receiver/member/value-construction work is complete except for the remaining dynamic-map cases in `membrum`

### Stage C: Stabilize Collection / Iterator Translation

Goal:

- Remove TS-style or placeholder collection behavior from Go output.

Targets:

- translated list/text methods
- iterator bindings
- cursor/range lowering

Primary files:

- `compilers/radix-rs/src/codegen/go/expr.rs`
- `compilers/radix-rs/src/codegen/go/stmt.rs`

Exit criteria:

- `clausa`, `morphologia`, and remaining iterator-driven examples stop failing

Status:

- complete for currently failing collection exempla

### Stage D: Target-Surface Policy

Goal:

- Decide what is supported, remapped, or gated for Go.

Targets:

- `externa`
- `ad`
- `proba`

Primary files:

- `compilers/radix-rs/src/codegen/go/*`
- maybe driver/diagnostic surfaces if explicit unsupported diagnostics are needed

Exit criteria:

- these surfaces either compile valid Go or fail with explicit, intentional diagnostics/gating

Status:

- `proba` is now syntactically valid Go
- `externa` and `ad` remain unresolved policy/runtime-shape work

---

## 6. Parallel Workstreams

### Workstream 1: Expression Semantics

Scope:

- `qua`
- `conversio`
- `vel`
- nil handling
- return coercion

Files:

- `compilers/radix-rs/src/codegen/go/expr.rs`
- `compilers/radix-rs/src/codegen/go/types.rs`
- relevant tests in `compilers/radix-rs/src/codegen/go/mod_test.rs`

### Workstream 2: Receiver / Construction Semantics

Scope:

- methods
- field access
- variant construction

Files:

- `compilers/radix-rs/src/codegen/go/decl.rs`
- `compilers/radix-rs/src/codegen/go/expr.rs`
- relevant tests in `compilers/radix-rs/src/codegen/go/mod_test.rs`

### Workstream 3: Iterator / Collection Translation

Scope:

- cursor/range
- collection methods
- unresolved loop bindings

Files:

- `compilers/radix-rs/src/codegen/go/stmt.rs`
- `compilers/radix-rs/src/codegen/go/expr.rs`

### Workstream 4: Unsupported-Surface Policy

Scope:

- `externa`
- `ad`
- `proba`

Files:

- `compilers/radix-rs/src/codegen/go/*`
- maybe `compilers/radix-rs/src/driver/*` if diagnostics need routing

Parallel safety note:

- `expr.rs` is shared territory. If multiple workers are used later, this file should have one owner at a time or changes should be serialized.

---

## 7. Checkpoints and Gates

### Checkpoint 1: Expression Semantics Stable

Required:

- `cargo test --manifest-path compilers/radix-rs/Cargo.toml`
- `bun run build:exempla:radix-go`
- `conversio` compile failure removed
- failure count materially below current `18/130`

Observed:

- achieved; current gate is `4/131`

### Checkpoint 2: Receiver / Construction Stable

Required:

- `cargo test --manifest-path compilers/radix-rs/Cargo.toml`
- Go backend tests expanded for method/member/variant behavior
- `genus`, `vocatio`, `pactum`, `finge`, `ordo` failures retired or intentionally gated

Observed:

- achieved for `genus`, `vocatio`, `pactum`, `finge`, `ordo`
- `membrum` remains as the last member-typing holdout

### Checkpoint 3: Iterator / Collection Stable

Required:

- no remaining `unresolved_def` in emitted Go exempla
- collection-method leftovers removed from Go output

### Checkpoint 4: Policy Freeze

Required:

- each remaining Go failure is either:
  - fixed
  - intentionally unsupported with explicit diagnostic
  - excluded by a documented target policy decision

---

## 8. Immediate Implementation Queue

Order of attack:

1. `conversio` compile failure
2. `qua` lowering
3. nullable/coalesce/value-vs-pointer semantics
4. receiver/member lowering
5. variant construction
6. collection methods and iterator bindings
7. interop/proba policy

This order is intentional:

- it maximizes retirement of real failures early
- it attacks shared semantic bugs before target-surface exceptions
- it keeps the gate honest

---

## 9. Delivery Handoff

This file is the current implementation baseline for continued Go emitter work.

Delivery orchestration should treat:

- this plan as the repo-aware baseline artifact
- `bun run build:exempla:radix-go` as the main implementation gate
- `cargo test --manifest-path compilers/radix-rs/Cargo.toml` as the safety gate

The next implementation wave should start at **Stage A: Stabilize Type-Directed Expression Semantics**.
