# Epic 2 Phase 34 Delivery: Rust Optional Member Access

## Interpreted Problem

After Phase 33, the Rust e2e corpus is `98/100`. The remaining Epic 2 file is
`examples/exempla/membrum/membrum.fab`. Its plain map-backed member access now
works, but optional member access still emits Rust struct field syntax for map
receivers and tries to access fields on a `nihil` local.

## Normalized Spec

- Lower optional member access on map-backed object values through keyed lookup.
- Preserve nullable chaining shape:
  - non-optional maps should produce `.get("field").cloned()`;
  - optional maps should produce `as_ref().and_then(...get("field").cloned())`;
  - optional map values that already store `Option<T>` should flatten with
    `and_then`.
- Treat optional access on a `nihil` receiver as a nullable dynamic result,
  not as Rust unit field access.
- Keep the phase scoped to Rust backend and the minimum semantic type recovery
  needed to avoid invalid `Option</* error */>` output.
- Do not solve `ad/ad.fab`; that remains Epic 3.

## Repo-Aware Baseline

- Plain Rust field access for map-backed objects lives in
  `crates/radix/src/codegen/rust/expr/access.rs`.
- Rust optional-chain emission lives in
  `crates/radix/src/codegen/rust/expr/option.rs`.
- Optional-chain typing lives in
  `crates/radix/src/semantic/passes/typecheck/access.rs`.
- `membrum/membrum.fab` currently fails rustc on `.present`, `.missing`, and
  `let empty: () = None`.

## Stage Graph

1. Add optional member map lookup emission for optional and non-optional map
   receivers.
2. Add nil-receiver optional-chain handling so `nihil?.field` emits a typed
   `None`.
3. Adjust semantic recovery for optional chaining over `nihil` so generated
   local types do not become `Option</* error */>`.
4. Add focused Rust codegen tests for map optional member chaining and nil
   optional member chaining.
5. Recheck `membrum/membrum.fab` directly and run the Rust e2e harness.

## Checkpoints

- Focused tests prove optional map member access uses `.get(...).cloned()`.
- Focused tests prove `nihil?.missing` does not emit field access on unit or an
  error Rust type.
- `examples/exempla/membrum/membrum.fab` emits Rust that compiles with `rustc`.
- Full Rust e2e count is recorded after the phase.

## Gate Plan

- `cargo test -p radix rust_optional_member_access`
- `cargo test -p radix`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`
- Poker-face audit before commit.

## Open Questions

- None for this phase. Any remaining `ad/ad.fab` failure is outside Epic 2.

## Closeout Update

Post-phase cleanup on 2026-05-24 confirms the live Rust e2e state is `99/100`,
with `examples/exempla/ad/ad.fab` as the only expected failure deferred to Epic
3. The Rust e2e harness now asserts that full expected corpus state rather than
only proving `salve-munde.fab`.
