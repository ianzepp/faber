# Phase 02: CLI IR and Validation

## Goal

Build a target-independent CLI model from parsed annotations and validate the user-facing command contract before codegen.

## Scope

- Introduce a `CliProgram` model for binary metadata, global options, operands, commands, aliases, and mode.
- Detect single-command mode versus subcommand mode.
- Validate required placement rules, including top-level-only `ubique`.
- Validate option names, short flags, long flags, duplicate bindings, duplicate command paths, duplicate aliases, and global/local collisions.
- Validate operand ordering, including the single final `ceteri` rule.
- Validate supported CLI types at the compiler boundary.
- Decide where diagnostics live in the existing semantic or analysis pipeline.

## Out Of Scope

- Generated argument parsing.
- Runtime dispatch.
- Module imports and `@ imperia` mounting.

## Design Questions

- Should CLI validation run before HIR lowering, during HIR lowering, or as a semantic analysis pass after HIR exists?
- Should `incipit argumenta` be required for `@ cli`, or should `@ cli` alone be enough to trigger generated dispatch?
- How should the typed `args` object be represented in HIR and semantic type information?

## Acceptance

- A valid CLI source file produces a validated CLI IR.
- Invalid CLI declarations produce actionable compiler diagnostics.
- The compiler can report whether a program is not a CLI, a single-command CLI, or a subcommand CLI.

