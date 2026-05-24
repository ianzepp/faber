# Faber macOS arm64 Host Architecture Goal

**Status**: architecture direction captured, not implemented
**Created**: 2026-05-24
**Host Crate**: `hosts/macos-arm64`
**Implementation Language**: Rust
**Syscall Model**: [`SYSCALL_MODEL.md`](SYSCALL_MODEL.md)

## Summary

The macOS arm64 host should become the installed runtime environment for Faber artifacts. Faber programs should compile core language behavior into a portable artifact and express outside-world functionality as host capabilities. The host supplies those capabilities at runtime. This moves standard-library implementation concerns out of each Faber-produced binary and into a versioned, platform-specific host layer.

## Target Model

The intended execution path is:

```text
Faber source
  -> typed Faber program
  -> Faber MIR
  -> Wasm component
  -> Rust host runtime
  -> macOS arm64
```

Assume the Wasm Component Model as the compatibility boundary unless later evidence proves a simpler or stricter ABI is better. Bare Wasm alone is too low-level for the capability/interface model because it does not naturally carry the typed import/export world that Faber needs for host capabilities.

## Core Split

Faber should move toward a **core + capabilities** architecture:

### Compiler-Owned Core

The compiler owns pure language behavior and lowers it directly through MIR/Wasm or target codegen:

- primitive values such as `textus`, `numerus`, `fractus`, `bivalens`, `nihil`, and `vacuum`,
- control flow,
- functions and closures,
- option/null behavior,
- lists, maps, sets, tuples, structs, and variants,
- pattern matching,
- pure collection operations that are part of the language contract,
- formatting or conversion operations that are defined as language semantics.

These should not require a separately linked standard library implementation in each generated program.

### Host-Owned Capabilities

Anything that touches the outside world should be represented as a host capability:

- filesystem and path access,
- console and terminal IO,
- process execution,
- environment access,
- clock/time,
- randomness,
- HTTP/network,
- database calls such as `pg:query`,
- OS-specific adapters,
- future installed provider plugins.

Faber source can request these capabilities through `ad` capability calls or through higher-level interfaces that lower to the same host import model.

## Norma Direction

`norma` should stop being treated as "stdlib implementation code copied or linked into every Faber binary" as the final architecture.

Instead, split today's norma surface into:

- **core language contracts** that the compiler owns directly,
- **host capability/interface contracts** that the host provides,
- **temporary Rust backend support** that exists only while generated Rust remains the practical executable path.

The `.fab` files under `stdlib/norma` can remain useful as interface definitions, metadata, and canonical Faber-facing contracts. They should not imply that all implementation must be bundled into each compiled artifact.

The future role of `norma` may become:

```text
norma = canonical namespace for built-in Faber capability/interface contracts
```

not:

```text
norma = standard library code statically linked into every Faber program
```

## Capability Compilation Modes

Capability calls should support two compilation modes.

### Non-Strict Mode

Non-strict mode is the default authoring mode.

- Source-declared capability names do not need to exist on a host at compile time.
- Source-declared result types are accepted as the expected success shape.
- The body typechecks against that declared shape.
- The compiled artifact records the required capability import.
- Running without a provider fails clearly at runtime.

Example:

```fab
ad "pg:query" ("select * from users") → lista<tabula<textus, ignotum>> pro rows {
    nota rows
}
```

This should compile without a known `pg:query` provider. It should fail only when executed by a host that cannot resolve `pg:query`, unless strict verification is requested.

### Strict Mode

Strict mode compiles against a selected host profile or exported capability manifest.

The compiler verifies:

- each required capability exists,
- argument count and types are compatible,
- declared result types are compatible with host-exported result contracts,
- required permissions or grants are declared,
- capability versions/features match policy.

Strict mode is for deployment certainty. It should not be required for ordinary local authoring.

## Host Responsibilities

The Rust host is responsible for:

- loading Faber-produced Wasm components,
- exposing capability imports as host syscalls,
- maintaining a registry of installed providers,
- exporting a machine-readable capability contract manifest for strict compilation,
- enforcing capability grants and permissions,
- translating host errors into Faber-visible runtime failures,
- delegating to WASI where WASI fits,
- implementing macOS-specific adapters where WASI is insufficient,
- providing launcher behavior for native-feeling Faber tools.

## Lowering Boundary

Faber-to-Wasm lowering should not encode Rust crate paths, Cargo dependencies, or platform library names into source-level language semantics.

The lowered artifact should express imports such as:

```text
capability: pg:query
args:       [textus]
result:     lista<tabula<textus, ignotum>>
```

The host decides whether that maps to:

- a Rust function built into the host,
- a dynamically loaded provider,
- a system library,
- a WASI interface,
- a network service,
- or an unresolved-capability failure.

Internally, these capability imports should route through the frame/syscall model described in [`SYSCALL_MODEL.md`](SYSCALL_MODEL.md). The host should treat `ad "pg:query"` as a request frame whose `call` is `pg:query`, then route it through built-in syscall handlers or registered external providers.

## First Implementation Slice

The first useful host implementation should be deliberately small:

1. Select Wasmtime and the Wasm Component Model, unless a repo-aware design pass finds a better current Rust implementation path.
2. Add the smallest host-internal frame/syscall router needed for one request and one terminal response.
3. Define a tiny Faber-owned component world with one entrypoint and one host capability import.
4. Build a Rust host that loads a minimal component and reports unresolved capability imports clearly.
5. Add a host capability registry shape and a static exported manifest.
6. Prove non-strict vs strict behavior with one fake capability such as `pg:query` or `host:echo`.

This first slice should not attempt to migrate all of `norma`.

## Later Migration Shape

After the first host slice works:

- classify `stdlib/norma` files into compiler core, host capability, or temporary Rust backend support,
- move HAL/provider surfaces toward host capability contracts,
- keep pure collection and data operations in compiler-owned core or ordinary built-in type APIs,
- add strict host manifest checks to `faber build`,
- add host bundle/launcher packaging,
- gradually reduce generated Rust runtime linkage as Wasm-host execution matures.

## Open Questions

- Should the first component world be `wasi:cli/command`, a Faber-owned command world, or a wrapper that supports both?
- What exact format should the host capability manifest use: WIT, Faber interface files, JSON/TOML metadata, or generated Rust metadata?
- How should capability grants be represented for filesystem, process, network, database, and environment access?
- Should unresolved capabilities fail during component instantiation or only when called?
- Which parts of `norma` are true compiler core versus host capability interfaces?
- How should host provider versions be matched in strict mode?

## Non-Goals

- Do not write the host in Faber, Go, or generated user code.
- Do not migrate all standard-library behavior in one pass.
- Do not make ordinary Faber programs depend on per-build Rust codegen as the final runtime model.
- Do not require strict host verification for non-strict authoring builds.
- Do not design a full plugin marketplace or package manager as part of the first host slice.

## Stop Conditions

- Stop if the implementation starts embedding platform-specific host details into Faber source semantics.
- Stop if the host design requires all providers to exist before non-strict builds can compile.
- Stop if raw Wasm forces ad hoc untyped ABI glue where the Component Model would preserve better contracts.
- Stop before replacing current Rust backend validation; the host path should be additive until proven.
