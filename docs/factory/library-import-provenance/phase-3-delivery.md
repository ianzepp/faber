# Phase 3 Delivery: Runtime Call Bridge By Provenance

## Objective

Replace receiver-name runtime call bridging with provider-aware lookup keyed by
the receiver binding `DefId`.

## Implementation

- Changed the Norma runtime bridge helper to accept a receiver `DefId` and
  `LibraryRegistry`.
- Mapped only `BuiltinNorma` library bindings to Rust runtime modules:
  - `["json"]` -> `norma::json`
  - `["toml"]` -> `norma::toml`
  - `["hal", "consolum"]` -> `norma::hal::consolum`
  - `["hal", "http"]` -> `norma::hal::http`
- Updated Rust method-call emission to bridge only when the receiver `DefId`
  has imported library binding provenance.
- Added package-level positive coverage proving an alias import:
  `importa ex "norma:hal/http" privata http ut rete`
  lowers `rete.petet(...)` to `norma::hal::http::petet(...)`.
- Replaced the old Radix direct-source HTTP bridge test with a negative test:
  a user-defined `pactum http` without library provenance no longer emits a
  Norma runtime call.

## Checkpoint Evidence

- Aliased Norma imports bridge by provider identity, not local spelling.
- A user-defined same-name `http` surface does not trigger Norma call bridging.
- Existing non-aliased HTTP package fixture still compiles, builds, runs, and
  bridges through the Norma runtime.

## Validation

Run during the phase:

```bash
cargo test -p faber aliased_norma_http_import_lowers_by_provider_identity
cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
cargo fmt --check
```

Result: all passed. The Faber tests passed for both duplicated test targets.

The Phase 6 plan's older focused Radix test name
`http_hal_calls_emit_norma_runtime_bridge_and_concrete_response_type` was
replaced here because its previous source shape intentionally exercised the
unsafe no-provenance bridge. Positive runtime bridging is now covered by the
package-level provenance test.
