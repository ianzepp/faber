# CLI Framework Rework Epic

## Purpose

Turn the declarative CLI surface described in `docs/grammatica/cli.md` from an aspirational and historical design into an implemented `radix-rs` compiler capability.

Phases 1-5 are implemented in the active `radix-rs` compiler. Phase 6 was deferred into future target/runtime
strategy work, and Phase 7 closes the initial effort with docs, examples, and tests.

## Current Truth

- `@ cli`, `@ imperium`, `@ optio`, and `@ operandus` parse into structured AST forms.
- CLI analysis builds a validated target-independent IR.
- `incipit argumenta args` and command `functio ... argumenta args` lower to typed parsed-arguments records.
- Rust codegen emits runnable single-command CLIs, subcommand dispatch, help, version output, and module-mounted package CLIs.
- TypeScript, Go, and Faber runnable CLI codegen remain explicitly gated.
- Parser helper code is still emitted inline; runtime extraction is deferred.

## Phase Index

1. [Syntax and AST](phase-01-syntax-and-ast.md)
2. [CLI IR and Validation](phase-02-cli-ir-and-validation.md)
3. [Single Command Codegen](phase-03-single-command-codegen.md)
4. [Subcommand Dispatch](phase-04-subcommand-dispatch.md)
5. [Module Mounts](phase-05-module-mounts.md)
6. [Targets and Runtime Shape](phase-06-targets-and-runtime-shape.md) (deferred)
7. [Docs, Examples, and Tests](phase-07-docs-examples-and-tests.md)

## Working Principles

- Implement vertical slices that can be tested end to end.
- Prefer explicit compiler diagnostics over silent fallback behavior.
- Keep CLI meaning target-independent until the final codegen boundary.
- Treat missing type information as an upstream compiler issue, not something codegen should guess.
- Preserve historical behavior only when it still fits the current language and compiler architecture.
