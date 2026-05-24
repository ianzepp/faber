# Epic 4.1 Delivery Spec: Host Kernel Route Proof

## Interpreted Problem

Epic 4 needs the macOS host to stop being a placeholder and prove the frame-shaped syscall model inside `hosts/macos-arm64`. The first proof should not force a daemon, Wasm loader, shared host crate, or full Muninn import before the route contract exists.

## Normalized Spec

- Add Faber-owned host kernel source under `hosts/macos-arm64/src/`.
- Represent a host call as a `Frame` with request correlation, status, trace, and data.
- Route requests by call prefix to built-in syscalls.
- Return structured `E_NO_ROUTE` errors for unresolved calls.
- Add a built-in `host:echo` syscall.
- Add manifest output listing built-in syscalls and registered providers.
- Add CLI route-proof commands:
  - `manifest`
  - `call <name> [json-object]`
- Record Muninn provenance as semantic adaptation, not a runtime dependency.
- Record the first HAL migration candidate without migrating all of `norma`.

## Repo-Aware Baseline

- `hosts/macos-arm64` is already a workspace member.
- The crate currently has only a placeholder `src/main.rs` and no dependencies.
- `hosts/macos-arm64/SYSCALL_MODEL.md` allows the first route proof to be launcher-style and in-process.
- `stdlib/norma/hal/consolum.fab` and `crates/norma/hal/consolum.rs` are good first HAL migration candidates because console I/O is outside-world behavior already described as HAL.

## Stage Graph

1. Add kernel data contracts: `Frame`, `Status`, `HostError`.
2. Add syscall trait, prefix router, host kernel, and `host:echo`.
3. Add manifest model and CLI commands.
4. Add tests for manifest, success route, unknown route, and CLI JSON behavior.
5. Update host README with route-proof usage, provenance, topology, and HAL candidate.

## Checkpoints

- `cargo test -p faber-host-macos-arm64` passes.
- `cargo run -p faber-host-macos-arm64 -- manifest` prints JSON containing `host:echo`.
- `cargo run -p faber-host-macos-arm64 -- call host:echo '{"value":"salve"}'` returns a done frame.
- `cargo run -p faber-host-macos-arm64 -- call pg:query '{}'` returns an error frame with `E_NO_ROUTE`.
- `cargo tree -p faber-host-macos-arm64` has no Muninn dependency.

## Gate Plan

The phase is complete when the route proof works from both tests and CLI, provenance is documented, and the crate has no Muninn runtime dependency. Wasm/component loading remains the next phase unless the route proof exposes a blocking architectural decision.
