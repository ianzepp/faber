# Faber macOS arm64 Host

This crate is a placeholder for the future prebuilt Faber host runtime for
macOS on Apple Silicon.

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

## Non-Goals For This Placeholder

- No Wasmtime or WASI dependency is selected yet.
- No Faber HAL import ABI is locked yet.
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

Status: placeholder only. Bene currit when this crate can execute a minimal
Faber-produced Wasm component without the program passing through generated Rust.
