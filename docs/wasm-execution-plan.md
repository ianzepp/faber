# Faber WASM Execution Plan

**Status**: Aspirational design direction captured (May 2026); updated after MIR closeout

This document records a possible future execution strategy for Faber as a first-class language, focused on WASM as the preferred long-term target. It is not a description of current compiler support: the active `radix-rs` target list is still `rust`, `go`, `ts`, and `faber`.

The compiler now has a validated MIR inspection branch and a deliberately temporary MIR-to-Rust probe. That work is an important prerequisite for WASM because it proves a typed, execution-shaped compiler boundary below HIR, but it is not yet a WASM backend, ABI, runtime, package format, or user-facing target.

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

**Current enabling work**:
- `radix mir` prints validated MIR for compiler-development inspection.
- MIR lowering currently covers functions, locals, temporaries, primitive expressions, calls, control flow, alternate exits, structured local handlers, aggregate and option/null operations, selected runtime intrinsics, provider identity, and selected collection operations.
- Successful MIR lowering runs through validation before it leaves the MIR layer.
- A temporary MIR Rust probe proves that validated MIR can be consumed by an executable target-like emitter for a narrow primitive/control-flow subset.

**Preferred path**:
- Future product behavior: `faber build --target wasm <file.fab>` should produce a `.wasm` artifact with minimal manual steps.
- The implementation may go through the existing high-quality Rust code generator in `radix-rs`, followed by `rustc` targeting `wasm32-wasip1`, `wasm32-wasip2`, or `wasm32-unknown-unknown`.
- A thin Rust layer is acceptable provided it feels like a natural extension of the compiler toolchain rather than a hidden target language.
- A future direct or lower-level WASM backend should start from validated MIR rather than from source-shaped HIR. The MIR layer is the right place to make control flow, storage, runtime calls, aggregate operations, and nullable behavior explicit before choosing a WASM ABI.

**Rationale for WASM**:
- Runs in the browser (enables web applications).
- Excellent support on Cloudflare Workers (a frequent deployment target for the author).
- Portable across Mac, Linux, and other environments via WASI or lightweight hosts.
- Provides a single, coherent execution model instead of maintaining multiple backends.

---

## Technical Approach

### Compilation Pipeline (Current Conservative Path)

1. `radix-rs` compiles Faber source to high-quality Rust (reusing the existing, most mature codegen backend).
2. The Rust output is compiled with `rustc --target wasm32-wasip1`, `wasm32-wasip2`, or an equivalent Cloudflare-friendly target.
3. A small Faber-specific WASM runtime provides the necessary support for language features and the Hardware Abstraction Layer (HAL).

This approach maximizes reuse of the strongest existing compiler component while still delivering a first-class "Faber to WASM" user experience.

This path remains the shortest product path because normal Rust output still uses the stable HIR-to-Rust backend. The MIR Rust probe is intentionally not a replacement for that backend.

### Compilation Pipeline (MIR-Backed Path)

The MIR closeout adds a more durable path for future lower targets:

```text
Source -> Lex -> Parse -> HIR -> Typecheck + Analysis -> Validated MIR -> WASM backend
```

For WASM, validated MIR should become the boundary where backend work begins. It already normalizes enough of the language to make the next design questions concrete:

- how functions and `incipit` export into a WASM module,
- how Faber primitives, strings, options, structs, enums, arrays, maps, and sets map to memory and ABI layout,
- how runtime intrinsics import host functions,
- how provider calls cross the host boundary,
- how diagnostics and traps surface through a WASI or Worker host,
- how source locations survive into debugging and observability.

The missing work is not another syntax pass. It is a backend/runtime project: ABI, layout, runtime imports, host integration, packaging, and a validation harness.

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

This is future product behavior. Today, declarative CLI lowering exists for the Rust target; a WASM target would need
its own launcher/runtime contract before compiled Faber CLI tools feel native.

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

MIR narrows the first implementation scope by separating language semantics from target emission. A WASM spike should begin with the validated MIR subset rather than the full source language:

- exported functions and `incipit`,
- primitive values and direct calls,
- simple branches and loops,
- strings and formatted diagnostics,
- option/null operations,
- small aggregate values,
- the minimal runtime intrinsic imports needed for console, arguments, environment, time, random, JSON, and HTTP.

Unsupported MIR shapes should fail closed with explicit diagnostics, matching the MIR Rust probe policy, instead of falling back through Rust silently or inventing partial WASM behavior.

---

## Open Questions and Future Work

- Whether the first user-visible WASM support should be Rust-mediated or MIR-backed.
- WASM ABI and memory layout for Faber primitives, strings, options, structs, enums, arrays, maps, and sets.
- Export model for top-level functions, `incipit`, tests, and CLI entrypoints.
- Runtime intrinsic import contract for diagnostics, formatting, conversions, collections, providers, and HAL calls.
- Validation harness, likely Wasmtime or an equivalent host, for compiling and executing MIR-backed WASM fixtures.
- Exact mechanism for the CLI launcher (custom host vs. thin wrapper binary).
- How much of the HAL should be implemented in Rust (as part of the runtime) vs. in Faber itself.
- Whether a more direct WASM codegen backend (bypassing Rust) becomes desirable later for performance or purity reasons.
- Packaging and distribution story for compiled `.wasm` artifacts.
- Debugging and observability experience when running under WASM.
- Whether the first implementation should split local WASI CLI support from Cloudflare Worker support instead of treating them as one runtime target.

---

## Related Documents

- `explain/cli.md` — Current user reference for the declarative CLI annotation surface.
- `explain/targets.md` — Current target compatibility entry; deeper target notes belong in `EBNF.md` or implementation plans.
- `docs/factory/mir-layer/ledger.md` — Current MIR closeout record, including validated MIR scope, temporary MIR Rust probe, and lower-target prerequisites.
- `docs/factory/mir-layer/phase-9.5-delivery.md` — MIR closeout and hardening delivery plan.

---

*This document captures the strategic intent as of May 2026. It is intended as a living reference for implementation planning.*
