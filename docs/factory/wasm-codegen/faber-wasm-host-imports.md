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

This is sufficient to prove **instantiate-valid** honestly.

## Entry Export (Phase 024)

Synthetic entry MIR functions (no `source`, no `name`) export as Wasm `incipit` while
keeping internal `$fN` names for intra-module calls. Named functions keep their sanitized
export names.

The exempla harness invokes export `incipit` after stub-host instantiation. Programs
without an entry export remain `instantiate-valid` only.

## Behavior Recording

`faber_diag` imports are wired to record `import_name:param` events (for example
`nota_text:3`). A small fixture table in `wasm_behavior_fixtures.rs` asserts stable traces
for selected exempla under the stub host.