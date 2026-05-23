# Radix Diagnostics Mode Plan

**Status**: constrained delivery plan  
**Updated**: 2026-05-23  
**Scope**: add a developer-facing diagnostics reporting mode without rewriting
all CLI output paths

## Purpose

Faber already has a diagnostics model, catalog, and renderer. The missing piece
is a focused way to answer: "why did this source fail, and in which compiler
phase?"

This plan replaces the older broad diagnostics plan. The first deliverable is
not a full `cargo check` clone, not a batch-reporting system, and not a parser
message rewrite. It is a bounded diagnostics mode that exposes the structured
information the compiler already has, then adds only the smallest model fields
needed to make that mode useful.

## Current State

Current diagnostics infrastructure:

- `crates/radix/src/diagnostics/diagnostic.rs`
- `crates/radix/src/diagnostics/catalog.rs`
- `crates/radix/src/diagnostics/render.rs`
- `crates/radix/src/driver/mod.rs`

The existing `Diagnostic` model already carries:

- severity,
- stable code,
- message,
- file,
- byte span,
- captured source line,
- help text.

The driver already normalizes lexer, parser, semantic, and selected backend
failures into `Diagnostic` values. Some CLI/tool commands still print ad hoc
messages directly, but fixing every output path is explicitly outside the first
slice.

## Product Decision

Add an explicit diagnostics mode for compiler and language debugging.

Normal user output should remain clean. Diagnostics mode should intentionally be
more verbose and more internal, because its job is debugging the compiler
pipeline rather than presenting polished end-user help.

The mode should make phase ownership obvious:

```text
error[PARSE022] parse src/main.fab:14:9 expected block or 'ergo'
phase: parse
span: 143..149
source: si ready tacet
help: use `{ ... }` for a block body or `ergo`/`∴` for a single statement
```

## First Deliverable

Add a diagnostics reporting mode for the most useful single-file paths:

- `radix check --diagnostics <file>`
- `radix emit --diagnostics <file>`
- `faber check --diagnostics <file>` if wiring through `faber` is cheap

If `faber` wiring is not cheap, keep the first slice to `radix` and record the
`faber` follow-up explicitly.

The first deliverable should:

1. Add a small CLI mode enum, such as `DiagnosticMode::{Normal, Diagnostics}`.
2. Add a shared reporting helper for normalized `Diagnostic` values.
3. Print the existing human diagnostic output in normal mode.
4. Print expanded diagnostic records in diagnostics mode.
5. Add a phase field to `Diagnostic` if phase cannot be inferred reliably from
   the existing code/catalog.

Do not add short mode, JSON mode, grouped summaries, batch suppression, or a
large parser wording pass in this slice.

## Diagnostics Mode Contract

Diagnostics mode should print one expanded record per diagnostic.

Required fields:

- severity,
- code when available,
- phase,
- message,
- file,
- byte span,
- source line when available,
- help text when available.

Nice-to-have fields if trivial:

- line and column,
- end line and end column,
- diagnostic kind/category.

Line/column is useful, but it should not block the first slice if byte span and
source line are already available. A follow-up can centralize byte-span to
line/column mapping.

## Phase Model

Use a small explicit phase enum:

```rust
pub enum DiagnosticPhase {
    Io,
    Lex,
    Parse,
    Resolve,
    Lower,
    Typecheck,
    Analysis,
    Mir,
    Codegen,
    Tool,
}
```

The exact names can follow existing module language. The important part is that
diagnostics mode tells the reader where to look next.

Initial phase assignment can be coarse:

- lexer errors -> `Lex`
- parser errors -> `Parse`
- semantic errors -> `Analysis` unless a more precise phase is already known
- MIR lowering/validation errors -> `Mir`
- codegen errors -> `Codegen`
- filesystem/tool errors -> `Io` or `Tool`

Do not block on perfectly splitting every semantic subphase. Coarse attribution
is already better than no attribution.

## Implementation Boundaries

### In Scope

- A diagnostics mode flag on the narrow command surface chosen for the first
  slice.
- A reporting helper that can render existing `Diagnostic` values in expanded
  form.
- A minimal `DiagnosticPhase` field or equivalent.
- Tests/snapshots for one parse diagnostic and one semantic diagnostic in
  diagnostics mode.

### Out Of Scope

- Making old archived/self-hosting code compile.
- Package-wide batch summaries.
- Grouped top-category reports.
- JSON output.
- Rewriting all `eprintln!` call sites.
- Rewording the parser catalog broadly.
- Changing compiler phase behavior.
- Full line/column redesign if the first slice can use byte spans.

## Suggested Stage Graph

### Stage 1: Contract And Flag

Add the CLI flag and mode enum for the selected commands. Keep normal behavior
unchanged when the flag is absent.

Exit criteria:

- `radix check --diagnostics <file>` selects diagnostics mode.
- Existing `radix check <file>` output remains unchanged.

### Stage 2: Expanded Renderer

Add a deterministic expanded renderer for `Diagnostic`.

Exit criteria:

- The renderer prints severity, code, phase, file, span, message, source line,
  and help.
- It handles missing file/span/source/help without panicking.

### Stage 3: Phase Attribution

Add minimal phase attribution at diagnostic construction points.

Exit criteria:

- Lex, parse, semantic, MIR, codegen, and IO/tool diagnostics have reasonable
  phase values where they are already normalized.
- Unknown or coarse phases are acceptable for the first slice if they are named
  honestly.

### Stage 4: Focused Verification

Add focused tests for diagnostics mode output.

Exit criteria:

- One parse failure snapshot proves phase/code/span/source/help output.
- One semantic failure snapshot proves semantic diagnostics are routed through
  the same expanded renderer.
- Existing normal-mode tests still pass.

## Follow-Ups

Only after the first slice lands, consider:

- `--diagnostics=short` for one-line CI output.
- `--diagnostics=json` for tooling.
- line/column fields on `Diagnostic`.
- package/batch summaries.
- grouped counts by code/message family.
- broader CLI output cleanup across `parse`, `hir`, `mir`, `emit`, `build`, and
  `faber`.
- parser wording improvements for high-frequency diagnostics.

## Recommended First Issue

Implement:

`Add radix check --diagnostics for expanded phase-aware diagnostics on a single file.`

Acceptance criteria:

- Normal `radix check <file>` behavior is unchanged.
- `radix check --diagnostics <file>` prints expanded diagnostic records.
- Expanded records include phase, severity, code, message, file, byte span,
  source line, and help when available.
- Tests cover at least one parse error and one semantic error.

This keeps the work useful without turning it into a repo-wide output rewrite.
