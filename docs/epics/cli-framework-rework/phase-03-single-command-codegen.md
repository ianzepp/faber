# Phase 03: Single Command Codegen

## Goal

Make a small single-command CLI compile into a working executable target with parsed arguments, help, version, options, operands, and exit behavior.

## Scope

- Generate argument parsing for top-level `@ optio` and `@ operandus` declarations.
- Bind parsed values into the `incipit argumenta <ident>` object.
- Support boolean flags, string values, numeric values, rest operands, and defaults.
- Generate `--help` and `-h` output.
- Generate `--version` output when `@ versio` is present.
- Respect `exitus` behavior for fixed or variable exit codes.
- Pick one initial target for the first vertical slice.

## Out Of Scope

- Subcommands.
- Module mounts.
- Repeatable list options unless explicitly pulled into the first slice.
- Multi-target parity.

## Design Questions

- Should TypeScript be the first implementation target because of the historical reference, or should Rust be first because `radix-rs` already has strong Rust output?
- Should the generated parser be handwritten per target, or should targets call into a small runtime helper?
- What exact null/default representation should absent options use?

## Acceptance

- A minimal `echo`-style Faber CLI compiles and runs.
- Help and version output are stable enough for tests.
- Parsed args are available through `args.<name>` in user code.

