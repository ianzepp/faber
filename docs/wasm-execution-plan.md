# Faber WASM Execution Plan

**Status**: Aspirational design direction captured (May 2026)

This document records a possible future execution strategy for Faber as a first-class language, focused on WASM as the preferred long-term target. It is not a description of current compiler support: the active `radix-rs` target list is still `rust`, `go`, `ts`, and `faber`.

---

## Context and Motivation

Faber originated as an intermediate representation intended to make it easier for LLMs to generate, and humans to review, code that would ultimately be compiled to systems languages (Rust, Go, Zig, etc.).

Over time, the premise shifted. With current LLMs capable of writing high-quality Rust (and other target languages) directly, the original value proposition of Faber as a "safer" or "more reviewable" IR has diminished for the author's personal use.

The remaining value of Faber is now primarily:

- Aesthetic and personal ownership ("I designed this language and I like writing in it").
- The ability to actually write and run real programs in it, rather than treating it purely as a staging language.

The author's primary personal use cases are:

- Small, sharp CLI tools (replacing the current pattern of asking an LLM to generate one-off Rust CLIs).
- Applications that integrate with the MintedGeek Swarm platform over standard HTTP.

This change in motivation drives the execution strategy below.

---

## Strategic Direction

Faber is evolving into a **first-class, standalone language** with its own execution story, rather than remaining a multi-target intermediate representation.

Key decisions:

- Multi-target flexibility is deprioritized for the author's personal work.
- WASM is the preferred execution vehicle.
- The goal is to be able to write real, useful programs (especially CLI tools and Swarm-integrated services) that feel native to run and deploy.

---

## Execution Model

**Primary target**: WebAssembly (WASM).

**Preferred path**:
- Future product behavior: `faber build --target wasm <file.fab>` should produce a `.wasm` artifact with minimal manual steps.
- The implementation may go through the existing high-quality Rust code generator in `radix-rs`, followed by `rustc` targeting `wasm32-wasip1`, `wasm32-wasip2`, or `wasm32-unknown-unknown`.
- A thin Rust layer is acceptable provided it feels like a natural extension of the compiler toolchain rather than a hidden target language.

**Rationale for WASM**:
- Runs in the browser (enables web applications).
- Excellent support on Cloudflare Workers (a frequent deployment target for the author).
- Portable across Mac, Linux, and other environments via WASI or lightweight hosts.
- Provides a single, coherent execution model instead of maintaining multiple backends.

---

## Technical Approach

### Compilation Pipeline (Initial)

1. `radix-rs` compiles Faber source to high-quality Rust (reusing the existing, most mature codegen backend).
2. The Rust output is compiled with `rustc --target wasm32-wasip1`, `wasm32-wasip2`, or an equivalent Cloudflare-friendly target.
3. A small Faber-specific WASM runtime provides the necessary support for language features and the Hardware Abstraction Layer (HAL).

This approach maximizes reuse of the strongest existing compiler component while still delivering a first-class "Faber to WASM" user experience.

### Runtime Requirements

Faber will require a minimal WASM runtime to support:

- Core language primitives and collections.
- The Hardware Abstraction Layer defined in `stdlib/norma/hal/` (`pactum solum`, `consolum`, `nuncius`/HTTP, `processus`, `tempus`, `crypta`, etc.).
- Basic I/O, filesystem access (via WASI), and networking as needed for CLI tools and HTTP clients.

The runtime should be kept as small as possible for the initial use cases (CLI tools + HTTP calls to Swarm).

WASI details need a separate design pass. The old Rust target name `wasm32-wasi` has been replaced by `wasm32-wasip1`, and Cloudflare Workers support for WASI is experimental with only some syscalls implemented. Local CLI execution and Workers deployment may therefore require different runtime shims even if both use WASM.

---

## CLI Tooling and Ergonomics

A major goal is to make CLI tools written in Faber feel like normal native tools.

Desired experience:

```bash
faber build --target wasm src/mycli.fab
./mycli --help
```

This is future product behavior. Today, Faber CLI ergonomics also depend on reviving the declarative CLI lowering described in `docs/grammatica/cli.md`.

This implies the need for a small launcher story:

- Either a tiny static binary wrapper (potentially generated as part of the build).
- Or integration with a lightweight WASM host (e.g., `wasmtime`, `wasmer`, or a custom minimal host written in Rust).

The exact launcher mechanism is still to be designed, but the requirement is clear: running a compiled Faber CLI tool should not require the user to manually invoke a WASM runtime with arguments every time.

---

## Deployment Targets

### Primary

- **Cloudflare Workers** — High priority. The author frequently deploys services here. WASM modules should be deployable with a small JavaScript/Rust bindings shim or another explicit host integration; direct WASI-style deployment should not be assumed until the runtime design proves it.

### Secondary

- **Browser** — Enables writing web applications entirely in Faber.
- **Local / server execution** via WASI for CLI tools and background services.

---

## Scope for Initial Implementation

The first version of WASM support should target the author's actual near-term needs:

- Writing and running personal CLI tools.
- Building small services that speak HTTP to the MintedGeek Swarm backend.

This means the initial runtime and HAL implementation can be scoped to:

- Console / arguments / environment
- Basic filesystem access (for CLIs)
- HTTP client functionality (critical for Swarm integration)
- Time, random, and basic JSON handling

Full coverage of the existing `norma` stdlib surface is not required upfront.

---

## Open Questions and Future Work

- Exact mechanism for the CLI launcher (custom host vs. thin wrapper binary).
- How much of the HAL should be implemented in Rust (as part of the runtime) vs. in Faber itself.
- Whether a more direct WASM codegen backend (bypassing Rust) becomes desirable later for performance or purity reasons.
- Packaging and distribution story for compiled `.wasm` artifacts.
- Debugging and observability experience when running under WASM.
- Whether the first implementation should split local WASI CLI support from Cloudflare Worker support instead of treating them as one runtime target.

---

## Related Documents

- `docs/grammatica/cli.md` — Aspirational design for the declarative CLI annotation surface (the primary way the author expects to write CLI tools in Faber).
- `docs/grammatica/targets.md` — Existing (older) target compatibility notes.

---

*This document captures the strategic intent as of May 2026. It is intended as a living reference for implementation planning.*
