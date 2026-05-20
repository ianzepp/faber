# Phase 04: Subcommand Dispatch

## Goal

Support multi-command CLIs where `@ imperium` functions become dispatchable subcommands.

## Scope

- Build nested command paths from `@ imperium` values.
- Support command aliases.
- Parse command-local options and operands.
- Merge validated global options into each command's argument object.
- Generate dispatch logic from command path to function call.
- Support command-specific help output.
- Decide and implement behavior for missing commands and unknown commands.
- Support the `optiones <ident>` function modifier if it remains part of the accepted design.

## Out Of Scope

- `@ imperia` module mounts.
- Shell completions.
- Advanced choice or enum option types.

## Design Questions

- Should command functions receive explicit parameters, a generated args object, or both depending on declaration style?
- How should command aliases appear in generated help?
- Should nested command paths be declared with `/`, nested modules, or both?

## Acceptance

- A program with at least two commands dispatches to the correct handler.
- Command-local options and operands are parsed independently.
- Global option collisions are rejected before codegen.

