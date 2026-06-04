# Phase 2 Delivery Spec: Norma Runtime Client

## Scope

Add the Rust runtime implementation for the client side of
`norma/hal/http`.

## Required Changes

- Add `crates/norma/hal/http.rs`.
- Export the module from `crates/norma/hal/mod.rs`.
- Add `reqwest` to `crates/norma/Cargo.toml` with rustls TLS and JSON support.
- Implement owned `Replicatio` snapshots:
  - status code,
  - response headers,
  - response body bytes.
- Implement client request helpers:
  - `petet`,
  - `mittet`,
  - `ponet`,
  - `delet`,
  - `mutabit`,
  - `rogabit`.
- Return synthetic failure responses for ordinary request construction,
  transport, and body-read errors.
- Return `Valor::Nihil` for JSON parse/conversion failures.

## Tests

Add dedicated runtime tests for:

- response accessors,
- header lookup,
- JSON body conversion to `Valor`,
- JSON parse failure returning `Valor::Nihil`,
- synthetic request failure response behavior.

## Checkpoint

Run:

```bash
cargo test -p norma http
cargo check -p norma
```
