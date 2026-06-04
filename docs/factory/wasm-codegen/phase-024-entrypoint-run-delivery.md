# Phase 024: Entrypoint And Run Policy

## Interpreted Problem

Phase 023 reaches `instantiate-valid` for all compile-valid exemplars, but the harness
does not invoke a program entry or capture observable stub-host output. Runnable and
behavior-checked tiers are still unclaimed.

## Normalized Spec

- Export synthetic entry MIR functions as Wasm `incipit` while keeping internal `$fN`
  names for intra-module calls.
- Named top-level functions keep their sanitized export names.
- After stub-host instantiation, invoke export `incipit` when present.
- Record `faber_diag` calls in the stub host for behavior fixtures.
- Add a small fixture table for exemplars with stable diagnostic traces under stubs.
- Do not add CLI, filesystem, HTTP, or real text/aggregate semantics.

## Checkpoints

- `salve-munde.fab`, primitive `nota` exemplars, and `incipit`/`functio` examples reach
  `runnable`.
- Behavior-checked applies only where fixtures exist and diag traces match.
- Compile-valid and instantiate-valid floors remain protected.

## Gate Plan

- `cargo test -p radix wasm_host -- --nocapture`
- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix --lib mir -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`