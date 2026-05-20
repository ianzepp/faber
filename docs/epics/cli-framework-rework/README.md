# CLI Framework Rework Epic

## Purpose

Turn the declarative CLI surface described in `docs/grammatica/cli.md` from an aspirational and historical design into an implemented `radix-rs` compiler capability.

This epic is intentionally high level at first. Each phase file is a working spec that should be refined before implementation begins for that phase.

## Current Truth

- CLI-shaped annotations are preserved as generic annotation metadata today.
- `AnnotationKind::Cli`, `OptioAnnotation`, and `OperandusAnnotation` exist, but the parser does not populate them.
- The prepared AST structs are not yet rich enough for the current planned grammar. They are missing distinctions and fields for several planned concepts, including `@ cli` versus `@ imperium`, option types, defaults, and globals.
- `incipit argumenta args` parses and lowers, but `args` currently becomes `lista<textus>`, not a typed parsed-arguments object.
- There is no CLI IR, mode detection, argument parser generation, help formatting, version handling, subcommand dispatch, or module command mounting in `radix-rs`.

## Phase Index

1. [Syntax and AST](phase-01-syntax-and-ast.md)
2. [CLI IR and Validation](phase-02-cli-ir-and-validation.md)
3. [Single Command Codegen](phase-03-single-command-codegen.md)
4. [Subcommand Dispatch](phase-04-subcommand-dispatch.md)
5. [Module Mounts](phase-05-module-mounts.md)
6. [Targets and Runtime Shape](phase-06-targets-and-runtime-shape.md)
7. [Docs, Examples, and Tests](phase-07-docs-examples-and-tests.md)

## Working Principles

- Implement vertical slices that can be tested end to end.
- Prefer explicit compiler diagnostics over silent fallback behavior.
- Keep CLI meaning target-independent until the final codegen boundary.
- Treat missing type information as an upstream compiler issue, not something codegen should guess.
- Preserve historical behavior only when it still fits the current language and compiler architecture.

