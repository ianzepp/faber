# Goal: Define And Lower `ad` Capability Calls

**Status**: problem defined, not started
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/capability-calls/`
**Mode**: language/runtime boundary design and compiler lowering
**Commit Policy**: Commit after each completed phase and validation gate pass
**Coordinating Roadmap**: [`../faber-execution-roadmap/goal.md`](../faber-execution-roadmap/goal.md)

## Summary

Reframe `ad` as Faber's capability-call syntax: a source-level request for a named host/provider capability that can compile even when no provider implementation exists yet, then fail clearly at runtime if the capability is unresolved. Future strict verification may check provider existence and type signatures, but permissive compilation is the default goal for now.

## Problem

- `ad` is parsed and preserved through AST/HIR, but Rust codegen currently rejects it.
- Existing docs describe `ad` as an HTTP endpoint, while `examples/exempla/ad/ad.fab` uses broader provider-style calls such as file read, HTTP fetch, process execution, and notification.
- The desired model is not ordinary library import resolution. It is late-bound host/provider capability dispatch, especially in the future Faber MIR -> Wasm component -> host runtime model.
- Today there is no host implementation, so the near-term executable Rust behavior should be: compile successfully, then fail clearly at runtime when a capability has no linked provider.

## Goals

- Rename the conceptual model from endpoint dispatch to **capability calls** in planning/docs.
- Preserve `ad "name:operation" (...) → Type pro binding { ... }` as the source shape for capability calls unless later grammar work chooses otherwise.
- Allow unresolved capabilities during normal compilation.
- Emit executable target code that represents unresolved capability calls and fails clearly at runtime when no host/provider implementation is linked.
- Keep room for a future strict flag that verifies capability existence and provider signatures against expected source types.
- Align capability calls with the long-term host-runtime direction in `hosts/macos-arm64/README.md`.

## Non-Goals

- Do not implement the future Wasm host runtime in this goal.
- Do not require all capability providers to exist during normal compilation.
- Do not require provider metadata for every `ad` call before code can compile.
- Do not turn `ad` into ordinary Rust library import syntax.
- Do not solve `iace`/`cape` alternate-exit semantics beyond what is needed to produce a clear unresolved-capability failure.

## Ground Truth Researched

- `examples/exempla/ad/ad.fab`: uses `ad` for file, HTTP, process, and notification-style provider dispatch.
- `EBNF.md`: documents `adStmt` and notes that `ad` is parsed but codegen is not yet implemented.
- `explain/ad.md`: currently describes `ad` as an HTTP endpoint, which is too narrow for the desired capability-call model.
- `crates/radix/src/syntax/ast.rs`: has an `AdStmt` statement variant.
- `crates/radix/src/hir/nodes.rs`: has `HirAd` and preserves path, args, optional binding, body, and catch block.
- `crates/radix/src/codegen/rust/stmt.rs`: rejects `ad` with `ad is not supported for Rust codegen`.
- `hosts/macos-arm64/README.md`: records the long-term split where Faber source lowers to MIR/Wasm and a prebuilt host supplies HAL imports and standard capabilities.
- `hosts/macos-arm64/ARCHITECTURE.md`: captures the host-owned capability model, non-strict and strict capability compilation modes, and the direction for moving `norma` implementation into compiler core or host capabilities.
- `hosts/macos-arm64/SYSCALL_MODEL.md`: models capability calls as host syscalls using the Muninn frame/kernel pattern from `/Users/ianzepp/work/ianzepp/muninn/protocol/frames-rs` and `/Users/ianzepp/work/ianzepp/muninn/runtimes/kernel-rs`.

## Reference Packet

Before editing, inspect:

- `EBNF.md`: current `ad` grammar and explanatory note.
- `explain/ad.md`: rename/rewrite from HTTP endpoint to capability call.
- `examples/exempla/ad/ad.fab`: executable example and desired source shape.
- `crates/radix/src/parser/stmt.rs`: `ad` parsing and binding/catch shape.
- `crates/radix/src/syntax/ast.rs`: `AdStmt` representation.
- `crates/radix/src/hir/lower/stmt.rs`, `crates/radix/src/hir/nodes.rs`, `crates/radix/src/hir/visit.rs`: lowered capability-call representation.
- `crates/radix/src/semantic/passes/typecheck/stmt.rs`: argument, result-binding, body, and catch typechecking behavior.
- `crates/radix/src/codegen/rust/stmt.rs`: current fail-closed Rust rejection.
- `crates/radix/src/codegen/go/stmt.rs`: existing Go stub behavior may inform a temporary unresolved-provider shape.
- `hosts/macos-arm64/README.md`: future host-runtime constraints and capability-grant open questions.
- `hosts/macos-arm64/ARCHITECTURE.md`: host-layer architecture that should own the longer-term capability implementation plan.
- `hosts/macos-arm64/SYSCALL_MODEL.md`: syscall/frame routing model for the host side of capability calls.

## Constraints And Invariants

- Normal compilation should be permissive: unresolved capability names are allowed.
- Runtime behavior must be explicit: an unresolved capability should fail with a clear diagnostic rather than silently doing nothing.
- Future strict mode should be possible without changing ordinary source syntax.
- Source-declared result types remain meaningful even when the provider is unresolved; they tell the compiler the expected shape of the success binding.
- If the result type is omitted and no provider metadata exists, the compiler should either require an explicit type or choose a documented escape type; do not guess a specific concrete type in codegen.
- `ad` is not the collection DSL. The retired collection DSL is `ab`.
- Capability calls should fit both current Rust output and future host/Wasm imports without encoding Rust-specific library paths in language semantics.
- Capability calls are host syscalls at runtime: source syntax names a capability, generated artifacts submit a named request, and the host routes it through built-in syscall handlers or registered providers.
- Missing runtime providers should fail as structured host errors, initially equivalent to `E_NO_ROUTE` unless a more explicit `E_CAPABILITY_UNRESOLVED` code is added.

## Implementation Shape

### Phase 0: Policy And Terminology Lock

Update docs and examples to consistently call `ad` a capability call. Record the permissive default policy: unresolved capability calls compile, strict existence/signature verification is future work, and missing providers fail at runtime.

### Phase 1: Typecheck Contract

Define the minimum typechecking contract for unresolved capability calls. The important decision is how the success binding gets a type when no provider metadata exists: explicit `→ Type` should be accepted; omitted type should produce a clear diagnostic unless a deliberate escape type is chosen.

### Phase 2: Temporary Rust Runtime Stub

Teach Rust codegen to emit executable code for `ad` that calls a temporary capability-dispatch stub. The stub may panic, return a structured error, or route through a placeholder function, but it must preserve statement/body control flow enough that `ad/ad.fab` can compile and fail clearly when executed without a provider.

### Phase 3: Body And Failure Flow

Define how success bodies and `cape` handlers interact with unresolved runtime failures for the current Rust backend. If `cape` is not ready, keep that limitation explicit and do not pretend the handler works.

### Phase 4: Future Strict Verification Hook

Add only the design surface needed for a future strict flag: where capability metadata would be loaded, what diagnostic would fire for missing providers, and how declared result types would be checked. Do not require strict metadata in normal compilation.

### Phase 5: Host/Wasm Alignment Notes

Update host-runtime planning notes so capability calls are the source-level form that can later lower to Wasm component imports or host-provided functions. The host-side implementation model is a frame-shaped syscall request routed by the macOS host kernel. This phase is documentation/design alignment, not host implementation.

### Phase 6: Exempla And E2E Integration

Migrate `examples/exempla/ad/ad.fab` from unsupported-codegen failure into the right e2e class. If the near-term expected behavior is runtime failure, the e2e harness must distinguish "compiled but expected unresolved capability failure" from compiler failure.

## Acceptance Criteria

- `ad` is documented as a capability call, not only as an HTTP endpoint.
- Normal compilation of an explicit-typed unresolved capability call succeeds.
- Generated Rust for unresolved `ad` calls compiles.
- Running generated Rust without a provider fails clearly at runtime.
- The e2e corpus can classify `ad/ad.fab` according to expected unresolved-capability behavior instead of counting it as unsupported Rust codegen.
- Future strict provider verification remains possible and is not conflated with default compilation.

## Validation

- A focused `ad`/capability-call compiler test should prove an explicit result type gives the success binding the expected type even without provider metadata.
- A focused Rust codegen test should prove generated code compiles for an unresolved capability call.
- A runtime/e2e test should prove unresolved capability execution fails with the intended diagnostic.
- `cargo test -p radix ad` should pass after implementation.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` should no longer report `ad/ad.fab` as a compile/codegen failure.

## Open Questions

- What exact runtime failure shape should unresolved capability calls use before the host runtime exists: panic, process diagnostic, structured error value, or temporary dispatcher stub?
- Should source require `→ Type` for unresolved capability calls, or should omitted result types default to `ignotum`?
- How should fire-and-forget calls such as `ad "notificatio:mitte" (...)` report unresolved providers?
- Should capability names use colon namespaces (`pg:query`, `fasciculus:lege`) as the canonical spelling?
- How will future strict mode be requested: compiler flag, package config, annotation, or target profile?

## Stop Conditions

- Stop if implementation starts requiring provider metadata for normal compilation.
- Stop if Rust codegen needs to guess a concrete result type not declared in source or provider metadata.
- Stop if unresolved capability behavior would silently succeed.
- Stop before implementing a broad host runtime or Wasm component ABI as part of this goal.
- Stop if `cape`/alternate-exit semantics become the real blocker; split that into its own effects goal rather than hiding it inside capability calls.
