# Phase 4 Delivery: Runtime Type Mapping By Provenance

## Objective

Replace HTTP runtime interface recognition by local interface name and method
list with imported library item provenance.

## Implementation

- Changed Rust backend runtime-interface collection to use
  `LibraryRegistry.items` keyed by interface `DefId`.
- Mapped only imported `BuiltinNorma + ["hal", "http"]` interface items:
  - exported `http` -> elide module interface declaration;
  - exported `Replicatio` -> `norma::hal::http::Replicatio`;
  - exported `Rogatio` -> `norma::hal::http::Rogatio`;
  - exported `Servitor` -> `norma::hal::http::Servitor`.
- Removed the old method-list matcher from Rust backend identity selection.
- Updated the direct-source HTTP test to prove a user-defined `pactum http` and
  matching `pactum Replicatio` emit ordinary Rust traits and do not receive
  Norma runtime call or type mapping.
- Extended the package alias test to prove imported Norma HTTP still emits the
  concrete `norma::hal::http::Replicatio` type.

## Checkpoint Evidence

- A user-defined same-name, same-shape `Replicatio` is rendered as a normal
  `dyn Replicatio` user trait type.
- A package importing `norma:hal/http` renders the imported `Replicatio` as
  `norma::hal::http::Replicatio`.
- HTTP runtime type identity now depends on imported item provenance, not
  method names.

## Validation

Run during the phase:

```bash
cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call
cargo test -p faber aliased_norma_http_import_lowers_by_provider_identity
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
cargo fmt --check
```

Result: all passed. The Faber tests passed for both duplicated test targets.
