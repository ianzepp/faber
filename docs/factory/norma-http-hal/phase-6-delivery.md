# Phase 6 Delivery Spec: Validation Gate

## Scope

Validate the completed Norma HTTP HAL implementation against the factory plan.

## Required Focused Commands

```bash
cargo test -p norma
cargo test -p radix
cargo test -p faber
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

Package fixture coverage is included in `cargo test -p faber` through
`package_fixture_runs_norma_http_hal_against_local_server`.

## Broad Gate

```bash
./scripta/test
```

## Completion Rule

Do not mark the factory complete until the required commands pass or any
remaining failure is documented with evidence and explicitly outside this
factory's scope.

## Validation Results

Recorded on 2026-06-04:

- `cargo test -p norma` passed.
- `cargo test -p radix` passed.
- `cargo test -p faber` passed after tightening the local HTTP fixture to avoid
  a generated-build/server-listener race and inherited nonblocking stream read.
- `cargo run -p faber -- check stdlib/norma/hal/http.fab` passed.
- `./scripta/test` passed.

The package-level HTTP HAL fixture is
`package_fixture_runs_norma_http_hal_against_local_server`. It builds and runs a
generated Faber package against a local `TcpListener`, then checks status,
header access, text body access, and JSON-as-`valor` behavior without public
network dependency.
