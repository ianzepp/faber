# Phase 7 Delivery: Metadata-Driven Runtime Linkage

## Objective

Remove the remaining standard-library-specific Rust runtime match tables from
library provenance codegen. Runtime linkage must come from resolved library
identity plus library metadata, not from enumerating known Norma modules or HTTP
types in backend code.

## Changes

- Generalized `LibraryProvider::BuiltinNorma` to `LibraryProvider::Builtin(String)`.
- Added Rust runtime metadata to library provenance:
  - `LibraryBinding::rust_runtime_module`
  - `LibraryItem::rust_runtime_type`
  - `LibraryItem::elide_rust_decl`
- Populated that metadata from `@ subsidia rs ...` on imported library
  interfaces.
- Added missing Rust support metadata to `stdlib/norma/hal/http.fab`.
- Replaced the Rust backend module/type match arms with registry lookups.

## Validation

```bash
cargo test -p faber aliased_norma_import_preserves_provider_identity_in_analysis
cargo test -p faber compile_package_resolves_builtin_norma_library_imports_without_local_modules
cargo test -p faber compile_package_resolves_builtin_norma_toml_library_imports
cargo test -p faber aliased_norma_http_import_lowers_by_provider_identity
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call
```

Results: all passed.

## Gate Result

Phase 7 passes the corrected design gate: adding a new runtime-backed standard
library module requires `.fab` metadata and runtime implementation, not a new
Rust codegen match arm for that module or its exported HTTP-like types.
