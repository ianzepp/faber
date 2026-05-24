# Epic 5 Ledger: Rust-To-Wasm Faber Host Syscall Bridge

## Status

- `5.1` implemented: generated Rust helper bridge from `__faber_ad(...)` to a Wasm host import.
- `5.2` implemented: core Wasm host runner for the generated Rust helper's `capability-call` import.

## Evidence Log

- 2026-05-24: Local toolchain check found Homebrew `rustc 1.95.0` and no `rustup`.
- 2026-05-24: `rustc --target wasm32-unknown-unknown` failed with `can't find crate for std`, so this machine cannot currently compile generated Rust to a Wasm artifact without adding the target standard library/tooling.
- 2026-05-24: `crates/radix/src/codegen/rust/prelude.rs` now emits cfg-separated native and Wasm `__faber_ad(...)` helpers. Native builds keep explicit unresolved-provider `E_NO_ROUTE` behavior. Wasm builds import `capability-call` as `__faber_syscall(...)` and map `host:echo` to route code `1` and `pg:query` to route code `2`.
- 2026-05-24: `cargo test -p radix ad_helper -- --nocapture` passed: 2 focused codegen tests.
- 2026-05-24: `cargo test -p faber-host-macos-arm64 component -- --nocapture` passed: 3 host component tests.
- 2026-05-24: Native generated Rust sanity check passed for an `ad "host:echo"` sample: generated Rust compiled with `rustc --edition=2021` and printed `E_NO_ROUTE: unresolved capability host:echo`.
- 2026-05-24: `hosts/macos-arm64/src/syscall_import.rs` now owns the temporary route-code ABI shared by Component Model and core Wasm host paths.
- 2026-05-24: `hosts/macos-arm64/src/wasm.rs` adds a core Wasm runner that provides import module `""`, function `capability-call`, routes through `HostKernel`, and records the frame response.
- 2026-05-24: `cargo test -p faber-host-macos-arm64 wasm -- --nocapture` passed: 3 focused core Wasm host tests.
- 2026-05-24: `cargo test -p faber-host-macos-arm64 component -- --nocapture` passed after the shared route-code extraction: 3 component tests.
- 2026-05-24: `cargo test -p radix ad_helper -- --nocapture` passed after the host runner addition: 2 generated-helper tests.
- 2026-05-24: `cargo test -p faber-host-macos-arm64 -- --nocapture` passed after adding the core Wasm runner: 11 host tests.

## Remaining Epic 5 Proofs

- Compile a generated Faber Rust artifact to Wasm once the local Wasm target is available.
- Run that generated artifact through `faber-host-macos-arm64 wasm-call` or an equivalent test harness.
- Prove `ad "host:echo"` success and `ad "pg:query"` structured `E_NO_ROUTE` from the generated artifact, not only from the core/component WAT fixtures.
