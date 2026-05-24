# Faber Host Syscall Model

**Status**: design direction captured, not implemented
**Created**: 2026-05-24
**Host Crate**: `hosts/macos-arm64`
**Primary References**:

- `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs`
- `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs`

## Summary

Faber capability calls should be modeled as host syscalls.

The source-level spelling can remain:

```fab
ad "pg:query" ("select * from users") → lista<tabula<textus, ignotum>> pro rows {
    nota rows
}
```

but the runtime meaning should be:

```text
Faber component asks host kernel to perform syscall "pg:query"
```

This keeps `ad` as a language feature while making the implementation use a known operating-system shape: a caller submits a named operation and a payload to a privileged host layer, the host routes the request to a handler, then returns success, stream items, cancellation, or a structured error.

## Borrow From Muninn

The relevant Muninn material now lives in the monorepo at `/Users/ianzepp/work/ianzepp/muninn`, specifically:

- `protocol/frames-rs`: transport-friendly frame envelope and protobuf codec.
- `runtimes/kernel-rs`: in-memory kernel, syscall routing, sigcall registry, cancellation, backpressure, and structured errors.

The Faber host should adopt the model before deciding whether to copy, vendor, or depend on code directly.

The pieces worth carrying forward are:

- A universal `Frame` envelope for host calls.
- Request correlation through `id` and `parent_id`.
- A lifecycle of `Request -> Item*/Bulk* -> Done|Error|Cancel`.
- Business payload separated from trace/observability metadata.
- Prefix routing from names such as `pg:query`, `http:get`, and `host:echo`.
- A syscall table for host built-ins.
- A sigcall-style registry for external providers and plugins.
- Structured error codes with retryability.
- Cooperative cancellation.
- Backpressure for streaming responses.
- Transport-agnostic routing inside the host kernel.

## Frame Shape

Muninn's frame shape is the right starting point for host-internal messages:

```text
Frame {
  id,
  parent_id,
  created_ms,
  expires_in,
  from,
  call,
  status,
  trace,
  data,
}
```

For Faber:

- `call` is the capability/syscall name, such as `pg:query`.
- `data` carries the Faber arguments after ABI conversion.
- `trace` carries diagnostics, spans, timings, grants, and host observability metadata.
- `parent_id` correlates response frames to the request.
- `status` defines whether the frame starts, continues, completes, fails, or cancels the call.

The host kernel should keep this as a Rust-native in-memory type. Serialization belongs at boundaries: Wasm component imports, provider processes, sockets, logs, or future remote hosts.

## Routing Model

Capability names should use colon namespaces:

```text
prefix:verb
```

Examples:

- `host:echo`
- `pg:query`
- `http:get`
- `fs:read`
- `process:run`

The host kernel routes by prefix first. Built-in host handlers own prefixes. External providers may register exact names or provider-owned prefixes through a sigcall-style registry.

```text
Faber component
  -> host syscall frame
  -> host kernel
  -> built-in syscall handler or registered provider
  -> response frames
  -> Faber component
```

If no built-in route and no registered provider exists, the host returns a structured error frame. For default non-strict compilation, this is the expected unresolved-capability failure mode.

## Syscalls And Sigcalls

The useful distinction from Muninn is:

```text
Syscall: caller -> host kernel -> built-in handler -> host kernel -> caller
Sigcall: caller -> host kernel -> registered external handler -> host kernel -> caller
```

For Faber:

- **Syscalls** are capabilities implemented by the host itself.
- **Sigcalls** are capabilities implemented by installed providers, plugins, sidecars, or future dynamically loaded libraries.

This gives Faber a clean path for both standard host functionality and application/provider extensions without changing source syntax.

## Error Model

Host errors should be machine-readable and stable. Muninn's `E_`-prefixed error convention is appropriate:

- `E_NO_ROUTE`: no host or provider handler exists for the call.
- `E_INVALID_ARGS`: the payload cannot be decoded into the handler's expected arguments.
- `E_FORBIDDEN`: grants or policy deny the call.
- `E_TIMEOUT`: the operation exceeded a timeout and may be retryable.
- `E_CANCELLED`: caller cancelled the operation.
- `E_INTERNAL`: host/provider invariant failure.

The first Faber-specific unresolved-capability runtime failure can map to `E_NO_ROUTE`. If later diagnostics need a more explicit code, add `E_CAPABILITY_UNRESOLVED` as a host-layer alias or subtype without changing the source language.

## Wasm Boundary

The host-internal model should be frame-shaped even if the first Wasm import is smaller than a full frame ABI.

Two plausible first boundaries are:

```text
host.capabilities.call(name, args) -> result
```

or:

```text
host.kernel.syscall(frame) -> frame-stream
```

The first is easier to implement. The second exposes the real model more directly. Either way, the host should translate the import into a frame request internally, route it through the same kernel path, and translate the response back to the component.

The long-term target should support streaming (`Item`/`Bulk`), cancellation, and structured errors. A first slice may support only one request and one terminal response.

## Compiler Meaning

An `ad` statement should lower to a capability syscall request:

```text
ad "pg:query" (sql) → Rows pro rows { body }
```

becomes:

```text
request call: "pg:query"
data:         encoded arguments
expected:     Rows
on done:      decode response payload as Rows, bind rows, run body
on error:     raise or route to capability failure handling
```

In non-strict mode, the compiler does not need the provider's implementation or exported contract. It only needs enough source type information to typecheck the success body.

In strict mode, the compiler checks the selected host manifest before build:

- the syscall exists,
- argument shapes are compatible,
- result shape is compatible,
- grants are declared,
- provider version/policy is acceptable.

## Relationship To Norma

This model supports removing the idea that all standard-library behavior must be linked into each Faber binary.

The split becomes:

- Language core: compiled directly by the compiler.
- Host syscalls: outside-world and provider functionality routed through the host kernel.
- Norma interfaces: canonical Faber-facing names, contracts, and metadata for language/core and host/syscall surfaces.

Norma can describe `http:get`, `fs:read`, `pg:query`, and other capabilities without forcing their implementations into generated artifacts.

## First Implementation Slice

Do not create a common host crate first. Prove the model in `hosts/macos-arm64`, then extract only after duplication or cross-host pressure exists.

The first implementation should:

1. Add a host-internal `Frame`, `Status`, and structured `HostError` modeled after Muninn.
2. Add prefix routing and an unresolved-route error.
3. Add one built-in syscall, such as `host:echo`.
4. Add a manifest command that lists built-in syscalls and any registered providers.
5. Wire one Wasm/component import to the frame router.
6. Return `E_NO_ROUTE` for a call such as `pg:query` when no provider is installed.
7. Keep strict host verification out of the first runtime slice, except for the manifest shape needed later.

## What Not To Steal Yet

Do not copy the entire Muninn kernel into Faber before the first host slice has a concrete component calling into it.

The risks are:

- importing abstractions before the Wasm boundary is proven,
- creating a shared crate too early,
- carrying transport features before Faber needs them,
- binding Faber's host lifecycle to Muninn's runtime lifecycle accidentally.

The right near-term move is to borrow the frame/kernel/syscall semantics and implement the smallest macOS host version. After that, decide whether to vendor/copy Muninn modules, depend on Muninn crates, or extract a common package.

## Open Questions

- Should the Wasm import expose a full frame call immediately, or should it start as `call(name, args)` and wrap frames internally?
- Should provider registration use exact call names first, prefix ownership first, or both?
- Where should grants live in the frame: `trace`, `data`, a separate host context, or the host manifest?
- Should `E_CAPABILITY_UNRESOLVED` exist as a distinct error code, or is `E_NO_ROUTE` enough?
- How much of Muninn's stream controller is needed before Faber supports streaming capability calls?
