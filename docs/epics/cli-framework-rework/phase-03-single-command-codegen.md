# Phase 03: Single Command Codegen

## Goal

Make a small single-command CLI compile into a working executable target with parsed arguments, help, version, options, operands, and exit behavior.

## Scope

- Use Rust as the first runnable CLI target.
- Emit a self-contained Rust argument parser for the first vertical slice.
- Generate argument parsing for top-level `@ optio` and `@ operandus` declarations.
- Make parsed values typecheck through the `incipit argumenta <ident>` object so user code can access `args.<name>`.
- Support boolean flags, string values, numeric values, rest operands, and defaults.
- Generate `--help` and `-h` output.
- Generate `--version` output only when `@ versio` is present.
- Respect `exitus` behavior for fixed or variable exit codes.
- Remove the Phase 03 runnable CLI codegen gate for valid single-command Rust CLI programs.
- Keep explicit diagnostics for CLI shapes or targets that are not implemented in this phase.

## Phase Decisions

- The first implementation target is Rust.
- Phase 03 emits parser logic directly into generated Rust output instead of introducing a runtime crate or helper API.
- Runtime extraction remains a Phase 06 decision after the single-command Rust behavior is proven.
- Boolean options default to `falsum`.
- Non-boolean options without `vel` are represented as absent optional values (`si T`) in the typed `args` contract.
- Non-boolean options with `vel` are represented as concrete values of their declared type.
- `ceteri` operands become `lista<T>` values.
- `--version` is recognized only when the source declares `@ versio`; otherwise it is treated like any other unknown flag.
- Subcommand CLI programs remain gated for Phase 04.
- TypeScript, Go, and Faber runnable CLI codegen remain gated until later phases.

## Out Of Scope

- Subcommands.
- Module mounts.
- Repeatable list options unless explicitly pulled into the first slice.
- Multi-target parity.
- A reusable CLI runtime crate or `norma::cli` helper.

## Design Questions

- Should `exitus` support land in the first implementation PR, or as a small follow-up inside Phase 03?
- Should operands without `vel` be required at parse time, or should missing operands surface as absent option values for user code?
- Which single-command example should become the canonical e2e fixture?

## Acceptance

- A minimal `echo`-style Faber CLI compiles to Rust and runs.
- Generated Rust compiles with `rustc` in tests.
- The Phase 03 codegen gate no longer fires for valid single-command Rust CLI programs.
- Subcommand CLIs still fail with a Phase 04 diagnostic.
- Non-Rust runnable CLI targets still fail with explicit diagnostics.
- Help and version output are stable enough for tests.
- Parsed args are available through `args.<name>` in user code.
- Tests cover boolean flags, text options, numeric options, defaults, rest operands, missing/unknown flags, `--help`, and `--version`.
