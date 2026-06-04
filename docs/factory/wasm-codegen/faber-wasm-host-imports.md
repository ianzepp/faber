# Faber Wasm Host Imports (Harness Stub Phase)

This note describes the import modules emitted by the MIR Wasm text probe and how the
exempla e2e harness satisfies them during Phase 023.

## Import Modules

| Module | Role in emitted Wasm |
| --- | --- |
| `faber_diag` | Diagnostic output (`nota_*`, `vide_*`, `mone`, `mori`, etc.) |
| `faber_text` | Text handles, formatting, concatenation, comparisons |
| `faber_aggregate` | Aggregate construction, projection, and indexed access |
| `faber_runtime` | Collection mutation, length, panic, and assertion helpers |

## Phase 023 Stub Policy

The harness uses wasmtime `define_unknown_imports_as_default_values`:

- Parameters are ignored.
- Results use wasmtime default values (numeric zero, null reference where applicable).
- No real text, aggregate, or collection state is modeled.

This is sufficient to prove **instantiate-valid** honestly. Runnable and behavior-checked
tiers require real host semantics and entrypoint policy in later phases.