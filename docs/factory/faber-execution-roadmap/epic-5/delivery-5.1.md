# Epic 5 Delivery 5.1: Generated Rust `ad` Syscall Bridge

## Interpreted Problem

Epic 5 requires Faber source `ad` calls to keep using the stable HIR-to-Rust backend while gaining a Wasm-host syscall path. Epic 4 already proves that the macOS host can route a tiny component import through `HostKernel` for `host:echo` and unresolved `pg:query`. The missing slice is generated Rust that lowers `ad` through `__faber_ad(...)`, delegates Wasm builds to a target syscall import named `capability-call`, and preserves native Rust's explicit unresolved-provider behavior.

## Normalized Spec

- Keep native generated Rust behavior unchanged: unresolved `ad` calls compile and fail clearly at runtime with `E_NO_ROUTE`.
- Add a Wasm-only generated helper path:
  - `__faber_ad(...)` maps source capability names to host route codes.
  - `__faber_ad(...)` calls `__faber_syscall(...)`.
  - `__faber_syscall(...)` is a Wasm import with import name `capability-call`.
- Align route codes with `hosts/macos-arm64/src/component.rs`:
  - `host:echo` -> `1`
  - `pg:query` -> `2`
- Add codegen tests proving the generated Rust contains the native and Wasm helper shapes.
- Add a ledger entry with validation evidence and the remaining stronger artifact proof.

## Repo-Aware Baseline

- `crates/radix/src/codegen/rust/expr/mod.rs` already emits `__faber_ad::<T, _>(...)`.
- `crates/radix/src/codegen/rust/prelude.rs` currently emits only a native unresolved `__faber_ad` helper.
- `hosts/macos-arm64/src/component.rs` already exposes a `capability-call` component import and routes codes `1` and `2` through `HostKernel`.
- The local Rust toolchain is Homebrew Rust without `rustup`; `rustc --target wasm32-unknown-unknown` currently fails because the target standard library is not installed.

## Stage Graph

1. Persist this delivery spec.
2. Update the generated Rust `__faber_ad` helper with native and Wasm cfg branches.
3. Add focused Rust codegen tests for `ad` helper shape and route-code mapping.
4. Run focused compiler and host tests.
5. Update the Epic 5 ledger with commands and remaining proof gap.

## Checkpoints

- Native generated Rust still contains `E_NO_ROUTE: unresolved capability`.
- Wasm generated Rust contains `#[link_name = "capability-call"]`, `__faber_syscall`, and route mappings for `host:echo` and `pg:query`.
- Host component tests still pass for `host:echo` and `pg:query`.
- Any missing end-to-end generated Rust-to-Wasm artifact proof is explicitly recorded instead of claimed.

## Gate Plan

This phase is complete only if focused tests pass and the ledger distinguishes implemented bridge code from the stronger rustc-to-wasm execution proof that depends on local Wasm target/tooling availability.
