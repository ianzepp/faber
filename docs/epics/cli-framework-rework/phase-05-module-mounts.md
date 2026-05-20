# Phase 05: Module Mounts

## Goal

Support `@ imperia <path> ex <ident>` so imported modules can contribute command trees to a parent CLI.

## Scope

- Resolve wildcard imports used as CLI command modules.
- Extract CLI commands from imported modules.
- Mount imported command trees under the requested path.
- Detect import and mount cycles.
- Validate collisions between parent commands, mounted commands, aliases, and globals.
- Preserve useful source spans in diagnostics across module boundaries.

## Out Of Scope

- General import system redesign.
- Package-level CLI discovery outside the existing compiler project model.
- Dynamic plugin loading.

## Design Questions

- Should mounted modules require their own `@ cli`, or should `@ imperium` functions be enough?
- How should module-level descriptions combine with mounted command descriptions?
- What is the simplest diagnostic model for cross-file command collisions?

## Acceptance

- A parent CLI can mount commands from at least one imported module.
- Cycles and duplicate command paths fail with clear diagnostics.
- Mounted commands participate in generated help and dispatch.

