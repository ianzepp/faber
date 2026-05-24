# Faber macOS arm64 Host

This crate is the first Faber-owned proof of the future prebuilt host runtime
for macOS on Apple Silicon.

## Intent

The long-term execution model is:

```text
Faber source
  -> Faber MIR
  -> Wasm component
  -> Faber host runtime
  -> macOS arm64
```

The important split is that ordinary Faber programs should not generate Rust
source and invoke `rustc` for every build. Rust remains a practical
implementation language for the host, but the host is built once for each
platform and then runs portable Faber-produced Wasm artifacts.

This host is expected to eventually:

- load a Faber-produced Wasm component,
- provide Faber HAL imports for `norma:hal/*`,
- delegate standard capabilities to WASI where the WASI world fits,
- provide macOS-specific adapters where WASI is not enough,
- expose a launcher shape that makes Faber tools feel like native commands.

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the current host-layer goal:
Rust host implementation, Wasm Component Model compatibility, non-strict and
strict capability compilation modes, and the direction for moving `norma`
functionality into compiler-owned core behavior or host-provided capabilities.
See [`SYSCALL_MODEL.md`](SYSCALL_MODEL.md) for the host-internal model where
Faber capability calls become frame-shaped syscalls routed through built-in host
handlers or registered providers.

## Current Route Proof

The current executable proves the host-internal frame route contract without
requiring a background daemon or Wasm loader yet.

```bash
cargo run -p faber-host-macos-arm64 -- manifest
cargo run -p faber-host-macos-arm64 -- call host:echo '{"value":"salve"}'
cargo run -p faber-host-macos-arm64 -- call pg:query '{}'
cargo run -p faber-host-macos-arm64 -- component-call hosts/macos-arm64/tests/fixtures/route-proof.wat route 1
```

The `manifest` command emits a JSON capability manifest containing built-in
syscalls and registered providers. The first built-in syscall is `host:echo`.

The `call` command builds a request `Frame`, routes it through the in-process
host kernel, and prints the response frame as JSON. Unknown calls such as
`pg:query` return a structured `E_NO_ROUTE` error frame. The command exits with
status `2` when the response frame has `status = "error"`.

The `component-call` command loads a Wasm component, calls a named exported
function, and exposes one host import named `capability-call`. The current proof
ABI is intentionally tiny: route code `1` maps to `host:echo`, and route code
`2` maps to unresolved `pg:query`. The host import wraps that route code into a
`Frame`, routes it through the same `HostKernel`, and prints the routed response
frame as JSON. This proves the Wasm/component boundary without committing the
final Faber WIT world or string ABI.

## Frame Topology

Frames are the durable contract. The first route proof keeps frames as Rust
values inside a launcher-style process, but the same frame shape is intended to
cross later boundaries:

- Wasm component imports that the host wraps into frames,
- Unix domain sockets for a future `serve` mode,
- JSON for local/debug transport,
- compact binary frame streams for production provider communication.

This means the host can grow into a daemon or hybrid in-process/provider model
without replacing the syscall protocol.

## Muninn Provenance

The kernel shape is semantically adapted from the Muninn frame and kernel work
under:

- `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs`
- `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs`

This crate does not depend on Muninn as a runtime crate. The current code
reimplements only the small Faber-owned subset needed for `Frame`, `Status`,
prefix routing, structured `E_` errors, `host:echo`, and manifest output.

## First HAL Migration Candidate

`norma:hal/consolum` is the first recorded HAL migration candidate. Console I/O
is outside-world behavior, and the existing split between
`stdlib/norma/hal/consolum.fab` and `crates/norma/hal/consolum.rs` makes it a
good next surface to move toward host-owned frame syscalls after the route proof.

## Non-Goals For This Slice

- No Wasmtime or WASI dependency is selected yet.
- No Faber HAL import ABI is locked yet.
- No background daemon/server lifecycle is implemented yet.
- No packaging format is defined yet.
- No per-program Rust codegen should move here.

## Open Design Questions

- Which Wasm component world should be the first supported surface:
  `wasi:cli/command`, a Faber-owned `faber:cli/command`, or both?
- Should HAL imports use direct Faber WIT packages such as
  `faber:hal/consolum`, or should they map one-to-one onto existing WASI
  interfaces when possible?
- Should `faber build` produce a bare `.wasm` component, a host-specific
  launcher bundle, or both?
- How should capability grants be represented for filesystem, process, network,
  and environment access?

Status: route proof and minimal component import proof implemented. Bene currit
for this epic when direct calls and component imports both route through the host
kernel and unresolved calls produce `E_NO_ROUTE`.
