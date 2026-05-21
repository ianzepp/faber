# Phase 05: Module Mounts

## Goal

Support `@ imperia <path> ex <ident>` so imported modules can contribute command trees to a parent CLI.

Phase 05 extends the Phase 04 subcommand dispatcher across package-local modules. A command endpoint is still
defined by `@ imperium` plus a `functio`; `@ imperia` only decides where an imported module's endpoints are
exposed in the root CLI command tree.

## Scope

- Use Rust/package compilation as the first runnable target.
- Resolve package-local wildcard imports used as CLI command modules:

    ```fab
    importa ex "./jobs" * ut jobs

    @ cli "tool"
    @ imperia "jobs" ex jobs
    incipit argumenta args {}
    ```

- Extract `@ imperium` functions from explicitly mounted imported modules.
- Do not require mounted modules to declare their own `@ cli`.
- Mount imported command trees under the requested path by flattening the mount prefix with each local command path.
- Detect import and mount cycles.
- Validate collisions between parent commands, mounted commands, aliases, and globals.
- Preserve useful source spans in diagnostics across module boundaries.
- Generate Rust dispatch, parser structs, help output, and command calls for mounted commands.

## Phase Decisions

- `@ imperium` plus its `functio` is the command endpoint. The function body is ordinary Faber code.
- Root `@ imperia` is the explicit exposure boundary. The compiler should not scan every package file for CLI
  commands automatically.
- Mounted modules are command libraries. They should contain `@ imperium` functions, not a second root `@ cli`.
- `@ imperia` must reference a wildcard import alias from the root module, such as `importa ex "./jobs" * ut jobs`.
- Named imports are not valid mount targets in Phase 05.
- Root-level global options and operands remain root-only. Mounted modules may not declare `ubique` options or operands.
- Nested paths are represented by normal command path segments. No separate command tree model is needed in the CLI IR.
- The mounted canonical path is `mount-prefix + local-imperium-path`.
- Aliases from mounted commands are mount-local. The alias replaces the command's local path under the mount prefix.

    ```fab
    # root
    @ imperia "jobs" ex jobs

    # mounted module
    @ imperium "config/set"
    @ alias "set"
    functio set_config() {}
    ```

    This produces:

    ```text
    tool jobs config set
    tool jobs set
    ```

- Two mount forms can intentionally normalize to the same command path and must collide:

    ```fab
    @ imperia "jobs" ex jobs
    # jobs module has @ imperium "config/set"

    @ imperia "jobs/config" ex config
    # config module has @ imperium "set"
    ```

    Both expose `jobs/config/set`.

## Out Of Scope

- General import system redesign.
- Automatic package-wide CLI discovery outside explicit `@ imperia` mounts.
- Dynamic plugin loading.
- Independent runnable CLIs inside mounted modules.
- Root-level aliases for mounted commands.
- TypeScript, Go, or Faber runnable module-mounted CLI codegen.

## Design Questions

- How should module-level descriptions combine with mounted command descriptions?
- What is the simplest diagnostic model for cross-file command collisions?

## Acceptance

- A Rust package CLI can mount commands from at least one package-local wildcard import.
- Mounted modules do not need `@ cli`; their `@ imperium` functions become dispatch endpoints only when mounted.
- Unmounted imported modules are not exposed as CLI commands.
- A mounted nested command such as local `@ imperium "config/set"` under `@ imperia "jobs"` dispatches as
  `tool jobs config set`.
- A mounted alias such as `@ alias "set"` under `@ imperia "jobs"` dispatches as `tool jobs set`, not `tool set`.
- Duplicate normalized command paths fail with clear diagnostics across root and mounted commands.
- Duplicate mounted alias paths fail with clear diagnostics.
- Named imports used as mount targets fail with a clear diagnostic.
- Mounted `ubique` options or operands fail with a clear diagnostic.
- Import and mount cycles fail with clear diagnostics.
- Mounted commands participate in generated help and dispatch.
- Non-Rust mounted CLI codegen remains explicitly gated.
