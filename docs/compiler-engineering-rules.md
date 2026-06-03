# Compiler Engineering Rules

This document preserves the durable parts of the old TypeScript compiler prompt
for the current Rust workspace. It is guidance for `crates/radix`, not a language
specification.

## Principle

Compiler code should make the language contract visible. Prefer explicit syntax
rules, typed phase boundaries, recoverable diagnostics, and fail-closed lowering
over convenient guesses.

Write each module as if a PL-theory reader and a production compiler engineer
were pair reviewing it: know which phase you are in, what invariants hold, handle
malformed input without panicking, document the WHY, and keep phases testable in
isolation where practical.

## Module Documentation

Large `crates/radix` modules should open with a Rust module doc that states:

- **Compiler phase** — lexical, syntactic, semantic, lowering, codegen, or runtime.
- **Role** — how the module fits the pipeline.
- **Input/output contract** — what the phase receives, produces, and which errors it may report.
- **Invariants** — facts later phases may rely on.
- **Grammar** (when applicable) — the EBNF production or note pointing at `EBNF.md`.

Parser helpers should carry `GRAMMAR:` notes for subtle productions. Codegen
helpers should carry `TARGET:` notes when behavior differs by backend. Use section
dividers for major responsibilities when a file spans multiple concerns.

## IR Design

- Preserve spans (and other source anchors) on syntax-shaped nodes for diagnostics.
- Prefer Rust enums and typed IDs over stringly node or diagnostic kinds.
- Keep parser output source-shaped; do not resolve names, infer types, or lower
  control flow in the parser.
- Keep semantic facts in HIR/MIR tables (`DefId`, `TypeId`, resolved structure),
  not duplicated ad hoc in codegen.

## Parser And Syntax

- Keep parser functions close to the grammar they implement. Add `GRAMMAR:` notes
  when a production is subtle, new, or easy to confuse with legacy syntax.
- Capture spans before consuming the token that proves a construct exists or
  fails. Diagnostics should point at the source contract that broke.
- Use enum-shaped syntax and token models for finite language surfaces. Do not
  represent node kinds, token kinds, target kinds, or diagnostic classes as raw
  strings when Rust can make the set explicit.
- Preserve source-shaped ASTs. Parsing should not guess semantic meaning that
  belongs to collection, resolution, lowering, typecheck, analysis, or codegen.
- Reject tempting aliases and legacy forms with diagnostics unless the current
  grammar explicitly accepts them.

## Diagnostics

- Collect errors where the phase can continue honestly. Stop only when the next
  phase cannot trust its input.
- Stable user-facing diagnostic codes belong in the diagnostics catalog. Phase
  internals may move, but codes and baseline help text should remain stable.
- Error messages should identify the broken contract. Help text should tell the
  user how to repair it, not restate the message.
- Tests should assert stable codes and meaningful fragments, not full prose when
  the exact wording is incidental.

## Semantic Phases

- Keep type information upstream. Missing HIR or MIR type data is a bug in
  analysis or lowering, not a reason for codegen to infer a fallback shape.
- Use `ignotum` only as the explicit escape hatch described by the language
  rules. Do not use it as nullability, inference, or error recovery.
- Empty collections need declared collection types. `vacua` must not become a
  backend-specific guess.
- Error sentinel nodes are poison. Later phases may carry them to report more
  diagnostics, but emission should fail rather than hide them.

## Lowering And Codegen

- Lowering should preserve semantic identity: `DefId`, `TypeId`, spans, and
  resolved structure are contracts between phases.
- Unsupported target surfaces should produce explicit diagnostics. Do not emit
  partial Rust, TypeScript, Go, or Faber output that pretends an unsupported
  construct was understood.
- Target backends may be pragmatic, but target-specific compromises must be local
  and documented with `TARGET:`, `EDGE:`, or `WHY:` comments when the behavior is
  not obvious.
- Shared codegen helpers should encode real cross-target structure, not paper
  over missing target semantics.

## Tests

- Every new syntax form needs parser coverage for success and at least one
  malformed case.
- Every new semantic rule needs a positive example and a diagnostic example.
- Every new codegen behavior needs output coverage for the target it changes.
- Prefer feeding a phase directly (tokens into the parser, HIR into a lowering
  helper) instead of always running the full pipeline when the contract under
  test is local.
- Malformed-input tests should assert recoverable continuation where the phase
  promises it: later statements or nodes still parse, and diagnostics are
  collected rather than panicking.
- Branch-heavy compiler code should be written so important paths are reachable
  from focused tests. If a branch cannot be tested, reconsider the structure.

## Pre-Commit Checklist

- Module or item docs state phase role and contracts where non-obvious.
- Parser changes reference grammar; codegen changes reference target behavior.
- Diagnostics use stable codes; help text teaches repair, not repetition.
- No panics on malformed input; error sentinels do not reach emission silently.
- Spans preserved on user-visible nodes; tests cover success and at least one
  failure path for new behavior.

## Style

- Prefer guard clauses over nested conditionals when rejecting invalid compiler
  state.
- Name complex loop conditions or helper predicates when the positive meaning is
  not obvious.
- Use comments to explain grammar, invariants, target constraints, or recovery
  policy. Avoid comments that merely narrate the next line.
- Keep modules readable in slices: phase docs at the top, section dividers for
  major responsibilities, and small helpers near the code that uses them.
