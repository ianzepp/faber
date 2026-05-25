# Goal: Rewrite `ad` Around Frame Streams

**Status**: drafted, not started
**Created**: 2026-05-24
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/ad-frame-stream/`
**Mode**: language semantics, compiler lowering, host/runtime contract
**Commit Policy**: Commit after each completed implementation phase and validation gate pass
**Related Roadmap**: `docs/factory/faber-execution-roadmap/goal.md`

## Summary

Redefine `ad` as Faber's frame-stream syscall form. An `ad` call sends a host capability request and consumes the response as a stream of `item*` and/or `bulk*` frames followed by a terminal `done`, `error`, or `cancel`. The success binding type in `ad "route" (...) → T item { ... }` describes each successful item frame, not a single scalar return value. Scalar behavior remains expressible through helper functions that consume the stream and return from the enclosing function.

## Problem

- Current `ad` planning and Rust lowering treat `→ T name` as a scalar `Result<T, String>` success path.
- The host model now treats capability calls as frame-shaped syscalls, and the useful runtime primitive is a response stream, not a provider SDK return value.
- Provider SDKs should terminate at the host/provider boundary. Faber should receive only frame data and decode it into caller-owned local shapes.
- The natural source form is stream-oriented:

```fab
ad "db:query" (sql) → Row row {
  nota row.name
}
```

- That form reads as "for each returned row, run this block." Treating it as a scalar continuation hides the real runtime shape and makes future database rows, HTTP chunks, log streams, subscriptions, and provider processes awkward.

## Goals

- Make `ad` cursor-shaped by default.
- Define the success binding type as the per-item frame data decode target.
- Define the `ad` body as the per-item body, executed once for each successful item frame.
- Preserve the current source spelling where possible:

```fab
ad "provider:operation" (args) → Type item {
  ...
}
```

- Treat terminal `done` as completion of the `ad` statement, not as the scalar value.
- Treat terminal `error` or `cancel` as failure routed through existing/future `cape` or alternate-exit behavior.
- Keep provider SDK and crate-specific response types out of Faber compilation. The provider adapter converts SDK values into frame data; Faber decodes frame data into local source-declared shapes.
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

## Non-Goals

- Do not implement a full provider registry or sigcall process transport in this goal.
- Do not hand-wrap HTTP, SQLite, Postgres, or other Rust crates as Faber interfaces.
- Do not require provider SDK contracts or generated Faber contracts for normal compilation.
- Do not require strict provider verification in the first phase.
- Do not solve every streaming abstraction (`cursor`, `rivus`, async task scheduling, backpressure) before locking the primitive `ad` semantics.
- Do not delete current non-strict unresolved-provider behavior until replacement tests prove the new behavior.
- Do not make scalar `ad` the primitive. Scalar behavior should be a consumer policy or helper over frame streams.

## Ground Truth Researched

- User design decision from 2026-05-24: Faber `ad` sends a syscall request, receives response frames, and the natural form `ad "db:query" (sql) → Row row { ... }` should mean per-item handling.
- `EBNF.md`: `adStmt` already supports `ad STRING(args) → Type name ⇥ ErrorType { body } cape ...`.
- `docs/factory/capability-calls/goal.md`: Epic 3 implemented scalar/non-strict Rust `ad` behavior and records unresolved providers as explicit runtime failures.
- `hosts/macos-arm64/SYSCALL_MODEL.md`: host capability calls are frame-shaped syscalls routed through built-in syscalls or sigcall providers; long-term target includes `Item`/`Bulk`, cancellation, structured errors, and provider routing.
- `crates/radix/src/parser/stmt.rs`: parser records the route string, arguments, optional success binding, body, and catch clause.
- `crates/radix/src/hir/nodes.rs`: `HirAd` currently preserves path, args, optional binding, error type, body, and catch block.
- `crates/radix/src/codegen/rust/stmt.rs`: current Rust codegen emits a scalar `match __faber_ad::<T, _>(...) { Ok(__faber_result) => body, Err(...) => ... }`.
- Prior local research into Abbot and Monk OS: Abbot demonstrates correlated frame streams and backpressure; Monk OS demonstrates dispatcher-owned syscall/sigcall routing where registered userspace providers stream responses back to the original caller.

## Reference Packet

Before implementation, inspect:

- `EBNF.md`, section "Capability Calls".
- `docs/factory/capability-calls/goal.md`.
- `hosts/macos-arm64/SYSCALL_MODEL.md`.
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
- The primitive `ad` body is per-item. `redde` inside the body returns from the enclosing function on the current item, just like other nested statement bodies.
- `done` completes the `ad` statement after all item/bulk frames have been handled.
- `error` and `cancel` are terminal failure states and must not silently succeed.
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

Rewrite the language and host docs to state that `ad` is frame-stream shaped by default. The success binding type is the type of each item frame, and the body runs per item. Record scalar convenience as a wrapper/helper pattern rather than primitive behavior.

### Phase 1: Compiler Contract

Audit AST/HIR/semantic assumptions for scalar `ad`. Decide whether `HirAd` needs an explicit result mode field such as `FrameStream` or whether the existing node can be reinterpreted without shape changes. Record the contract in comments and tests.

### Phase 2: Rust Lowering Rewrite

Change Rust codegen from scalar `Result<T, String>` handling to frame-stream iteration. A minimal generated shape may use a temporary helper such as:

```rust
for __faber_item in __faber_ad_items::<T, _>("db:query", args)? {
    let row = __faber_item;
    /* ad body */
}
```

The helper can still fail unresolved providers clearly in native Rust. For the Wasm path, it should align with host frame responses.

### Phase 3: Host Frame Semantics

Extend or clarify the macOS host proof so a route can return zero or more `Item`/`Bulk` frames and a terminal `Done`, not only a single terminal response. Keep the first implementation narrow if needed, but avoid adding a second non-frame protocol.

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

- If stream-shaped `ad` proves too disruptive for current Rust e2e, keep the old scalar helper under an internal temporary name while source docs and new tests move to stream semantics.
- Do not ship two source-level meanings for the same `ad` form. Temporary backend helpers are allowed; user-facing semantics should converge on frame streams.
- If `cape`/alternate-exit behavior blocks the rewrite, split failure handling into a separate effects goal and keep this goal focused on success item iteration.

## Acceptance Criteria

- `ad` is documented as frame-stream shaped by default.
- The success binding in `ad "route" (...) → T item { ... }` is documented and tested as an item-frame binding.
- The `ad` body is documented and tested as executing once per successful item frame.
- Terminal `done`, `error`, and `cancel` semantics are recorded.
- Generated Rust no longer treats primitive `ad` as a single scalar `Result<T, String>` success path.
- Existing unresolved-provider behavior still fails clearly.
- Host docs explain that provider SDKs convert to frame data at the host boundary, and Faber decodes frame data into local caller-owned shapes.
- Examples include both streaming per-item `ad` and scalar wrapper helper patterns.

## Validation

- `cargo test -p radix ad`
- `cargo test -p radix --test` or the focused Rust codegen tests that cover `ad`.
- `cargo test -p faber-host-macos-arm64`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture` if the active phase touches exempla or e2e classification.
- `git diff --check -- EBNF.md docs hosts crates examples`
- Review check: no docs describe `ad → T` as inherently scalar unless they are explicitly documenting a helper/consumer policy.

## Open Questions

- Should `bulk` frames use the same `→ T item` binding path as `item` frames, or should they require a distinct declared type such as `octeti` or a future stream type?
- Should an `ad` body with no success binding mean "ignore all items and only check terminal success"?
- Should there eventually be explicit source sugar for common consumer policies such as exactly one item, first item, collect all items, or cursor binding?
- Should strict mode check only route/effect/provider existence, or also broad frame-data schema compatibility with the local expected item type?
- How should cancellation be exposed to source once provider streams can be long-running?

## Stop Conditions

- Stop if implementation starts requiring provider SDK contracts or generated Faber provider interfaces.
- Stop if scalar `ad` remains the documented primitive behavior.
- Stop if codegen guesses item types from route names or provider assumptions.
- Stop if terminal `error` or `cancel` can be ignored accidentally and appear as success.
- Stop if the implementation needs a full provider registry or sigcall process transport before proving source/compiler semantics.
- Stop if the work requires broad redesign of `cursor`, `rivus`, or async function semantics; split that into a dedicated goal.
