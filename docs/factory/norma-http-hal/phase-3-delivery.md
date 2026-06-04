# Phase 3 Delivery Spec: Rust Codegen Bridge

## Scope

Teach the Rust backend to lower `norma/hal/http` module calls to the Norma
runtime and render the HTTP response pactum as the concrete Rust runtime type.

## Required Changes

- Add the HTTP module to the Rust Norma runtime bridge:
  - `http` to `norma::hal::http`.
- Recognize the exact HTTP HAL pactum shapes in HIR as runtime-owned interfaces.
- Elide generated local Rust trait declarations for those runtime-owned HTTP
  HAL pacta.
- Render the runtime-owned `Replicatio` interface as
  `norma::hal::http::Replicatio`.
- Preserve ordinary user-defined interfaces and non-HTTP pacta behavior.

## Tests

- Add focused Rust codegen tests proving:
  - `http.petet(url)` emits `norma::hal::http::petet(url)`.
  - the awaited response local is typed as `norma::hal::http::Replicatio`.
  - local HTTP HAL trait stubs are not emitted.

## Checkpoint

Run:

```bash
cargo test -p radix http
```

Also build a temporary Faber package that imports `norma/hal/http`, awaits
`http.petet`, and reads response status to prove generated Rust compiles.
