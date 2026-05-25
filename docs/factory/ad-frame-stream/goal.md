# Goal: Rewrite `ad` Around The Frame Protocol

**Status**: drafted, not started
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/ad-frame-stream/`
**Mode**: language semantics, compiler lowering, host/runtime contract
**Commit Policy**: Commit after each completed implementation phase and validation gate pass
**Related Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`

## Summary

Redefine `ad` as Faber's frame-protocol syscall form. An `ad` call sends a host capability request and consumes the response as zero or more `item` frames and/or transparent `bulk` batches followed by a hard terminal `done`, `error`, or `cancel` frame. The success binding type in `ad "route" (...) → T item { ... }` describes each successful item payload, not a single scalar return value. Scalar behavior remains expressible through helper functions that consume the frame stream and return from the enclosing function.

## Problem

- Current `ad` planning and Rust lowering treat `→ T name` as a scalar `Result<T, String>` success path.
- The host model now treats capability calls as frame-shaped syscalls, and the useful runtime primitive is a request/response frame protocol, not a provider SDK return value and not a Faber `@ cursor` generator.
- Provider SDKs should terminate at the host/provider boundary. Faber should receive only frame data and decode it into caller-owned local shapes.
- The natural source form is stream-oriented:

```fab
ad "db:query" (sql) → Row row {
  nota row.name
}
```

- That form reads as "for each returned row, run this block." Treating it as a scalar continuation hides the real runtime shape and makes future database rows, HTTP chunks, log streams, subscriptions, and provider processes awkward.

## Goals

- Make `ad` frame-protocol-shaped by default. A frame stream is not a Faber `@ cursor`: it has an explicit request, protocol envelopes, and a hard terminal `done`, `error`, or `cancel`.
- Define the success binding type as the per-item frame data decode target.
- Define the `ad` body as the per-item body, executed once for each successful item payload.
- Define `bulk` as a transparent transport batching optimization: a `bulk` frame carries multiple item payloads inside one frame envelope, and the compiler/runtime expands those payloads through the same per-item path used for individual `item` frames.
- Preserve the current source spelling where possible:

```fab
ad "provider:operation" (args) → Type item {
  ...
}
```

- Treat terminal `done` as completion of the `ad` statement, not as the scalar value. Primary success data arrives through `item` or expanded `bulk` payloads, while `done` may carry only terminal metadata for current/future helpers.
- Treat terminal `error` as host/provider-initiated failure routed through existing/future `cape` or alternate-exit behavior.
- Treat terminal `cancel` as caller-initiated cancellation. Even if cancellation is represented through an error-shaped value in early lowering, it must remain distinguishable as cancellation because the caller requested it.
- Keep provider SDK and crate-specific response types out of Faber compilation. The provider adapter converts SDK values into frame data; Faber decodes frame data into local source-declared shapes.
- Treat omitted normal success channels as effect-only, not as implicit value-return inference. A bodyful function or effect-only `ad` form with no `→` should not contain `redde`; if a caller wants to return a value from a frame item, it must declare a success binding and/or normal return type explicitly.
- Preserve scalar convenience through ordinary wrapper functions:

```fab
functio legeUnum(textus path) → textus {
  ad "solum:lege" (path) → textus body {
    redde body
  }

  iace "empty response"
}
```

- Leave room for future collection helpers such as "first item", "exactly one item", "collect all items", and cursor/rivus adapters without adding new syntax in the first phase.
- Effect-only calls should not invent a success binding. A call that intentionally waits only for terminal success may use an empty body without a `→` success binding, and may declare an error channel only when it needs typed failure handling, for example:

```fab
ad "notificatio:mitte" ("x") {}
ad "notificatio:mitte" ("x") ⇥ ignotum {} cape err {
  iace err
}
```

## Non-Goals

- Do not implement a full provider registry or sigcall process transport in this goal.
- Do not hand-wrap HTTP, SQLite, Postgres, or other Rust crates as Faber interfaces.
- Do not require provider SDK contracts or generated Faber contracts for normal compilation.
- Do not require strict provider verification in the first phase.
- Do not solve every streaming abstraction (`cursor`, `rivus`, async task scheduling, backpressure) before locking the primitive `ad` semantics.
- Do not delete current non-strict unresolved-provider behavior until replacement tests prove the new behavior.
- Do not make scalar `ad` the primitive. Scalar behavior should be a consumer policy or helper over frame streams.
- Do not expose `bulk` as a separate source-level success branch in the first phase. It is a host/protocol batching detail unless a future goal adds explicit bulk-aware source syntax.

## Ground Truth Researched

- User design decision from 2026-05-24: Faber `ad` sends a syscall request, receives response frames, and the natural form `ad "db:query" (sql) → Row row { ... }` should mean per-item handling.
- `EBNF.md`: `adStmt` already supports `ad STRING(args) → Type name ⇥ ErrorType { body } cape ...`.
- `docs/factory/capability-calls/goal.md`: Epic 3 implemented scalar/non-strict Rust `ad` behavior and records unresolved providers as explicit runtime failures.
- `hosts/macos-arm64/SYSCALL_MODEL.md`: host capability calls are frame-shaped syscalls routed through built-in syscalls or sigcall providers; long-term target includes `Item`/`Bulk`, cancellation, structured errors, and provider routing.
- Follow-up research into `fieldboard`: `bulk` was created to reduce protocol overhead when many small item frames made envelope cost dominate payload transfer. `board:join` batches many object rows into `Bulk` frames, and the client expands them through the same object queue path used for single `Item` frames.
- Current compiler behavior: bodyful functions without `→` are currently accepted and may infer a return type from `redde`. The desired hardened direction is stricter: omitting `→` means effect-only, and any `redde` inside that body should become a diagnostic in a dedicated language-hardening slice.
- `crates/radix/src/parser/stmt.rs`: parser records the route string, arguments, optional success binding, body, and catch clause.
- `crates/radix/src/hir/nodes.rs`: `HirAd` currently preserves path, args, optional binding, error type, body, and catch block.
- `crates/radix/src/codegen/rust/stmt.rs`: current Rust codegen emits a scalar `match __faber_ad::<T, _>(...) { Ok(__faber_result) => body, Err(...) => ... }`.
- Prior local research into Abbot and Monk OS: Abbot demonstrates correlated frame streams and backpressure; Monk OS demonstrates dispatcher-owned syscall/sigcall routing where registered userspace providers stream responses back to the original caller.
- `hosts/macos-arm64/SYSCALL_MODEL.md` predates this semantic correction and still contains scalar `Done` payload language. Treat that as stale ground truth to repair, not as an invariant.

## Reference Packet

Before implementation, inspect:

- `EBNF.md`, section "Capability Calls".
- `docs/factory/capability-calls/goal.md`.
- `hosts/macos-arm64/SYSCALL_MODEL.md`.
- `/Users/ianzepp/work/ianzepp/fieldboard/server/src/routes/ws.rs`, especially `JOIN_BULK_CHUNK_SIZE` and `board:join` response construction.
- `/Users/ianzepp/work/ianzepp/fieldboard/client/src/net/frame_client.rs`, especially `board:join` `Item` and `Bulk` handling.
- `/Users/ianzepp/work/ianzepp/fieldboard/client/src/net/frame_client_parse.rs`, especially `parse_board_object_bulk`.
- `hosts/macos-arm64/src/kernel/frame.rs`, `router.rs`, `syscall.rs`, and `syscall_import.rs`.
- `hosts/macos-arm64/src/hal/consolum.rs` for current built-in host-effect handling.
- `crates/radix/src/parser/stmt.rs`, especially `parse_ad_stmt`.
- `crates/radix/src/syntax/ast.rs`, `crates/radix/src/hir/nodes.rs`, `crates/radix/src/hir/lower/stmt.rs`, and `crates/radix/src/hir/visit.rs`.
- `crates/radix/src/semantic/passes/typecheck/stmt.rs`.
- `crates/radix/src/codegen/rust/stmt.rs`, `expr/mod.rs`, and `tests/ad_test.rs`.
- `examples/exempla/ad/ad.fab`.

## Constraints And Invariants

- `ad` names a capability route, not a Rust crate API and not a Faber package import.
- Provider SDKs terminate at the host/provider boundary. No provider SDK type should cross into Faber compilation.
- The interchange format between Faber and host capabilities is frame data.
- Faber result shapes are caller-owned decode targets over returned frame data.
- Extra provider frame fields may be allowed, but missing required local fields should fail explicitly.
- Normal compilation remains permissive: route existence and provider installation are not required unless strict mode is requested.
- The primitive `ad` body is per item payload. `redde` inside the body returns from the enclosing function on the current item, just like other nested statement bodies.
- `bulk` is semantically transparent to source item handling. After frame decode, each payload element in a `bulk` batch is processed as though it had arrived as an individual `item`, preserving order within the batch.
- `done` completes the `ad` statement after all `item` frames and expanded `bulk` payloads have been handled. `done` is not the primary success value.
- `error` is host/provider-initiated terminal failure and must not silently succeed.
- `cancel` is caller-initiated terminal cancellation and must remain recognizable as cancellation even if it initially travels through an error-shaped alternate-exit value.
- A non-empty per-item body requires a success binding. Effect-only calls that intentionally ignore response items should not fake a return channel; they may use an empty body with no success binding. The `⇥` error channel is for typed failure handling, not for declaring terminal success.
- Omitted `→` means effect-only in the desired hardened language shape. It must not be treated as permission to infer a value return from `redde`.
- The implementation must not guess source types in codegen. The per-item binding type must come from source or future strict metadata.
- Backpressure and cancellation are part of the host/frame model, but the first compiler rewrite may model only finite in-process item iteration if that is the smallest executable proof.

## Supporting Skills

- `delivery`: lower this goal into one implementation phase at a time.
- `factory`: use for long-running execution after the first delivery spec is accepted.
- `consequences`: use before changing parser/HIR/codegen semantics because this affects examples, docs, host ABI assumptions, and future provider design.
- `zombie-docs`: use after implementation to repair stale scalar `ad` documentation.
- `poker-face`: use after each phase to verify the old scalar assumption did not survive unnoticed.

## Implementation Shape

### Phase 0: Semantic Lock And Documentation

Rewrite the language and host docs to state that `ad` is a frame-protocol syscall form by default. The success binding type is the type of each item payload, and the body runs per item payload after `item` and `bulk` decoding. Record scalar convenience as a wrapper/helper pattern rather than primitive behavior.

### Phase 1: Compiler Contract

Audit AST/HIR/semantic assumptions for scalar `ad`. Decide whether `HirAd` needs an explicit result mode field such as `FrameProtocol` or whether the existing node can be reinterpreted without shape changes. Record the contract in comments and tests.

### Phase 2: Rust Lowering Rewrite

Change Rust codegen from scalar `Result<T, String>` handling to frame response iteration. A minimal generated shape may use a temporary helper such as:

```rust
for __faber_item in __faber_ad_items::<T, _>("db:query", args)? {
    let row = __faber_item;
    /* ad body */
}
```

The helper can still fail unresolved providers clearly in native Rust. For the Wasm path, it should align with host frame responses.

### Phase 3.1: Host Documentation Correction

Rewrite `hosts/macos-arm64/SYSCALL_MODEL.md` so it no longer says scalar success data is decoded from terminal `Done`. The corrected model is `Request -> (Item | Bulk)* -> Done | Error | Cancel`, where primary success data is carried by `Item` frames or transparent `Bulk` batches.

### Phase 3.2: Narrow Host Runtime Proof

Extend the macOS host proof only as far as needed to represent zero or more `Item`/`Bulk` responses followed by one terminal `Done`, `Error`, or `Cancel`. A narrow in-process vector/iterator proof is acceptable. Do not implement provider daemons, full sigcall process transport, durable backpressure, or broad cancellation token plumbing in this phase.

### Phase 3.3: Host ABI Follow-Up Decision

If the Wasm or native host ABI cannot carry actual frame response sequences yet, record the smallest temporary bridge and open a follow-up goal. Do not introduce a second user-visible `ad` protocol or preserve scalar `Done` payload semantics as the source-level model.

### Phase 4: Examples And Tests

Add examples that make the per-item semantics obvious:

```fab
ad "db:query" (sql) → Row row {
  nota row.name
}
```

Add scalar wrapper examples where `redde` inside the per-item body returns the first or only item.

### Phase 5: Compatibility And Migration Notes

Review existing `ad` examples and tests. Migrate scalar-style assumptions to helper form or explicitly classify them as old behavior. Decide whether current scalar behavior gets a compatibility diagnostic, a direct rewrite, or no compatibility layer because the project is still pre-stable.

## Exit Strategy

Included.

- If frame-protocol `ad` proves too disruptive for current Rust e2e, keep the old scalar helper under an internal temporary name while source docs and new tests move to frame semantics.
- Do not ship two source-level meanings for the same `ad` form. Temporary backend helpers are allowed; user-facing semantics should converge on the frame protocol.
- If `cape`/alternate-exit behavior blocks the rewrite, split failure handling into a separate effects goal and keep this goal focused on success item iteration.

## Acceptance Criteria

- `ad` is documented as a frame-protocol syscall form by default, not as a Faber `@ cursor` generator.
- The success binding in `ad "route" (...) → T item { ... }` is documented and tested as an item-frame binding.
- The `ad` body is documented and tested as executing once per successful item payload.
- `bulk` is documented and tested as transparent batching: each batch element is expanded into the same per-item path as an `item` frame.
- Terminal `done`, `error`, and `cancel` semantics are recorded, including the distinction that `error` is host/provider initiated and `cancel` is caller initiated.
- Effect-only calls are documented as terminal-success-only calls with no fake success binding. Optional `⇥` error typing is documented as failure handling, not as a required return channel.
- The goal records the desired hardening that bodyful no-`→` forms are effect-only and must not contain `redde`; if that compiler behavior is changed, it should be done as an explicit language-hardening slice.
- Generated Rust no longer treats primitive `ad` as a single scalar `Result<T, String>` success path.
- Existing unresolved-provider behavior still fails clearly.
- Host docs explain that provider SDKs convert to frame data at the host boundary, and Faber decodes frame data into local caller-owned shapes.
- Examples include both streaming per-item `ad` and scalar wrapper helper patterns.

## Validation

- `cargo test -p radix ad`
- The focused Rust codegen tests that cover `ad`, using the actual test target name discovered during implementation.
- `cargo test -p faber-host-macos-arm64`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` if the active phase touches exempla or e2e classification.
- `git diff --check -- EBNF.md docs hosts crates examples`
- Review check: no docs describe `ad → T` as inherently scalar unless they are explicitly documenting a helper/consumer policy.

## Open Questions

- What canonical payload field should Faber host frames use for `bulk` batches (`items` unless a host ABI decision says otherwise)?
- Should body-without-success-binding be a parser/semantic diagnostic when the body is non-empty, or should a future goal define it as a terminal-success continuation?
- Should there eventually be explicit source sugar for common consumer policies such as exactly one item, first item, collect all items, or cursor binding?
- Should strict mode check only route/effect/provider existence, or also broad frame-data schema compatibility with the local expected item type?
- How should cancellation be exposed to source once provider streams can be long-running, beyond the initial requirement that `cancel` remains distinguishable from host/provider `error`?

## Stop Conditions

- Stop if implementation starts requiring provider SDK contracts or generated Faber provider interfaces.
- Stop if scalar `ad` remains the documented primitive behavior.
- Stop if codegen guesses item types from route names or provider assumptions.
- Stop if terminal `error` or `cancel` can be ignored accidentally and appear as success.
- Stop if `bulk` becomes a different source-level success branch instead of transparent batching for item payloads.
- Stop if the implementation needs a full provider registry or sigcall process transport before proving source/compiler semantics.
- Stop if the work requires broad redesign of `cursor`, `rivus`, or async function semantics; split that into a dedicated goal.
