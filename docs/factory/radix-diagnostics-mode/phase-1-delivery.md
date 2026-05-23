# Phase 1 Delivery: `radix check --diagnostics`

## Interpreted Problem

The compiler already normalizes many failures into `Diagnostic`, but the developer CLI does not expose a focused mode that answers which compiler phase owns a failure. The first implementation slice should add a narrow diagnostics mode for single-file `radix check` without changing normal user output.

## Normalized Spec

Implement `radix check --diagnostics <file>` for single-file inputs.

Required behavior:

- Add a small command-mode enum for normal versus expanded diagnostics output.
- Preserve current `radix check <file>` behavior when the flag is absent.
- Route diagnostics-mode check failures through normalized `Diagnostic` values.
- Print one expanded record per diagnostic with severity, code, phase, message, file, byte span, source line, and help when those fields are available.
- Add an explicit phase field to `Diagnostic` because phase cannot be reliably inferred from code prefixes alone.
- Add focused tests for one parse diagnostic and one semantic diagnostic in diagnostics mode.

Out of scope:

- JSON diagnostics.
- Batch or package diagnostics.
- Parser wording rewrites.
- Broad cleanup of older direct `eprintln!` inspection commands.

## Repo-Aware Baseline

Relevant current files:

- `crates/radix/src/tool.rs` owns `RadixCli`, `CheckArgs`, `CheckCommand`, `cmd_check`, `EmitArgs`, and command helpers.
- `crates/radix/src/diagnostics/diagnostic.rs` owns the `Diagnostic` model and constructors from lex, parse, semantic, IO, and codegen errors.
- `crates/radix/src/diagnostics/render.rs` owns terminal and deterministic plain renderers.
- `crates/radix/src/driver/mod.rs` already normalizes full compile failures for `emit`, but `cmd_check` currently performs phase work directly and prints ad hoc messages.

Existing tests live in `crates/radix/src/tool_test.rs` and run inside the crate test binary.

## Stage Graph

1. Extend the diagnostic model with `DiagnosticPhase`.
2. Add deterministic expanded rendering in `diagnostics::render`.
3. Add `--diagnostics` parsing to `check` and a mode field to `CheckCommand`.
4. Refactor diagnostics-mode `cmd_check` to reuse normalized diagnostic construction while keeping normal mode unchanged.
5. Add focused unit tests for the renderer and CLI flag parsing.

## Epic Candidates And Scopable Issues

- Diagnostic model: phase enum, builder, and constructor defaults.
- Renderer: expanded text format that tolerates missing fields.
- Command surface: `CheckArgs` flag and `CheckCommand` mode.
- Check execution: diagnostics-mode branch that reports lex, parse, CLI analysis, and semantic errors via `Diagnostic`.
- Tests: parse and semantic output assertions without spawning a process.

## Checkpoints

Phase checkpoint passes when:

- `radix check --diagnostics <file>` selects diagnostics mode through clap.
- Normal `radix check <file>` behavior is unchanged by inspection and tests.
- Expanded parse diagnostic includes phase/code/span/source/help.
- Expanded semantic diagnostic includes phase/code/span/source/help when available.
- Focused tests pass.

## Companion Skill Plan

- `factory`: supervise phase boundary, verification, poker-face gate, and commit.
- `delivery`: this saved artifact is the single-phase delivery plan.

## Gate Plan

Run:

- `cargo test -p radix tool::tests::`
- A direct CLI smoke command for `radix check --diagnostics` if binary invocation is cheap after tests.

Gate result must be `PASS` before commit.

## Open Questions

None blocking.

