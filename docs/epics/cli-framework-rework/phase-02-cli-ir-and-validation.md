# Phase 02: CLI IR and Validation

## Goal

Build a target-independent CLI model from parsed annotations and validate the user-facing command contract before runnable CLI codegen.

After this phase, CLI programs should parse, check, and expose their normalized CLI structure for inspection. They do not yet need to emit runnable argument parsing or dispatch code.

## Scope

- Introduce a `CliProgram` model for binary metadata, global options, operands, commands, aliases, and mode.
- Detect single-command mode versus subcommand mode.
- Validate required placement rules, including top-level-only `ubique`.
- Validate option names, short flags, long flags, duplicate bindings, duplicate command paths, duplicate aliases, and global/local collisions.
- Validate operand ordering, including the single final `ceteri` rule.
- Validate supported CLI types at the compiler boundary.
- Add an inspection surface for the normalized CLI IR, either as a dedicated command or an extension of an existing inspection command.
- Gate runnable CLI codegen with an explicit diagnostic until Phase 03 implements it.

## Phase Decisions

- A `@ cli` program must use an explicit `incipit argumenta <ident>` entry point.
- CLI analysis should use a hybrid shape: build the CLI surface from AST annotations first, preserving source spans and placement information, then attach semantic links as needed for later phases.
- Phase 02 should validate the CLI contract during `check`.
- Omitted option `typus` resolves to `textus`.
- `typus bivalens` declares a flag and defaults to `falsum`.
- Non-boolean options and operands are absent/`ignotum` unless `vel` provides a default.
- Historical option syntaxes are rejected during parsing in Phase 01, so Phase 02 does not carry compatibility logic for them.

## Out Of Scope

- Generated argument parsing.
- Runtime dispatch.
- Module imports and `@ imperia` mounting.
- Target-specific CLI runtime implementation.

## Design Questions

- How should the typed `args` object be represented in HIR and semantic type information?
- Should CLI IR inspection be a new `cli-ir` command, part of `hir`, or a JSON mode on `check`?
- Should `vel` on `typus bivalens` be allowed as a normal default, or should boolean defaults remain fixed to `falsum`?

## Acceptance

- A valid CLI source file produces a validated CLI IR.
- Invalid CLI declarations produce actionable compiler diagnostics.
- The compiler can report whether a program is not a CLI, a single-command CLI, or a subcommand CLI.
- `parse` output shows structured CLI annotations from Phase 01.
- `check` validates CLI rules and succeeds for valid CLI programs.
- A normalized CLI IR is inspectable through a compiler-facing command or mode.
- Emitting runnable CLI behavior remains explicitly gated until Phase 03.
