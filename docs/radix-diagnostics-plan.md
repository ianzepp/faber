# Radix Diagnostics Plan

Internal delivery plan for improving `compilers/radix-rs` diagnostics without attempting to make the `rivus` codebase pass semantic checking.

Last updated: 2026-04-10

---

## 1. Interpreted Problem

### Claimed problem

Improve `radix-rs` diagnostics so they are genuinely useful when checking difficult codebases such as `compilers/rivus`.

### Inferred actual problem

The main blocker is not only diagnostic correctness. `radix-rs` already has the start of a real diagnostics layer, but the CLI still emits too much ad hoc text and too little structured context:

- single-file diagnostics are too sparse for fast triage
- batch failures degrade into a wall of repeated messages
- parser errors are too local to explain what construct the parser thought it was in
- the existing diagnostics infrastructure is not yet the canonical CLI output path

### Confidence

High.

### Non-goal

Do **not** try to make `rivus` pass in this effort. The target is diagnostic quality, not compatibility.

---

## 2. Product Decision

Treat diagnostics as a first-class compiler output contract.

`radix-rs` should follow the same broad split that users expect from `cargo check` / `rustc`:

1. each diagnostic is precise, contextual, and individually readable
2. each run ends with a short aggregate summary

The compiler should not dump a giant undifferentiated error wall and call that good enough.

---

## 3. Current State

### Existing strengths

`radix-rs` already has a diagnostics subsystem:

- [`compilers/radix-rs/src/diagnostics/diagnostic.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/diagnostic.rs)
- [`compilers/radix-rs/src/diagnostics/render.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/render.rs)
- [`compilers/radix-rs/src/diagnostics/catalog.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/catalog.rs)

The data model already supports:

- severity
- code
- file
- span
- help text
- one captured source line

### Existing weaknesses

The CLI still bypasses this in important paths:

- [`compilers/radix-rs/src/main.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/main.rs)

Today the output often falls back to plain `eprintln!` strings such as:

- `expected identifier`
- `expected expression`
- `name does not refer to a type`

That is directionally correct, but weak for broad compatibility passes.

### Evidence from rivus sweep

A per-file `check` sweep across `compilers/rivus/**/*.fab` produced:

- `total=137`
- `failed=132`

The messages were useful enough to prove broad incompatibility, but not useful enough to cluster failures quickly or identify the dominant grammar families with confidence.

---

## 4. Design Goals

The improved diagnostics system should:

1. render precise line/column locations for human output
2. show source snippets and highlight spans consistently
3. distinguish single-file readability from batch-run readability
4. support explicit output modes instead of one overloaded text stream
5. preserve machine-readable extension paths later without forcing them first
6. improve parser wording where it matters most, but only after transport and rendering are fixed

---

## 5. Output Contract

### Human mode

Default mode should behave like a compact `rustc`-style diagnostic stream:

- severity
- diagnostic code when available
- message
- file, line, column
- source snippet
- underline / label
- optional help / note

Each diagnostic should stand on its own.

### Short mode

Short mode should produce one line per diagnostic:

```text
error[PARSE012]: compilers/rivus/foo.fab:12:8: expected identifier
warning[SEM031]: compilers/rivus/bar.fab:44:3: unused local
```

This is for CI logs, filtering, and large sweeps.

### Session summary

Every run should end with a concise summary, analogous to `cargo check`:

- files checked
- files failed
- total errors
- total warnings
- top error categories by count

Example shape:

```text
check failed: 132/137 files failed, 842 errors, 17 warnings
top categories: expected identifier (211), expected expression (173), name does not refer to a type (146)
```

### Batch behavior

Batch mode should not invent a new narrative format. It should:

1. emit normal diagnostics
2. optionally suppress repetition in non-verbose mode
3. finish with grouped counts and representative examples

In non-verbose mode, show:

- grouped counts by code/message family
- first few example files for each category

In verbose mode, show full detail for every diagnostic.

### JSON mode

Optional later.

Do not make JSON the first milestone unless another tool is already waiting for it. The immediate need is stronger human triage.

---

## 6. Recommended Diagnostic Model Changes

Extend `Diagnostic` to carry richer location and grouping data.

### Required additions

1. line
2. column
3. optional end_line
4. optional end_column
5. phase/category marker
6. optional construct context

### Why

Current byte offsets and one captured line are not enough for:

- batch summaries
- precise short-mode formatting
- clearer parser context

### Notes

This does not require redesigning the whole compiler. It requires moving more location knowledge into the canonical diagnostic object.

---

## 7. Cargo Check Analogue

The intended model is:

1. compiler owns precise atomic diagnostics
2. CLI owns session summary and presentation mode

This mirrors `rustc` + `cargo check`:

- `rustc` emits exact errors with spans and context
- `cargo` provides run-level framing and a final summary

`radix-rs` should do the same:

- keep diagnostics atomic
- add a short end-of-run aggregate
- avoid giant prose reports

---

## 8. Stage Graph

### Stage A: Contract Freeze

Scope:

- define `human` and `short` output modes
- define required fields for all diagnostics
- define required fields for session summaries

Exit criteria:

- output contract written down
- no ambiguity about what a CLI command must print

### Stage B: Diagnostic Model Upgrade

Scope:

- extend `Diagnostic` with line/column and grouping fields
- centralize span-to-line/column mapping

Exit criteria:

- all diagnostics can render precise file:line:column locations
- plain/short format no longer relies on byte offsets alone

### Stage C: Renderer Upgrade

Scope:

- strengthen [`render.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/render.rs)
- ensure human output uses the richer model consistently
- bring plain rendering into parity with the contract

Exit criteria:

- parse, semantic, and IO diagnostics render in the same style
- source snippet + location + help all appear consistently

### Stage D: CLI Integration

Scope:

- route `check`, `emit`, `build`, `parse`, and `hir` through shared reporting helpers
- remove ad hoc `eprintln!` diagnostics where practical

Exit criteria:

- `main.rs` no longer defines its own competing diagnostic formats
- compiler diagnostics flow through one reporting path

### Stage E: Batch Summary

Scope:

- add grouped end-of-run summaries
- add category counting and representative examples
- keep full detail behind `--verbose`

Exit criteria:

- large runs are scannable
- top failure families are obvious without reading every line

### Stage F: Parser Message Improvement

Scope:

- target the most common parse failures found in `rivus`
- add construct-aware wording where possible

Examples:

- instead of only `expected identifier`
- prefer `expected identifier in genus field declaration`

Exit criteria:

- the top recurring parse errors become materially more specific

---

## 9. Workstreams

### Workstream 1: Diagnostic Core

Primary files:

- [`compilers/radix-rs/src/diagnostics/diagnostic.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/diagnostic.rs)
- [`compilers/radix-rs/src/diagnostics/catalog.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/catalog.rs)

Responsibilities:

- richer diagnostic payload
- stable codes/categories
- grouping keys

### Workstream 2: Rendering

Primary file:

- [`compilers/radix-rs/src/diagnostics/render.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/diagnostics/render.rs)

Responsibilities:

- human formatting
- short formatting
- session summaries

### Workstream 3: CLI Integration

Primary file:

- [`compilers/radix-rs/src/main.rs`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/main.rs)

Responsibilities:

- mode selection
- consistent command output
- batch reporting hooks

### Workstream 4: Parser Specificity

Primary area:

- [`compilers/radix-rs/src/parser`](/Users/ianzepp/github/ianzepp/faber/compilers/radix-rs/src/parser)

Responsibilities:

- improve high-frequency parse messages
- add construct-aware context

---

## 10. Verification Plan

### Required tests

1. single parse diagnostic snapshot
2. single semantic diagnostic snapshot
3. single IO diagnostic snapshot
4. short-mode snapshot
5. grouped batch summary snapshot

### Test corpus

Use:

- small curated invalid Faber fixtures
- a narrow `rivus` sample subset representing top failure categories

Do **not** put the whole `rivus` tree in a normal unit-test path.

### Manual verification

Run a `rivus` sweep after each checkpoint and confirm:

1. location fidelity improved
2. category grouping is meaningful
3. the top failure families are easier to identify

---

## 11. Checkpoints

### Checkpoint 1: Contract Freeze

Pass if:

- the human/short/session-summary contract is stable

### Checkpoint 2: Foundation Merge

Pass if:

- line/column rendering works
- CLI uses shared diagnostic reporting

### Checkpoint 3: Batch Readability

Pass if:

- a `rivus` sweep ends with useful grouped summaries
- repeated failures are no longer just a flat wall

### Checkpoint 4: Parser Improvement

Pass if:

- the top recurring parse diagnostics are materially more specific

---

## 12. Recommended First Issue

Implement:

`Make radix-rs CLI diagnostics line/column aware and route check/build/emit through the shared diagnostics renderer.`

Why this first:

- highest leverage
- improves every command immediately
- creates the foundation needed before batch summaries or parser message tuning

---

## 13. Explicit Non-Goals

Do not fold these into the first pass:

1. making `rivus` compile
2. making `check` package-aware
3. rewriting the entire parser wording surface at once
4. adding a complex JSON diagnostics protocol before human output is good
