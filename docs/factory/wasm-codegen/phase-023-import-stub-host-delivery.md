# Phase 023: Minimal Import Stub Host

## Interpreted Problem

Phase 022 proves all 72 compile-valid exemplars stop at `missing-import` under a stubless
linker. Instantiation cannot advance until the harness supplies no-op host functions for the
`faber_diag`, `faber_text`, `faber_aggregate`, and `faber_runtime` import modules.

## Normalized Spec

- Use wasmtime `define_unknown_imports_as_default_values` to satisfy every function import
  with zero/null results and ignored parameters.
- Promote compile-valid exemplars to `instantiate-valid` when stubbed linker instantiation
  succeeds.
- Keep stubless `missing-import` bucket reporting for transparency.
- Record the stub policy in a small host-import reference note.
- Do not claim runnable or behavior-checked tiers.

## Checkpoints

- All current compile-valid exemplars (`72/101`) reach `instantiate-valid` under the stub host.
- Phase 021/025 compile-valid floors remain protected.

## Gate Plan

- `cargo test -p radix wasm_host -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix --lib mir -- --nocapture`
- `cargo test -p radix --lib wasm -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`