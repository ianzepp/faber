# Epic 5 Ledger: Rust-To-Wasm Faber Host Syscall Bridge

## Status

- `5.1` implemented: generated Rust helper bridge from `__faber_ad(...)` to a Wasm host import.

## Evidence Log

- 2026-05-24: Local toolchain check found Homebrew `rustc 1.95.0` and no `rustup`.
- 2026-05-24: `rustc --target wasm32-unknown-unknown` failed with `can't find crate for std`, so this machine cannot currently compile generated Rust to a Wasm artifact without adding the target standard library/tooling.
- 2026-05-24: `crates/radix/src/codegen/rust/prelude.rs` now emits cfg-separated native and Wasm `__faber_ad(...)` helpers. Native builds keep explicit unresolved-provider `E_NO_ROUTE` behavior. Wasm builds import `capability-call` as `__faber_syscall(...)` and map `host:echo` to route code `1` and `pg:query` to route code `2`.
- 2026-05-24: `cargo test -p radix ad_helper -- --nocapture` passed: 2 focused codegen tests.
- 2026-05-24: `cargo test -p faber-host-macos-arm64 component -- --nocapture` passed: 3 host component tests.
- 2026-05-24: Native generated Rust sanity check passed for an `ad "host:echo"` sample: generated Rust compiled with `rustc --edition=2021` and printed `E_NO_ROUTE: unresolved capability host:echo`.

## Remaining Epic 5 Proofs

- Compile a generated Faber Rust artifact to Wasm once the local Wasm target is available.
- Run that generated artifact through the macOS host import path.
- Prove `ad "host:echo"` success and `ad "pg:query"` structured `E_NO_ROUTE` from the generated artifact, not only from the host fixture.
