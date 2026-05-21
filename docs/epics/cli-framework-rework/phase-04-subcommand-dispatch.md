# Phase 04: Subcommand Dispatch

## Goal

Support multi-command CLIs where `@ imperium` functions become dispatchable subcommands.

Phase 04 extends the Phase 03 Rust single-command path. It should not redesign CLI parsing or introduce a runtime crate.

## Scope

- Use Rust as the first runnable subcommand target.
- Extend the self-contained generated Rust parser from Phase 03.
- Add `argumenta <ident>` as a function modifier in the parser, AST, HIR lowering, and semantic binding paths.
- Build nested command paths from `@ imperium` values.
- Support command aliases.
- Parse command-local options and operands.
- Merge validated global options into each command's argument object.
- Add and implement a command-handler argument binding using `argumenta <ident>` as a function modifier:

    ```fab
    @ imperium "jobs/list"
    @ optio limit longum "limit" typus numerus vel 20
    functio list() argumenta args -> vacuum {
        nota args.limit
    }
    ```

- Generate dispatch logic from command path to function call.
- Make parsed command values typecheck through the command handler's `argumenta <ident>` object.
- Support command-specific help output.
- Implement behavior for missing commands and unknown commands.
- Remove the Phase 04 Rust subcommand codegen gate for supported subcommand CLIs.
- Keep explicit diagnostics for module mounts and non-Rust runnable subcommand targets.

## Phase Decisions

- The command args binding keyword is `argumenta`, not `optiones`.
- `optiones <ident>` remains historical/unimplemented CLI syntax and should not be used for Phase 04 command arguments.
- Command functions may use `argumenta <ident>` to access merged global and command-local CLI values.
- Command functions without `argumenta <ident>` may still dispatch, but cannot access parsed CLI values.
- Ordinary command function parameters remain unsupported for CLI dispatch in this phase.
- Nested paths continue to use `/` inside `@ imperium` values.
- Aliases dispatch to the same handler as their command.
- Root `--help` lists global options and available commands.
- Command `--help` lists globals plus command-local options and operands.
- If no command is provided, print root help and exit with code `2`.
- If an unknown command is provided, print an error and exit with code `2`.
- In Phase 04, subcommand CLIs must use an empty root `incipit` body. A non-empty root body should produce a diagnostic instead of being silently ignored or treated as setup code.

## Out Of Scope

- `@ imperia` module mounts.
- Shell completions.
- Advanced choice or enum option types.
- Lifecycle/setup semantics for running the root `incipit` body before every command.
- A reusable CLI runtime crate or `norma::cli` helper.
- TypeScript, Go, or Faber runnable subcommand codegen.

## Design Questions

- How should command aliases appear in generated help?
- Should command functions without `argumenta <ident>` be allowed permanently, or only as a Phase 04 compatibility convenience?
- Should the root `incipit` body ever run for subcommand CLIs, or should setup/lifecycle behavior wait for a later explicit design?

## Acceptance

- A program with at least two commands dispatches to the correct handler.
- Command-local options and operands are parsed independently.
- Command handlers can access parsed values through `argumenta <ident>` and `args.<name>` typechecks.
- Global options are available in command args objects.
- Global option collisions are rejected before codegen.
- Aliases dispatch correctly.
- Nested command paths dispatch correctly.
- Root help and command help are stable enough for tests.
- Missing and unknown commands produce clear errors and exit code `2`.
- Non-empty root `incipit` bodies in subcommand CLIs are rejected with a clear diagnostic.
- Rust subcommand CLIs compile and run in tests.
- Non-Rust subcommand CLI codegen remains explicitly gated.
- Module-mounted commands remain explicitly gated for Phase 05.
