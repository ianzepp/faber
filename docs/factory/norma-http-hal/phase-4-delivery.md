# Phase 4 Delivery Spec: Package Fixture

## Scope

Add a repeatable package-level fixture proving that a Faber package can import
`norma/hal/http`, issue a client request through generated Rust, and inspect
the response without public internet access.

## Required Changes

- Add a package-level test that starts a local HTTP listener.
- Generate a temporary Faber package importing `norma/hal/http`.
- Build the generated Rust crate through the normal package writer.
- Run the generated binary.
- Assert observable response inspection:
  - `status()`,
  - `caput(...)`,
  - `corpus()`,
  - `corpus_json()`.
- Ensure generated async entrypoints use a Tokio runtime so the HTTP client can
  run inside package binaries.
- Ensure generated package `Cargo.toml` declares `tokio` directly.

## Checkpoint

Run:

```bash
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
```

The fixture must not depend on public network access.
