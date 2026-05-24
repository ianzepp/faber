# Epic 6.2 Delivery: Host-Owned Consolum Syscalls

## Interpreted Problem

Epic 6.1 classified `stdlib/norma/hal/consolum.fab` as the first
`host-effect` contract and `crates/norma/hal/consolum.rs` as temporary
`rust-bridge` support. Epic 6.2 should make that classification executable in
the macOS host by exposing the current `consolum:<member>` identities through
the host syscall router and manifest.

## Normalized Spec

Add a Faber-owned `consolum` built-in syscall handler under
`hosts/macos-arm64`:

- register the `consolum` prefix in `HostKernel`,
- list all current `stdlib/norma/hal/consolum.fab` members in the host
  manifest with canonical names from `norma-classification.md`,
- route output and TTY predicate calls through frame-shaped host syscalls,
- keep invalid payloads on the existing generic `E_INVALID_ARGS` path,
- keep unresolved names on the existing generic `E_NO_ROUTE` path,
- preserve `crates/norma/hal/consolum.rs` unchanged as native Rust bridge
  support.

## Repo-Aware Baseline

The current host kernel already provides:

- `Syscall` and `SyscallInfo` in `hosts/macos-arm64/src/kernel/syscall.rs`,
- prefix routing in `hosts/macos-arm64/src/kernel/router.rs`,
- manifest export in `hosts/macos-arm64/src/manifest.rs`,
- `HostError::invalid_args` and `HostError::no_route`,
- direct route and CLI manifest tests in
  `hosts/macos-arm64/tests/host_kernel_test.rs`.

`consolum` should follow the existing `host:echo` pattern rather than creating
a new routing abstraction.

## Stage Graph

1. Add a host kernel module for the `consolum` syscall handler.
2. Register the handler in `HostKernel::new`.
3. Export the handler through `kernel::mod`.
4. Add focused tests proving:
   - manifest lists `consolum` identities,
   - a text output call returns a `done` frame,
   - TTY predicates return boolean frame data,
   - bad payloads return `E_INVALID_ARGS`,
   - unknown `consolum:*` names return `E_NO_ROUTE`.
5. Update the Epic 6 ledger with implementation and validation evidence.

## Constraints

- Do not change compiler lowering.
- Do not add strict-mode validation.
- Do not alter host dependency provisioning.
- Do not delete or replace `crates/norma/hal/consolum.rs`.
- Do not invent new Faber annotation syntax.
- Do not create a shared host crate.

## Gate Plan

Run:

```bash
cargo test -p faber-host-macos-arm64
git diff --check -- hosts/macos-arm64 docs/factory/faber-execution-roadmap/epic-6
```

Then run a poker-face pass against this delivery spec and the Epic 6 checkpoint
before committing.
