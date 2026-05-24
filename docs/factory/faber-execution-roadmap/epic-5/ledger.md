# Epic 5 Ledger: Rust-To-Wasm Faber Host Syscall Bridge

## Status

- `5.1` implemented: generated Rust helper bridge from `__faber_ad(...)` to a Wasm host import.
- `5.2` implemented: core Wasm host runner for the generated Rust helper's `capability-call` import.
- `5.3` implemented: generated Faber Rust-to-Wasm artifact proof passed through the macOS host.

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
- 2026-05-24: Generated Wasm helper now imports `capability-text-len` and `capability-text-read`; `__faber_ad::<String, _>(...)` reads the host-returned text payload instead of returning `String::default()`.
- 2026-05-24: `scripta/prove-epic5-wasm` added as the durable generated-artifact proof harness. It uses `WASM_RUSTC` for the Wasm compile so Cargo workspace builds remain on the normal host compiler.
- 2026-05-24: A temporary rustup toolchain was installed under `/tmp/faber-rustup-epic5` with `wasm32-unknown-unknown`. This did not modify the repo or shell profile.
- 2026-05-24: `CARGO_HOME=/tmp/faber-rustup-epic5/cargo RUSTUP_HOME=/tmp/faber-rustup-epic5/rustup WASM_RUSTC=/tmp/faber-rustup-epic5/cargo/bin/rustc ./scripta/prove-epic5-wasm` passed. The generated Faber Rust artifact compiled to Wasm, `host:echo` returned a done frame containing `"salve"`, and `pg:query` returned an error frame with `E_NO_ROUTE`.

## Completion Audit

- Native HIR-to-Rust behavior remains explicit unresolved-provider behavior, proven by `cargo test -p radix ad_helper -- --nocapture` and the earlier native generated Rust sanity check.
- Wasm-targeted generated Rust lowers `ad` through `__faber_ad(...)` to `__faber_syscall(...)`, proven by generated-helper tests.
- The macOS host can instantiate core Wasm modules and provide the generated helper imports, proven by `cargo test -p faber-host-macos-arm64 -- --nocapture`.
- Generated Faber Rust compiled to Wasm and reached `HostKernel` for `host:echo` and `pg:query`, proven by `scripta/prove-epic5-wasm`.
- `host:echo` returned the frame-derived text value to generated Faber code before the export harness reported success.
- `pg:query` returned structured `E_NO_ROUTE`.
- No strict provider verification, final WIT world, daemon transport, full `norma` migration, or MIR-exclusive Wasm codegen was introduced.
