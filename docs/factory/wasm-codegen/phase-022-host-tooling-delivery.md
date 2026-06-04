# Phase 022: Host Tooling And Instantiation Harness

## Interpreted Problem

Compile-valid Wasm coverage is protected at 72/101, but every compile-valid exemplar stops below
instantiate-valid because the harness only checks for a `wasmtime` binary on `PATH`. There is no
repo-local runtime, and all failures collapse into one `no wasmtime on PATH` bucket.

## Normalized Spec

- Add `wasmtime` as a `radix` dev-dependency so ignored e2e tests always have a Wasm host in CI
  and local `cargo test`.
- Replace PATH-only instantiation probing with an in-process `wasmtime` linker probe.
- Classify compile-valid modules into explicit instantiation buckets:
  `missing-import`, `instantiation-trap`, `instantiate-valid`, plus `no-runtime` only if the host
  cannot be constructed.
- Parse emitted WAT import sites for reporting; do not implement Faber runtime behavior yet.
- Preserve Phase 021/025 expected compile-valid floors (`72/101`).

## Checkpoints

- Harness report prints per-bucket counts for the compile-valid subset.
- `instantiate-valid` tier means linker instantiation succeeded without stubs, not merely
  `wasmtime` being installed.
- Existing `72/101` compile-valid exemplars remain compile-valid or better.

## Gate Plan

- `cargo test -p radix wasm_host -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`