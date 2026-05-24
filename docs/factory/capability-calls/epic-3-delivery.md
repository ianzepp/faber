# Epic 3 Delivery: Non-Strict `ad` Capability Calls

**Status**: implemented, validation passed
**Created**: 2026-05-24
**Scope**: current Rust backend only

## Interpreted Problem

Epic 2 left the executable Rust corpus cleaner, but `examples/exempla/ad/ad.fab` is still classified as an unsupported Rust codegen failure. Epic 3 should make `ad` mean a permissive capability call in normal compilation: unresolved providers compile, then fail clearly at runtime when no provider is linked.

## Normalized Spec

- Fix the standalone `ad` example by declaring `userId`.
- Rename user-facing `ad` docs from endpoint dispatch to capability calls.
- Require explicit result types for success-binding `ad` calls when no provider metadata exists.
- Generate Rust for `ad` through a temporary unresolved dispatcher.
- Preserve body/catch control flow enough for generated Rust to compile.
- Keep host runtime, Wasm lowering, provider manifests, and strict mode out of this phase.

## Repo-Aware Baseline

- Parser and HIR already preserve `ad`.
- Typecheck currently only visits `ad` arguments/body/catch.
- Rust codegen rejects `HirStmtKind::Ad`.
- Go codegen has a useful temporary-stub pattern.
- Rust e2e currently lists `ad/ad.fab` as an expected failure.

## Stage Graph

1. Policy/docs and example cleanup.
2. Typecheck contract for omitted binding result types.
3. Rust codegen temporary unresolved dispatcher.
4. E2E classification for expected unresolved runtime failure.
5. Focused tests and validation.

## Checkpoints

- `cargo test -p radix ad`
- `cargo test -p radix codegen::rust`
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`

## Validation Results

- `cargo test -p radix ad -- --nocapture`: passed.
- `cargo test -p radix codegen::rust -- --nocapture`: passed.
- `cargo test -p radix exempla_rust_e2e -- --ignored --nocapture`: passed, `100/100` Rust exempla.

## Gate Plan

Stop if this phase starts requiring provider metadata, embedding host ABI decisions, or solving broader `cape`/alternate-exit semantics beyond the temporary Rust `ad` statement path.
