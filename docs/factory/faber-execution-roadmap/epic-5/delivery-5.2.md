# Epic 5 Delivery 5.2: Core Wasm Host Runner

## Interpreted Problem

Phase 5.1 taught generated Rust to import a core Wasm function named `capability-call`, but the macOS host only had a Component Model runner. That left the generated Rust helper with no direct host runtime target even before the local Rust Wasm target/tooling issue. This phase adds the missing host-side core Wasm runner while keeping the Epic 4 component proof intact.

## Normalized Spec

- Share the temporary route-code ABI between component and core Wasm host paths.
- Add a `WasmHost` runner that:
  - instantiates a core Wasm module,
  - provides import module `""`, function `capability-call`,
  - routes route codes through `HostKernel`,
  - records the last frame response for host-side validation.
- Add a CLI command for manual proof runs:
  - `faber-host-macos-arm64 wasm-call <module> <export> <route-code>`.
- Add tests proving `host:echo` success and `pg:query` `E_NO_ROUTE` through the core Wasm import path.
- Preserve existing component runner behavior.

## Repo-Aware Baseline

- `crates/radix/src/codegen/rust/prelude.rs` emits a core Wasm import for `capability-call`.
- `hosts/macos-arm64/src/component.rs` routed component imports through a private helper that duplicated the route-code policy.
- `hosts/macos-arm64/tests/fixtures/route-proof.wat` already proved the Component Model wrapper path.

## Stage Graph

1. Extract shared route-code mapping into `hosts/macos-arm64/src/syscall_import.rs`.
2. Update `component.rs` to use the shared route-code router.
3. Add `hosts/macos-arm64/src/wasm.rs` for core Wasm module instantiation.
4. Add the `wasm-call` CLI command.
5. Add a core WAT fixture and host tests.
6. Run focused host and codegen validation.

## Checkpoints

- `host:echo` reaches `HostKernel` through core Wasm import and returns a done frame with echoed payload.
- `pg:query` reaches `HostKernel` through core Wasm import and returns structured `E_NO_ROUTE`.
- Component import tests still pass.
- Generated Rust helper tests still pass.

## Gate Plan

This phase does not claim the final generated Rust artifact proof. It removes the host-side blocker for that proof and keeps the remaining blocker limited to local Rust-to-Wasm compilation tooling.
