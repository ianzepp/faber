# Phase 2 Delivery: Provider Metadata Model

## Objective

Carry target-neutral library identity through analysis so later codegen phases
can use `DefId` provenance instead of source spelling or local binding names.

## Implementation

- Added neutral HIR-side data types:
  - `LibraryProvider`
  - `LibraryIdentity`
  - `LibraryBinding`
  - `LibraryItem`
  - `LibraryItemKind`
  - `LibraryRegistry`
- Added `libraries: LibraryRegistry` to `radix::driver::AnalyzedUnit`.
- Added provenance-aware Rust backend constructors and module-generation helpers.
- Updated Faber package codegen to pass `analysis.libraries` into the Rust
  backend.
- Populated library provenance during package analysis from resolved
  `LibraryImportBinding` values:
  - imported module binding `DefId` -> provider/module identity;
  - declarations injected from the library interface -> provider/module
    identity plus exported library name and item kind.
- Added a focused test proving `importa ex "norma:hal/http" privata http ut
  rete` preserves `BuiltinNorma + ["hal", "http"]` for both the aliased binding
  `rete` and an imported item `Replicatio`.

## Checkpoint Evidence

- Code can ask whether a specific `DefId` came from `BuiltinNorma + ["hal",
  "http"]` through `analysis.libraries`.
- Source aliases change local names only; the aliased `rete` binding keeps
  `BuiltinNorma + ["hal", "http"]`.
- Imported interface items keep their own item provenance keyed by their own
  `DefId`.
- Rust backend construction can receive the side table without relying on
  target-specific Rust paths in the Faber resolver.

## Validation

Run during the phase:

```bash
cargo fmt --check
cargo test -p faber library
cargo test -p faber aliased_norma_import_preserves_provider_identity_in_analysis
cargo test -p radix http_hal_calls_emit_norma_runtime_bridge_and_concrete_response_type
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
```

Result: all passed. The Faber tests passed for both duplicated test targets.
